//! R9.19 BFF primitives for custom-domain enrollment.
//!
//! These functions intentionally remain transport-agnostic. A caller must
//! provide an already verified opaque browser session, trusted same-origin
//! decision, exact tenant UUID from server-side routing context, and a
//! restricted PostgreSQL client. Tenant authority is re-checked in PostgreSQL
//! for every operation.

use identity_core::browser_session::{BrowserSessionVault, RequestKind};
use std::{fs::File, io::Read};
use tokio_postgres::Client;
use uuid::Uuid;

use crate::tenant_domain::{canonical_dns_name, valid_custom_domain};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RequestedDomain {
    pub id: Uuid,
    pub hostname: String,
    pub verification_name: String,
    pub verification_value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DomainStatus {
    pub id: Uuid,
    pub hostname: String,
    pub routing_mode: Option<String>,
    pub verification_name: Option<String>,
    pub verification_value: Option<String>,
    pub activation_state: String,
    pub ownership_verified_at: Option<String>,
    pub routing_ready_at: Option<String>,
    pub tls_ready_at: Option<String>,
    pub activated_at: Option<String>,
    pub last_checked_at: Option<String>,
    pub last_error_code: Option<String>,
}

fn valid_routing_mode(value: &str) -> bool {
    matches!(value, "a_record" | "cname" | "nameserver")
}

fn new_request_id() -> Option<Uuid> {
    let mut bytes = [0u8; 16];
    File::open("/dev/urandom").ok()?.read_exact(&mut bytes).ok()?;
    // RFC 4122/9562-compatible UUIDv4 version and variant bits.
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Some(Uuid::from_bytes(bytes))
}

pub(super) async fn request_custom_domain_for_session(
    vault: &mut BrowserSessionVault,
    cookie: &str,
    csrf: &str,
    trusted_same_origin: bool,
    db: &Client,
    tenant: Uuid,
    hostname: &str,
    routing_mode: &str,
    now: u64,
) -> Option<RequestedDomain> {
    let identity = vault.authenticate(
        cookie,
        Some(csrf),
        RequestKind::Write,
        trusted_same_origin,
        now,
    )?;
    let host = canonical_dns_name(hostname)?;
    if !valid_custom_domain(&host) || !valid_routing_mode(routing_mode) {
        return None;
    }

    let id = new_request_id()?;
    let verification_value = format!("ipat-domain={id}");
    let row = db
        .query_opt(
            "SELECT ipat_platform.request_tenant_custom_domain(
               $1,$2,$3::uuid,$4::uuid,$5,$6,$7
             )",
            &[
                &identity.issuer(),
                &identity.subject(),
                &tenant,
                &id,
                &host.as_str(),
                &routing_mode,
                &verification_value,
            ],
        )
        .await
        .ok()??;
    let accepted: Option<Uuid> = row.get(0);
    let accepted = accepted?;
    if accepted != id {
        return None;
    }

    Some(RequestedDomain {
        id,
        hostname: host.as_str().to_string(),
        verification_name: format!("_ipat-verify.{}", host.as_str()),
        verification_value,
    })
}

pub(super) async fn list_custom_domains_for_session(
    vault: &mut BrowserSessionVault,
    cookie: &str,
    trusted_same_origin: bool,
    db: &Client,
    tenant: Uuid,
    now: u64,
) -> Option<Vec<DomainStatus>> {
    let identity = vault.authenticate(
        cookie,
        None,
        RequestKind::Read,
        trusted_same_origin,
        now,
    )?;
    let rows = db
        .query(
            "SELECT id,hostname,routing_mode,verification_name,verification_value,
                    activation_state,
                    ownership_verified_at::text,routing_ready_at::text,
                    tls_ready_at::text,activated_at::text,last_checked_at::text,
                    last_error_code
             FROM ipat_platform.list_tenant_domains_for_member($1,$2,$3::uuid)",
            &[&identity.issuer(), &identity.subject(), &tenant],
        )
        .await
        .ok()?;
    if rows.len() > 100 {
        return None;
    }

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let activation_state: String = row.get(5);
        if !matches!(
            activation_state.as_str(),
            "pending_dns"
                | "ownership_verified"
                | "routing_ready"
                | "tls_ready"
                | "active"
                | "disabled"
        ) {
            return None;
        }
        out.push(DomainStatus {
            id: row.get(0),
            hostname: row.get(1),
            routing_mode: row.get(2),
            verification_name: row.get(3),
            verification_value: row.get(4),
            activation_state,
            ownership_verified_at: row.get(6),
            routing_ready_at: row.get(7),
            tls_ready_at: row.get(8),
            activated_at: row.get(9),
            last_checked_at: row.get(10),
            last_error_code: row.get(11),
        });
    }
    Some(out)
}

pub(super) async fn disable_custom_domain_for_session(
    vault: &mut BrowserSessionVault,
    cookie: &str,
    csrf: &str,
    trusted_same_origin: bool,
    db: &Client,
    tenant: Uuid,
    domain_id: Uuid,
    now: u64,
) -> bool {
    let Some(identity) = vault.authenticate(
        cookie,
        Some(csrf),
        RequestKind::Write,
        trusted_same_origin,
        now,
    ) else {
        return false;
    };
    let row = db
        .query_opt(
            "SELECT ipat_platform.disable_tenant_custom_domain(
               $1,$2,$3::uuid,$4::uuid
             )",
            &[
                &identity.issuer(),
                &identity.subject(),
                &tenant,
                &domain_id,
            ],
        )
        .await;
    matches!(row, Ok(Some(row)) if row.get::<_, bool>(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routing_mode_is_not_customer_arbitrary_text() {
        assert!(valid_routing_mode("a_record"));
        assert!(valid_routing_mode("cname"));
        assert!(valid_routing_mode("nameserver"));
        assert!(!valid_routing_mode("http_proxy"));
        assert!(!valid_routing_mode("A"));
    }

    #[test]
    fn server_request_id_is_uuid_v4_with_rfc_variant() {
        let id = new_request_id().expect("Ubuntu target must provide /dev/urandom");
        assert_eq!(id.get_version_num(), 4);
        assert_eq!(id.as_bytes()[8] & 0xc0, 0x80);
    }
}
