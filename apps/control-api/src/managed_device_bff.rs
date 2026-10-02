//! R9.57 transport-neutral production Device Registry BFF primitives.
//!
//! Deliberately UNMOUNTED until HTTPS + pinned IdP/MFA + durable, per-tenant
//! session storage + canonical verified-domain context are independently proven.
//! Caller MUST derive `tenant` from verified Host and compare to current
//! trusted identity membership. Database re-checks membership for each call.
//! This module cannot start a network probe, save plaintext secrets or adopt.
use identity_core::browser_session::{BrowserSessionVault, RequestKind};
use std::{fs::File, io::Read};
use tokio_postgres::Client;
use uuid::Uuid;

pub(super) struct NewDevice<'a> {
    pub request_id: Uuid,
    pub pop_id: &'a str,
    pub display_name: &'a str,
    pub device_kind: &'a str,
    pub vendor: &'a str,
    pub intended_model: Option<&'a str>,
    pub management_transport: &'a str,
    pub management_host: Option<&'a str>,
    pub management_port: Option<i32>,
    pub secret_ref: Option<&'a str>,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct DeviceRow {
    pub id: Uuid,
    pub pop_id: String,
    pub display_name: String,
    pub device_kind: String,
    pub vendor: String,
    pub intended_model: Option<String>,
    pub management_transport: String,
    pub lifecycle_state: String,
}

fn id_from_os() -> Option<Uuid> {
    let mut bytes = [0u8; 16];
    File::open("/dev/urandom")
        .ok()?
        .read_exact(&mut bytes)
        .ok()?;
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Some(Uuid::from_bytes(bytes))
}

fn valid_new_device(tenant: Uuid, d: &NewDevice<'_>) -> bool {
    let pop = !d.pop_id.is_empty()
        && d.pop_id.len() <= 128
        && d.pop_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'));
    let name = !d.display_name.is_empty()
        && d.display_name.len() <= 120
        && d.display_name.trim() == d.display_name
        && !d.display_name.chars().any(char::is_control);
    let vendor = !d.vendor.is_empty()
        && d.vendor.len() <= 64
        && d.vendor
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'));
    let model = d
        .intended_model
        .is_none_or(|m| !m.is_empty() && m.len() <= 128 && !m.chars().any(char::is_control));
    let kind = matches!(d.device_kind, "olt" | "ont" | "router");
    let protocol = matches!(
        d.management_transport,
        "ssh" | "snmp" | "routeros_api_ssl" | "cwmp" | "usp"
    );
    let endpoint = if matches!(d.management_transport, "cwmp" | "usp") {
        d.management_host.is_none() && d.management_port.is_none()
    } else {
        d.management_host.is_some_and(|h| {
            (3..=253).contains(&h.len())
                && h.bytes().next().is_some_and(|b| b.is_ascii_alphanumeric())
                && h.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b':' | b'-'))
        }) && d.management_port.is_some_and(|p| (1..=65535).contains(&p))
    };
    // A vault URI is metadata. Its use by future workers needs independent
    // vault ACL checks; this prevents accidentally storing raw passwords here.
    let prefix = format!("vault://tenant/{tenant}/");
    let reference = d.secret_ref.is_none_or(|r| {
        (55..=255).contains(&r.len())
            && r.starts_with(&prefix)
            && r[prefix.len()..]
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'/'))
    });
    pop && name
        && vendor
        && model
        && kind
        && protocol
        && endpoint
        && reference
        && (d.management_transport != "routeros_api_ssl" || d.device_kind == "router")
        && d.request_id.get_version_num() == 4
}

