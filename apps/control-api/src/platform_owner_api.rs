//! Dedicated source-only Platform Owner BFF. NEVER mount in tenant/device service.
//! This router accepts only a previously issued real-MFA platform session;
//! it deliberately has no unverified login or role-claim endpoint.
use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{net::Ipv4Addr, path::PathBuf, sync::Arc};
use tokio_postgres::{Client, Config, NoTls};
use uuid::Uuid;

use crate::tenant_domain::canonical_host;

const CSRF_HEADER: HeaderName = HeaderName::from_static("x-ipat-platform-csrf");
const HTML: &str = include_str!("../../../web/console/platform/dashboard.html");
const JS: &str = include_str!("../../../web/console/platform/app.js");
const CSS: &str = include_str!("../../../web/console/platform/style.css");

#[derive(Clone)]
struct PlatformState {
    db: Arc<Client>,
    expected_host: String,
}
fn strict_host(host: &str) -> bool {
    host.len() <= 253
        && host.split('.').count() >= 2
        && host.split('.').all(|part| {
            !part.is_empty()
                && part.len() <= 63
                && !part.starts_with('-')
                && !part.ends_with('-')
                && part
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        })
        && host
            .rsplit('.')
            .next()
            .is_some_and(|t| t.len() >= 2 && t.bytes().all(|b| b.is_ascii_lowercase()))
}

