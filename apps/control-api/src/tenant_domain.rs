//! Production-path tenant hostname resolution.
//! A verified Host selects routing context only. Authorization still requires
//! independently verified identity + DB membership + RBAC/ABAC on every API.
use axum::{
    extract::State,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    routing::get,
    Json, Router,
};
use serde_json::{json, Value};
use std::{
    fs::OpenOptions,
    io::Read,
    os::unix::{fs::MetadataExt, fs::OpenOptionsExt},
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};
use tokio_postgres::{config::Host, Config, NoTls};

const EXPECTED_DOMAIN_READER: &str = "ipat_domain_reader";
const MAX_CONNFILE_BYTES: u64 = 4096;

pub(crate) struct Store {
    db: Config,
}

fn response_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    headers
}
fn canonical_hostname(headers: &HeaderMap) -> Result<String, StatusCode> {
    if headers.get_all(header::HOST).iter().count() != 1 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let raw = headers
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
        .ok_or(StatusCode::BAD_REQUEST)?;
    if raw.is_empty()
        || raw.len() > 259
        || !raw.is_ascii()
        || raw.chars().any(|c| matches!(c, '/' | '\\' | '@' | ' '))
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let host = match raw.rsplit_once(':') {
        Some((name, port))
            if !name.contains(':')
                && !port.is_empty()
                && port.bytes().all(|b| b.is_ascii_digit()) =>
        {
            name
        }
        Some(_) if raw.contains(':') => return Err(StatusCode::BAD_REQUEST),
        _ => raw,
    };
    let host = host.to_ascii_lowercase();
    if host.len() < 3
        || host.len() > 253
        || host.starts_with('.')
        || host.ends_with('.')
        || host.contains("..")
        || !host.contains('.')
        || !host.bytes().any(|b| b.is_ascii_lowercase())
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    for label in host.split('.') {
        if label.is_empty()
            || label.len() > 63
            || label.starts_with('-')
            || label.ends_with('-')
            || !label
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        {
            return Err(StatusCode::BAD_REQUEST);
        }
    }
    Ok(host)
}

async fn context(
    State(store): State<Arc<Store>>,
    headers: HeaderMap,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let host = match canonical_hostname(&headers) {
        Ok(host) => host,
        Err(status) => {
            return (
                status,
                response_headers(),
                Json(json!({"tenant_context":false})),
            )
        }
    };
    let Ok((client, connection)) = store.db.connect(NoTls).await else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            response_headers(),
            Json(json!({"tenant_context":false})),
        );
    };
    let task = tokio::spawn(async move {
        let _ = connection.await;
    });
    let row = client
        .query_opt(
            "SELECT tenant_slug,fqdn FROM ipat_platform.resolve_verified_tenant_domain($1)",
            &[&host],
        )
        .await;
    drop(client);
    task.abort();

    let Ok(Some(row)) = row else {
        return match row {
            Ok(None) => (
                StatusCode::NOT_FOUND,
                response_headers(),
                Json(json!({"tenant_context":false})),
            ),
            Err(_) => (
                StatusCode::SERVICE_UNAVAILABLE,
                response_headers(),
                Json(json!({"tenant_context":false})),
            ),
            _ => unreachable!(),
        };
    };
    let tenant_slug: String = row.get(0);
    let fqdn: String = row.get(1);
    if fqdn != host {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            response_headers(),
            Json(json!({"tenant_context":false})),
        );
    }
    (
        StatusCode::OK,
        response_headers(),
        Json(json!({
            "tenant_context":true,
            "tenant_slug":tenant_slug,
            "hostname":fqdn,
            "authentication_required":true,
            "business_access_enabled":false
        })),
    )
}

pub(crate) fn router(store: Arc<Store>) -> Router {
    Router::new()
        .route("/v1/tenant-context", get(context))
        .with_state(store)
}
fn read_private_file(path: &Path) -> Result<String, &'static str> {
    let parent = path.parent().ok_or("missing private config directory")?;
    if !path.is_absolute() {
        return Err("domain DB config path must be absolute");
    }
    for (point, mode) in [(parent, 0o700), (path, 0o600)] {
        let metadata = std::fs::symlink_metadata(point).map_err(|_| "missing domain DB config")?;
        if metadata.file_type().is_symlink()
            || metadata.uid() != unsafe { libc::geteuid() }
            || metadata.mode() & 0o777 != mode
        {
            return Err("unsafe domain DB config ownership/mode");
        }
    }
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .map_err(|_| "invalid domain DB config")?;
    let metadata = file.metadata().map_err(|_| "invalid domain DB config")?;
    if !metadata.is_file()
        || metadata.nlink() != 1
        || metadata.len() == 0
        || metadata.len() > MAX_CONNFILE_BYTES
    {
        return Err("unsafe domain DB config file");
    }
    let mut value = String::new();
    file.take(MAX_CONNFILE_BYTES + 1)
        .read_to_string(&mut value)
        .map_err(|_| "invalid domain DB config encoding")?;
    if value.len() as u64 > MAX_CONNFILE_BYTES {
        return Err("oversized domain DB config");
    }
    Ok(value)
}
fn valid_db_config(config: &Config) -> bool {
    config.get_user() == Some(EXPECTED_DOMAIN_READER)
        && config.get_dbname().is_some()
        && config.get_hosts().len() == 1
        && config.get_hostaddrs().is_empty()
        && config.get_options().is_none()
        && matches!(config.get_hosts()[0], Host::Unix(ref path) if path.is_absolute())
}