pub(super) async fn save_for_session(
    vault: &mut BrowserSessionVault,
    cookie: &str,
    csrf: &str,
    same_origin: bool,
    db: &Client,
    tenant: Uuid,
    d: &NewDevice<'_>,
    now: u64,
) -> Option<Uuid> {
    let identity =
        vault.authenticate(cookie, Some(csrf), RequestKind::Mutation, same_origin, now)?;
    if !valid_new_device(tenant, d) {
        return None;
    }
    let new_id = id_from_os()?;
    let row = db
        .query_one(
            "SELECT ipat_platform.register_managed_device(\
          $1,$2,$3::uuid,$4::uuid,$5::uuid,$6,$7,$8,$9,$10,$11,$12,$13,$14)",
            &[
                &identity.issuer(),
                &identity.subject(),
                &tenant,
                &new_id,
                &d.request_id,
                &d.pop_id,
                &d.display_name,
                &d.device_kind,
                &d.vendor,
                &d.intended_model,
                &d.management_transport,
                &d.management_host,
                &d.management_port,
                &d.secret_ref,
            ],
        )
        .await
        .ok()?;
    row.get::<usize, Option<Uuid>>(0)
}

pub(super) async fn list_for_session(
    vault: &mut BrowserSessionVault,
    cookie: &str,
    same_origin: bool,
    db: &Client,
    tenant: Uuid,
    pop: Option<&str>,
    now: u64,
) -> Option<Vec<DeviceRow>> {
    let identity = vault.authenticate(cookie, None, RequestKind::Read, same_origin, now)?;
    if pop.is_some_and(|p| {
        p.is_empty()
            || p.len() > 128
            || !p
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
    }) {
        return None;
    }
    let rows = db
        .query(
            "SELECT id,pop_id,display_name,device_kind,vendor,intended_model,\
          management_transport,lifecycle_state\
          FROM ipat_platform.list_managed_devices($1,$2,$3::uuid,$4)",
            &[&identity.issuer(), &identity.subject(), &tenant, &pop],
        )
        .await
        .ok()?;
    if rows.len() > 100 {
        return None;
    }
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let state: String = row.get(7);
        if state != "SAVED" {
            return None;
        }
        out.push(DeviceRow {
            id: row.get(0),
            pop_id: row.get(1),
            display_name: row.get(2),
            device_kind: row.get(3),
            vendor: row.get(4),
            intended_model: row.get(5),
            management_transport: row.get(6),
            lifecycle_state: state,
        });
    }
    Some(out)
}

// R9.65 production-path metadata-only detail/edit/archive. Intentionally
// UNMOUNTED until the commercial HTTPS IdP/MFA, verified Host+tenant and
// durable shared browser session release gates pass. The SQL functions make
// independent current PostgreSQL membership/tenant/firmware checks.
pub(super) struct MetadataUpdate<'a> {
    pub id: Uuid,
    pub expected_revision: i64,
    pub pop_id: &'a str,
    pub display_name: &'a str,
    pub intended_model: Option<&'a str>,
    pub management_host: Option<&'a str>,
    pub management_port: Option<i32>,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct DeviceMetadataDetail {
    pub id: Uuid,
    pub pop_id: String,
    pub display_name: String,
    pub device_kind: String,
    pub vendor: String,
    pub intended_model: Option<String>,
    pub management_transport: String,
    pub management_host: Option<String>,
    pub management_port: Option<i32>,
    pub lifecycle_state: String,
    pub metadata_revision: i64,
    pub created_at: String,
    pub archived_at: Option<String>,
    pub archived_site_instance_id: Option<Uuid>,
    pub archived_site_name: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct AdminDeviceRow {
    pub id: Uuid,
    pub pop_id: String,
    pub display_name: String,
    pub device_kind: String,
    pub vendor: String,
    pub intended_model: Option<String>,
    pub management_transport: String,
    pub lifecycle_state: String,
    pub metadata_revision: i64,
    pub created_cursor: String,
}
#[derive(Debug, PartialEq, Eq)]
pub(super) struct AdminDevicePage {
    pub devices: Vec<AdminDeviceRow>,
    pub next_after: Option<(String, Uuid)>,
}

fn valid_metadata_update(input: &MetadataUpdate<'_>) -> bool {
    let valid_site = !input.pop_id.is_empty()
        && input.pop_id.len() <= 128
        && input
            .pop_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'));
    let valid_name = !input.display_name.is_empty()
        && input.display_name.len() <= 120
        && input.display_name.trim() == input.display_name
        && !input.display_name.chars().any(char::is_control);
    let valid_model = input.intended_model.is_none_or(|model| {
        !model.is_empty() && model.len() <= 128 && !model.chars().any(char::is_control)
    });
    let valid_endpoint = match (input.management_host, input.management_port) {
        (None, None) => true, // inbound CWMP/USP; exact type verified in SQL
        (Some(host), Some(port)) => {
            (3..=253).contains(&host.len())
                && host
                    .bytes()
                    .next()
                    .is_some_and(|first| first.is_ascii_alphanumeric())
                && host
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b':' | b'-'))
                && (1..=65535).contains(&port)
        }
        _ => false,
    };
    input.expected_revision > 0 && valid_site && valid_name && valid_model && valid_endpoint
}

