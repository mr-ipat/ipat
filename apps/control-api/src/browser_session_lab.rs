//! R8.8 PRIVATE OFFLINE bridge: truly SIGNED synthetic OIDC ID+access pair,
//! independently sealed REAL PostgreSQL membership, then opaque identity-only
//! session. NEVER mount on HTTP until actual real confidential IdP+MFA exists.
//! A fresh exact DB lookup remains REQUIRED PER REQUEST on every tenant API.
use identity_core::{
    browser_session::{BrowserSessionVault, IssuedSession, RequestKind},
    PinnedIssuer,
};
use tokio_postgres::Client;
use uuid::Uuid;

fn valid_scope(role: &str, pop: Option<&str>) -> bool {
    match role {
        "tenant_admin" => pop.is_none(),
        "noc_engineer" => pop.is_some_and(|p| {
            !p.is_empty()
                && p.len() <= 128
                && p.bytes()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_' | b'.'))
        }),
        _ => false,
    }
}
async fn member(
    db: &Client,
    issuer: &str,
    subject: &str,
    tenant: Uuid,
    role: &str,
    pop: Option<&str>,
    now: u64,
) -> bool {
    if !valid_scope(role, pop) || issuer.is_empty() || subject.is_empty() {
        return false;
    }
    // EXACT restricted EXECUTE-only PostgreSQL function; never SQL string
    // interpolation, GUC role changes, supplied client-side tenant headers.
    let row = db
        .query_opt(
            "SELECT approved_by,EXTRACT(EPOCH FROM expires_at)::bigint,tenant_slug
         FROM ipat_platform.lookup_active_membership($1,$2,$3::uuid,$4,$5)",
            &[&issuer, &subject, &tenant, &role, &pop],
        )
        .await;
    let Ok(Some(row)) = row else { return false };
    let approver: String = row.get(0);
    let expires: i64 = row.get(1);
    let slug: String = row.get(2);
    !approver.trim().is_empty()
        && !slug.is_empty()
        && u64::try_from(expires).is_ok_and(|expiry| now < expiry)
}
/// PRIVATE, no runtime caller yet. The actual IdP enrollment, transport proof
/// and MFA challenge are external hard gates; these synthetic signatures are
/// NOT production browser authorization or real hardware admission.
pub(super) async fn issue_after_sealed_membership(
    vault: &mut BrowserSessionVault,
    verifier: &PinnedIssuer,
    id_token: &str,
    access_token: &str,
    client_id: &str,
    original_server_nonce: &str,
    restricted_db: &Client,
    tenant: Uuid,
    role: &str,
    pop: Option<&str>,
    now: u64,
) -> Option<IssuedSession> {
    let verified = verifier
        .verify_offline_browser_pair(client_id, original_server_nonce, id_token, access_token)
        .ok()?;
    if now >= verified.expires_at()
        || !member(
            restricted_db,
            verified.issuer(),
            verified.subject(),
            tenant,
            role,
            pop,
            now,
        )
        .await
    {
        return None;
    }
    vault.issue(verified, now)
}
/// Each request must independently re-check *current* DB membership after
/// cookie+CSRF, NOT trust a cached role/POP from session issuance. No live route.
pub(super) async fn current_scope_allowed(
    vault: &mut BrowserSessionVault,
    cookie: &str,
    csrf: Option<&str>,
    kind: RequestKind,
    independently_verified_origin: bool,
    restricted_db: &Client,
    tenant: Uuid,
    role: &str,
    pop: Option<&str>,
    now: u64,
) -> bool {
    let Some(identity) = vault.authenticate(cookie, csrf, kind, independently_verified_origin, now)
    else {
        return false;
    };
    member(
        restricted_db,
        identity.issuer(),
        identity.subject(),
        tenant,
        role,
        pop,
        now,
    )
    .await
}

