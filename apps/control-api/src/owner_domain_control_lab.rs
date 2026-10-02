//! Owner-private platform/domain STAGING. Does not activate public DNS/TLS,
//! grant any tenant identity, issue TXT proofs or touch host ingress.
use axum::{
    extract::{DefaultBodyLimit, Path, State},
    http::{HeaderMap, StatusCode},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    net::Ipv4Addr,
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::{Path as FsPath, PathBuf},
    sync::Arc,
};
use tokio::sync::Mutex;

const FOLDER: &str = "/home/openai/.local/share/ipat/r962-domain-control";
const MAX_DRAFTS: usize = 32;
const HTML: &str = include_str!("../../../web/lab/platform-domain-admin.html");
const JS: &str = include_str!("../../../web/lab/platform-domain-admin.js");
const CSS: &str = include_str!("../../../web/lab/platform-domain-admin.css");
#[derive(Clone)]
struct Store {
    path: PathBuf,
    guard: Arc<Mutex<()>>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DomainRequest {
    id: String,
    tenant_reference: String,
    hostname: String,
    kind: String,
    revision: u64,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Event {
    sequence: u64,
    operation: String,
    reference: String,
}
#[derive(Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Settings {
    revision: u64,
    platform_hostname: Option<String>,
    public_ipv4: Option<Ipv4Addr>,
    next_request: u64,
    drafts: Vec<DomainRequest>,
    audit: Vec<Event>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfileInput {
    expected_revision: u64,
    platform_hostname: Option<String>,
    public_ipv4: Option<Ipv4Addr>,
    confirm: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RequestInput {
    tenant_reference: String,
    kind: String,
    custom_hostname: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RemoveInput {
    expected_revision: u64,
    confirm: String,
}
fn error(status: StatusCode, code: &'static str) -> (StatusCode, HeaderMap, Json<Value>) {
    (
        status,
        super::private_lab_headers("application/json; charset=utf-8"),
        Json(json!({
        "error":code,"public_route_changed":false,"owner_private_lab_only":true})),
    )
}
fn owner(headers: &HeaderMap, mutation: bool) -> bool {
    super::owner_device_registry_lab::strict_owner(headers, mutation)
}
fn valid_slug(s: &str) -> bool {
    (1..=48).contains(&s.len())
        && !s.starts_with('-')
        && !s.ends_with('-')
        && s.bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
        && !matches!(
            s,
            "admin" | "api" | "platform" | "cwmp" | "usp" | "www" | "localhost"
        )
}
fn hostname(raw: &str) -> Option<String> {
    let normalized = super::tenant_domain::canonical_dns_name(raw)?;
    if !super::tenant_domain::valid_custom_domain(&normalized) {
        return None;
    }
    let lower = normalized.as_str().to_owned();
    if lower.ends_with(".example.com") || lower == "example.com" || lower == "ipat.fadly.id" {
        return None;
    }
    Some(lower)
}
fn public_address(ip: Ipv4Addr) -> bool {
    let [a, b, c, _] = ip.octets();
    // Exclude private, shared, link-local, documentation, loopback, broadcast,
    // multicast and protocol/reserved IP space. Not a proof of VPS ownership.
    !(a == 0
        || a == 10
        || a == 127
        || a >= 224
        || (a == 100 && (64..=127).contains(&b))
        || (a == 169 && b == 254)
        || (a == 172 && (16..=31).contains(&b))
        || (a == 192 && (b == 168 || (b == 0 && c == 2)))
        || (a == 198 && (b == 18 || b == 19 || (b == 51 && c == 100)))
        || (a == 203 && b == 0 && c == 113))
}
fn read(path: &FsPath) -> std::io::Result<Settings> {
    let parent = path
        .parent()
        .ok_or_else(|| std::io::Error::other("invalid folder"))?;
    super::owner_device_registry_lab::private_dir(parent)?;
    if !path.exists() {
        if path.symlink_metadata().is_ok() {
            return Err(std::io::Error::other("unsafe symlink"));
        }
        return Ok(Settings::default());
    }
    super::owner_device_registry_lab::private_file(path)?;
    let mut data = Vec::new();
    File::open(path)?.take(131073).read_to_end(&mut data)?;
    if data.len() > 131072 {
        return Err(std::io::Error::other("domain registry oversized"));
    }
    let s: Settings = serde_json::from_slice(&data)
        .map_err(|_| std::io::Error::other("invalid domain registry"))?;
    if s.drafts.len() > MAX_DRAFTS
        || s.audit.len() > 512
        || s.platform_hostname
            .as_deref()
            .is_some_and(|v| hostname(v).as_deref() != Some(v))
        || s.public_ipv4.is_some_and(|v| !public_address(v))
        || s.drafts.iter().any(|d| {
            !valid_slug(&d.tenant_reference)
                || hostname(&d.hostname).as_deref() != Some(&d.hostname)
        })
    {
        return Err(std::io::Error::other("invalid staged domain registry"));
    }
    Ok(s)
}
fn write(path: &FsPath, s: &Settings) -> std::io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| std::io::Error::other("invalid folder"))?;
    super::owner_device_registry_lab::private_dir(parent)?;
    let body = serde_json::to_vec(s).map_err(std::io::Error::other)?;
    if body.len() > 131072 {
        return Err(std::io::Error::other("domain registry exceeds limit"));
    }
    let mut rnd = [0u8; 8];
    File::open("/dev/urandom")?.read_exact(&mut rnd)?;
    let tmp = path.with_extension(format!("json.pending-{:016x}", u64::from_ne_bytes(rnd)));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&tmp)?;
    let result = (|| {
        file.write_all(&body)?;
        file.sync_all()?;
        if path.symlink_metadata().is_ok() {
            super::owner_device_registry_lab::private_file(path)?;
        }
        fs::rename(&tmp, path)?;
        File::open(parent)?.sync_all()?;
        Ok::<(), std::io::Error>(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(tmp);
    }
    result
}
fn record(s: &mut Settings, operation: &str, reference: &str) -> bool {
    if s.audit.len() >= 512 {
        return false;
    }
    s.audit.push(Event {
        sequence: s.audit.len() as u64 + 1,
        operation: operation.into(),
        reference: reference.into(),
    });
    true
}
fn staged(s: &Settings) -> Value {
    json!({"owner_private_lab_only":true,"persisted":true,
        "platform":{"revision":s.revision,"hostname":s.platform_hostname,"temporary_public_ipv4":s.public_ipv4,
        "access_mode":"PRIVATE_SSH_ONLY","public_https_ready":false,
        "dns_ownership_verified":false,"ingress_tenant_routing_ready":false,
        "safe_to_point_now":false,"runtime_changes_applied":false},
        "drafts":s.drafts.iter().map(|d|json!({"id":d.id,"hostname":d.hostname,
            "tenant_reference":d.tenant_reference,"kind":d.kind,"revision":d.revision,
            "state":"PLANNED_NOT_ROUTED","tls_ready":false,"ownership_verified":false})).collect::<Vec<_>>(),
        "audit_count":s.audit.len(),"tenant_identity_verified":false})
}
async fn list(
    State(state): State<Store>,
    headers: HeaderMap,
) -> Result<(HeaderMap, Json<Value>), (StatusCode, HeaderMap, Json<Value>)> {
    if !owner(&headers, false) {
        return Err(error(StatusCode::FORBIDDEN, "OWNER_TUNNEL_REQUIRED"));
    }
    let _lock = state.guard.lock().await;
    let data = read(&state.path)
        .map_err(|_| error(StatusCode::SERVICE_UNAVAILABLE, "DOMAIN_STORE_UNAVAILABLE"))?;
    Ok((
        super::private_lab_headers("application/json; charset=utf-8"),
        Json(staged(&data)),
    ))
}
async fn profile(
    State(state): State<Store>,
    headers: HeaderMap,
    Json(input): Json<ProfileInput>,
) -> Result<(HeaderMap, Json<Value>), (StatusCode, HeaderMap, Json<Value>)> {
    if !owner(&headers, true) {
        return Err(error(StatusCode::FORBIDDEN, "OWNER_TUNNEL_REQUIRED"));
    }
    if input.confirm != "STAGE_DOMAIN_CONFIG"
        || input
            .platform_hostname
            .as_deref()
            .is_some_and(|h| hostname(h).as_deref() != Some(h))
        || input.public_ipv4.is_some_and(|a| !public_address(a))
    {
        return Err(error(
            StatusCode::BAD_REQUEST,
            "INVALID_PLATFORM_DOMAIN_PLAN",
        ));
    }
    let _lock = state.guard.lock().await;
    let mut s = read(&state.path)
        .map_err(|_| error(StatusCode::SERVICE_UNAVAILABLE, "DOMAIN_STORE_UNAVAILABLE"))?;
    if s.revision != input.expected_revision {
        return Err(error(StatusCode::CONFLICT, "STALE_REVISION"));
    }
    if s.platform_hostname != input.platform_hostname
        && s.drafts.iter().any(|d| d.kind == "managed_subdomain")
    {
        return Err(error(
            StatusCode::CONFLICT,
            "MANAGED_SUBDOMAINS_MUST_BE_REVIEWED",
        ));
    }
    let next = s
        .revision
        .checked_add(1)
        .ok_or_else(|| error(StatusCode::CONFLICT, "STALE_REVISION"))?;
    if !record(&mut s, "STAGE_PLATFORM_DOMAIN", "platform") {
        return Err(error(StatusCode::CONFLICT, "DOMAIN_AUDIT_LIMIT"));
    }
    s.platform_hostname = input.platform_hostname;
    s.public_ipv4 = input.public_ipv4;
    s.revision = next;
    write(&state.path, &s)
        .map_err(|_| error(StatusCode::SERVICE_UNAVAILABLE, "DOMAIN_STORE_UNAVAILABLE"))?;
    Ok((
        super::private_lab_headers("application/json; charset=utf-8"),
        Json(staged(&s)),
    ))
}
async fn create(
    State(state): State<Store>,
    headers: HeaderMap,
    Json(input): Json<RequestInput>,
) -> Result<(StatusCode, HeaderMap, Json<Value>), (StatusCode, HeaderMap, Json<Value>)> {
    if !owner(&headers, true) {
        return Err(error(StatusCode::FORBIDDEN, "OWNER_TUNNEL_REQUIRED"));
    }
    if !valid_slug(&input.tenant_reference) {
        return Err(error(
            StatusCode::BAD_REQUEST,
            "INVALID_UNVERIFIED_TENANT_REFERENCE",
        ));
    }
    let _lock = state.guard.lock().await;
    let mut s = read(&state.path)
        .map_err(|_| error(StatusCode::SERVICE_UNAVAILABLE, "DOMAIN_STORE_UNAVAILABLE"))?;
    if s.drafts.len() >= MAX_DRAFTS {
        return Err(error(StatusCode::CONFLICT, "DOMAIN_DRAFT_LIMIT"));
    }
    let name = match input.kind.as_str() {
        "managed_subdomain" if input.custom_hostname.is_none() => {
            let base = s.platform_hostname.as_ref().ok_or_else(|| {
                error(
                    StatusCode::PRECONDITION_FAILED,
                    "PLATFORM_DOMAIN_NOT_CONFIGURED",
                )
            })?;
            hostname(&format!("{}.{}", input.tenant_reference, base))
        }
        "custom_domain" => input.custom_hostname.as_deref().and_then(hostname),
        _ => None,
    }
    .ok_or_else(|| error(StatusCode::BAD_REQUEST, "INVALID_DOMAIN_DRAFT"))?;
    if name == s.platform_hostname.as_deref().unwrap_or("")
        || s.drafts.iter().any(|d| d.hostname == name)
    {
        return Err(error(StatusCode::CONFLICT, "DOMAIN_RESERVED_OR_DUPLICATE"));
    }
    let id = s
        .next_request
        .checked_add(1)
        .ok_or_else(|| error(StatusCode::CONFLICT, "DOMAIN_DRAFT_LIMIT"))?;
    let entry = DomainRequest {
        id: format!("DOM-{id:06}"),
        tenant_reference: input.tenant_reference,
        hostname: name,
        kind: input.kind,
        revision: 1,
    };
    if !record(&mut s, "CREATE_DRAFT", &entry.id) {
        return Err(error(StatusCode::CONFLICT, "DOMAIN_AUDIT_LIMIT"));
    }
    s.next_request = id;
    s.drafts.push(entry.clone());
    write(&state.path, &s)
        .map_err(|_| error(StatusCode::SERVICE_UNAVAILABLE, "DOMAIN_STORE_UNAVAILABLE"))?;
    Ok((
        StatusCode::CREATED,
        super::private_lab_headers("application/json; charset=utf-8"),
        Json(
            json!({"draft":entry,"state":"PLANNED_NOT_ROUTED","dns_ownership_verified":false,
            "tls_ready":false,"public_route_changed":false}),
        ),
    ))
}
async fn preview(
    State(state): State<Store>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<(HeaderMap, Json<Value>), (StatusCode, HeaderMap, Json<Value>)> {
    if !owner(&headers, false) {
        return Err(error(StatusCode::FORBIDDEN, "OWNER_TUNNEL_REQUIRED"));
    }
    let _lock = state.guard.lock().await;
    let s = read(&state.path)
        .map_err(|_| error(StatusCode::SERVICE_UNAVAILABLE, "DOMAIN_STORE_UNAVAILABLE"))?;
    let d = s
        .drafts
        .iter()
        .find(|d| d.id == id)
        .ok_or_else(|| error(StatusCode::NOT_FOUND, "DOMAIN_DRAFT_NOT_FOUND"))?;
    Ok((
        super::private_lab_headers("application/json; charset=utf-8"),
        Json(json!({
        "hostname":d.hostname,"state":"PLANNED_NOT_ROUTED", "routing_preference":"a_record",
        "suggested_a_record":{"name":d.hostname,"value":s.public_ipv4},
        "ownership_txt_name":format!("_ipat-verify.{}",d.hostname),
        "ownership_txt_value":null,"dns_ownership_verified":false,"tls_ready":false,
        "safe_to_point_now":false,"public_route_changed":false,
        "warning":"PREVIEW_ONLY: do not point DNS until verified tenant/domain ownership, public ingress and TLS are independently ready."})),
    ))
}
async fn remove(
    State(state): State<Store>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<RemoveInput>,
) -> Result<(HeaderMap, Json<Value>), (StatusCode, HeaderMap, Json<Value>)> {
    if !owner(&headers, true) {
        return Err(error(StatusCode::FORBIDDEN, "OWNER_TUNNEL_REQUIRED"));
    }
    if input.confirm != format!("REMOVE DRAFT {id}") {
        return Err(error(
            StatusCode::BAD_REQUEST,
            "EXPLICIT_DRAFT_REMOVE_REQUIRED",
        ));
    }
    let _lock = state.guard.lock().await;
    let mut s = read(&state.path)
        .map_err(|_| error(StatusCode::SERVICE_UNAVAILABLE, "DOMAIN_STORE_UNAVAILABLE"))?;
    let i = s
        .drafts
        .iter()
        .position(|d| d.id == id)
        .ok_or_else(|| error(StatusCode::NOT_FOUND, "DOMAIN_DRAFT_NOT_FOUND"))?;
    if s.drafts[i].revision != input.expected_revision {
        return Err(error(StatusCode::CONFLICT, "STALE_REVISION"));
    }
    if !record(&mut s, "REMOVE_DRAFT", &id) {
        return Err(error(StatusCode::CONFLICT, "DOMAIN_AUDIT_LIMIT"));
    }
    s.drafts.remove(i);
    write(&state.path, &s)
        .map_err(|_| error(StatusCode::SERVICE_UNAVAILABLE, "DOMAIN_STORE_UNAVAILABLE"))?;
    Ok((
        super::private_lab_headers("application/json; charset=utf-8"),
        Json(json!({"removed":true,"id":id,"public_route_changed":false})),
    ))
}
async fn html() -> (HeaderMap, axum::response::Html<&'static str>) {
    (
        super::private_lab_headers("text/html; charset=utf-8"),
        axum::response::Html(HTML),
    )
}
async fn js() -> (HeaderMap, &'static str) {
    (
        super::private_lab_headers("text/javascript; charset=utf-8"),
        JS,
    )
}
async fn css() -> (HeaderMap, &'static str) {
    (super::private_lab_headers("text/css; charset=utf-8"), CSS)
}
fn router_at(path: PathBuf) -> Router {
    let state = Store {
        path,
        guard: Arc::new(Mutex::new(())),
    };
    Router::new()
        .route("/lab/platform-admin/domains", get(html))
        .route("/lab/platform-admin/domains.js", get(js))
        .route("/lab/platform-admin/domains.css", get(css))
        .route("/lab/owner/platform-domains", get(list).put(profile))
        .route(
            "/lab/owner/platform-domains/drafts",
            axum::routing::post(create),
        )
        .route(
            "/lab/owner/platform-domains/drafts/{id}",
            axum::routing::delete(remove),
        )
        .route(
            "/lab/owner/platform-domains/drafts/{id}/preview",
            get(preview),
        )
        .layer(DefaultBodyLimit::max(1024))
        .with_state(state)
}
pub(super) fn router() -> Router {
    router_at(PathBuf::from(FOLDER).join("settings.json"))
}
#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::Request,
    };
    use tower::ServiceExt;
    async fn call(
        app: &Router,
        m: &str,
        p: &str,
        payload: &str,
        authorized: bool,
    ) -> (StatusCode, Value) {
        let mut req = Request::builder()
            .method(m)
            .uri(p)
            .header("host", "127.0.0.1:3002");
        if authorized {
            req = req
                .header("origin", "http://127.0.0.1:3002")
                .header("x-ipat-owner-private", "1");
        }
        let res = app
            .clone()
            .oneshot(
                req.header("content-type", "application/json")
                    .body(Body::from(payload.to_owned()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let code = res.status();
        let bytes = to_bytes(res.into_body(), 65536).await.unwrap();
        (code, serde_json::from_slice(&bytes).unwrap_or(json!({})))
    }
    #[tokio::test]
    async fn private_staging_preserves_false_public_status_across_restart() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("owner").join("domains.json");
        let app = router_at(path.clone());
        let (code, current) = call(&app, "GET", "/lab/owner/platform-domains", "", false).await;
        assert_eq!(code, StatusCode::OK);
        assert_eq!(current["platform"]["public_https_ready"], false);
        let profile = r#"{"expected_revision":0,"platform_hostname":"ipat.id","public_ipv4":"202.162.204.121","confirm":"STAGE_DOMAIN_CONFIG"}"#;
        assert_eq!(
            call(&app, "PUT", "/lab/owner/platform-domains", profile, false)
                .await
                .0,
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            call(&app, "PUT", "/lab/owner/platform-domains", profile, true)
                .await
                .0,
            StatusCode::OK
        );
        assert_eq!(
            call(&app, "PUT", "/lab/owner/platform-domains", profile, true)
                .await
                .0,
            StatusCode::CONFLICT
        );
        assert_eq!(call(&app,"POST","/lab/owner/platform-domains/drafts",
          r#"{"tenant_reference":"company-a","kind":"managed_subdomain","custom_hostname":null}"#,true).await.0,StatusCode::CREATED);
        let (code, list) = call(
            &router_at(path.clone()),
            "GET",
            "/lab/owner/platform-domains",
            "",
            false,
        )
        .await;
        assert_eq!(code, StatusCode::OK);
        assert_eq!(list["drafts"][0]["hostname"], "company-a.ipat.id");
        assert_eq!(list["platform"]["runtime_changes_applied"], false);
        let (code, p) = call(
            &app,
            "GET",
            "/lab/owner/platform-domains/drafts/DOM-000001/preview",
            "",
            false,
        )
        .await;
        assert_eq!(code, StatusCode::OK);
        assert_eq!(p["suggested_a_record"]["value"], "202.162.204.121");
        assert_eq!(p["ownership_txt_value"], Value::Null);
        assert_eq!(p["safe_to_point_now"], false);
        assert_eq!(
            call(
                &app,
                "DELETE",
                "/lab/owner/platform-domains/drafts/DOM-000001",
                r#"{"expected_revision":1,"confirm":"REMOVE DRAFT DOM-000001"}"#,
                true
            )
            .await
            .0,
            StatusCode::OK
        );
        assert_eq!(read(&path).unwrap().audit.len(), 3);
    }
    #[tokio::test]
    async fn reject_unsafe_ip_reserved_names_and_domain_takeover() {
        let dir = tempfile::tempdir().unwrap();
        let app = router_at(dir.path().join("safe").join("domains.json"));
        for bad in [
            "127.0.0.1",
            "192.0.2.10",
            "10.0.0.1",
            "203.0.113.1",
            "100.64.0.1",
        ] {
            let body = format!(
                r#"{{"expected_revision":0,"platform_hostname":"ipat.id","public_ipv4":"{bad}","confirm":"STAGE_DOMAIN_CONFIG"}}"#
            );
            assert_eq!(
                call(&app, "PUT", "/lab/owner/platform-domains", &body, true)
                    .await
                    .0,
                StatusCode::BAD_REQUEST
            );
        }
        assert_eq!(
            call(
                &app,
                "POST",
                "/lab/owner/platform-domains/drafts",
                r#"{"tenant_reference":"admin","kind":"managed_subdomain","custom_hostname":null}"#,
                true
            )
            .await
            .0,
            StatusCode::BAD_REQUEST
        );
        assert_eq!(call(&app,"PUT","/lab/owner/platform-domains",
          r#"{"expected_revision":0,"platform_hostname":"ipat.id","public_ipv4":"202.162.204.121","confirm":"STAGE_DOMAIN_CONFIG"}"#,true).await.0,StatusCode::OK);
        assert_eq!(call(&app,"POST","/lab/owner/platform-domains/drafts",
          r#"{"tenant_reference":"example","kind":"custom_domain","custom_hostname":"example.com"}"#,true).await.0,StatusCode::BAD_REQUEST);
        assert_eq!(call(&app,"POST","/lab/owner/platform-domains/drafts",
          r#"{"tenant_reference":"company-b","kind":"custom_domain","custom_hostname":"portal.customer.co.id"}"#,true).await.0,StatusCode::CREATED);
        assert_eq!(call(&app,"POST","/lab/owner/platform-domains/drafts",
          r#"{"tenant_reference":"company-c","kind":"custom_domain","custom_hostname":"portal.customer.co.id"}"#,true).await.0,StatusCode::CONFLICT);
    }
}