// All pagination cursors are generated by PostgreSQL in UTC. The frontend
// cannot get management endpoints or Vault paths from a list page.
fn valid_utc_cursor(cursor: &str) -> bool {
    let value = cursor.as_bytes();
    if value.len() != 27 {
        return false;
    }
    for (i, b) in value.iter().enumerate() {
        match i {
            4 | 7 => {
                if *b != b'-' {
                    return false;
                }
            }
            10 => {
                if *b != b'T' {
                    return false;
                }
            }
            13 | 16 => {
                if *b != b':' {
                    return false;
                }
            }
            19 => {
                if *b != b'.' {
                    return false;
                }
            }
            26 => {
                if *b != b'Z' {
                    return false;
                }
            }
            _ => {
                if !b.is_ascii_digit() {
                    return false;
                }
            }
        }
    }
    true
}

pub(super) async fn edit_metadata_for_session(
    sessions: &mut BrowserSessionVault,
    cookie: &str,
    csrf: &str,
    origin_verified: bool,
    db: &Client,
    verified_host_tenant: Uuid,
    update: &MetadataUpdate<'_>,
    now: u64,
) -> Option<i64> {
    let actor = sessions.authenticate(
        cookie,
        Some(csrf),
        RequestKind::Mutation,
        origin_verified,
        now,
    )?;
    if !valid_metadata_update(update) {
        return None;
    }
    let row = db
        .query_one(
            "SELECT ipat_platform.edit_managed_device_metadata(\
          $1,$2,$3::uuid,$4::uuid,$5::bigint,$6,$7,$8,$9,$10)",
            &[
                &actor.issuer(),
                &actor.subject(),
                &verified_host_tenant,
                &update.id,
                &update.expected_revision,
                &update.display_name,
                &update.pop_id,
                &update.intended_model,
                &update.management_host,
                &update.management_port,
            ],
        )
        .await
        .ok()?;
    row.get::<_, Option<i64>>(0)
        .filter(|revision| *revision > update.expected_revision)
}

pub(super) async fn archive_metadata_for_session(
    sessions: &mut BrowserSessionVault,
    cookie: &str,
    csrf: &str,
    origin_verified: bool,
    db: &Client,
    verified_host_tenant: Uuid,
    device: Uuid,
    expected_revision: i64,
    now: u64,
) -> bool {
    let Some(actor) = sessions.authenticate(
        cookie,
        Some(csrf),
        RequestKind::Mutation,
        origin_verified,
        now,
    ) else {
        return false;
    };
    if expected_revision < 1 {
        return false;
    }
    db.query_one(
        "SELECT ipat_platform.archive_managed_device_metadata($1,$2,$3::uuid,$4::uuid,$5::bigint)",
        &[
            &actor.issuer(),
            &actor.subject(),
            &verified_host_tenant,
            &device,
            &expected_revision,
        ],
    )
    .await
    .ok()
    .is_some_and(|row| row.get::<_, bool>(0))
}