// Explicit temporary IPv4 is allowed as an exact platform Host only. A public
// address is NOT an HTTPS identity: a separately reviewed trusted certificate
// with an iPAddress SAN, recovery and external reachability are still required.
fn eligible_temporary_ipv4(host: &str) -> bool {
    let Ok(ip) = host.parse::<Ipv4Addr>() else {
        return false;
    };
    let [a, b, c, _] = ip.octets();
    !(a == 0
        || a == 10
        || a == 127
        || a >= 224
        || (a == 100 && (64..=127).contains(&b))
        || (a == 169 && b == 254)
        || (a == 172 && (16..=31).contains(&b))
        || (a == 192 && (b == 168 || (b == 0 && c == 2)))
        || (a == 198 && (b == 18 || b == 19 || (b == 51 && c == 100)))
        || (a == 203 && b == 0 && c == 113)
        || (a == 255))
}
pub(super) fn allowed_platform_host(host: &str, ip_mode: bool, ip_san_reviewed: bool) -> bool {
    if host.parse::<Ipv4Addr>().is_ok() {
        ip_mode && ip_san_reviewed && eligible_temporary_ipv4(host)
    } else {
        strict_host(host)
    }
}
pub(super) fn platform_host_shape(host: &str) -> bool {
    strict_host(host) || eligible_temporary_ipv4(host)
}
pub(super) async fn connect_from_environment() -> Result<(Arc<Client>, String), String> {
    if unsafe { libc::geteuid() } == 0
        || std::env::var("IPAT_R981_PLATFORM_SERVICE").as_deref() != Ok("YES")
        || std::env::var("IPAT_R981_REVIEWED_TRUSTED_HTTPS_EDGE").as_deref() != Ok("YES")
    {
        return Err(
            "dedicated Platform Owner mode requires nonroot and reviewed HTTPS edge".into(),
        );
    }
    let host =
        std::env::var("IPAT_R981_EXACT_PLATFORM_HOST").map_err(|_| "platform Host missing")?;
    let ip_mode = std::env::var("IPAT_R982_TEMPORARY_IPV4_MODE").as_deref() == Ok("YES");
    let ip_san_reviewed = std::env::var("IPAT_R982_IP_SAN_TLS_REVIEWED").as_deref() == Ok("YES");
    if !allowed_platform_host(&host, ip_mode, ip_san_reviewed) {
        return Err("unapproved exact platform hostname or temporary public IPv4".into());
    }
    // IP mode still also requires the existing independent trusted-edge gate
    // plus a separately provisioned preapproved platform_console_hosts record.
    // A configuration value alone is NEVER proof of an actual valid certificate.
    let user = std::env::var("IPAT_R981_PLATFORM_DB_USER").map_err(|_| "DB user missing")?;
    if user != "ipat_platform_session_api_login" {
        return Err("wrong dedicated DB user".into());
    }
    let name = std::env::var("IPAT_R981_PLATFORM_DB_NAME").map_err(|_| "DB name missing")?;
    if name.is_empty()
        || name.len() > 63
        || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
    {
        return Err("invalid dedicated DB name".into());
    }
    let path = std::env::var("IPAT_R981_PLATFORM_DB_SOCKET").map_err(|_| "DB socket missing")?;
    if !PathBuf::from(&path).is_absolute() || path.len() > 200 || path.contains("..") {
        return Err("invalid local DB socket".into());
    }
    let mut config = Config::new();
    config.host_path(PathBuf::from(path));
    config.user(&user);
    config.dbname(&name);
    let (client, connection) = config
        .connect(NoTls)
        .await
        .map_err(|_| "dedicated Platform Owner database unavailable")?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    Ok((Arc::new(client), host))
}
fn headers(content: &'static str) -> HeaderMap {
    let mut h = HeaderMap::new();
    h.insert(header::CONTENT_TYPE, HeaderValue::from_static(content));
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    h.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    h.insert("x-frame-options", HeaderValue::from_static("DENY"));
    h.insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    h
}
fn html_headers() -> HeaderMap {
    let mut h = headers("text/html; charset=utf-8");
    h.insert(
        "content-security-policy",
        HeaderValue::from_static(
            "default-src 'none'; base-uri 'none'; frame-ancestors 'none'; object-src 'none'; form-action 'self'; connect-src 'self'; style-src 'self'; script-src 'self'",
        ),
    );
    h
}
fn json_resp(status: StatusCode, value: Value) -> (StatusCode, HeaderMap, Json<Value>) {
    (
        status,
        headers("application/json; charset=utf-8"),
        Json(value),
    )
}
fn denied(status: StatusCode) -> (StatusCode, HeaderMap, Json<Value>) {
    json_resp(status, json!({"ok":false,"error":"REQUEST_REJECTED"}))
}
fn request_host(h: &HeaderMap, allowed: &str) -> Option<String> {
    if h.get_all(header::HOST).iter().count() != 1 {
        return None;
    }
    let canonical = canonical_host(h.get(header::HOST)?)?;
    (canonical.as_str() == allowed).then(|| allowed.to_string())
}
fn secret(h: &HeaderMap) -> Option<&str> {
    if h.get_all(header::COOKIE).iter().count() != 1 {
        return None;
    }
    let raw = h.get(header::COOKIE)?.to_str().ok()?;
    let mut found = None;
    for field in raw.split(';') {
        if let Some(value) = field.trim().strip_prefix("__Host-ipat_platform_session=") {
            if found.is_some()
                || value.len() != 43
                || !value
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
            {
                return None;
            }
            found = Some(value);
        }
    }
    found
}
fn csrf(h: &HeaderMap) -> Option<&str> {
    if h.get_all(&CSRF_HEADER).iter().count() != 1 {
        return None;
    }
    let value = h.get(&CSRF_HEADER)?.to_str().ok()?;
    (value.len() == 43
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_'))
    .then_some(value)
}
fn sha256_hex(input: &str) -> String {
    let mut digest = String::with_capacity(64);
    for byte in Sha256::digest(input.as_bytes()) {
        use std::fmt::Write as _;
        write!(&mut digest, "{byte:02x}").expect("write fixed digest");
    }
    digest
}
fn session_inputs(
    h: &HeaderMap,
    allowed: &str,
    mutation: bool,
) -> Option<(String, String, Option<String>)> {
    let host = request_host(h, allowed)?;
    if mutation
        && (h.get_all(header::ORIGIN).iter().count() != 1
            || h.get(header::ORIGIN).and_then(|v| v.to_str().ok())
                != Some(format!("https://{host}").as_str()))
    {
        return None;
    }
    let cookie = sha256_hex(secret(h)?);
    let csrf = if mutation {
        Some(sha256_hex(csrf(h)?))
    } else {
        None
    };
    Some((cookie, host, csrf))
}
async fn verified(
    s: &PlatformState,
    h: &HeaderMap,
    mutation: bool,
) -> Option<(String, String, Option<String>)> {
    let (cookie, host, csrf) = session_inputs(h, &s.expected_host, mutation)?;
    let row = s
        .db
        .query_opt(
            "SELECT issuer,subject FROM ipat_platform.authenticate_platform_browser_session($1,$2,$3,$4)",
            &[&cookie, &host, &csrf, &mutation],
        )
        .await
        .ok()??;
    let issuer: String = row.get(0);
    let subject: String = row.get(1);
    if issuer.is_empty() || subject.is_empty() {
        return None;
    }
    Some((cookie, host, csrf))
}
async fn sign_in_page() -> Response {
    (
        StatusCode::OK,
        html_headers(),
        Html("<!doctype html><html lang=\"en-US\"><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width\"><title>IPAT Platform Owner sign in</title><h1>IPAT Platform Owner</h1><p>Use the approved Platform Owner identity provider and multi-factor authentication.</p><a href=\"/platform/auth/oidc/start\">Sign in securely</a></html>"),
    )
        .into_response()
}
async fn dashboard(State(s): State<Arc<PlatformState>>, h: HeaderMap) -> Response {
    if verified(&s, &h, false).await.is_none() {
        return (
            StatusCode::UNAUTHORIZED,
            html_headers(),
            "Sign in through an approved Platform Owner issuer.",
        )
            .into_response();
    }
    (StatusCode::OK, html_headers(), Html(HTML)).into_response()
}
async fn js() -> Response {
    (
        StatusCode::OK,
        headers("application/javascript; charset=utf-8"),
        JS,
    )
        .into_response()
}
async fn css() -> Response {
    (StatusCode::OK, headers("text/css; charset=utf-8"), CSS).into_response()
}
async fn list(
    State(s): State<Arc<PlatformState>>,
    h: HeaderMap,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let Some((cookie, host, _)) = verified(&s, &h, false).await else {
        return denied(StatusCode::UNAUTHORIZED);
    };
    let Ok(rows) = s.db.query(
        "SELECT tenant_id,tenant_slug,tenant_state,reserved_at::text FROM ipat_platform.list_reservations_from_platform_session($1,$2)",
        &[&cookie, &host],
    ).await else {
        return denied(StatusCode::SERVICE_UNAVAILABLE);
    };
    json_resp(
        StatusCode::OK,
        json!({
            "ok":true,
            "reservations":rows.into_iter().map(|r| json!({
                "tenant_id":r.get::<_,Uuid>(0).to_string(),
                "tenant_slug":r.get::<_,String>(1),
                "tenant_state":r.get::<_,String>(2),
                "reserved_at":r.get::<_,String>(3)
            })).collect::<Vec<_>>()
        }),
    )
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Reservation {
    request_id: String,
    tenant_id: String,
    slug: String,
}
async fn create(
    State(s): State<Arc<PlatformState>>,
    h: HeaderMap,
    Json(v): Json<Reservation>,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let Some((cookie, host, csrf)) = verified(&s, &h, true).await else {
        return denied(StatusCode::UNAUTHORIZED);
    };
    let (Ok(request), Ok(tenant)) = (
        Uuid::parse_str(&v.request_id),
        Uuid::parse_str(&v.tenant_id),
    ) else {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    };
    if v.slug.is_empty()
        || v.slug.len() > 63
        || !v
            .slug
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        || v.slug.starts_with('-')
        || v.slug.ends_with('-')
    {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let result=s.db.query_one(
        "SELECT ipat_platform.reserve_tenant_from_platform_session($1,$2,$3,$4::uuid,$5::uuid,$6)",
        &[&cookie,&host,&csrf,&request,&tenant,&v.slug]
    ).await;
    match result.ok().and_then(|r| r.get::<_, Option<Uuid>>(0)) {
        Some(id) => json_resp(
            StatusCode::CREATED,
            json!({"ok":true,"tenant_id":id.to_string(),"state":"suspended"}),
        ),
        None => denied(StatusCode::CONFLICT),
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ActivationRequestInput {
    request_id: String,
    tenant_id: String,
    domain_id: String,
    hostname: String,
    routing_mode: String,
    admin_issuer: String,
    admin_subject: String,
    admin_expires_at: String,
    evidence_sha256: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ActivationReviewInput {
    approve: bool,
}

fn safe_admin_identity(issuer: &str, subject: &str) -> bool {
    issuer.len() >= 10
        && issuer.len() <= 512
        && issuer.starts_with("https://")
        && !issuer.bytes().any(|b| b.is_ascii_whitespace())
        && !subject.is_empty()
        && subject.len() <= 128
        && subject
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b':' | b'/' | b'.' | b'-'))
}
fn safe_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn safe_rfc3339_shape(value: &str) -> bool {
    value.len() >= 20
        && value.len() <= 40
        && value.contains('T')
        && (value.ends_with('Z') || value.rfind(['+', '-']).is_some_and(|i| i > 10))
        && value.bytes().all(|b| b.is_ascii_graphic())
}

async fn list_activation_requests(
    State(s): State<Arc<PlatformState>>,
    h: HeaderMap,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let Some((cookie, host, _)) = verified(&s, &h, false).await else {
        return denied(StatusCode::UNAUTHORIZED);
    };
    let rows =
        s.db.query(
            "SELECT request_id,tenant_id,tenant_slug,state,customer_hostname,routing_mode,\
             admin_issuer,admin_subject,admin_expires_at::text,evidence_sha256,\
             requested_by_subject,requested_at::text,reviewed_by_subject,reviewed_at::text,\
             verification_name,verification_value \
             FROM ipat_platform.list_company_activation_requests_from_platform_session($1,$2)",
            &[&cookie, &host],
        )
        .await;
    let Ok(rows) = rows else {
        return denied(StatusCode::SERVICE_UNAVAILABLE);
    };
    json_resp(
        StatusCode::OK,
        json!({"ok":true,"requests":rows.into_iter().map(|r|json!({
          "request_id":r.get::<_,Uuid>(0).to_string(),
          "tenant_id":r.get::<_,Uuid>(1).to_string(),
          "tenant_slug":r.get::<_,String>(2),
          "state":r.get::<_,String>(3),
          "hostname":r.get::<_,String>(4),
          "routing_mode":r.get::<_,String>(5),
          "admin_issuer":r.get::<_,String>(6),
          "admin_subject":r.get::<_,String>(7),
          "admin_expires_at":r.get::<_,String>(8),
          "evidence_sha256":r.get::<_,String>(9),
          "requested_by":r.get::<_,String>(10),
          "requested_at":r.get::<_,String>(11),
          "reviewed_by":r.get::<_,Option<String>>(12),
          "reviewed_at":r.get::<_,Option<String>>(13),
          "verification_name":r.get::<_,Option<String>>(14),
          "verification_value":r.get::<_,Option<String>>(15)
        })).collect::<Vec<_>>() }),
    )
}

async fn create_activation_request(
    State(s): State<Arc<PlatformState>>,
    h: HeaderMap,
    Json(v): Json<ActivationRequestInput>,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let Some((cookie, host, csrf)) = verified(&s, &h, true).await else {
        return denied(StatusCode::UNAUTHORIZED);
    };
    let (Ok(request), Ok(tenant), Ok(domain)) = (
        Uuid::parse_str(&v.request_id),
        Uuid::parse_str(&v.tenant_id),
        Uuid::parse_str(&v.domain_id),
    ) else {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    };
    if !strict_host(&v.hostname)
        || !matches!(v.routing_mode.as_str(), "a_record" | "cname" | "nameserver")
        || !safe_admin_identity(&v.admin_issuer, &v.admin_subject)
        || !safe_sha256(&v.evidence_sha256)
        || !safe_rfc3339_shape(&v.admin_expires_at)
    {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let row =
        s.db.query_one(
            "SELECT ipat_platform.request_company_activation_from_platform_session(\
             $1,$2,$3,$4::uuid,$5::uuid,$6::uuid,$7,$8,$9,$10,$11::text::timestamptz,$12)",
            &[
                &cookie,
                &host,
                &csrf,
                &request,
                &tenant,
                &domain,
                &v.hostname,
                &v.routing_mode,
                &v.admin_issuer,
                &v.admin_subject,
                &v.admin_expires_at,
                &v.evidence_sha256,
            ],
        )
        .await;
    // A text wire parameter must be cast inside PostgreSQL; tokio-postgres cannot
    // serialize String directly as TIMESTAMPTZ. A driver/DB failure is not a
    // business conflict, and its details must never be exposed to the caller.
    match row {
        Ok(r) => match r.get::<_, Option<Uuid>>(0) {
            Some(id) => json_resp(
                StatusCode::CREATED,
                json!({"ok":true,"request_id":id.to_string(),"state":"REQUESTED"}),
            ),
            None => denied(StatusCode::CONFLICT),
        },
        Err(_) => denied(StatusCode::SERVICE_UNAVAILABLE),
    }
}

async fn review_activation_request(
    State(s): State<Arc<PlatformState>>,
    h: HeaderMap,
    Path(id): Path<String>,
    Json(v): Json<ActivationReviewInput>,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let Some((cookie, host, csrf)) = verified(&s, &h, true).await else {
        return denied(StatusCode::UNAUTHORIZED);
    };
    let Ok(id) = Uuid::parse_str(&id) else {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    };
    let row =
        s.db.query_one(
            "SELECT ipat_platform.review_company_activation_from_platform_session(\
             $1,$2,$3,$4::uuid,$5)",
            &[&cookie, &host, &csrf, &id, &v.approve],
        )
        .await;
    if row.ok().is_some_and(|r| r.get::<_, bool>(0)) {
        json_resp(
            StatusCode::OK,
            json!({"ok":true,"reviewed":true,"approved":v.approve}),
        )
    } else {
        denied(StatusCode::CONFLICT)
    }
}

async fn logout(
    State(s): State<Arc<PlatformState>>,
    h: HeaderMap,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let Some((cookie, host, csrf)) = verified(&s, &h, true).await else {
        return denied(StatusCode::UNAUTHORIZED);
    };
    let result =
        s.db.query_one(
            "SELECT ipat_platform.revoke_platform_browser_session($1,$2,$3)",
            &[&cookie, &host, &csrf],
        )
        .await;
    if !result.ok().is_some_and(|r| r.get::<_, bool>(0)) {
        return denied(StatusCode::CONFLICT);
    }
    let mut h = headers("application/json; charset=utf-8");
    h.append(
        header::SET_COOKIE,
        HeaderValue::from_static(
            "__Host-ipat_platform_session=; Secure; HttpOnly; SameSite=Strict; Path=/; Max-Age=0",
        ),
    );
    h.append(
        header::SET_COOKIE,
        HeaderValue::from_static(
            "__Host-ipat_platform_csrf=; Secure; SameSite=Strict; Path=/; Max-Age=0",
        ),
    );
    (StatusCode::OK, h, Json(json!({"ok":true})))
}
pub(super) fn router(db: Arc<Client>, expected_host: String) -> Router {
    let state = Arc::new(PlatformState { db, expected_host });
    Router::new()
        .route("/", get(sign_in_page))
        .route("/platform/dashboard", get(dashboard))
        .route("/platform/assets/app.js", get(js))
        .route("/platform/assets/style.css", get(css))
        .route("/api/v1/platform/reservations", get(list).post(create))
        .route(
            "/api/v1/platform/activation-requests",
            get(list_activation_requests).post(create_activation_request),
        )
        .route(
            "/api/v1/platform/activation-requests/{id}/review",
            axum::routing::post(review_activation_request),
        )
        .route("/api/v1/platform/logout", axum::routing::post(logout))
        .with_state(state)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn r981_platform_host_cookie_csrf_are_separate() {
        assert!(strict_host("admin.platform.example"));
        assert!(!allowed_platform_host("202.162.204.121", false, true));
        assert!(!allowed_platform_host("202.162.204.121", true, false));
        assert!(allowed_platform_host("202.162.204.121", true, true));
        assert!(!allowed_platform_host("127.0.0.1", true, true));
        assert!(!allowed_platform_host("10.10.13.233", true, true));
        assert!(!allowed_platform_host("203.0.113.99", true, true));
        assert!(!allowed_platform_host("202.162.204.121:3005", true, true));
        assert!(allowed_platform_host(
            "admin.platform.example",
            false,
            false
        ));
        assert!(safe_admin_identity(
            "https://identity.customer.net/realm/main",
            "initial-admin_01"
        ));
        assert!(!safe_admin_identity(
            "http://identity.customer.net",
            "admin"
        ));
        assert!(!safe_admin_identity(
            "https://identity.customer.net",
            "bad subject"
        ));
        assert!(safe_sha256(&"a".repeat(64)));
        assert!(!safe_sha256(&"A".repeat(64)));
        assert!(safe_rfc3339_shape("2026-11-01T00:00:00Z"));
        assert!(!safe_rfc3339_shape("tomorrow"));
        for bad in [
            "a",
            "admin.EXAMPLE.org",
            "admin..example.org",
            "-admin.example.org",
            "admin.example.org:443",
        ] {
            assert!(!strict_host(bad));
        }
        let mut h = HeaderMap::new();
        h.insert(
            header::HOST,
            HeaderValue::from_static("admin.platform.example"),
        );
        h.insert(
            header::COOKIE,
            HeaderValue::from_static(
                "__Host-ipat_platform_session=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            ),
        );
        h.insert(
            header::ORIGIN,
            HeaderValue::from_static("https://admin.platform.example"),
        );
        h.insert(
            &CSRF_HEADER,
            HeaderValue::from_static("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
        );
        assert!(session_inputs(&h, "admin.platform.example", true).is_some());
        assert!(session_inputs(&h, "other.platform.example", false).is_none());
        h.append(
            &CSRF_HEADER,
            HeaderValue::from_static("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
        );
        assert!(session_inputs(&h, "admin.platform.example", true).is_none());
    }
}

#[cfg(test)]
mod pg_integration {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::Request,
    };
    use tower::ServiceExt;

    const HOST: &str = "platform-r981.synthetic.invalid";
    const ISSUER: &str = "https://identity.r981.synthetic.invalid/realm/platform";
    const SUBJECT: &str = "owner-r981";
    const COOKIE: &str = "ccccccccccccccccccccccccccccccccccccccccccc";
    const CSRF: &str = "ddddddddddddddddddddddddddddddddddddddddddd";

    async fn db() -> Client {
        let mut cfg = Config::new();
        cfg.host("127.0.0.1");
        cfg.port(
            std::env::var("PGPORT")
                .ok()
                .and_then(|x| x.parse().ok())
                .unwrap_or(5432),
        );
        cfg.user(std::env::var("PGUSER").as_deref().unwrap_or("postgres"));
        if let Ok(password) = std::env::var("PGPASSWORD") {
            cfg.password(password);
        }
        cfg.dbname("ipat_synthetic");
        let (client, conn) = cfg.connect(NoTls).await.unwrap();
        tokio::spawn(async move {
            let _ = conn.await;
        });
        client
    }
    fn req(
        method: &str,
        path: &str,
        host: &str,
        cookie: Option<&str>,
        csrf: Option<&str>,
        body: Option<&str>,
    ) -> Request<Body> {
        let mut b = Request::builder()
            .method(method)
            .uri(path)
            .header("Host", host);
        if let Some(c) = cookie {
            b = b.header("Cookie", format!("__Host-ipat_platform_session={c}"));
        }
        if let Some(c) = csrf {
            b = b
                .header("Origin", format!("https://{host}"))
                .header("X-IPAT-Platform-CSRF", c);
        }
        if body.is_some() {
            b = b.header("Content-Type", "application/json");
        }
        b.body(Body::from(body.unwrap_or_default().to_string()))
            .unwrap()
    }
    async fn json_body(r: Response) -> Value {
        serde_json::from_slice(&to_bytes(r.into_body(), 64 * 1024).await.unwrap()).unwrap()
    }
    #[tokio::test]
    async fn r981_real_pg_platform_api_owner_exact_host_csrf_suspended_reservation_logout() {
        if std::env::var("IPAT_PG_EPHEMERAL_TEST").as_deref() != Ok("1")
            || std::env::var("PGDATABASE").as_deref() != Ok("ipat_synthetic")
        {
            return;
        }
        let admin = db().await;
        admin.execute("INSERT INTO ipat_platform.platform_principals(issuer,subject,role,approved_by,expires_at) VALUES($1,$2,'platform_owner','independent-synthetic-reviewer',clock_timestamp()+interval '1 day')",&[&ISSUER,&SUBJECT]).await.unwrap();
        admin.execute("INSERT INTO ipat_platform.platform_console_hosts(hostname,verified_at,tls_ready_at) VALUES($1,clock_timestamp()-interval '1 day',clock_timestamp()-interval '1 hour')",&[&HOST]).await.unwrap();
        let issuer_db = db().await;
        issuer_db
            .batch_execute("SET ROLE ipat_platform_session_issuer_login")
            .await
            .unwrap();
        let session = Uuid::parse_str("81818181-8181-4181-8181-818181818181").unwrap();
        let issued=issuer_db.query_one(
            "SELECT ipat_platform.issue_platform_browser_session($1,$2,$3,$4::uuid,$5,$6,clock_timestamp()+interval '8 minutes')",
            &[&ISSUER,&SUBJECT,&HOST,&session,&sha256_hex(COOKIE),&sha256_hex(CSRF)]
        ).await.unwrap();
        assert_eq!(issued.get::<_, Option<Uuid>>(0), Some(session));
        let api_db = db().await;
        api_db
            .batch_execute("SET ROLE ipat_platform_session_api_login")
            .await
            .unwrap();
        let app = router(Arc::new(api_db), HOST.to_string());
        let unauthorized = app
            .clone()
            .oneshot(req("GET", "/platform/dashboard", HOST, None, None, None))
            .await
            .unwrap();
        assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);
        let login_page = app
            .clone()
            .oneshot(req("GET", "/", HOST, None, None, None))
            .await
            .unwrap();
        assert_eq!(login_page.status(), StatusCode::OK);
        let login_html = to_bytes(login_page.into_body(), 32 * 1024).await.unwrap();
        let login_html = String::from_utf8(login_html.to_vec()).unwrap();
        assert!(login_html.contains("/platform/auth/oidc/start"));
        assert!(!login_html.contains("password"));
        let wrong_host = app
            .clone()
            .oneshot(req(
                "GET",
                "/api/v1/platform/reservations",
                "other-r981.synthetic.invalid",
                Some(COOKIE),
                None,
                None,
            ))
            .await
            .unwrap();
        assert_eq!(wrong_host.status(), StatusCode::UNAUTHORIZED);
        let tenant_cookie = app
            .clone()
            .oneshot(req(
                "GET",
                "/api/v1/platform/reservations",
                HOST,
                Some("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"),
                None,
                None,
            ))
            .await
            .unwrap();
        assert_eq!(tenant_cookie.status(), StatusCode::UNAUTHORIZED);
        let dashboard = app
            .clone()
            .oneshot(req(
                "GET",
                "/platform/dashboard",
                HOST,
                Some(COOKIE),
                None,
                None,
            ))
            .await
            .unwrap();
        assert_eq!(dashboard.status(), StatusCode::OK);
        let req_id = Uuid::parse_str("81818181-8181-4181-8181-818181818182").unwrap();
        let tenant_id = Uuid::parse_str("81818181-8181-4181-8181-818181818183").unwrap();
        let body=json!({"request_id":req_id.to_string(),"tenant_id":tenant_id.to_string(),"slug":"r981-company"}).to_string();
        let missing_csrf = app
            .clone()
            .oneshot(req(
                "POST",
                "/api/v1/platform/reservations",
                HOST,
                Some(COOKIE),
                None,
                Some(&body),
            ))
            .await
            .unwrap();
        assert_eq!(missing_csrf.status(), StatusCode::UNAUTHORIZED);
        let wrong_origin = app
            .clone()
            .oneshot(req(
                "POST",
                "/api/v1/platform/reservations",
                "other-r981.synthetic.invalid",
                Some(COOKIE),
                Some(CSRF),
                Some(&body),
            ))
            .await
            .unwrap();
        assert_eq!(wrong_origin.status(), StatusCode::UNAUTHORIZED);
        for _ in 0..2 {
            let created = app
                .clone()
                .oneshot(req(
                    "POST",
                    "/api/v1/platform/reservations",
                    HOST,
                    Some(COOKIE),
                    Some(CSRF),
                    Some(&body),
                ))
                .await
                .unwrap();
            assert_eq!(created.status(), StatusCode::CREATED);
            assert_eq!(json_body(created).await["state"], "suspended");
        }
        let forged=json!({"request_id":req_id.to_string(),"tenant_id":tenant_id.to_string(),"slug":"forged"}).to_string();
        let denied_replay = app
            .clone()
            .oneshot(req(
                "POST",
                "/api/v1/platform/reservations",
                HOST,
                Some(COOKIE),
                Some(CSRF),
                Some(&forged),
            ))
            .await
            .unwrap();
        assert_eq!(denied_replay.status(), StatusCode::CONFLICT);
        let listed = app
            .clone()
            .oneshot(req(
                "GET",
                "/api/v1/platform/reservations",
                HOST,
                Some(COOKIE),
                None,
                None,
            ))
            .await
            .unwrap();
        assert_eq!(listed.status(), StatusCode::OK);
        let entries = json_body(listed).await;
        assert!(entries["reservations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x["tenant_slug"] == "r981-company"));
        assert_eq!(
            admin
                .query_one(
                    "SELECT state FROM ipat_platform.tenants WHERE id=$1",
                    &[&tenant_id]
                )
                .await
                .unwrap()
                .get::<_, String>(0),
            "suspended"
        );
        let logout = app
            .clone()
            .oneshot(req(
                "POST",
                "/api/v1/platform/logout",
                HOST,
                Some(COOKIE),
                Some(CSRF),
                None,
            ))
            .await
            .unwrap();
        assert_eq!(logout.status(), StatusCode::OK);
        assert_eq!(
            logout.headers().get_all(header::SET_COOKIE).iter().count(),
            2
        );
        let replay = app
            .clone()
            .oneshot(req(
                "GET",
                "/api/v1/platform/reservations",
                HOST,
                Some(COOKIE),
                None,
                None,
            ))
            .await
            .unwrap();
        assert_eq!(replay.status(), StatusCode::UNAUTHORIZED);
    }
    #[tokio::test]
    async fn r1011_real_pg_platform_company_activation_requires_independent_owner_and_keeps_domain_pending(
    ) {
        if std::env::var("IPAT_PG_EPHEMERAL_TEST").as_deref() != Ok("1")
            || std::env::var("PGDATABASE").as_deref() != Ok("ipat_synthetic")
        {
            return;
        }
        const H: &str = "platform-r1011.synthetic.net";
        const I: &str = "https://identity.r1011.synthetic.invalid/realm/platform";
        const O1: &str = "owner-r1011-maker-http";
        const O2: &str = "owner-r1011-checker-http";
        const C1: &str = "mmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm";
        const X1: &str = "nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn";
        const C2: &str = "ooooooooooooooooooooooooooooooooooooooooooo";
        const X2: &str = "ppppppppppppppppppppppppppppppppppppppppppp";
        let admin = db().await;
        admin.execute(
            "INSERT INTO ipat_platform.platform_principals(issuer,subject,role,approved_by,expires_at)
             VALUES($1,$2,'platform_owner','r1011-review',clock_timestamp()+interval '1 day'),
                   ($1,$3,'platform_owner','r1011-review',clock_timestamp()+interval '1 day')",
            &[&I, &O1, &O2],
        ).await.unwrap();
        admin.execute(
            "INSERT INTO ipat_platform.platform_console_hosts(hostname,verified_at,tls_ready_at)
             VALUES($1,clock_timestamp()-interval '1 day',clock_timestamp()-interval '1 hour')",
            &[&H],
        ).await.unwrap();
        let issuer_db = db().await;
        issuer_db
            .batch_execute("SET ROLE ipat_platform_session_issuer_login")
            .await
            .unwrap();
        for (subject, cookie, csrf, sid) in [
            (O1, C1, X1, "11111111-1111-4111-8111-111111111111"),
            (O2, C2, X2, "11111111-1111-4111-8111-111111111112"),
        ] {
            let sid = Uuid::parse_str(sid).unwrap();
            let issued = issuer_db
                .query_one(
                    "SELECT ipat_platform.issue_platform_browser_session(
                 $1,$2,$3,$4::uuid,$5,$6,clock_timestamp()+interval '8 minutes')",
                    &[
                        &I,
                        &subject,
                        &H,
                        &sid,
                        &sha256_hex(cookie),
                        &sha256_hex(csrf),
                    ],
                )
                .await
                .unwrap();
            assert_eq!(issued.get::<_, Option<Uuid>>(0), Some(sid));
        }
        let api_db = db().await;
        api_db
            .batch_execute("SET ROLE ipat_platform_session_api_login")
            .await
            .unwrap();
        let app = router(Arc::new(api_db), H.to_string());

        let tenant = "11111111-1111-4111-8111-111111111121";
        let reserve = json!({
            "request_id":"11111111-1111-4111-8111-111111111122",
            "tenant_id":tenant,
            "slug":"r1011-http-company"
        })
        .to_string();
        let reserved = app
            .clone()
            .oneshot(req(
                "POST",
                "/api/v1/platform/reservations",
                H,
                Some(C1),
                Some(X1),
                Some(&reserve),
            ))
            .await
            .unwrap();
        assert_eq!(reserved.status(), StatusCode::CREATED);

        let expires: String = admin
            .query_one(
                "SELECT to_char((clock_timestamp()+interval '30 days') AT TIME ZONE 'UTC',
             'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"')",
                &[],
            )
            .await
            .unwrap()
            .get(0);
        let activation = json!({
            "request_id":"11111111-1111-4111-8111-111111111123",
            "tenant_id":tenant,
            "domain_id":"11111111-1111-4111-8111-111111111124",
            "hostname":"portal.r1011-company.net",
            "routing_mode":"a_record",
            "admin_issuer":"https://identity.r1011-company.net/realm/customer",
            "admin_subject":"initial-admin-r1011-http",
            "admin_expires_at":expires,
            "evidence_sha256":"eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
        })
        .to_string();

        let missing_csrf = app
            .clone()
            .oneshot(req(
                "POST",
                "/api/v1/platform/activation-requests",
                H,
                Some(C1),
                None,
                Some(&activation),
            ))
            .await
            .unwrap();
        assert_eq!(missing_csrf.status(), StatusCode::UNAUTHORIZED);
        let created = app
            .clone()
            .oneshot(req(
                "POST",
                "/api/v1/platform/activation-requests",
                H,
                Some(C1),
                Some(X1),
                Some(&activation),
            ))
            .await
            .unwrap();
        assert_eq!(created.status(), StatusCode::CREATED);
        let retry = app
            .clone()
            .oneshot(req(
                "POST",
                "/api/v1/platform/activation-requests",
                H,
                Some(C1),
                Some(X1),
                Some(&activation),
            ))
            .await
            .unwrap();
        assert_eq!(retry.status(), StatusCode::CREATED);

        let self_review = app
            .clone()
            .oneshot(req(
                "POST",
                "/api/v1/platform/activation-requests/11111111-1111-4111-8111-111111111123/review",
                H,
                Some(C1),
                Some(X1),
                Some(r#"{"approve":true}"#),
            ))
            .await
            .unwrap();
        assert_eq!(self_review.status(), StatusCode::CONFLICT);
        let approved = app
            .clone()
            .oneshot(req(
                "POST",
                "/api/v1/platform/activation-requests/11111111-1111-4111-8111-111111111123/review",
                H,
                Some(C2),
                Some(X2),
                Some(r#"{"approve":true}"#),
            ))
            .await
            .unwrap();
        assert_eq!(approved.status(), StatusCode::OK);

        let tenant_id = Uuid::parse_str(tenant).unwrap();
        let state: String = admin
            .query_one(
                "SELECT state FROM ipat_platform.tenants WHERE id=$1",
                &[&tenant_id],
            )
            .await
            .unwrap()
            .get(0);
        assert_eq!(state, "active");
        let membership: i64 = admin
            .query_one(
                "SELECT count(*) FROM ipat_platform.identity_memberships
             WHERE tenant_id=$1 AND role='tenant_admin' AND revoked_at IS NULL",
                &[&tenant_id],
            )
            .await
            .unwrap()
            .get(0);
        assert_eq!(membership, 1);
        let domain_state: String = admin
            .query_one(
                "SELECT activation_state FROM ipat_platform.tenant_domains WHERE tenant_id=$1",
                &[&tenant_id],
            )
            .await
            .unwrap()
            .get(0);
        assert_eq!(domain_state, "pending_dns");

        let listing = app
            .clone()
            .oneshot(req(
                "GET",
                "/api/v1/platform/activation-requests",
                H,
                Some(C2),
                None,
                None,
            ))
            .await
            .unwrap();
        assert_eq!(listing.status(), StatusCode::OK);
        let body = json_body(listing).await;
        let row = body["requests"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["tenant_id"] == tenant)
            .unwrap();
        assert_eq!(row["state"], "APPROVED");
        assert_eq!(
            row["verification_name"],
            "_ipat-verify.portal.r1011-company.net"
        );
        assert_eq!(
            row["verification_value"],
            "ipat-domain=11111111-1111-4111-8111-111111111124"
        );
    }
}
