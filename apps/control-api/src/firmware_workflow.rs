//! R9.58 operator-driven firmware control-plane primitives, NOT a flash worker.
//! Wire only after genuine MFA/OIDC, durable secure browser sessions, trusted
//! same-origin/CSRF, active Host->tenant equality and restricted DB roles exist.
//! Image bytes belong in a tenant-isolated encrypted artifact service: SQL
//! accepts an object pointer, digest and vendor manifest reference, never bytes.
use identity_core::browser_session::{BrowserSessionVault, RequestKind};
use std::{fs::File, io::Read};
use tokio_postgres::Client;
use uuid::Uuid;

pub(super) struct Artifact<'a> {
    pub id: Uuid,
    pub vendor: &'a str,
    pub exact_model: &'a str,
    pub target_version: &'a str,
    pub size_bytes: i64,
    pub sha256: &'a str,
    pub object_ref: &'a str,
    pub vendor_release_ref: &'a str,
}

pub(super) struct FirmwareRequest<'a> {
    pub id: Uuid,
    pub device_id: Uuid,
    pub artifact_id: Uuid,
    pub request_id: Uuid,
    pub reason: &'a str,
    pub window_start_epoch: i64,
    pub window_end_epoch: i64,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct FirmwareChangeRow {
    pub id: Uuid,
    pub device_id: Uuid,
    pub target_version: String,
    pub state: String,
    pub window_start: String,
    pub window_end: String,
}

fn safe_name(s: &str, max: usize) -> bool {
    !s.is_empty() && s.len() <= max && !s.chars().any(char::is_control)
}

fn safe_firmware_digest(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn safe_artifact(tenant: Uuid, a: &Artifact<'_>) -> bool {
    let prefix = format!("artifact://tenant/{tenant}/firmware/");
    a.id.get_version_num() == 4
        && safe_name(a.vendor, 64)
        && a.vendor
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
        && safe_name(a.exact_model, 128)
        && safe_name(a.target_version, 128)
        && (1024..=1_073_741_824).contains(&a.size_bytes)
        && safe_firmware_digest(a.sha256)
        && (75..=255).contains(&a.object_ref.len())
        && a.object_ref.starts_with(&prefix)
        && !a.object_ref[prefix.len()..].is_empty()
        && a.object_ref[prefix.len()..]
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'/' | b'_' | b'-'))
        && (12..=180).contains(&a.vendor_release_ref.len())
        && a.vendor_release_ref.bytes().all(|b| {
            b.is_ascii_alphanumeric()
                || matches!(b, b'_' | b'.' | b':' | b'/' | b'(' | b')' | b' ' | b'-')
        })
}

fn safe_request(r: &FirmwareRequest<'_>, now: i64) -> bool {
    r.id.get_version_num() == 4
        && r.request_id.get_version_num() == 4
        && (12..=180).contains(&r.reason.len())
        && r.reason.bytes().all(|b| {
            b.is_ascii_alphanumeric()
                || matches!(
                    b,
                    b' ' | b'_' | b'.' | b',' | b':' | b'/' | b'(' | b')' | b'-'
                )
        })
        && r.window_start_epoch > now
        && r.window_start_epoch <= now + 30 * 24 * 3600
        && r.window_end_epoch > r.window_start_epoch
        && r.window_end_epoch <= r.window_start_epoch + 8 * 3600
}

pub(super) async fn stage_artifact_for_session(
    vault: &mut BrowserSessionVault,
    cookie: &str,
    csrf: &str,
    same_origin: bool,
    db: &Client,
    tenant: Uuid,
    artifact: &Artifact<'_>,
    now: u64,
) -> Option<Uuid> {
    let identity =
        vault.authenticate(cookie, Some(csrf), RequestKind::Mutation, same_origin, now)?;
    if !safe_artifact(tenant, artifact) {
        return None;
    }
    let row = db
        .query_one(
            "SELECT ipat_platform.stage_firmware_artifact(\
             $1,$2,$3::uuid,$4::uuid,$5,$6,$7,$8,$9,$10,$11)",
            &[
                &identity.issuer(),
                &identity.subject(),
                &tenant,
                &artifact.id,
                &artifact.vendor,
                &artifact.exact_model,
                &artifact.target_version,
                &artifact.size_bytes,
                &artifact.sha256,
                &artifact.object_ref,
                &artifact.vendor_release_ref,
            ],
        )
        .await
        .ok()?;
    row.get(0)
}

pub(super) async fn propose_for_session(
    vault: &mut BrowserSessionVault,
    cookie: &str,
    csrf: &str,
    same_origin: bool,
    db: &Client,
    tenant: Uuid,
    request: &FirmwareRequest<'_>,
    now: u64,
) -> Option<Uuid> {
    let identity =
        vault.authenticate(cookie, Some(csrf), RequestKind::Mutation, same_origin, now)?;
    if !safe_request(request, now as i64) {
        return None;
    }
    let row = db
        .query_one(
            "SELECT ipat_platform.propose_firmware_change(\
             $1,$2,$3::uuid,$4::uuid,$5::uuid,$6::uuid,$7::uuid,$8,\
             to_timestamp($9::bigint::double precision),to_timestamp($10::bigint::double precision))",
            &[
                &identity.issuer(),
                &identity.subject(),
                &tenant,
                &request.id,
                &request.device_id,
                &request.artifact_id,
                &request.request_id,
                &request.reason,
                &request.window_start_epoch,
                &request.window_end_epoch,
            ],
        )
        .await
        .ok()?;
    row.get(0)
}

