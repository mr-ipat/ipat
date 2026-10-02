//! R9.67 production-path durable Host-bound tenant browser session adapter.
//!
//! This is deliberately transport-neutral and UNMOUNTED until a real public
//! HTTPS callback performs confidential OIDC authorization-code exchange and
//! `PinnedIssuer::verify_offline_browser_pair` validates the signed ID/access
//! pair, fresh MFA, nonce and at_hash. Session/CSRF plaintext never enters DB.
//! Host selects tenant; there is no client-supplied tenant or role parameter.
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use identity_core::oidc_id_token::VerifiedBrowserIdentity;
use sha2::{Digest, Sha256};
use std::{fs::File, io::Read};
use tokio_postgres::Client;
use uuid::Uuid;

use crate::tenant_domain::{canonical_host, CanonicalHost};
use axum::http::HeaderValue;

const SECRET_BYTES: usize = 32;
const ABSOLUTE_MAX_SECONDS: u64 = 15 * 60;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DurableSessionContext {
    pub tenant_id: Uuid,
    pub domain_id: Uuid,
    pub issuer: String,
    pub subject: String,
}

pub(super) struct IssuedDurableSession {
    cookie: String,
    csrf: String,
    expires_at: u64,
    max_age_secs: u64,
}
impl IssuedDurableSession {
    pub fn cookie_secret(&self) -> &str {
        &self.cookie
    }
    pub fn csrf_secret(&self) -> &str {
        &self.csrf
    }
    pub fn expires_at(&self) -> u64 {
        self.expires_at
    }
    pub fn secure_cookie_header(&self) -> String {
        format!(
            "__Host-ipat_session={}; Secure; HttpOnly; SameSite=Strict; Path=/; Max-Age={}",
            self.cookie, self.max_age_secs
        )
    }
}

fn random_secret() -> Option<String> {
    let mut raw = [0u8; SECRET_BYTES];
    File::open("/dev/urandom").ok()?.read_exact(&mut raw).ok()?;
    Some(URL_SAFE_NO_PAD.encode(raw))
}
fn valid_secret(secret: &str) -> bool {
    secret.len() == 43
        && secret
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
}
fn sha256_hex(secret: &str) -> Option<String> {
    if !valid_secret(secret) {
        return None;
    }
    {
        let digest = Sha256::digest(secret.as_bytes());
        let mut out = String::with_capacity(64);
        for byte in digest {
            use std::fmt::Write as _;
            write!(&mut out, "{byte:02x}").ok()?;
        }
        Some(out)
    }
}
fn canonical_request_host(raw: &str) -> Option<CanonicalHost> {
    let value = HeaderValue::from_str(raw).ok()?;
    canonical_host(&value)
}
fn new_uuid_v4() -> Option<Uuid> {
    let mut raw = [0u8; 16];
    File::open("/dev/urandom").ok()?.read_exact(&mut raw).ok()?;
    raw[6] = (raw[6] & 0x0f) | 0x40;
    raw[8] = (raw[8] & 0x3f) | 0x80;
    Some(Uuid::from_bytes(raw))
}

/// Called ONLY after cryptographic browser identity+fresh MFA validation.
/// The DB re-checks active Host/domain and any current tenant membership.
pub(super) async fn issue_after_verified_identity(
    db: &Client,
    identity: &VerifiedBrowserIdentity,
    request_host: &str,
    now: u64,
) -> Option<IssuedDurableSession> {
    let host = canonical_request_host(request_host)?;
    let binding=db.query_opt(
        "SELECT domain_id,tenant_id FROM ipat_platform.resolve_active_tenant_domain_binding($1)",
        &[&host.as_str()]).await.ok()??;
    let domain_id: Uuid = binding.get(0);
    let tenant_id: Uuid = binding.get(1);
    let cookie = random_secret()?;
    let csrf = random_secret()?;
    if cookie == csrf {
        return None;
    }
    let cookie_hash = sha256_hex(&cookie)?;
    let csrf_hash = sha256_hex(&csrf)?;
    let session_id = new_uuid_v4()?;
    let expires_at = identity
        .expires_at()
        .min(now.saturating_add(ABSOLUTE_MAX_SECONDS));
    if expires_at <= now {
        return None;
    }
    let expires_i64 = i64::try_from(expires_at).ok()?;
    let row=db.query_one(
        "SELECT ipat_platform.issue_tenant_browser_session($1,$2,$3::uuid,$4::uuid,$5::uuid,$6,$7,to_timestamp($8))",
        &[&identity.issuer(),&identity.subject(),&tenant_id,&domain_id,&session_id,&cookie_hash,&csrf_hash,&expires_i64]
    ).await.ok()?;
    let accepted: Option<Uuid> = row.get(0);
    if accepted != Some(session_id) {
        return None;
    }
    Some(IssuedDurableSession {
        cookie,
        csrf,
        expires_at,
        max_age_secs: expires_at.saturating_sub(now),
    })
}

/// Host is the only tenant selector. A valid cookie replayed on another tenant
/// hostname fails in PostgreSQL. Mutation also requires exact CSRF secret.
pub(super) async fn authenticate(
    db: &Client,
    cookie: &str,
    csrf: Option<&str>,
    mutation: bool,
    request_host: &str,
) -> Option<DurableSessionContext> {
    let host = canonical_request_host(request_host)?;
    let cookie_hash = sha256_hex(cookie)?;
    let csrf_hash = match (mutation, csrf) {
        (true, Some(v)) => Some(sha256_hex(v)?),
        (true, None) => return None,
        (false, _) => None,
    };
    let row=db.query_opt(
        "SELECT tenant_id,domain_id,issuer,subject FROM ipat_platform.authenticate_tenant_browser_session($1,$2,$3,$4)",
        &[&cookie_hash,&host.as_str(),&csrf_hash,&mutation]
    ).await.ok()??;
    Some(DurableSessionContext {
        tenant_id: row.get(0),
        domain_id: row.get(1),
        issuer: row.get(2),
        subject: row.get(3),
    })
}

pub(super) async fn revoke(db: &Client, cookie: &str, request_host: &str) -> bool {
    let Some(host) = canonical_request_host(request_host) else {
        return false;
    };
    let Some(cookie_hash) = sha256_hex(cookie) else {
        return false;
    };
    db.query_one(
        "SELECT ipat_platform.revoke_tenant_browser_session($1,$2)",
        &[&cookie_hash, &host.as_str()],
    )
    .await
    .ok()
    .is_some_and(|row| row.get::<_, bool>(0))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn secret_shape_and_hash_are_strict() {
        let s = random_secret().unwrap();
        assert!(valid_secret(&s));
        assert_eq!(s.len(), 43);
        let h = sha256_hex(&s).unwrap();
        assert_eq!(h.len(), 64);
        assert!(h
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()));
        assert!(sha256_hex("not a session").is_none());
    }
    #[test]
    fn host_parser_is_strict_and_canonical() {
        assert_eq!(
            canonical_request_host("Tenant.Example.COM:443")
                .unwrap()
                .as_str(),
            "tenant.example.com"
        );
        for raw in [
            "https://tenant.example.com",
            "tenant.example.com/path",
            "[::1]:443",
            "tenant..example.com",
            "evil example.com",
        ] {
            assert!(canonical_request_host(raw).is_none(), "{raw}");
        }
    }
}
