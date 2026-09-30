//! Production-shaped tenant-domain routing.
//!
//! A verified hostname may select public tenant bootstrap context, but NEVER
//! grants user membership, role, POP scope, device access, or secrets.

use axum::{
    extract::State,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    routing::get,
    Json, Router,
};
use serde_json::{json, Value};
use std::{path::PathBuf, sync::Arc};
use tokio_postgres::{Config, NoTls};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalHost(String);

impl CanonicalHost {
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

pub(crate) fn canonical_host(value: &HeaderValue) -> Option<CanonicalHost> {
    let raw = value.to_str().ok()?.trim();
    if raw.is_empty() || raw.len() > 255 || raw.contains(['/', '\\', ' ', '\t', '\r', '\n']) {
        return None;
    }
    if raw.starts_with('[') {
        return None; // tenant selectors are DNS names, not IP literals
    }
    let host = match raw.rsplit_once(':') {
        Some((left, port))
            if !left.is_empty() && !port.is_empty() && port.bytes().all(|b| b.is_ascii_digit()) =>
        {
            left
        }
        Some(_) => return None,
        None => raw,
    };
    let host = host.strip_suffix('.').unwrap_or(host);
    if host.len() < 3
        || host.len() > 253
        || host.starts_with('.')
        || host.ends_with('.')
        || host.contains("..")
    {
        return None;
    }
    let lower = host.to_ascii_lowercase();
    if !lower
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'.')
    {
        return None;
    }
    for label in lower.split('.') {
        if label.is_empty() || label.len() > 63 || label.starts_with('-') || label.ends_with('-') {
            return None;
        }
    }
    Some(CanonicalHost(lower))
}

pub(crate) struct DomainStore {
    db: Config,
}

fn safe_name(value: &str, max: usize) -> bool {
    !value.is_empty()
        && value.len() <= max
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
}

/// Initial runtime adapter uses a local Unix socket with peer/local credential
/// isolation. It intentionally refuses TCP without a reviewed TLS DB adapter.
pub(crate) fn from_environment() -> Result<Arc<DomainStore>, String> {
    let socket =
        std::env::var("IPAT_TENANT_DOMAIN_DB_SOCKET").map_err(|_| "missing domain DB socket")?;
    let database =
        std::env::var("IPAT_TENANT_DOMAIN_DB_NAME").map_err(|_| "missing domain DB name")?;
    let user = std::env::var("IPAT_TENANT_DOMAIN_DB_USER").map_err(|_| "missing domain DB user")?;
    if user != "ipat_domain_reader_login" || !safe_name(&database, 63) {
        return Err("unexpected domain DB identity".into());
    }
    let path = PathBuf::from(&socket);
    if !path.is_absolute() || socket.len() > 200 || socket.contains("..") {
        return Err("domain DB socket must be absolute and canonical".into());
    }
    let mut db = Config::new();
    db.host_path(path);
    db.user(&user);
    db.dbname(&database);
    Ok(Arc::new(DomainStore { db }))
}

fn headers() -> HeaderMap {
    let mut h = HeaderMap::new();
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    h.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    h
}

async fn bootstrap(
    State(store): State<Arc<DomainStore>>,
    request_headers: HeaderMap,
) -> (StatusCode, HeaderMap, Json<Value>) {
    if request_headers.get_all(header::HOST).iter().count() != 1 {
        return (
            StatusCode::BAD_REQUEST,
            headers(),
            Json(json!({"tenant":null})),
        );
    }
    let Some(host) = request_headers.get(header::HOST).and_then(canonical_host) else {
        return (
            StatusCode::BAD_REQUEST,
            headers(),
            Json(json!({"tenant":null})),
        );
    };

    let Ok((client, connection)) = store.db.connect(NoTls).await else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            headers(),
            Json(json!({"tenant":null})),
        );
    };
    let task = tokio::spawn(async move {
        let _ = connection.await;
    });
    let row = client
        .query_opt(
            "SELECT tenant_id::text,tenant_slug,hostname,domain_type
             FROM ipat_platform.resolve_active_tenant_domain($1)",
            &[&host.as_str()],
        )
        .await;
    drop(client);
    task.abort();

    let Ok(Some(row)) = row else {
        return match row {
            Ok(None) => (
                StatusCode::NOT_FOUND,
                headers(),
                Json(json!({"tenant":null})),
            ),
            Err(_) => (
                StatusCode::SERVICE_UNAVAILABLE,
                headers(),
                Json(json!({"tenant":null})),
            ),
            _ => unreachable!(),
        };
    };
    let tenant_id: String = row.get(0);
    let tenant_slug: String = row.get(1);
    let hostname: String = row.get(2);
    let domain_type: String = row.get(3);
    if hostname != host.as_str() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            headers(),
            Json(json!({"tenant":null})),
        );
    }
    (
        StatusCode::OK,
        headers(),
        Json(json!({
            "tenant":{
                "id":tenant_id,
                "slug":tenant_slug,
                "hostname":hostname,
                "domain_type":domain_type
            },
            "authorization_granted":false,
            "membership_required":true
        })),
    )
}

pub(crate) fn router(store: Arc<DomainStore>) -> Router {
    Router::new()
        .route("/v1/bootstrap/tenant", get(bootstrap))
        .with_state(store)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalizes_dns_host_without_conferring_authority() {
        let h = HeaderValue::from_static("IPAT.FADLY.ID:443");
        assert_eq!(canonical_host(&h).unwrap().as_str(), "ipat.fadly.id");
    }

    #[test]
    fn accepts_managed_customer_subdomain() {
        let h = HeaderValue::from_static("kangnet.ipat.id");
        assert_eq!(canonical_host(&h).unwrap().as_str(), "kangnet.ipat.id");
    }

    #[test]
    fn rejects_ambiguous_or_non_dns_authorities() {
        for raw in [
            "ipat.fadly.id:notaport",
            "ipat..fadly.id",
            "-bad.ipat.id",
            "bad-.ipat.id",
            "[::1]:443",
            "ipat.fadly.id/path",
            "ipat.fadly.id evil.example",
        ] {
            let value = HeaderValue::from_bytes(raw.as_bytes()).unwrap();
            assert!(canonical_host(&value).is_none(), "{raw}");
        }
    }

    #[test]
    fn runtime_db_adapter_refuses_unexpected_identity_and_relative_socket() {
        std::env::set_var("IPAT_TENANT_DOMAIN_DB_SOCKET", "relative/socket");
        std::env::set_var("IPAT_TENANT_DOMAIN_DB_NAME", "ipat");
        std::env::set_var("IPAT_TENANT_DOMAIN_DB_USER", "postgres");
        assert!(from_environment().is_err());
        std::env::remove_var("IPAT_TENANT_DOMAIN_DB_SOCKET");
        std::env::remove_var("IPAT_TENANT_DOMAIN_DB_NAME");
        std::env::remove_var("IPAT_TENANT_DOMAIN_DB_USER");
    }
}
