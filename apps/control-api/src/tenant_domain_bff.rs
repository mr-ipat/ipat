//! R9.54 session-bound BFF primitives for the reconciled R9.52/R9.53 domain schema.
//!
//! Transport mounting remains separately gated. Every operation requires an
//! already verified opaque browser session and the sealed PostgreSQL functions
//! re-check current tenant_admin membership.

use identity_core::browser_session::{BrowserSessionVault, RequestKind};
use std::{fs::File, io::Read};
use tokio_postgres::Client;
use uuid::Uuid;

use crate::tenant_domain::canonical_requested_domain;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RequestedDomain {
    pub hostname: String,
    pub verification_name: String,
    pub verification_value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DomainStatus {
    pub hostname: String,
    pub routing_mode: Option<String>,
    pub verification_name: Option<String>,
    pub verification_value: Option<String>,
    pub activation_state: String,
    pub ownership_verified_at: Option<String>,
    pub activated_at: Option<String>,
    pub last_checked_at: Option<String>,
    pub last_error_code: Option<String>,
}

fn valid_routing_mode(value: &str) -> bool {
    matches!(value, "a_record" | "cname" | "nameserver")
}

fn new_verification_value() -> Option<String> {
    let mut bytes = [0u8; 16];
    File::open("/dev/urandom")
        .ok()?
        .read_exact(&mut bytes)
        .ok()?;
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Some(format!("ipat-domain={}", Uuid::from_bytes(bytes)))
}

fn lifecycle_state(
    state: &str,
    ownership_verified: bool,
    routing_ready: bool,
    tls_ready: bool,
) -> Option<&'static str> {
    match state {
        "verified" if ownership_verified && routing_ready && tls_ready => Some("active"),
        "pending" if tls_ready && routing_ready && ownership_verified => Some("tls_ready"),
        "pending" if routing_ready && ownership_verified => Some("routing_ready"),
        "pending" if ownership_verified => Some("ownership_verified"),
        "pending" => Some("pending_dns"),
        "revoked" | "suspended" => Some("disabled"),
        _ => None,
    }
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
        RequestKind::Mutation,
        trusted_same_origin,
        now,
    )?;
    let host = canonical_requested_domain(hostname)?;
    if !valid_routing_mode(routing_mode) {
        return None;
    }
    let verification_value = new_verification_value()?;
    let row = db
        .query_opt(
            "SELECT ipat_platform.request_tenant_custom_domain(
               $1,$2,$3::uuid,$4,$5,$6
             )",
            &[
                &identity.issuer(),
                &identity.subject(),
                &tenant,
                &host,
                &routing_mode,
                &verification_value,
            ],
        )
        .await
        .ok()??;
    let accepted: Option<String> = row.get(0);
    if accepted.as_deref() != Some(host.as_str()) {
        return None;
    }
    Some(RequestedDomain {
        hostname: host.clone(),
        verification_name: format!("_ipat-verify.{host}"),
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
    let identity = vault.authenticate(cookie, None, RequestKind::Read, trusted_same_origin, now)?;
    let rows = db
        .query(
            "SELECT fqdn,state,routing_mode,verification_name,verification_value,
                    ownership_verified_at::text,routing_ready,tls_ready,
                    verified_at::text
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
        let state: String = row.get(1);
        let ownership_verified_at: Option<String> = row.get(5);
        let routing_ready: bool = row.get(6);
        let tls_ready: bool = row.get(7);
        let activation_state = lifecycle_state(
            &state,
            ownership_verified_at.is_some(),
            routing_ready,
            tls_ready,
        )?;
        out.push(DomainStatus {
            hostname: row.get(0),
            routing_mode: row.get(2),
            verification_name: row.get(3),
            verification_value: row.get(4),
            activation_state: activation_state.to_string(),
            ownership_verified_at,
            activated_at: row.get(8),
            last_checked_at: None,
            last_error_code: None,
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
    hostname: &str,
    now: u64,
) -> bool {
    let Some(identity) = vault.authenticate(
        cookie,
        Some(csrf),
        RequestKind::Mutation,
        trusted_same_origin,
        now,
    ) else {
        return false;
    };
    let Some(host) = canonical_requested_domain(hostname) else {
        return false;
    };
    let row = db
        .query_opt(
            "SELECT ipat_platform.revoke_tenant_custom_domain(
               $1,$2,$3::uuid,$4
             )",
            &[&identity.issuer(), &identity.subject(), &tenant, &host],
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
        assert!(!valid_routing_mode("auto"));
        assert!(!valid_routing_mode("http_proxy"));
    }

    #[test]
    fn verification_value_uses_uuid_v4_shape() {
        let value = new_verification_value().expect("Ubuntu target provides /dev/urandom");
        let id = value.strip_prefix("ipat-domain=").unwrap();
        let parsed = Uuid::parse_str(id).unwrap();
        assert_eq!(parsed.get_version_num(), 4);
        assert_eq!(parsed.as_bytes()[8] & 0xc0, 0x80);
    }

    #[test]
    fn lifecycle_projection_is_fail_closed_and_ordered() {
        assert_eq!(
            lifecycle_state("pending", false, false, false),
            Some("pending_dns")
        );
        assert_eq!(
            lifecycle_state("pending", true, false, false),
            Some("ownership_verified")
        );
        assert_eq!(
            lifecycle_state("pending", true, true, false),
            Some("routing_ready")
        );
        assert_eq!(
            lifecycle_state("pending", true, true, true),
            Some("tls_ready")
        );
        assert_eq!(
            lifecycle_state("verified", true, true, true),
            Some("active")
        );
        assert_eq!(lifecycle_state("verified", false, true, true), None);
    }
}