pub(super) async fn detail_for_session(
    sessions: &mut BrowserSessionVault,
    cookie: &str,
    origin_verified: bool,
    db: &Client,
    verified_host_tenant: Uuid,
    device: Uuid,
    now: u64,
) -> Option<DeviceMetadataDetail> {
    let actor = sessions.authenticate(cookie, None, RequestKind::Read, origin_verified, now)?;
    let row = db
        .query_opt(
            "SELECT id,pop_id,display_name,device_kind,vendor,intended_model,\
          management_transport,management_host,management_port,lifecycle_state,\
          metadata_revision,created_at::text,archived_at::text,\
          archived_site_instance_id,archived_site_name\
          FROM ipat_platform.get_managed_device_metadata($1,$2,$3::uuid,$4::uuid)",
            &[
                &actor.issuer(),
                &actor.subject(),
                &verified_host_tenant,
                &device,
            ],
        )
        .await
        .ok()??;
    let state: String = row.get(9);
    if !matches!(state.as_str(), "SAVED" | "ARCHIVED") {
        return None;
    }
    Some(DeviceMetadataDetail {
        id: row.get(0),
        pop_id: row.get(1),
        display_name: row.get(2),
        device_kind: row.get(3),
        vendor: row.get(4),
        intended_model: row.get(5),
        management_transport: row.get(6),
        management_host: row.get(7),
        management_port: row.get(8),
        lifecycle_state: state,
        metadata_revision: row.get(10),
        created_at: row.get(11),
        archived_at: row.get(12),
        archived_site_instance_id: row.get(13),
        archived_site_name: row.get(14),
    })
}