// Reviewer must have independently verified MFA in the caller, and DB checks
// current exact security_admin membership + separation from the requester.
pub(super) async fn review_for_session(
    vault: &mut BrowserSessionVault,
    cookie: &str,
    csrf: &str,
    same_origin: bool,
    db: &Client,
    tenant: Uuid,
    change: Uuid,
    approve: bool,
    now: u64,
) -> Option<String> {
    let identity =
        vault.authenticate(cookie, Some(csrf), RequestKind::Mutation, same_origin, now)?;
    let row = db
        .query_one(
            "SELECT ipat_platform.review_firmware_change($1,$2,$3::uuid,$4::uuid,$5)",
            &[
                &identity.issuer(),
                &identity.subject(),
                &tenant,
                &change,
                &approve,
            ],
        )
        .await
        .ok()?;
    row.get(0)
}

/// Explicit operator click. Returns execution INTENT ONLY. This project has
/// no qualified actuator mounted, so no real device firmware can be changed.
pub(super) async fn request_execution_for_session(
    vault: &mut BrowserSessionVault,
    cookie: &str,
    csrf: &str,
    same_origin: bool,
    db: &Client,
    tenant: Uuid,
    change: Uuid,
    now: u64,
) -> Option<String> {
    let identity =
        vault.authenticate(cookie, Some(csrf), RequestKind::Mutation, same_origin, now)?;
    let row = db
        .query_one(
            "SELECT ipat_platform.request_firmware_execution($1,$2,$3::uuid,$4::uuid)",
            &[&identity.issuer(), &identity.subject(), &tenant, &change],
        )
        .await
        .ok()?;
    row.get(0)
}

pub(super) async fn list_for_session(
    vault: &mut BrowserSessionVault,
    cookie: &str,
    same_origin: bool,
    db: &Client,
    tenant: Uuid,
    now: u64,
) -> Option<Vec<FirmwareChangeRow>> {
    let identity = vault.authenticate(cookie, None, RequestKind::Read, same_origin, now)?;
    let rows = db
        .query(
            "SELECT id,device_id,target_version,state,window_start::text,window_end::text\
             FROM ipat_platform.list_firmware_changes($1,$2,$3::uuid)",
            &[&identity.issuer(), &identity.subject(), &tenant],
        )
        .await
        .ok()?;
    if rows.len() > 100 {
        return None;
    }
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let state: String = row.get(3);
        if !matches!(
            state.as_str(),
            "AWAITING_EVIDENCE" | "APPROVED" | "REJECTED" | "EXECUTION_REQUESTED"
        ) {
            return None;
        }
        out.push(FirmwareChangeRow {
            id: row.get(0),
            device_id: row.get(1),
            target_version: row.get(2),
            state,
            window_start: row.get(4),
            window_end: row.get(5),
        });
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tenant() -> Uuid {
        Uuid::parse_str("11111111-1111-4111-8111-111111111111").unwrap()
    }
    fn artifact<'a>() -> Artifact<'a> {
        Artifact {
            id: Uuid::parse_str("f0000000-0000-4000-8000-000000000001").unwrap(),
            vendor: "ZTE",
            exact_model: "C320",
            target_version: "SYNTHETIC-TEST-NOT-FIRMWARE",
            size_bytes: 1024,
            sha256: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            object_ref:
                "artifact://tenant/11111111-1111-4111-8111-111111111111/firmware/test-object",
            vendor_release_ref: "synthetic-only-do-not-flash",
        }
    }
    #[test]
    fn artifact_reference_is_not_binary_or_vendor_attestation() {
        let mut a = artifact();
        assert!(safe_artifact(tenant(), &a));
        a.sha256 = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
        assert!(!safe_artifact(tenant(), &a));
        a.sha256 = artifact().sha256;
        a.object_ref =
            "artifact://tenant/22222222-2222-4222-8222-222222222222/firmware/test-object";
        assert!(!safe_artifact(tenant(), &a));
    }
    #[test]
    fn rejects_password_url_and_unbounded_window() {
        let now = 1_800_000_000;
        let mut r = FirmwareRequest {
            id: artifact().id,
            device_id: artifact().id,
            artifact_id: artifact().id,
            request_id: artifact().id,
            reason: "Scheduled synthetic operator test",
            window_start_epoch: now + 3600,
            window_end_epoch: now + 7200,
        };
        assert!(safe_request(&r, now));
        r.reason = "admin:password@example";
        assert!(!safe_request(&r, now));
        r.reason = "Scheduled synthetic operator test";
        r.window_end_epoch = now + 12 * 3600;
        assert!(!safe_request(&r, now));
        r.window_end_epoch = now + 7200;
        r.window_start_epoch = now - 1;
        assert!(!safe_request(&r, now));
    }
}
