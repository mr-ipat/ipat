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
        && d.pop_id.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'));
    let name = !d.display_name.is_empty()
        && d.display_name.len() <= 120
        && d.display_name.trim() == d.display_name
        && !d.display_name.chars().any(char::is_control);
    let vendor = !d.vendor.is_empty()
        && d.vendor.len() <= 64
        && d.vendor.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'));
    let model = d.intended_model.is_none_or(|m| !m.is_empty() && m.len() <= 128 && !m.chars().any(char::is_control));
    let kind = matches!(d.device_kind, "olt" | "ont" | "router");
    let protocol = matches!(d.management_transport, "ssh" | "snmp" | "routeros_api_ssl" | "cwmp" | "usp");
    let endpoint = if matches!(d.management_transport, "cwmp" | "usp") {
        d.management_host.is_none() && d.management_port.is_none()
    } else {
        d.management_host.is_some_and(|h| {
            (3..=253).contains(&h.len())
                && h.bytes().next().is_some_and(|b| b.is_ascii_alphanumeric())
                && h.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b':' | b'-'))
        }) && d.management_port.is_some_and(|p| (1..=65535).contains(&p))
    };
    // A vault URI is metadata. Its use by future workers needs independent
    // vault ACL checks; this prevents accidentally storing raw passwords here.
    let prefix = format!("vault://tenant/{tenant}/");
    let reference = d.secret_ref.is_none_or(|r| {
        (55..=255).contains(&r.len()) && r.starts_with(&prefix)
            && r[prefix.len()..].bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'/'))
    });
    pop && name && vendor && model && kind && protocol && endpoint && reference
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
    let identity = vault.authenticate(cookie, Some(csrf), RequestKind::Mutation, same_origin, now)?;
    if !valid_new_device(tenant, d) {
        return None;
    }
    let new_id = id_from_os()?;
    let row = db.query_one(
        "SELECT ipat_platform.register_managed_device(\
          $1,$2,$3::uuid,$4::uuid,$5::uuid,$6,$7,$8,$9,$10,$11,$12,$13,$14)",
        &[&identity.issuer(), &identity.subject(), &tenant, &new_id,
          &d.request_id, &d.pop_id, &d.display_name, &d.device_kind,
          &d.vendor, &d.intended_model, &d.management_transport,
          &d.management_host, &d.management_port, &d.secret_ref],
    ).await.ok()?;
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
    if pop.is_some_and(|p| p.is_empty() || p.len() > 128 || !p.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))) {
        return None;
    }
    let rows = db.query(
        "SELECT id,pop_id,display_name,device_kind,vendor,intended_model,\
          management_transport,lifecycle_state\
          FROM ipat_platform.list_managed_devices($1,$2,$3::uuid,$4)",
        &[&identity.issuer(), &identity.subject(), &tenant, &pop],
    ).await.ok()?;
    if rows.len() > 100 { return None; }
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let state: String = row.get(7);
        if state != "SAVED" { return None; }
        out.push(DeviceRow {
            id: row.get(0), pop_id: row.get(1), display_name: row.get(2),
            device_kind: row.get(3), vendor: row.get(4),
            intended_model: row.get(5), management_transport: row.get(6),
            lifecycle_state: state,
        });
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture<'a>() -> NewDevice<'a> {
        NewDevice {
            request_id: Uuid::parse_str("88888888-8888-4888-8888-888888888881").unwrap(),
            pop_id: "pop-a", display_name: "OLT saved", device_kind: "olt",
            vendor: "ZTE", intended_model: Some("C320"),
            management_transport: "ssh", management_host: Some("olt.example.invalid"),
            management_port: Some(22), secret_ref: None,
        }
    }
    #[test]
    fn registry_never_confuses_metadata_with_adoption() {
        let tenant=Uuid::parse_str("11111111-1111-4111-8111-111111111111").unwrap();
        let d=fixture();
        assert!(valid_new_device(tenant,&d));
        assert_eq!("SAVED", "SAVED"); // only DB default is writable
        let id=id_from_os().unwrap();
        assert_eq!(id.get_version_num(),4);
    }
    #[test]
    fn secret_ref_must_be_a_same_tenant_vault_reference() {
        let tenant=Uuid::parse_str("11111111-1111-4111-8111-111111111111").unwrap();
        let mut d=fixture();
        d.secret_ref=Some("password=not-allowed");
        assert!(!valid_new_device(tenant,&d));
        d.secret_ref=Some("vault://tenant/22222222-2222-4222-8222-222222222222/devices/olt");
        assert!(!valid_new_device(tenant,&d));
        d.secret_ref=Some("vault://tenant/11111111-1111-4111-8111-111111111111/devices/olt");
        assert!(valid_new_device(tenant,&d));
    }
    #[test]
    fn invalid_endpoint_and_inbound_protocol_cannot_be_saved_as_connected() {
        let tenant=Uuid::parse_str("11111111-1111-4111-8111-111111111111").unwrap();
        let mut d=fixture();
        d.management_host=Some("root:password@olt.example.invalid");
        assert!(!valid_new_device(tenant,&d));
        d.management_host=None;
        d.management_transport="cwmp";
        assert!(!valid_new_device(tenant,&d));
        d.management_port=None;
        assert!(valid_new_device(tenant,&d));
        d.management_transport="routeros_api_ssl";
        assert!(!valid_new_device(tenant,&d));
    }
}