pub(super) async fn admin_list_for_session(
    sessions: &mut BrowserSessionVault,
    cookie: &str,
    origin_verified: bool,
    db: &Client,
    verified_host_tenant: Uuid,
    include_archived: bool,
    after: Option<(&str, Uuid)>,
    now: u64,
) -> Option<AdminDevicePage> {
    let actor = sessions.authenticate(cookie, None, RequestKind::Read, origin_verified, now)?;
    if after.is_some_and(|(cursor, _)| !valid_utc_cursor(cursor)) {
        return None;
    }
    let after_time = after.map(|(cursor, _)| cursor);
    let after_id = after.map(|(_, id)| id);
    let rows = db
        .query(
            "SELECT id,pop_id,display_name,device_kind,vendor,intended_model,\
          management_transport,lifecycle_state,metadata_revision,\
          to_char(created_at AT TIME ZONE 'UTC',\
            'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"') AS created_cursor\
          FROM ipat_platform.list_managed_devices_admin(\
            $1,$2,$3::uuid,$4::boolean,$5::timestamptz,$6::uuid)",
            &[
                &actor.issuer(),
                &actor.subject(),
                &verified_host_tenant,
                &include_archived,
                &after_time,
                &after_id,
            ],
        )
        .await
        .ok()?;
    if rows.len() > 101 {
        return None;
    }
    let has_more = rows.len() == 101;
    let mut devices = Vec::with_capacity(rows.len().min(100));
    for row in rows.into_iter().take(100) {
        let state: String = row.get(7);
        let revision: i64 = row.get(8);
        let cursor: String = row.get(9);
        if !matches!(state.as_str(), "SAVED" | "ARCHIVED")
            || revision < 1
            || !valid_utc_cursor(&cursor)
        {
            return None;
        }
        devices.push(AdminDeviceRow {
            id: row.get(0),
            pop_id: row.get(1),
            display_name: row.get(2),
            device_kind: row.get(3),
            vendor: row.get(4),
            intended_model: row.get(5),
            management_transport: row.get(6),
            lifecycle_state: state,
            metadata_revision: revision,
            created_cursor: cursor,
        });
    }
    let next_after = if has_more {
        devices.last().map(|d| (d.created_cursor.clone(), d.id))
    } else {
        None
    };
    Some(AdminDevicePage {
        devices,
        next_after,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture<'a>() -> NewDevice<'a> {
        NewDevice {
            request_id: Uuid::parse_str("88888888-8888-4888-8888-888888888881").unwrap(),
            pop_id: "pop-a",
            display_name: "OLT saved",
            device_kind: "olt",
            vendor: "ZTE",
            intended_model: Some("C320"),
            management_transport: "ssh",
            management_host: Some("olt.example.invalid"),
            management_port: Some(22),
            secret_ref: None,
        }
    }
    #[test]
    fn metadata_edit_requires_revision_tenant_site_and_safe_endpoint() {
        let mut input = MetadataUpdate {
            id: Uuid::parse_str("96500000-0000-4000-8000-000000000001").unwrap(),
            expected_revision: 1,
            pop_id: "pop-a",
            display_name: "POP A primary OLT",
            intended_model: Some("C320"),
            management_host: Some("olt.fixture.invalid"),
            management_port: Some(22),
        };
        assert!(valid_metadata_update(&input));
        input.expected_revision = 0;
        assert!(!valid_metadata_update(&input));
        input.expected_revision = 1;
        input.pop_id = "../foreign-site";
        assert!(!valid_metadata_update(&input));
        input.pop_id = "pop-a";
        input.management_host = Some("root:password@olt.fixture.invalid");
        assert!(!valid_metadata_update(&input));
        input.management_host = None;
        assert!(!valid_metadata_update(&input));
        input.management_port = None;
        assert!(valid_metadata_update(&input)); // future inbound CWMP, validated by DB
    }
    #[test]
    fn pagination_cursor_is_strictly_generated_utc_format() {
        assert!(valid_utc_cursor("2026-10-02T13:20:59.123456Z"));
        for candidate in [
            "",
            "2026-10-02 13:20:59.123456Z",
            "2026-10-02T13:20:59.123456+00",
            "../2026-10-02",
        ] {
            assert!(!valid_utc_cursor(candidate), "{candidate}");
        }
    }
    #[test]
    fn registry_never_confuses_metadata_with_adoption() {
        let tenant = Uuid::parse_str("11111111-1111-4111-8111-111111111111").unwrap();
        let d = fixture();
        assert!(valid_new_device(tenant, &d));
        assert_eq!("SAVED", "SAVED"); // only DB default is writable
        let id = id_from_os().unwrap();
        assert_eq!(id.get_version_num(), 4);
    }
    #[test]
    fn secret_ref_must_be_a_same_tenant_vault_reference() {
        let tenant = Uuid::parse_str("11111111-1111-4111-8111-111111111111").unwrap();
        let mut d = fixture();
        d.secret_ref = Some("password=not-allowed");
        assert!(!valid_new_device(tenant, &d));
        d.secret_ref = Some("vault://tenant/22222222-2222-4222-8222-222222222222/devices/olt");
        assert!(!valid_new_device(tenant, &d));
        d.secret_ref = Some("vault://tenant/11111111-1111-4111-8111-111111111111/devices/olt");
        assert!(valid_new_device(tenant, &d));
    }
    #[test]
    fn invalid_endpoint_and_inbound_protocol_cannot_be_saved_as_connected() {
        let tenant = Uuid::parse_str("11111111-1111-4111-8111-111111111111").unwrap();
        let mut d = fixture();
        d.management_host = Some("root:password@olt.example.invalid");
        assert!(!valid_new_device(tenant, &d));
        d.management_host = None;
        d.management_transport = "cwmp";
        assert!(!valid_new_device(tenant, &d));
        d.management_port = None;
        assert!(valid_new_device(tenant, &d));
        d.management_transport = "routeros_api_ssl";
        assert!(!valid_new_device(tenant, &d));
    }
}