/// R8.9 UNMOUNTED BFF Device Manager bridge: only tenant/POP-authorized
/// candidate metadata, including honestly recorded pending/review outcomes.
/// Never returns a management IP, secret or invented connectivity measurement.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct PendingDevice {
    pub id: Uuid,
    pub pop_id: String,
    pub display_name: String,
    pub kind: String,
    pub vendor: String,
    pub adoption_state: String,
    pub connectivity: String,
    pub health: String,
}
/// Every request proves the cryptographic session again AND joins current
/// membership and strictly scoped rows in ONE PostgreSQL statement/snapshot.
/// No HTTP route until a REAL independently verified IdP/MFA/HTTPS BFF exists.
pub(super) async fn pending_devices_for_session(
    vault: &mut BrowserSessionVault,
    cookie: &str,
    trusted_host_origin: bool,
    restricted_db: &Client,
    tenant: Uuid,
    role: &str,
    pop: Option<&str>,
    now: u64,
) -> Option<Vec<PendingDevice>> {
    if !valid_scope(role, pop) {
        return None;
    }
    let identity = vault.authenticate(cookie, None, RequestKind::Read, trusted_host_origin, now)?;
    let rows = restricted_db
        .query(
            "WITH permit AS MATERIALIZED (
                SELECT approved_by, EXTRACT(EPOCH FROM expires_at)::bigint AS expiry
                FROM ipat_platform.lookup_active_membership($1,$2,$3::uuid,$4,$5)
             )
             SELECT permit.approved_by,permit.expiry,d.id,d.pop_id,
               d.display_name,d.device_kind,d.vendor,d.adoption_state,
               d.connectivity,d.health
             FROM permit LEFT JOIN LATERAL
               ipat_platform.list_lab_device_candidates($1,$2,$3::uuid,$4,$5) d
               ON true ORDER BY d.requested_at DESC,d.id",
            &[
                &identity.issuer(),
                &identity.subject(),
                &tenant,
                &role,
                &pop,
            ],
        )
        .await
        .ok()?;
    let first = rows.first()?;
    let approved_by: String = first.get(0);
    let expires: i64 = first.get(1);
    if approved_by.trim().is_empty()
        || !u64::try_from(expires).is_ok_and(|expiry| now < expiry)
        || rows.len() > 100
    {
        return None;
    }
    let mut inventory = Vec::new();
    for row in rows {
        let Some(id): Option<Uuid> = row.get(2) else {
            continue;
        };
        let row_pop: String = row.get(3);
        if role == "noc_engineer" && Some(row_pop.as_str()) != pop {
            return None;
        }
        let adoption_state: String = row.get(7);
        let connectivity: String = row.get(8);
        let health: String = row.get(9);
        if !matches!(
            adoption_state.as_str(),
            "pending_review" | "approved" | "rejected" | "quarantined"
        ) || !matches!(
            connectivity.as_str(),
            "unknown" | "reachable" | "unreachable"
        ) || !matches!(
            health.as_str(),
            "not_measured" | "normal" | "degraded" | "critical"
        ) {
            return None;
        }
        inventory.push(PendingDevice {
            id,
            pop_id: row_pop,
            display_name: row.get(4),
            kind: row.get(5),
            vendor: row.get(6),
            adoption_state,
            connectivity,
            health,
        });
    }
    Some(inventory)
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use identity_core::browser_session::RequestKind;
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
    use serde_json::{json, Value};
    use sha2::{Digest, Sha256};
    use std::{
        fs,
        process::Command,
        str::FromStr,
        time::{SystemTime, UNIX_EPOCH},
    };
    use tokio_postgres::{Config, NoTls};

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
    const NONCE: &str = "nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn";
    const ISS: &str = "https://id.example.invalid/realms/lab";
    const CLIENT: &str = "ipat-private-browser";
    fn sign(key: &[u8], claims: &Value, typ: &str) -> String {
        let mut h = Header::new(Algorithm::RS256);
        h.kid = Some("private-lab".into());
        h.typ = Some(typ.into());
        encode(&h, claims, &EncodingKey::from_rsa_pem(key).unwrap()).unwrap()
    }
    fn synthetic_pair() -> (PinnedIssuer, String, String, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let privpath = dir.path().join("test-only.key");
        let pubpath = dir.path().join("test-only.pub");
        assert!(Command::new("openssl")
            .args([
                "genpkey",
                "-algorithm",
                "RSA",
                "-pkeyopt",
                "rsa_keygen_bits:2048",
                "-out"
            ])
            .arg(&privpath)
            .output()
            .unwrap()
            .status
            .success());
        assert!(Command::new("openssl")
            .args(["pkey", "-pubout", "-in"])
            .arg(&privpath)
            .arg("-out")
            .arg(&pubpath)
            .output()
            .unwrap()
            .status
            .success());
        let verifier = PinnedIssuer::new(
            ISS,
            "ipat-control-api",
            "private-lab",
            &fs::read(&pubpath).unwrap(),
        )
        .unwrap();
        let key = fs::read(&privpath).unwrap();
        let now = now();
        let access = sign(
            &key,
            &json!({
                "iss":ISS,"aud":"ipat-control-api","sub":"synthetic-operator",
                "iat":now,"nbf":now,"exp":now+180,"amr":["pwd","mfa"],
                "tenant_id":"fake-untrusted-escalation","role":"platform_owner"
            }),
            "at+jwt",
        );
        let hash = Sha256::digest(access.as_bytes());
        let id = sign(
            &key,
            &json!({
                "iss":ISS,"aud":CLIENT,"sub":"synthetic-operator",
                "iat":now,"nbf":now,"exp":now+180,"auth_time":now,
                "nonce":NONCE,"at_hash":URL_SAFE_NO_PAD.encode(&hash[..16]),
                "amr":["pwd","mfa"],"azp":CLIENT
            }),
            "JWT",
        );
        (verifier, id, access, dir)
    }
    #[tokio::test]
    async fn r89_actual_signed_opaque_session_reads_real_sealed_pop_candidate_rows() {
        if std::env::var("IPAT_PG_EPHEMERAL_TEST").as_deref() != Ok("1") {
            return; // real DB test ONLY in mandatory disposable CI
        }
        assert_eq!(std::env::var("PGHOST").as_deref(), Ok("127.0.0.1"));
        assert_eq!(std::env::var("PGDATABASE").as_deref(), Ok("ipat_synthetic"));
        assert_eq!(
            std::env::var("IPAT_PG_SYNTHETIC_PASSWORD").as_deref(),
            Ok("local_ci_synthetic_only")
        );
        let db = Config::from_str(
            "host=127.0.0.1 port=5432 user=ipat_lab_identity_reader password=local_ci_synthetic_only dbname=ipat_synthetic"
        ).unwrap();
        let (reader, read_conn) = db.connect(NoTls).await.unwrap();
        let rt = tokio::spawn(async move {
            let _ = read_conn.await;
        });
        let writer_cfg = Config::from_str(
            "host=127.0.0.1 port=5432 user=ipat_lab_device_registrar password=local_ci_synthetic_only dbname=ipat_synthetic"
        ).unwrap();
        let (writer, write_conn) = writer_cfg.connect(NoTls).await.unwrap();
        let wt = tokio::spawn(async move {
            let _ = write_conn.await;
        });
        let a = Uuid::parse_str("11111111-1111-4111-8111-111111111111").unwrap();
        let b = Uuid::parse_str("22222222-2222-4222-8222-222222222222").unwrap();
        let (verifier, id, access, _keys) = synthetic_pair();
        let n = now();
        let mut vault = BrowserSessionVault::default();
        assert!(pending_devices_for_session(
            &mut vault,
            "forged",
            true,
            &reader,
            a,
            "noc_engineer",
            Some("pop-a"),
            n
        )
        .await
        .is_none());
        let issued = issue_after_sealed_membership(
            &mut vault,
            &verifier,
            &id,
            &access,
            CLIENT,
            NONCE,
            &reader,
            a,
            "noc_engineer",
            Some("pop-a"),
            n,
        )
        .await
        .expect("actual restricted SQL and verified independently signed pair");
        let before = pending_devices_for_session(
            &mut vault,
            issued.cookie_secret(),
            true,
            &reader,
            a,
            "noc_engineer",
            Some("pop-a"),
            n,
        )
        .await
        .expect("real empty or preseeded allowed rows");
        assert!(before.iter().all(|x| x.pop_id == "pop-a"));
        for (tenant, request, pop, name) in [
            (
                a,
                "b0000000-0000-4000-8000-000000000089",
                "pop-a",
                "LAB-R89-OLT-A",
            ),
            (
                b,
                "b0000000-0000-4000-8000-000000000090",
                "pop-b",
                "LAB-R89-ONT-B",
            ),
        ] {
            let request = Uuid::parse_str(request).unwrap();
            let created = writer
                .query_one(
                    "SELECT ipat_platform.propose_lab_device_candidate(
                   $1,$2,$3::uuid,$4::uuid,$5,$6,$7,$8,$9,$10)",
                    &[
                        &ISS,
                        &"synthetic-operator",
                        &tenant,
                        &request,
                        &pop,
                        &name,
                        &"olt",
                        &"ZTE",
                        &Some("VIRTUAL-C320"),
                        &Some("10.20.3.4"),
                    ],
                )
                .await
                .unwrap();
            let _: Uuid = created
                .get::<_, Option<Uuid>>(0)
                .expect("genuine pending draft");
        }
        let rows_a = pending_devices_for_session(
            &mut vault,
            issued.cookie_secret(),
            true,
            &reader,
            a,
            "noc_engineer",
            Some("pop-a"),
            n,
        )
        .await
        .expect("exact POP after independent recheck");
        assert!(rows_a.iter().any(|d| d.display_name == "LAB-R89-OLT-A"
            && d.adoption_state == "pending_review"
            && d.connectivity == "unknown"
            && d.health == "not_measured"));
        assert!(rows_a.iter().all(|d| d.pop_id == "pop-a"));
        assert!(!rows_a.iter().any(|d| d.display_name == "LAB-R89-ONT-B"));
        assert!(pending_devices_for_session(
            &mut vault,
            issued.cookie_secret(),
            true,
            &reader,
            b,
            "noc_engineer",
            Some("pop-a"),
            n
        )
        .await
        .is_none());
        assert!(pending_devices_for_session(
            &mut vault,
            issued.cookie_secret(),
            true,
            &reader,
            a,
            "noc_engineer",
            Some("pop-b"),
            n
        )
        .await
        .is_none());
        assert!(pending_devices_for_session(
            &mut vault,
            issued.cookie_secret(),
            true,
            &reader,
            a,
            "platform_owner",
            None,
            n
        )
        .await
        .is_none());
        assert!(pending_devices_for_session(
            &mut vault,
            issued.cookie_secret(),
            false,
            &reader,
            a,
            "noc_engineer",
            Some("pop-a"),
            n
        )
        .await
        .is_none());
        let rows_b = pending_devices_for_session(
            &mut vault,
            issued.cookie_secret(),
            true,
            &reader,
            b,
            "noc_engineer",
            Some("pop-b"),
            n,
        )
        .await
        .expect("separately approved SECOND company NOC POP");
        assert!(rows_b.iter().any(|d| d.display_name == "LAB-R89-ONT-B"));
        assert!(!rows_b.iter().any(|d| d.display_name == "LAB-R89-OLT-A"));
        assert!(pending_devices_for_session(
            &mut vault,
            issued.cookie_secret(),
            true,
            &reader,
            a,
            "noc_engineer",
            Some("pop-a"),
            issued.expires_at()
        )
        .await
        .is_none());
        drop(reader);
        drop(writer);
        rt.abort();
        wt.abort();
    }

    #[tokio::test]
    async fn r88_real_rsa_pair_to_disposable_postgres_tenant_pop_then_opaque_session() {
        // Separate REQUIRED CI fixture; on owner Mac/actual VPS never
        // auto-connect to real DB or provision a fake MFA operator.
        if std::env::var("IPAT_PG_EPHEMERAL_TEST").as_deref() != Ok("1") {
            return;
        }
        assert_eq!(std::env::var("PGHOST").as_deref(), Ok("127.0.0.1"));
        assert_eq!(std::env::var("PGDATABASE").as_deref(), Ok("ipat_synthetic"));
        assert_eq!(
            std::env::var("IPAT_PG_SYNTHETIC_PASSWORD").as_deref(),
            Ok("local_ci_synthetic_only")
        );
        let db=Config::from_str("host=127.0.0.1 port=5432 user=ipat_lab_identity_reader password=local_ci_synthetic_only dbname=ipat_synthetic").unwrap();
        let (client, conn) = db.connect(NoTls).await.unwrap();
        let task = tokio::spawn(async move {
            let _ = conn.await;
        });
        let (verifier, id, access, _key) = synthetic_pair();
        let a = Uuid::parse_str("11111111-1111-4111-8111-111111111111").unwrap();
        let b = Uuid::parse_str("22222222-2222-4222-8222-222222222222").unwrap();
        let mut vault = BrowserSessionVault::default();
        let n = now();
        assert!(issue_after_sealed_membership(
            &mut vault,
            &verifier,
            &id,
            &access,
            CLIENT,
            &"x".repeat(43),
            &client,
            a,
            "noc_engineer",
            Some("pop-a"),
            n
        )
        .await
        .is_none());
        assert!(issue_after_sealed_membership(
            &mut vault,
            &verifier,
            &id,
            &access,
            CLIENT,
            NONCE,
            &client,
            a,
            "platform_owner",
            None,
            n
        )
        .await
        .is_none());
        assert!(issue_after_sealed_membership(
            &mut vault,
            &verifier,
            &id,
            &access,
            CLIENT,
            NONCE,
            &client,
            b,
            "noc_engineer",
            Some("pop-a"),
            n
        )
        .await
        .is_none());
        assert!(issue_after_sealed_membership(
            &mut vault,
            &verifier,
            &id,
            &access,
            CLIENT,
            NONCE,
            &client,
            a,
            "noc_engineer",
            Some("pop-b"),
            n
        )
        .await
        .is_none());
        assert_eq!(vault.active_count(), 0);
        let issued = issue_after_sealed_membership(
            &mut vault,
            &verifier,
            &id,
            &access,
            CLIENT,
            NONCE,
            &client,
            a,
            "noc_engineer",
            Some("pop-a"),
            n,
        )
        .await
        .expect("actually restricted SQL");
        assert!(issued
            .secure_cookie_header()
            .contains("Secure; HttpOnly; SameSite=Strict"));
        assert!(
            current_scope_allowed(
                &mut vault,
                issued.cookie_secret(),
                None,
                RequestKind::Read,
                true,
                &client,
                a,
                "noc_engineer",
                Some("pop-a"),
                n
            )
            .await
        );
        assert!(
            !current_scope_allowed(
                &mut vault,
                issued.cookie_secret(),
                None,
                RequestKind::Mutation,
                true,
                &client,
                a,
                "noc_engineer",
                Some("pop-a"),
                n
            )
            .await
        );
        assert!(
            !current_scope_allowed(
                &mut vault,
                issued.cookie_secret(),
                Some("wrong"),
                RequestKind::Mutation,
                true,
                &client,
                a,
                "noc_engineer",
                Some("pop-a"),
                n
            )
            .await
        );
        assert!(
            !current_scope_allowed(
                &mut vault,
                issued.cookie_secret(),
                Some(issued.csrf_secret()),
                RequestKind::Mutation,
                false,
                &client,
                a,
                "noc_engineer",
                Some("pop-a"),
                n
            )
            .await
        );
        assert!(
            current_scope_allowed(
                &mut vault,
                issued.cookie_secret(),
                Some(issued.csrf_secret()),
                RequestKind::Mutation,
                true,
                &client,
                a,
                "noc_engineer",
                Some("pop-a"),
                n
            )
            .await
        );
        assert!(
            !current_scope_allowed(
                &mut vault,
                issued.cookie_secret(),
                Some(issued.csrf_secret()),
                RequestKind::Mutation,
                true,
                &client,
                b,
                "noc_engineer",
                Some("pop-a"),
                n
            )
            .await
        );
        assert!(
            !current_scope_allowed(
                &mut vault,
                issued.cookie_secret(),
                Some(issued.csrf_secret()),
                RequestKind::Mutation,
                true,
                &client,
                a,
                "tenant_admin",
                Some("pop-a"),
                n
            )
            .await
        );
        assert!(
            current_scope_allowed(
                &mut vault,
                issued.cookie_secret(),
                None,
                RequestKind::Read,
                true,
                &client,
                b,
                "noc_engineer",
                Some("pop-b"),
                n
            )
            .await
        );
        assert!(
            !current_scope_allowed(
                &mut vault,
                issued.cookie_secret(),
                None,
                RequestKind::Read,
                true,
                &client,
                a,
                "noc_engineer",
                Some("pop-a"),
                issued.expires_at()
            )
            .await
        );
        assert_eq!(vault.active_count(), 0);
        drop(client);
        task.abort();
    }
}