pub(crate) fn from_environment() -> Result<Arc<Store>, &'static str> {
    if std::env::var("IPAT_TENANT_DOMAIN_ROUTING").as_deref() != Ok("YES")
        || unsafe { libc::geteuid() } == 0
    {
        return Err("tenant domain routing requires explicit nonroot opt-in");
    }
    let path = PathBuf::from(
        std::env::var("IPAT_TENANT_DOMAIN_DB_CONNINFO_FILE")
            .map_err(|_| "missing tenant domain DB conninfo file")?,
    );
    let value = read_private_file(&path)?;
    let db = Config::from_str(value.trim()).map_err(|_| "invalid tenant domain DB config")?;
    if !valid_db_config(&db) {
        return Err("tenant domain reader must use the dedicated local Unix-socket role");
    }
    Ok(Arc::new(Store { db }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::Request,
    };
    use tower::ServiceExt;

    fn host(value: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(header::HOST, HeaderValue::from_str(value).unwrap());
        headers
    }

    #[test]
    fn canonicalizes_dns_case_and_numeric_port_only() {
        assert_eq!(
            canonical_hostname(&host("IPAT.FADLY.ID:443")).unwrap(),
            "ipat.fadly.id"
        );
        assert_eq!(
            canonical_hostname(&host("kangnet.ipat.id")).unwrap(),
            "kangnet.ipat.id"
        );
    }
    #[test]
    fn rejects_ip_literals_malformed_labels_and_injection() {
        for value in [
            "127.0.0.1",
            "[::1]",
            "bad",
            "-bad.example",
            "bad-.example",
            "bad..example",
            "bad.example.",
            "bad.example:abc",
            "bad.example/tenant",
            "bad.example tenant",
            "user@bad.example",
        ] {
            assert!(canonical_hostname(&host(value)).is_err(), "{value}");
        }
    }

    #[test]
    fn duplicate_host_is_rejected() {
        let mut headers = HeaderMap::new();
        headers.append(header::HOST, HeaderValue::from_static("ipat.fadly.id"));
        headers.append(header::HOST, HeaderValue::from_static("other.example"));
        assert_eq!(canonical_hostname(&headers), Err(StatusCode::BAD_REQUEST));
    }

    #[test]
    fn direct_tcp_or_unrestricted_database_config_is_not_allowed() {
        for dsn in [
            "host=127.0.0.1 user=ipat_domain_reader dbname=ipat",
            "host=/var/run/postgresql user=postgres dbname=ipat",
            "host=/var/run/postgresql user=ipat_domain_reader",
            "host=/var/run/postgresql user=ipat_domain_reader dbname=ipat options='-c role=postgres'",
        ] {
            let config = Config::from_str(dsn).unwrap();
            assert!(!valid_db_config(&config), "{dsn}");
        }
        let config =
            Config::from_str("host=/var/run/postgresql user=ipat_domain_reader dbname=ipat")
                .unwrap();
        assert!(valid_db_config(&config));
    }

    #[tokio::test]
    async fn r952_real_disposable_postgres_host_to_tenant_context_is_fail_closed() {
        if std::env::var("IPAT_PG_EPHEMERAL_TEST").as_deref() != Ok("1") {
            return;
        }
        assert_eq!(std::env::var("PGHOST").as_deref(), Ok("127.0.0.1"));
        let db = Config::from_str(
            "host=127.0.0.1 port=5432 user=ipat_domain_reader password=local_ci_synthetic_only dbname=ipat_synthetic",
        )
        .unwrap();
        let app = router(Arc::new(Store { db }));

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/v1/tenant-context")
                    .header("Host", "ipat.fadly.id")
                    .header("X-Forwarded-Host", "forged.other.invalid")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 4096).await.unwrap();
        let value: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(value["tenant_context"], true);
        assert_eq!(value["tenant_slug"], "nengnet");
        assert_eq!(value["hostname"], "ipat.fadly.id");
        assert_eq!(value["authentication_required"], true);
        assert_eq!(value["business_access_enabled"], false);
        assert!(value.get("tenant_id").is_none());

        let unknown = app
            .oneshot(
                Request::builder()
                    .uri("/v1/tenant-context")
                    .header("Host", "unknown.ipat.id")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(unknown.status(), StatusCode::NOT_FOUND);
    }
}
