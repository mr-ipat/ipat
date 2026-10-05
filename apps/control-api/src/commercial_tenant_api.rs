//! R9.68 commercial tenant JSON API over R9.67 durable Host-bound sessions.
//!
//! Source-complete router boundary, NOT publicly mounted until deployment has
//! trusted HTTPS + real confidential OIDC callback + production PostgreSQL.
//! Tenant and actor identity come only from durable session authentication.
use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{fs::File, io::Read, path::PathBuf, sync::Arc};
use tokio_postgres::{Client, Config, NoTls};
use uuid::Uuid;

use crate::{
    durable_tenant_session,
    tenant_domain::{
        canonical_dns_name, canonical_host, dns_profile_from_environment,
        saved_customer_dns_guidance, valid_custom_domain,
    },
};

const CSRF_HEADER: HeaderName = HeaderName::from_static("x-ipat-csrf");
const DASHBOARD_HTML: &str = include_str!("../../../web/console/tenant/dashboard.html");
const DASHBOARD_JS: &str = include_str!("../../../web/console/tenant/app.js");
const DASHBOARD_CSS: &str = include_str!("../../../web/console/tenant/style.css");
// This is only a syntax/transport envelope; the PostgreSQL catalog INSERT
// trigger is the authoritative platform-controlled allow/deny decision.
// No vendor name here can imply verified physical model/firmware support.
fn catalog_allows(kind: &str, vendor: &str, protocol: &str) -> bool {
    !vendor.is_empty()
        && vendor.len() <= 64
        && vendor
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
        && matches!(
            (kind, protocol),
            ("olt", "ssh" | "snmp")
                | ("ont", "cwmp" | "usp")
                | ("router", "ssh" | "snmp" | "routeros_api_ssl")
        )
}
fn html_headers() -> HeaderMap {
    let mut h = safe_headers();
    h.insert("content-security-policy", HeaderValue::from_static(
        "default-src 'none'; base-uri 'none'; frame-ancestors 'none'; object-src 'none'; form-action 'self'; connect-src 'self'; style-src 'self'; script-src 'self'"));
    h.insert("x-frame-options", HeaderValue::from_static("DENY"));
    h
}
async fn login_page() -> Response {
    (StatusCode::OK,html_headers(),Html("<!doctype html><html lang=\"en-US\"><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width\"><title>IPAT sign in</title><h1>IPAT company sign in</h1><p>Use your company's approved identity provider and multi-factor authentication.</p><a href=\"/auth/oidc/start\">Sign in securely</a></html>")).into_response()
}
async fn dashboard(State(s): State<Arc<StateData>>, headers: HeaderMap) -> Response {
    if actor(&s.db, &headers, false).await.is_none() {
        return (
            StatusCode::UNAUTHORIZED,
            html_headers(),
            "Sign in at /auth/oidc/start with your approved company account.",
        )
            .into_response();
    }
    (StatusCode::OK, html_headers(), Html(DASHBOARD_HTML)).into_response()
}
async fn js_asset() -> Response {
    let mut h = html_headers();
    h.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/javascript; charset=utf-8"),
    );
    (StatusCode::OK, h, DASHBOARD_JS).into_response()
}
async fn css_asset() -> Response {
    let mut h = html_headers();
    h.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/css; charset=utf-8"),
    );
    (StatusCode::OK, h, DASHBOARD_CSS).into_response()
}
async fn capabilities(
    State(s): State<Arc<StateData>>,
    headers: HeaderMap,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let Some(a) = actor(&s.db, &headers, false).await else {
        return denied(StatusCode::UNAUTHORIZED);
    };
    let row =
        s.db.query_one(
            "SELECT ipat_platform.tenant_admin_ui_capability($1,$2,$3::uuid)",
            &[&a.issuer, &a.subject, &a.tenant_id],
        )
        .await;
    let Ok(row) = row else {
        return denied(StatusCode::SERVICE_UNAVAILABLE);
    };
    let admin: bool = row.get(0);
    let scoped =
        s.db.query(
            "SELECT pop_id FROM ipat_platform.current_noc_pop_scopes($1,$2,$3::uuid)",
            &[&a.issuer, &a.subject, &a.tenant_id],
        )
        .await;
    let Ok(scoped) = scoped else {
        return denied(StatusCode::SERVICE_UNAVAILABLE);
    };
    let noc_pops = scoped
        .into_iter()
        .map(|r| r.get::<_, String>(0))
        .collect::<Vec<_>>();
    // A separate typed REAL POP grant never inherits the legacy exact-Site
    // identity_pop_grants column. Failure is deny-all, not legacy fallback.
    let real_scoped =
        s.db.query(
            "SELECT pop_code FROM ipat_platform.current_noc_real_pop_scopes($1,$2,$3::uuid)",
            &[&a.issuer, &a.subject, &a.tenant_id],
        )
        .await;
    let Ok(real_scoped) = real_scoped else {
        return denied(StatusCode::SERVICE_UNAVAILABLE);
    };
    let real_noc_pops = real_scoped
        .into_iter()
        .map(|r| r.get::<_, String>(0))
        .collect::<Vec<_>>();
    let can_read_noc = !noc_pops.is_empty() || !real_noc_pops.is_empty();
    response(
        StatusCode::OK,
        json!({"ok":true,"tenant_id":a.tenant_id.to_string(),
        "can_manage_sites":admin,"can_manage_pops":admin,
        "can_manage_devices":admin,"can_manage_domains":admin,
        "noc_pops":noc_pops,"real_noc_pops":real_noc_pops,
        "can_read_noc":can_read_noc}),
    )
}
async fn device_catalog(
    State(s): State<Arc<StateData>>,
    headers: HeaderMap,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let Some(a) = actor(&s.db, &headers, false).await else {
        return denied(StatusCode::UNAUTHORIZED);
    };
    let row =
        s.db.query_one(
            "SELECT ipat_platform.tenant_admin_ui_capability($1,$2,$3::uuid)",
            &[&a.issuer, &a.subject, &a.tenant_id],
        )
        .await;
    if !row.ok().is_some_and(|r| r.get::<_, bool>(0)) {
        return denied(StatusCode::FORBIDDEN);
    };
    // Current approved tenant membership is checked AGAIN inside the sealed
    // SQL catalog function. A missing migration or revoked role fails closed.
    let rows = s.db.query(
        "SELECT device_kind,vendor,management_transport,qualification,description FROM ipat_platform.list_tenant_vendor_catalog($1,$2,$3::uuid)",
        &[&a.issuer, &a.subject, &a.tenant_id],
    ).await;
    let Ok(rows) = rows else {
        return denied(StatusCode::SERVICE_UNAVAILABLE);
    };
    let mut catalog: Vec<Value> = Vec::new();
    for row in rows {
        let kind: String = row.get(0);
        let vendor: String = row.get(1);
        let transport: String = row.get(2);
        let qualification: String = row.get(3);
        let description: String = row.get(4);
        let item = json!({
            "transport":transport,"qualification":qualification,
            "description":description
        });
        if let Some(existing) = catalog
            .iter_mut()
            .find(|entry| entry["type"] == kind && entry["vendor"] == vendor)
        {
            if let Some(options) = existing["protocols"].as_array_mut() {
                options.push(item);
            }
            if let Some(transports) = existing["transports"].as_array_mut() {
                transports.push(json!(transport));
            }
        } else {
            catalog.push(json!({
                "type":kind,"vendor":vendor,
                "qualification":"metadata_candidate",
                "transports":[transport],"protocols":[item]
            }));
        }
    }
    response(
        StatusCode::OK,
        json!({"ok":true,"catalog":catalog,
        "catalog_semantics":"platform_curated_metadata_only_not_verified_physical_support"}),
    )
}
async fn logout(State(s): State<Arc<StateData>>, headers: HeaderMap) -> Response {
    if actor(&s.db, &headers, true).await.is_none() {
        return denied(StatusCode::UNAUTHORIZED).into_response();
    }
    let Some(h) = host(&headers) else {
        return denied(StatusCode::UNAUTHORIZED).into_response();
    };
    let Some(c) = cookie(&headers) else {
        return denied(StatusCode::UNAUTHORIZED).into_response();
    };
    if !durable_tenant_session::revoke(&s.db, c, &h).await {
        return denied(StatusCode::CONFLICT).into_response();
    }
    let mut out = html_headers();
    out.append(
        header::SET_COOKIE,
        HeaderValue::from_static(
            "__Host-ipat_session=; Secure; HttpOnly; SameSite=Strict; Path=/; Max-Age=0",
        ),
    );
    out.append(
        header::SET_COOKIE,
        HeaderValue::from_static("__Host-ipat_csrf=; Secure; SameSite=Strict; Path=/; Max-Age=0"),
    );
    (
        StatusCode::OK,
        out,
        Json(json!({"ok":true,"signed_out":true})),
    )
        .into_response()
}

#[derive(Clone)]
pub(super) struct StateData {
    pub db: Arc<Client>,
}

fn safe_db_name(v: &str) -> bool {
    !v.is_empty() && v.len() <= 63 && v.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

/// Production connection factory. The process still must bind behind a trusted
/// HTTPS edge on loopback/private transport; this function does not make HTTP public.
pub(super) async fn connect_from_environment() -> Result<Arc<Client>, String> {
    if unsafe { libc::geteuid() } == 0
        || std::env::var("IPAT_PRODUCTION_TENANT_API").as_deref() != Ok("YES")
        || std::env::var("IPAT_TRUSTED_HTTPS_EDGE").as_deref() != Ok("YES")
    {
        return Err("commercial tenant API not explicitly enabled".into());
    }
    let socket =
        std::env::var("IPAT_TENANT_API_DB_SOCKET").map_err(|_| "missing tenant API DB socket")?;
    let database =
        std::env::var("IPAT_TENANT_API_DB_NAME").map_err(|_| "missing tenant API DB name")?;
    let user =
        std::env::var("IPAT_TENANT_API_DB_USER").map_err(|_| "missing tenant API DB user")?;
    if user != "ipat_tenant_api_login" || !safe_db_name(&database) {
        return Err("unexpected tenant API database identity".into());
    }
    let path = PathBuf::from(&socket);
    if !path.is_absolute() || socket.len() > 200 || socket.contains("..") {
        return Err("tenant API DB socket must be absolute and canonical".into());
    }
    let mut cfg = Config::new();
    cfg.host_path(path);
    cfg.user(&user);
    cfg.dbname(&database);
    let (client, connection) = cfg
        .connect(NoTls)
        .await
        .map_err(|_| "tenant API DB unavailable")?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    Ok(Arc::new(client))
}

fn safe_headers() -> HeaderMap {
    let mut h = HeaderMap::new();
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    h.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    h.insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    h
}
fn response(code: StatusCode, body: Value) -> (StatusCode, HeaderMap, Json<Value>) {
    (code, safe_headers(), Json(body))
}
fn denied(code: StatusCode) -> (StatusCode, HeaderMap, Json<Value>) {
    response(code, json!({"ok":false,"error":"REQUEST_REJECTED"}))
}

fn host(headers: &HeaderMap) -> Option<String> {
    if headers.get_all(header::HOST).iter().count() != 1 {
        return None;
    }
    Some(
        canonical_host(headers.get(header::HOST)?)?
            .as_str()
            .to_string(),
    )
}
fn cookie(headers: &HeaderMap) -> Option<&str> {
    if headers.get_all(header::COOKIE).iter().count() != 1 {
        return None;
    }
    let raw = headers.get(header::COOKIE)?.to_str().ok()?;
    let mut found = None;
    for piece in raw.split(';') {
        if let Some(v) = piece.trim().strip_prefix("__Host-ipat_session=") {
            if found.is_some()
                || v.len() != 43
                || !v
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
            {
                return None;
            }
            found = Some(v);
        }
    }
    found
}
fn csrf(headers: &HeaderMap) -> Option<&str> {
    if headers.get_all(&CSRF_HEADER).iter().count() != 1 {
        return None;
    }
    let v = headers.get(&CSRF_HEADER)?.to_str().ok()?;
    (v.len() == 43
        && v.bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_')))
    .then_some(v)
}
fn mutation_origin(headers: &HeaderMap, canonical: &str) -> bool {
    if headers.get_all(header::ORIGIN).iter().count() != 1 {
        return false;
    }
    headers.get(header::ORIGIN).and_then(|v| v.to_str().ok())
        == Some(format!("https://{canonical}").as_str())
}
async fn actor(
    db: &Client,
    headers: &HeaderMap,
    mutation: bool,
) -> Option<durable_tenant_session::DurableSessionContext> {
    let h = host(headers)?;
    let c = cookie(headers)?;
    if mutation && !mutation_origin(headers, &h) {
        return None;
    }
    durable_tenant_session::authenticate(
        db,
        c,
        if mutation { Some(csrf(headers)?) } else { None },
        mutation,
        &h,
    )
    .await
}
fn uuid_v4() -> Option<Uuid> {
    let mut b = [0u8; 16];
    File::open("/dev/urandom").ok()?.read_exact(&mut b).ok()?;
    b[6] = (b[6] & 0x0f) | 0x40;
    b[8] = (b[8] & 0x3f) | 0x80;
    Some(Uuid::from_bytes(b))
}
fn valid_code(v: &str) -> bool {
    !v.is_empty()
        && v.len() <= 128
        && v.bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
}
fn valid_name(v: &str) -> bool {
    !v.is_empty() && v.len() <= 120 && v == v.trim() && !v.chars().any(char::is_control)
}
fn valid_host(v: &str) -> bool {
    (3..=253).contains(&v.len())
        && v.bytes().next().is_some_and(|b| b.is_ascii_alphanumeric())
        && v.bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b':' | b'-'))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SiteCreate {
    code: String,
    display_name: String,
    parent_pop_code: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SiteEdit {
    display_name: String,
    expected_revision: i64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ParentPopAssign {
    parent_pop_code: Option<String>,
    expected_revision: i64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NocPopQuery {
    pop: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NocGrantRequest {
    request_id: String,
    target_issuer: String,
    target_subject: String,
    pop_code: String,
    expires_at: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NocGrantReview {
    approve: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NocGrantRevoke {
    target_issuer: String,
    target_subject: String,
    pop_code: String,
}

async fn tenant_admin_actor(
    db: &Client,
    headers: &HeaderMap,
    mutation: bool,
) -> Result<durable_tenant_session::DurableSessionContext, StatusCode> {
    let a = actor(db, headers, mutation)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let row = db
        .query_one(
            "SELECT ipat_platform.tenant_admin_ui_capability($1,$2,$3::uuid)",
            &[&a.issuer, &a.subject, &a.tenant_id],
        )
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    if !row.get::<_, bool>(0) {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(a)
}

async fn list_noc_access(
    State(s): State<Arc<StateData>>,
    headers: HeaderMap,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let a = match tenant_admin_actor(&s.db, &headers, false).await {
        Ok(a) => a,
        Err(status) => return denied(status),
    };
    let members=s.db.query(
        "SELECT issuer,subject,expires_at::text FROM ipat_platform.list_current_noc_members_for_admin($1,$2,$3::uuid)",
        &[&a.issuer,&a.subject,&a.tenant_id],
    ).await;
    let access=s.db.query(
        "SELECT kind,request_id,target_issuer,target_subject,pop_code,state,expires_at::text,requested_by,reviewed_by FROM ipat_platform.list_noc_real_pop_access_for_admin($1,$2,$3::uuid)",
        &[&a.issuer,&a.subject,&a.tenant_id],
    ).await;
    let (Ok(members), Ok(access)) = (members, access) else {
        return denied(StatusCode::SERVICE_UNAVAILABLE);
    };
    let members = members
        .into_iter()
        .map(|r| {
            json!({
                "issuer":r.get::<_,String>(0),"subject":r.get::<_,String>(1),
                "expires_at":r.get::<_,String>(2)
            })
        })
        .collect::<Vec<_>>();
    let access = access
        .into_iter()
        .map(|r| {
            let kind: String = r.get(0);
            let state: String = r.get(5);
            let requested_by: String = r.get(7);
            let is_requester = requested_by == format!("{}#{}", a.issuer, a.subject);
            json!({
                "kind":kind,
                "request_id":r.get::<_,Option<Uuid>>(1).map(|v|v.to_string()),
                "target_issuer":r.get::<_,String>(2),"target_subject":r.get::<_,String>(3),
                "pop_code":r.get::<_,String>(4),
                "can_review":kind=="request" && state=="PENDING" && !is_requester,
                "state":state,"expires_at":r.get::<_,String>(6),
                "requested_by":requested_by,
                "reviewed_by":r.get::<_,Option<String>>(8)
            })
        })
        .collect::<Vec<_>>();
    response(
        StatusCode::OK,
        json!({"ok":true,"members":members,"access":access}),
    )
}

async fn create_noc_access_request(
    State(s): State<Arc<StateData>>,
    headers: HeaderMap,
    Json(q): Json<NocGrantRequest>,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let a = match tenant_admin_actor(&s.db, &headers, true).await {
        Ok(a) => a,
        Err(status) => return denied(status),
    };
    if !valid_code(&q.pop_code)
        || q.expires_at.len() < 20
        || q.expires_at.len() > 40
        || !q.expires_at.ends_with('Z')
        || q.target_issuer.len() < 10
        || q.target_issuer.len() > 512
        || !q.target_issuer.starts_with("https://")
        || q.target_subject.is_empty()
        || q.target_subject.len() > 128
    {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let Ok(request_id) = Uuid::parse_str(&q.request_id) else {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    };
    let row=s.db.query_one(
        "SELECT ipat_platform.request_noc_real_pop_grant($1,$2,$3::uuid,$4::uuid,$5,$6,$7,($8::text)::timestamptz)",
        &[&a.issuer,&a.subject,&a.tenant_id,&request_id,
          &q.target_issuer,&q.target_subject,&q.pop_code,&q.expires_at],
    ).await;
    match row.ok().and_then(|r| r.get::<_, Option<Uuid>>(0)) {
        Some(id) => response(
            StatusCode::CREATED,
            json!({"ok":true,"request_id":id.to_string()}),
        ),
        None => denied(StatusCode::CONFLICT),
    }
}

async fn review_noc_access_request(
    State(s): State<Arc<StateData>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(q): Json<NocGrantReview>,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let a = match tenant_admin_actor(&s.db, &headers, true).await {
        Ok(a) => a,
        Err(status) => return denied(status),
    };
    let Ok(id) = Uuid::parse_str(&id) else {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    };
    let row =
        s.db.query_one(
            "SELECT ipat_platform.review_noc_real_pop_grant($1,$2,$3::uuid,$4::uuid,$5)",
            &[&a.issuer, &a.subject, &a.tenant_id, &id, &q.approve],
        )
        .await;
    match row.ok().and_then(|r| r.get::<_, Option<String>>(0)) {
        Some(state) => response(StatusCode::OK, json!({"ok":true,"state":state})),
        None => denied(StatusCode::CONFLICT),
    }
}

async fn revoke_noc_access(
    State(s): State<Arc<StateData>>,
    headers: HeaderMap,
    Json(q): Json<NocGrantRevoke>,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let a = match tenant_admin_actor(&s.db, &headers, true).await {
        Ok(a) => a,
        Err(status) => return denied(status),
    };
    if !valid_code(&q.pop_code) || q.target_issuer.is_empty() || q.target_subject.is_empty() {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let row =
        s.db.query_one(
            "SELECT ipat_platform.revoke_noc_real_pop_grant($1,$2,$3::uuid,$4,$5,$6)",
            &[
                &a.issuer,
                &a.subject,
                &a.tenant_id,
                &q.target_issuer,
                &q.target_subject,
                &q.pop_code,
            ],
        )
        .await;
    if row.ok().is_some_and(|r| r.get::<_, bool>(0)) {
        response(StatusCode::OK, json!({"ok":true,"revoked":true}))
    } else {
        denied(StatusCode::CONFLICT)
    }
}

async fn noc_actor_for_pop(
    db: &Client,
    headers: &HeaderMap,
    pop: &str,
) -> Result<durable_tenant_session::DurableSessionContext, StatusCode> {
    let a = actor(db, headers, false)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;
    if !valid_code(pop) {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let rows = db
        .query(
            "SELECT pop_id FROM ipat_platform.current_noc_pop_scopes($1,$2,$3::uuid)",
            &[&a.issuer, &a.subject, &a.tenant_id],
        )
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    if !rows.iter().any(|row| row.get::<_, &str>(0) == pop) {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(a)
}

async fn noc_sites(
    State(s): State<Arc<StateData>>,
    headers: HeaderMap,
    Query(q): Query<NocPopQuery>,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let a = match noc_actor_for_pop(&s.db, &headers, &q.pop).await {
        Ok(a) => a,
        Err(status) => return denied(status),
    };
    let rows = s.db.query(
        "SELECT code,display_name,revision,assigned_devices FROM ipat_platform.list_tenant_sites($1,$2,$3::uuid,$4,NULL)",
        &[&a.issuer, &a.subject, &a.tenant_id, &q.pop],
    ).await;
    let Ok(rows) = rows else {
        return denied(StatusCode::SERVICE_UNAVAILABLE);
    };
    let sites = rows
        .into_iter()
        .map(|r| {
            json!({
                "code":r.get::<_,String>(0),"display_name":r.get::<_,String>(1),
                "revision":r.get::<_,i64>(2),"assigned_devices":r.get::<_,i64>(3)
            })
        })
        .collect::<Vec<_>>();
    response(
        StatusCode::OK,
        json!({"ok":true,"sites":sites,"read_only":true}),
    )
}

async fn noc_devices(
    State(s): State<Arc<StateData>>,
    headers: HeaderMap,
    Query(q): Query<NocPopQuery>,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let a = match noc_actor_for_pop(&s.db, &headers, &q.pop).await {
        Ok(a) => a,
        Err(status) => return denied(status),
    };
    let rows = s.db.query(
        "SELECT id,pop_id,display_name,device_kind,vendor,intended_model,management_transport,lifecycle_state FROM ipat_platform.list_managed_devices($1,$2,$3::uuid,$4)",
        &[&a.issuer, &a.subject, &a.tenant_id, &q.pop],
    ).await;
    let Ok(rows) = rows else {
        return denied(StatusCode::SERVICE_UNAVAILABLE);
    };
    let devices = rows
        .into_iter()
        .map(|r| {
            json!({
                "id":r.get::<_,Uuid>(0).to_string(),"site":r.get::<_,String>(1),
                "display_name":r.get::<_,String>(2),"device_kind":r.get::<_,String>(3),
                "vendor":r.get::<_,String>(4),"intended_model":r.get::<_,Option<String>>(5),
                "management_transport":r.get::<_,String>(6),"lifecycle_state":r.get::<_,String>(7)
            })
        })
        .collect::<Vec<_>>();
    response(
        StatusCode::OK,
        json!({"ok":true,"devices":devices,"read_only":true}),
    )
}

// True POP-scoped NOC read path, explicitly distinct from historical exact-
// Site grants. Both HTTP authorization AND SQL functions independently check
// the current typed grant, current approved NOC membership and active tenant.
async fn noc_real_actor_for_pop(
    db: &Client,
    headers: &HeaderMap,
    pop: &str,
) -> Result<durable_tenant_session::DurableSessionContext, StatusCode> {
    let a = actor(db, headers, false)
        .await
        .ok_or(StatusCode::UNAUTHORIZED)?;
    if !valid_code(pop) {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let rows = db
        .query(
            "SELECT pop_code FROM ipat_platform.current_noc_real_pop_scopes($1,$2,$3::uuid)",
            &[&a.issuer, &a.subject, &a.tenant_id],
        )
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    if !rows.iter().any(|r| r.get::<_, &str>(0) == pop) {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(a)
}
async fn noc_real_sites(
    State(s): State<Arc<StateData>>,
    headers: HeaderMap,
    Query(q): Query<NocPopQuery>,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let a = match noc_real_actor_for_pop(&s.db, &headers, &q.pop).await {
        Ok(a) => a,
        Err(status) => return denied(status),
    };
    let rows=s.db.query(
  "SELECT code,display_name,revision,assigned_devices FROM ipat_platform.list_noc_real_pop_sites($1,$2,$3::uuid,$4)",
  &[&a.issuer,&a.subject,&a.tenant_id,&q.pop]
 ).await;
    let Ok(rows) = rows else {
        return denied(StatusCode::SERVICE_UNAVAILABLE);
    };
    let sites = rows
        .into_iter()
        .map(|r| {
            json!({
             "code":r.get::<_,String>(0),"display_name":r.get::<_,String>(1),
             "revision":r.get::<_,i64>(2),"assigned_devices":r.get::<_,i64>(3)
            })
        })
        .collect::<Vec<_>>();
    response(
        StatusCode::OK,
        json!({"ok":true,"sites":sites,
  "read_only":true,"scope_type":"real_pop"}),
    )
}
async fn noc_real_devices(
    State(s): State<Arc<StateData>>,
    headers: HeaderMap,
    Query(q): Query<NocPopQuery>,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let a = match noc_real_actor_for_pop(&s.db, &headers, &q.pop).await {
        Ok(a) => a,
        Err(status) => return denied(status),
    };
    let rows=s.db.query(
  "SELECT id,site_code,display_name,device_kind,vendor,intended_model,management_transport,lifecycle_state FROM ipat_platform.list_noc_real_pop_devices($1,$2,$3::uuid,$4)",
  &[&a.issuer,&a.subject,&a.tenant_id,&q.pop]
 ).await;
    let Ok(rows) = rows else {
        return denied(StatusCode::SERVICE_UNAVAILABLE);
    };
    let devices = rows
        .into_iter()
        .map(|r| {
            json!({
             "id":r.get::<_,Uuid>(0).to_string(),"site":r.get::<_,String>(1),
             "display_name":r.get::<_,String>(2),"device_kind":r.get::<_,String>(3),
             "vendor":r.get::<_,String>(4),"intended_model":r.get::<_,Option<String>>(5),
             "management_transport":r.get::<_,String>(6),
             "lifecycle_state":r.get::<_,String>(7)
            })
        })
        .collect::<Vec<_>>();
    response(
        StatusCode::OK,
        json!({"ok":true,"devices":devices,
  "read_only":true,"scope_type":"real_pop"}),
    )
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ListSites {
    pop: Option<String>,
    after: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Revision {
    revision: i64,
}
async fn list_sites(
    State(s): State<Arc<StateData>>,
    headers: HeaderMap,
    Query(q): Query<ListSites>,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let Some(a) = actor(&s.db, &headers, false).await else {
        return denied(StatusCode::UNAUTHORIZED);
    };
    if q.pop.as_deref().is_some_and(|v| !valid_code(v))
        || q.after.as_deref().is_some_and(|v| !valid_code(v))
    {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let rows=s.db.query("SELECT code,display_name,revision,assigned_devices,parent_pop_code FROM ipat_platform.list_tenant_sites_with_parent_pop($1,$2,$3::uuid,$4,$5)",&[&a.issuer,&a.subject,&a.tenant_id,&q.pop.as_deref(),&q.after.as_deref()]).await;
    let Ok(rows) = rows else {
        return denied(StatusCode::SERVICE_UNAVAILABLE);
    };
    if rows.len() > 101 {
        return denied(StatusCode::SERVICE_UNAVAILABLE);
    }
    let has_more = rows.len() == 101;
    let mut out = Vec::new();
    for r in rows.into_iter().take(100) {
        out.push(json!({"code":r.get::<_,String>(0),"display_name":r.get::<_,String>(1),"revision":r.get::<_,i64>(2),"assigned_devices":r.get::<_,i64>(3),"parent_pop_code":r.get::<_,Option<String>>(4)}));
    }
    let next = if has_more {
        out.last().and_then(|v| v.get("code")).cloned()
    } else {
        None
    };
    response(
        StatusCode::OK,
        json!({"ok":true,"sites":out,"next_after":next}),
    )
}
async fn create_site(
    State(s): State<Arc<StateData>>,
    headers: HeaderMap,
    Json(q): Json<SiteCreate>,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let Some(a) = actor(&s.db, &headers, true).await else {
        return denied(StatusCode::UNAUTHORIZED);
    };
    if !valid_code(&q.code)
        || !valid_name(&q.display_name)
        || q.parent_pop_code.as_deref().is_some_and(|v| !valid_code(v))
    {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let r =
        s.db.query_one(
            "SELECT ipat_platform.create_tenant_site_with_pop($1,$2,$3::uuid,$4,$5,$6)",
            &[
                &a.issuer,
                &a.subject,
                &a.tenant_id,
                &q.code,
                &q.display_name,
                &q.parent_pop_code,
            ],
        )
        .await;
    match r.ok().and_then(|r| r.get::<_, Option<String>>(0)) {
        Some(code) => response(StatusCode::CREATED, json!({"ok":true,"code":code})),
        None => denied(StatusCode::CONFLICT),
    }
}
async fn edit_site(
    State(s): State<Arc<StateData>>,
    headers: HeaderMap,
    Path(code): Path<String>,
    Json(q): Json<SiteEdit>,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let Some(a) = actor(&s.db, &headers, true).await else {
        return denied(StatusCode::UNAUTHORIZED);
    };
    if !valid_code(&code) || !valid_name(&q.display_name) || q.expected_revision < 1 {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let r =
        s.db.query_one(
            "SELECT ipat_platform.rename_tenant_site($1,$2,$3::uuid,$4,$5,$6::bigint)",
            &[
                &a.issuer,
                &a.subject,
                &a.tenant_id,
                &code,
                &q.display_name,
                &q.expected_revision,
            ],
        )
        .await;
    match r.ok().and_then(|r| r.get::<_, Option<i64>>(0)) {
        Some(v) => response(StatusCode::OK, json!({"ok":true,"revision":v})),
        None => denied(StatusCode::CONFLICT),
    }
}
async fn delete_site(
    State(s): State<Arc<StateData>>,
    headers: HeaderMap,
    Path(code): Path<String>,
    Query(q): Query<Revision>,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let Some(a) = actor(&s.db, &headers, true).await else {
        return denied(StatusCode::UNAUTHORIZED);
    };
    if !valid_code(&code) || q.revision < 1 {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let r =
        s.db.query_one(
            "SELECT ipat_platform.delete_tenant_site($1,$2,$3::uuid,$4,$5::bigint)",
            &[&a.issuer, &a.subject, &a.tenant_id, &code, &q.revision],
        )
        .await;
    if r.ok().is_some_and(|r| r.get::<_, bool>(0)) {
        response(StatusCode::OK, json!({"ok":true}))
    } else {
        denied(StatusCode::CONFLICT)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PopPage {
    after: Option<String>,
}

async fn list_pops(
    State(s): State<Arc<StateData>>,
    headers: HeaderMap,
    Query(q): Query<PopPage>,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let Some(a) = actor(&s.db, &headers, false).await else {
        return denied(StatusCode::UNAUTHORIZED);
    };
    if q.after.as_deref().is_some_and(|v| !valid_code(v)) {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let admin =
        s.db.query_one(
            "SELECT ipat_platform.tenant_admin_ui_capability($1,$2,$3::uuid)",
            &[&a.issuer, &a.subject, &a.tenant_id],
        )
        .await;
    match admin {
        Ok(row) if !row.get::<_, bool>(0) => return denied(StatusCode::FORBIDDEN),
        Err(_) => return denied(StatusCode::SERVICE_UNAVAILABLE),
        _ => {}
    }
    let rows=s.db.query(
        "SELECT code,display_name,revision,assigned_sites FROM ipat_platform.list_tenant_pops($1,$2,$3::uuid,$4)",
        &[&a.issuer,&a.subject,&a.tenant_id,&q.after.as_deref()],
    ).await;
    let Ok(rows) = rows else {
        return denied(StatusCode::SERVICE_UNAVAILABLE);
    };
    if rows.len() > 101 {
        return denied(StatusCode::SERVICE_UNAVAILABLE);
    }
    let more = rows.len() == 101;
    let mut pops = Vec::new();
    for r in rows.into_iter().take(100) {
        pops.push(json!({"code":r.get::<_,String>(0),
            "display_name":r.get::<_,String>(1),
            "revision":r.get::<_,i64>(2),
            "assigned_sites":r.get::<_,i64>(3)}));
    }
    let next = if more {
        pops.last().and_then(|v| v.get("code")).cloned()
    } else {
        None
    };
    response(
        StatusCode::OK,
        json!({"ok":true,"pops":pops,"next_after":next}),
    )
}
async fn create_pop(
    State(s): State<Arc<StateData>>,
    headers: HeaderMap,
    Json(q): Json<SiteCreate>,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let Some(a) = actor(&s.db, &headers, true).await else {
        return denied(StatusCode::UNAUTHORIZED);
    };
    if !valid_code(&q.code) || !valid_name(&q.display_name) || q.parent_pop_code.is_some() {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let r =
        s.db.query_one(
            "SELECT ipat_platform.create_tenant_pop($1,$2,$3::uuid,$4,$5)",
            &[
                &a.issuer,
                &a.subject,
                &a.tenant_id,
                &q.code,
                &q.display_name,
            ],
        )
        .await;
    match r.ok().and_then(|r| r.get::<_, Option<String>>(0)) {
        Some(code) => response(StatusCode::CREATED, json!({"ok":true,"code":code})),
        None => denied(StatusCode::CONFLICT),
    }
}
async fn edit_pop(
    State(s): State<Arc<StateData>>,
    headers: HeaderMap,
    Path(code): Path<String>,
    Json(q): Json<SiteEdit>,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let Some(a) = actor(&s.db, &headers, true).await else {
        return denied(StatusCode::UNAUTHORIZED);
    };
    if !valid_code(&code) || !valid_name(&q.display_name) || q.expected_revision < 1 {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let r =
        s.db.query_one(
            "SELECT ipat_platform.rename_tenant_pop($1,$2,$3::uuid,$4,$5,$6::bigint)",
            &[
                &a.issuer,
                &a.subject,
                &a.tenant_id,
                &code,
                &q.display_name,
                &q.expected_revision,
            ],
        )
        .await;
    match r.ok().and_then(|r| r.get::<_, Option<i64>>(0)) {
        Some(revision) => response(StatusCode::OK, json!({"ok":true,"revision":revision})),
        None => denied(StatusCode::CONFLICT),
    }
}
async fn delete_pop(
    State(s): State<Arc<StateData>>,
    headers: HeaderMap,
    Path(code): Path<String>,
    Query(q): Query<Revision>,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let Some(a) = actor(&s.db, &headers, true).await else {
        return denied(StatusCode::UNAUTHORIZED);
    };
    if !valid_code(&code) || q.revision < 1 {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let r =
        s.db.query_one(
            "SELECT ipat_platform.delete_unused_tenant_pop($1,$2,$3::uuid,$4,$5::bigint)",
            &[&a.issuer, &a.subject, &a.tenant_id, &code, &q.revision],
        )
        .await;
    if r.ok().is_some_and(|r| r.get::<_, bool>(0)) {
        response(StatusCode::OK, json!({"ok":true}))
    } else {
        denied(StatusCode::CONFLICT)
    }
}
async fn assign_site_pop(
    State(s): State<Arc<StateData>>,
    headers: HeaderMap,
    Path(code): Path<String>,
    Json(q): Json<ParentPopAssign>,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let Some(a) = actor(&s.db, &headers, true).await else {
        return denied(StatusCode::UNAUTHORIZED);
    };
    if !valid_code(&code)
        || q.expected_revision < 1
        || q.parent_pop_code.as_deref().is_some_and(|p| !valid_code(p))
    {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let r =
        s.db.query_one(
            "SELECT ipat_platform.assign_tenant_site_to_pop($1,$2,$3::uuid,$4,$5,$6::bigint)",
            &[
                &a.issuer,
                &a.subject,
                &a.tenant_id,
                &code,
                &q.parent_pop_code,
                &q.expected_revision,
            ],
        )
        .await;
    match r.ok().and_then(|r| r.get::<_, Option<i64>>(0)) {
        Some(revision) => response(StatusCode::OK, json!({"ok":true,"revision":revision})),
        None => denied(StatusCode::CONFLICT),
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeviceCreate {
    request_id: String,
    pop_id: String,
    display_name: String,
    device_kind: String,
    vendor: String,
    intended_model: Option<String>,
    management_transport: String,
    management_host: Option<String>,
    management_port: Option<i32>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeviceEdit {
    expected_revision: i64,
    pop_id: String,
    display_name: String,
    intended_model: Option<String>,
    management_host: Option<String>,
    management_port: Option<i32>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeviceList {
    pop: Option<String>,
}
fn valid_device_create(v: &DeviceCreate) -> bool {
    valid_code(&v.pop_id)
        && catalog_allows(&v.device_kind, &v.vendor, &v.management_transport)
        && valid_name(&v.display_name)
        && matches!(v.device_kind.as_str(), "olt" | "ont" | "router")
        && !v.vendor.is_empty()
        && v.vendor.len() <= 64
        && v.vendor
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
        && matches!(
            v.management_transport.as_str(),
            "ssh" | "snmp" | "routeros_api_ssl" | "cwmp" | "usp"
        )
        && match v.management_transport.as_str() {
            "cwmp" | "usp" => v.management_host.is_none() && v.management_port.is_none(),
            _ => {
                v.management_host.as_deref().is_some_and(valid_host)
                    && v.management_port.is_some_and(|p| (1..=65535).contains(&p))
            }
        }
}
async fn list_devices(
    State(s): State<Arc<StateData>>,
    headers: HeaderMap,
    Query(q): Query<DeviceList>,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let Some(a) = actor(&s.db, &headers, false).await else {
        return denied(StatusCode::UNAUTHORIZED);
    };
    if q.pop.as_deref().is_some_and(|v| !valid_code(v)) {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let rows=s.db.query("SELECT id,pop_id,display_name,device_kind,vendor,intended_model,management_transport,lifecycle_state FROM ipat_platform.list_managed_devices($1,$2,$3::uuid,$4)",&[&a.issuer,&a.subject,&a.tenant_id,&q.pop.as_deref()]).await;
    let Ok(rows) = rows else {
        return denied(StatusCode::SERVICE_UNAVAILABLE);
    };
    let out=rows.into_iter().map(|r|json!({"id":r.get::<_,Uuid>(0).to_string(),"site":r.get::<_,String>(1),"display_name":r.get::<_,String>(2),"device_kind":r.get::<_,String>(3),"vendor":r.get::<_,String>(4),"intended_model":r.get::<_,Option<String>>(5),"management_transport":r.get::<_,String>(6),"lifecycle_state":r.get::<_,String>(7)})).collect::<Vec<_>>();
    response(StatusCode::OK, json!({"ok":true,"devices":out}))
}
async fn create_device(
    State(s): State<Arc<StateData>>,
    headers: HeaderMap,
    Json(q): Json<DeviceCreate>,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let Some(a) = actor(&s.db, &headers, true).await else {
        return denied(StatusCode::UNAUTHORIZED);
    };
    if !valid_device_create(&q) {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    };
    // Form and HTTP admission use the SAME current platform-maintained
    // catalog. A database trigger rechecks the row at transaction time so
    // concurrent catalog disable cannot create a stale admission bypass.
    let current_catalog = s.db.query(
        "SELECT 1 FROM ipat_platform.list_tenant_vendor_catalog($1,$2,$3::uuid) WHERE device_kind=$4 AND vendor=$5 AND management_transport=$6 LIMIT 1",
        &[&a.issuer, &a.subject, &a.tenant_id, &q.device_kind,
          &q.vendor, &q.management_transport],
    ).await;
    match current_catalog {
        Ok(rows) if rows.is_empty() => return denied(StatusCode::UNPROCESSABLE_ENTITY),
        Err(_) => return denied(StatusCode::SERVICE_UNAVAILABLE),
        _ => {}
    };
    let Ok(request_id) = Uuid::parse_str(&q.request_id) else {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    };
    let Some(id) = uuid_v4() else {
        return denied(StatusCode::SERVICE_UNAVAILABLE);
    };
    let r=s.db.query_one("SELECT ipat_platform.register_managed_device($1,$2,$3::uuid,$4::uuid,$5::uuid,$6,$7,$8,$9,$10,$11,$12,$13,NULL)",&[&a.issuer,&a.subject,&a.tenant_id,&id,&request_id,&q.pop_id,&q.display_name,&q.device_kind,&q.vendor,&q.intended_model.as_deref(),&q.management_transport,&q.management_host.as_deref(),&q.management_port]).await;
    match r.ok().and_then(|r| r.get::<_, Option<Uuid>>(0)) {
        Some(saved) => response(
            StatusCode::CREATED,
            json!({"ok":true,"id":saved.to_string(),"lifecycle_state":"SAVED"}),
        ),
        None => denied(StatusCode::CONFLICT),
    }
}
async fn device_detail(
    State(s): State<Arc<StateData>>,
    headers: HeaderMap,
    Path(id_raw): Path<String>,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let Some(a) = actor(&s.db, &headers, false).await else {
        return denied(StatusCode::UNAUTHORIZED);
    };
    let Ok(id) = Uuid::parse_str(&id_raw) else {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    };
    let r=s.db.query_opt("SELECT id,pop_id,display_name,device_kind,vendor,intended_model,management_transport,management_host,management_port,lifecycle_state,metadata_revision,created_at::text,archived_at::text,archived_site_instance_id,archived_site_name FROM ipat_platform.get_managed_device_metadata($1,$2,$3::uuid,$4::uuid)",&[&a.issuer,&a.subject,&a.tenant_id,&id]).await;
    let Ok(Some(r)) = r else {
        return denied(StatusCode::NOT_FOUND);
    };
    response(
        StatusCode::OK,
        json!({"ok":true,"device":{"id":r.get::<_,Uuid>(0).to_string(),"site":r.get::<_,String>(1),"display_name":r.get::<_,String>(2),"device_kind":r.get::<_,String>(3),"vendor":r.get::<_,String>(4),"intended_model":r.get::<_,Option<String>>(5),"management_transport":r.get::<_,String>(6),"management_host":r.get::<_,Option<String>>(7),"management_port":r.get::<_,Option<i32>>(8),"lifecycle_state":r.get::<_,String>(9),"revision":r.get::<_,i64>(10),"created_at":r.get::<_,String>(11),"archived_at":r.get::<_,Option<String>>(12),"archived_site_instance_id":r.get::<_,Option<Uuid>>(13).map(|v|v.to_string()),"archived_site_name":r.get::<_,Option<String>>(14)}}),
    )
}
async fn edit_device(
    State(s): State<Arc<StateData>>,
    headers: HeaderMap,
    Path(id_raw): Path<String>,
    Json(q): Json<DeviceEdit>,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let Some(a) = actor(&s.db, &headers, true).await else {
        return denied(StatusCode::UNAUTHORIZED);
    };
    let Ok(id) = Uuid::parse_str(&id_raw) else {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    };
    if q.expected_revision < 1
        || !valid_code(&q.pop_id)
        || !valid_name(&q.display_name)
        || !match (q.management_host.as_deref(), q.management_port) {
            (None, None) => true,
            (Some(h), Some(p)) => valid_host(h) && (1..=65535).contains(&p),
            _ => false,
        }
    {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let r=s.db.query_one("SELECT ipat_platform.edit_managed_device_metadata($1,$2,$3::uuid,$4::uuid,$5::bigint,$6,$7,$8,$9,$10)",&[&a.issuer,&a.subject,&a.tenant_id,&id,&q.expected_revision,&q.display_name,&q.pop_id,&q.intended_model.as_deref(),&q.management_host.as_deref(),&q.management_port]).await;
    match r.ok().and_then(|r| r.get::<_, Option<i64>>(0)) {
        Some(v) => response(StatusCode::OK, json!({"ok":true,"revision":v})),
        None => denied(StatusCode::CONFLICT),
    }
}
async fn delete_device(
    State(s): State<Arc<StateData>>,
    headers: HeaderMap,
    Path(id_raw): Path<String>,
    Query(q): Query<Revision>,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let Some(a) = actor(&s.db, &headers, true).await else {
        return denied(StatusCode::UNAUTHORIZED);
    };
    let Ok(id) = Uuid::parse_str(&id_raw) else {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    };
    if q.revision < 1 {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let r=s.db.query_one("SELECT ipat_platform.archive_managed_device_metadata($1,$2,$3::uuid,$4::uuid,$5::bigint)",&[&a.issuer,&a.subject,&a.tenant_id,&id,&q.revision]).await;
    if r.ok().is_some_and(|r| r.get::<_, bool>(0)) {
        response(StatusCode::OK, json!({"ok":true,"archived":true}))
    } else {
        denied(StatusCode::CONFLICT)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DomainCreate {
    hostname: String,
    routing_mode: String,
}
async fn list_domains(
    State(s): State<Arc<StateData>>,
    headers: HeaderMap,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let Some(a) = actor(&s.db, &headers, false).await else {
        return denied(StatusCode::UNAUTHORIZED);
    };
    let rows=s.db.query("SELECT id,hostname,routing_mode,verification_name,verification_value,activation_state,ownership_verified_at::text,routing_ready_at::text,tls_ready_at::text,activated_at::text,last_checked_at::text,last_error_code FROM ipat_platform.list_tenant_domains_for_member($1,$2,$3::uuid)",&[&a.issuer,&a.subject,&a.tenant_id]).await;
    let Ok(rows) = rows else {
        return denied(StatusCode::SERVICE_UNAVAILABLE);
    };
    let out=rows.into_iter().map(|r|json!({"id":r.get::<_,Uuid>(0).to_string(),"hostname":r.get::<_,String>(1),"routing_mode":r.get::<_,Option<String>>(2),"verification_name":r.get::<_,Option<String>>(3),"verification_value":r.get::<_,Option<String>>(4),"activation_state":r.get::<_,String>(5),"ownership_verified_at":r.get::<_,Option<String>>(6),"routing_ready_at":r.get::<_,Option<String>>(7),"tls_ready_at":r.get::<_,Option<String>>(8),"activated_at":r.get::<_,Option<String>>(9),"last_checked_at":r.get::<_,Option<String>>(10),"last_error_code":r.get::<_,Option<String>>(11)})).collect::<Vec<_>>();
    response(StatusCode::OK, json!({"ok":true,"domains":out}))
}
async fn create_domain(
    State(s): State<Arc<StateData>>,
    headers: HeaderMap,
    Json(q): Json<DomainCreate>,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let Some(a) = actor(&s.db, &headers, true).await else {
        return denied(StatusCode::UNAUTHORIZED);
    };
    let Some(h) = canonical_dns_name(&q.hostname) else {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    };
    if !valid_custom_domain(&h)
        || !matches!(q.routing_mode.as_str(), "a_record" | "cname" | "nameserver")
    {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    };
    let Some(id) = uuid_v4() else {
        return denied(StatusCode::SERVICE_UNAVAILABLE);
    };
    let value = format!("ipat-domain={id}");
    let r =
        s.db.query_one(
            "SELECT ipat_platform.request_tenant_custom_domain($1,$2,$3::uuid,$4::uuid,$5,$6,$7)",
            &[
                &a.issuer,
                &a.subject,
                &a.tenant_id,
                &id,
                &h.as_str(),
                &q.routing_mode,
                &value,
            ],
        )
        .await;
    match r.ok().and_then(|r| r.get::<_, Option<Uuid>>(0)) {
        Some(saved) => response(
            StatusCode::CREATED,
            json!({"ok":true,"id":saved.to_string(),"hostname":h.as_str(),"verification_name":format!("_ipat-verify.{}",h.as_str()),"verification_value":value,"activation_state":"pending_dns"}),
        ),
        None => denied(StatusCode::CONFLICT),
    }
}
// The customer sees DNS instructions ONLY for a domain they already saved
// inside their current authenticated tenant. Never invent TXT tokens.
async fn customer_domain_dns(
    State(s): State<Arc<StateData>>,
    headers: HeaderMap,
    Path(id_raw): Path<String>,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let Some(a) = actor(&s.db, &headers, false).await else {
        return denied(StatusCode::UNAUTHORIZED);
    };
    let Ok(id) = Uuid::parse_str(&id_raw) else {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    };
    let rows =
        s.db.query(
            "SELECT id,hostname,routing_mode,verification_name,verification_value,activation_state
         FROM ipat_platform.list_tenant_domains_for_member($1,$2,$3::uuid)",
            &[&a.issuer, &a.subject, &a.tenant_id],
        )
        .await;
    let Ok(rows) = rows else {
        return denied(StatusCode::SERVICE_UNAVAILABLE);
    };
    let Some(row) = rows.iter().find(|r| r.get::<_, Uuid>(0) == id) else {
        return denied(StatusCode::NOT_FOUND);
    };
    let hostname: String = row.get(1);
    let mode: Option<String> = row.get(2);
    let verification_name: Option<String> = row.get(3);
    let verification_value: Option<String> = row.get(4);
    let activation_state: String = row.get(5);
    let (Some(mode), Some(verification_name), Some(verification_value)) =
        (mode, verification_name, verification_value)
    else {
        return denied(StatusCode::NOT_FOUND);
    };
    let Some(host) = canonical_dns_name(&hostname) else {
        return denied(StatusCode::SERVICE_UNAVAILABLE);
    };
    if !valid_custom_domain(&host) || activation_state == "disabled" {
        return denied(StatusCode::NOT_FOUND);
    }
    let Ok(profile) = dns_profile_from_environment() else {
        // No made-up IP or dependency on an unowned main brand domain.
        return denied(StatusCode::SERVICE_UNAVAILABLE);
    };
    let edge_reviewed =
        std::env::var("IPAT_R982_CUSTOM_DOMAIN_TLS_EDGE_READY").as_deref() == Ok("YES");
    let Some(mut guide) =
        saved_customer_dns_guidance(&profile, host, &mode, &activation_state, edge_reviewed)
    else {
        return denied(StatusCode::SERVICE_UNAVAILABLE);
    };
    guide["verification_record_name"] = json!(verification_name);
    guide["verification_record_type"] = json!("TXT");
    guide["verification_value"] = json!(verification_value);
    guide["verification_value_issued_after_save"] = json!(true);
    guide["saved_domain_id"] = json!(id.to_string());
    response(StatusCode::OK, json!({"ok":true,"guide":guide}))
}

async fn delete_domain(
    State(s): State<Arc<StateData>>,
    headers: HeaderMap,
    Path(id_raw): Path<String>,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let Some(a) = actor(&s.db, &headers, true).await else {
        return denied(StatusCode::UNAUTHORIZED);
    };
    let Ok(id) = Uuid::parse_str(&id_raw) else {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    };
    let r =
        s.db.query_one(
            "SELECT ipat_platform.disable_tenant_custom_domain($1,$2,$3::uuid,$4::uuid)",
            &[&a.issuer, &a.subject, &a.tenant_id, &id],
        )
        .await;
    if r.ok().is_some_and(|r| r.get::<_, bool>(0)) {
        response(StatusCode::OK, json!({"ok":true,"disabled":true}))
    } else {
        denied(StatusCode::CONFLICT)
    }
}

pub(super) fn router(db: Arc<Client>) -> Router {
    let state = Arc::new(StateData { db });
    Router::new()
        .route("/", get(login_page))
        .route("/dashboard", get(dashboard))
        .route("/dashboard/assets/app.js", get(js_asset))
        .route("/dashboard/assets/style.css", get(css_asset))
        .route("/api/v1/capabilities", get(capabilities))
        .route("/api/v1/noc/sites", get(noc_sites))
        .route("/api/v1/noc/devices", get(noc_devices))
        .route("/api/v1/noc/real/sites", get(noc_real_sites))
        .route("/api/v1/noc/real/devices", get(noc_real_devices))
        .route("/api/v1/noc-access", get(list_noc_access))
        .route(
            "/api/v1/noc-access/requests",
            axum::routing::post(create_noc_access_request),
        )
        .route(
            "/api/v1/noc-access/requests/{id}/review",
            axum::routing::post(review_noc_access_request),
        )
        .route(
            "/api/v1/noc-access/revoke",
            axum::routing::post(revoke_noc_access),
        )
        .route("/api/v1/device-catalog", get(device_catalog))
        .route("/api/v1/logout", axum::routing::post(logout))
        .route("/api/v1/pops", get(list_pops).post(create_pop))
        .route(
            "/api/v1/pops/{code}",
            get(|| async { StatusCode::METHOD_NOT_ALLOWED })
                .patch(edit_pop)
                .delete(delete_pop),
        )
        .route(
            "/api/v1/sites/{code}/pop",
            axum::routing::patch(assign_site_pop),
        )
        .route("/api/v1/sites", get(list_sites).post(create_site))
        .route(
            "/api/v1/sites/{code}",
            get(|| async { StatusCode::METHOD_NOT_ALLOWED })
                .patch(edit_site)
                .delete(delete_site),
        )
        .route("/api/v1/devices", get(list_devices).post(create_device))
        .route(
            "/api/v1/devices/{id}",
            get(device_detail).patch(edit_device).delete(delete_device),
        )
        .route("/api/v1/domains", get(list_domains).post(create_domain))
        .route("/api/v1/domains/{id}/dns", get(customer_domain_dns))
        .route(
            "/api/v1/domains/{id}",
            get(|| async { StatusCode::METHOD_NOT_ALLOWED }).delete(delete_domain),
        )
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn r970_catalog_forbids_forged_or_mismatched_device_registration() {
        assert!(catalog_allows("olt", "ZTE", "ssh"));
        assert!(catalog_allows("olt", "FutureCatalogVendor", "ssh"));
        assert!(catalog_allows("ont", "VSOL", "cwmp"));
        assert!(catalog_allows("router", "MikroTik", "routeros_api_ssl"));
        for wrong in [
            ("olt", "bad vendor space", "ssh"),
            ("router", "MikroTik", "cwmp"),
            ("router", "VSOL", "cwmp"),
            ("ont", "VSOL", "ssh"),
            ("olt", "ZTE", "telnet"),
        ] {
            assert!(!catalog_allows(wrong.0, wrong.1, wrong.2));
        }
        assert!(DASHBOARD_HTML.contains("<html lang=\"en-US\">"));
        assert!(!DASHBOARD_HTML.contains("ipt-owner-pop-form"));
        assert!(DASHBOARD_JS.contains("'/api/v1/capabilities'"));
    }
    #[test]
    fn cookie_and_origin_are_strict() {
        let mut h = HeaderMap::new();
        h.insert(header::HOST, HeaderValue::from_static("Tenant.Example.NET"));
        h.insert(
            header::COOKIE,
            HeaderValue::from_static(
                "__Host-ipat_session=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            ),
        );
        h.insert(
            header::ORIGIN,
            HeaderValue::from_static("https://tenant.example.net"),
        );
        h.insert(
            &CSRF_HEADER,
            HeaderValue::from_static("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
        );
        assert_eq!(host(&h).as_deref(), Some("tenant.example.net"));
        assert!(cookie(&h).is_some());
        assert!(csrf(&h).is_some());
        assert!(mutation_origin(&h, "tenant.example.net"));
        h.append(header::COOKIE, HeaderValue::from_static("x=y"));
        assert!(cookie(&h).is_none());
    }
    #[test]
    fn validation_never_accepts_cli_or_arbitrary_protocol() {
        assert!(!valid_host("root:password@olt.example.net"));
        assert!(valid_code("JKT-CORE_01"));
        assert!(!valid_code("../JKT"));
        assert!(valid_name("Distribution OLT"));
        assert!(!valid_name(" Distribution OLT"));
    }
}

#[cfg(test)]
mod pg_integration {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::Request,
    };
    use sha2::{Digest, Sha256};
    use tokio_postgres::{Config, NoTls};
    use tower::ServiceExt;

    async fn connect_admin() -> Client {
        let mut c = Config::new();
        c.host("127.0.0.1");
        c.port(
            std::env::var("PGPORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5432),
        );
        c.user(std::env::var("PGUSER").as_deref().unwrap_or("postgres"));
        if let Ok(v) = std::env::var("PGPASSWORD") {
            c.password(v);
        }
        c.dbname("ipat_synthetic");
        let (client, connection) = c.connect(NoTls).await.unwrap();
        tokio::spawn(async move {
            let _ = connection.await;
        });
        client
    }
    fn hex(v: &str) -> String {
        let d = Sha256::digest(v.as_bytes());
        d.iter().map(|b| format!("{b:02x}")).collect()
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
            b = b.header("Cookie", format!("__Host-ipat_session={c}"));
        }
        if let Some(c) = csrf {
            b = b
                .header("Origin", format!("https://{host}"))
                .header("X-IPAT-CSRF", c);
        }
        if body.is_some() {
            b = b.header("Content-Type", "application/json");
        }
        b.body(Body::from(body.unwrap_or_default().to_owned()))
            .unwrap()
    }
    async fn body_json(r: axum::response::Response) -> Value {
        serde_json::from_slice(&to_bytes(r.into_body(), 1024 * 64).await.unwrap()).unwrap()
    }

    #[tokio::test]
    async fn r970_real_pg_tenant_dashboard_current_role_catalog_and_logout() {
        if std::env::var("IPAT_PG_EPHEMERAL_TEST").as_deref() != Ok("1")
            || std::env::var("PGDATABASE").as_deref() != Ok("ipat_synthetic")
        {
            return;
        }
        let admin = connect_admin().await;
        // 0023 is explicitly applied in the new R9.70 disposable CI step.
        if admin.query_one("SELECT to_regprocedure('ipat_platform.tenant_admin_ui_capability(text,text,uuid)') IS NOT NULL",&[]).await.unwrap().get::<_,bool>(0)==false {return}
        let tenant = Uuid::parse_str("70707070-7070-4070-8070-707070707071").unwrap();
        let domain = Uuid::parse_str("71717171-7171-4171-8171-717171717171").unwrap();
        let session_id = Uuid::parse_str("72727272-7272-4272-8272-727272727271").unwrap();
        let issuer = "https://id.r970.synthetic.invalid/realms/ipat";
        let subject = "r970-dashboard-admin";
        let host = "tenant-r970.example.net";
        admin.execute("INSERT INTO ipat_platform.tenants(id,tenant_slug,state) VALUES($1,'r970-dashboard-tenant','active') ON CONFLICT(id) DO UPDATE SET state='active'",&[&tenant]).await.unwrap();
        admin.execute("INSERT INTO ipat_platform.identity_memberships(tenant_id,issuer,subject,role,approved_by,expires_at) VALUES($1,$2,$3,'tenant_admin','synthetic-r970-reviewer',clock_timestamp()+interval '1 day') ON CONFLICT(tenant_id,issuer,subject,role) DO UPDATE SET revoked_at=NULL,expires_at=clock_timestamp()+interval '1 day'",&[&tenant,&issuer,&subject]).await.unwrap();
        admin.execute("INSERT INTO ipat_platform.tenant_domains(id,tenant_id,hostname,domain_type,verification_state,verification_method,verified_at,routing_mode,verification_name,verification_value,requested_by_issuer,requested_by_subject,requested_at,activation_state,ownership_verified_at,routing_ready_at,tls_ready_at,activated_at) VALUES($1,$2,$3,'custom_domain','verified','dns_txt',clock_timestamp(),'a_record',$4,$5,$6,$7,clock_timestamp(),'active',clock_timestamp(),clock_timestamp(),clock_timestamp(),clock_timestamp())",&[&domain,&tenant,&host,&format!("_ipat-verify.{host}"),&format!("ipat-domain={domain}"),&issuer,&subject]).await.unwrap();
        let cookie = "ccccccccccccccccccccccccccccccccccccccccccc";
        let csrf = "ddddddddddddddddddddddddddddddddddddddddddd";
        let issuer_db = connect_admin().await;
        issuer_db
            .batch_execute("SET ROLE ipat_oidc_session_issuer_login")
            .await
            .unwrap();
        let issued=issuer_db.query_one("SELECT ipat_platform.issue_tenant_browser_session($1,$2,$3::uuid,$4::uuid,$5::uuid,$6,$7,clock_timestamp()+interval '10 minutes')",&[&issuer,&subject,&tenant,&domain,&session_id,&hex(cookie),&hex(csrf)]).await.unwrap();
        assert_eq!(issued.get::<_, Option<Uuid>>(0), Some(session_id));
        let api_db = connect_admin().await;
        api_db
            .batch_execute("SET ROLE ipat_tenant_api_login")
            .await
            .unwrap();
        let app = router(Arc::new(api_db));
        let missing = app
            .clone()
            .oneshot(req("GET", "/dashboard", host, None, None, None))
            .await
            .unwrap();
        assert_eq!(missing.status(), StatusCode::UNAUTHORIZED);
        let wrong = app
            .clone()
            .oneshot(req(
                "GET",
                "/dashboard",
                "other-r970.example.net",
                Some(cookie),
                None,
                None,
            ))
            .await
            .unwrap();
        assert_eq!(wrong.status(), StatusCode::UNAUTHORIZED);
        let dash = app
            .clone()
            .oneshot(req("GET", "/dashboard", host, Some(cookie), None, None))
            .await
            .unwrap();
        assert_eq!(dash.status(), StatusCode::OK);
        assert_eq!(dash.headers().get("cache-control").unwrap(), "no-store");
        let text = String::from_utf8(
            to_bytes(dash.into_body(), 1024 * 128)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        assert!(text.contains("<html lang=\"en-US\">"));
        assert!(text.contains("id=\"device-create\""));
        let cap = app
            .clone()
            .oneshot(req(
                "GET",
                "/api/v1/capabilities",
                host,
                Some(cookie),
                None,
                None,
            ))
            .await
            .unwrap();
        assert_eq!(cap.status(), StatusCode::OK);
        let c = body_json(cap).await;
        assert_eq!(c["tenant_id"], tenant.to_string());
        assert_eq!(c["can_manage_sites"], true);
        let catalogue = app
            .clone()
            .oneshot(req(
                "GET",
                "/api/v1/device-catalog",
                host,
                Some(cookie),
                None,
                None,
            ))
            .await
            .unwrap();
        assert_eq!(catalogue.status(), StatusCode::OK);
        let verified_catalog = body_json(catalogue).await;
        assert_eq!(
            verified_catalog["catalog_semantics"],
            "platform_curated_metadata_only_not_verified_physical_support"
        );
        assert_eq!(verified_catalog["catalog"].as_array().unwrap().len(), 5);
        let zte_olt = verified_catalog["catalog"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["type"] == "olt" && entry["vendor"] == "ZTE")
            .unwrap();
        assert_eq!(zte_olt["qualification"], "metadata_candidate");
        assert_eq!(zte_olt["protocols"].as_array().unwrap().len(), 2);
        // R9.76: independently managed POP, atomic Site-to-POP creation
        // and no implicit expansion of old exact-site NOC grants.
        let initial_pops = app
            .clone()
            .oneshot(req("GET", "/api/v1/pops", host, Some(cookie), None, None))
            .await
            .unwrap();
        assert_eq!(initial_pops.status(), StatusCode::OK);
        assert_eq!(
            body_json(initial_pops).await["pops"]
                .as_array()
                .unwrap()
                .len(),
            0
        );
        let created_pop = app
            .clone()
            .oneshot(req(
                "POST",
                "/api/v1/pops",
                host,
                Some(cookie),
                Some(csrf),
                Some(r#"{"code":"DC-R976","display_name":"R976 synthetic POP"}"#),
            ))
            .await
            .unwrap();
        assert_eq!(created_pop.status(), StatusCode::CREATED);
        let invalid_pop_site=app.clone().oneshot(req(
            "POST","/api/v1/sites",host,Some(cookie),Some(csrf),
            Some(r#"{"code":"BAD-R976","display_name":"No parent","parent_pop_code":"NONEXISTENT"}"#)
        )).await.unwrap();
        assert_eq!(invalid_pop_site.status(), StatusCode::CONFLICT);
        let site = app
            .clone()
            .oneshot(req(
                "POST",
                "/api/v1/sites",
                host,
                Some(cookie),
                Some(csrf),
                Some(r#"{"code":"POP-R970","display_name":"Synthetic R970 Site","parent_pop_code":"DC-R976"}"#),
            ))
            .await
            .unwrap();
        assert_eq!(site.status(), StatusCode::CREATED);
        let linked_sites = app
            .clone()
            .oneshot(req("GET", "/api/v1/sites", host, Some(cookie), None, None))
            .await
            .unwrap();
        assert_eq!(linked_sites.status(), StatusCode::OK);
        assert_eq!(
            body_json(linked_sites).await["sites"][0]["parent_pop_code"],
            "DC-R976"
        );
        let linked_pops = app
            .clone()
            .oneshot(req("GET", "/api/v1/pops", host, Some(cookie), None, None))
            .await
            .unwrap();
        assert_eq!(body_json(linked_pops).await["pops"][0]["assigned_sites"], 1);
        let deny_assigned_pop_deletion = app
            .clone()
            .oneshot(req(
                "DELETE",
                "/api/v1/pops/DC-R976?revision=1",
                host,
                Some(cookie),
                Some(csrf),
                None,
            ))
            .await
            .unwrap();
        assert_eq!(deny_assigned_pop_deletion.status(), StatusCode::CONFLICT);
        let forged=app.clone().oneshot(req("POST","/api/v1/devices",host,Some(cookie),Some(csrf),Some(r#"{"request_id":"73737373-7373-4373-8373-737373737371","pop_id":"POP-R970","display_name":"Fake vendor","device_kind":"olt","vendor":"Forged","intended_model":"C320","management_transport":"ssh","management_host":"olt-r970.invalid","management_port":22}"#))).await.unwrap();
        assert_eq!(forged.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let wrong_transport=app.clone().oneshot(req("POST","/api/v1/devices",host,Some(cookie),Some(csrf),Some(r#"{"request_id":"73737373-7373-4373-8373-737373737372","pop_id":"POP-R970","display_name":"Invalid protocol","device_kind":"ont","vendor":"VSOL","management_transport":"ssh","management_host":"ont-r970.invalid","management_port":22}"#))).await.unwrap();
        assert_eq!(wrong_transport.status(), StatusCode::UNPROCESSABLE_ENTITY);
        // R9.72: exact POP NOC session on the SAME live Axum router and
        // independently restricted PostgreSQL connection. No mutation rights.
        let noc_subject = "r972-noc";
        admin.execute(
            "INSERT INTO ipat_platform.identity_memberships(tenant_id,issuer,subject,role,approved_by,expires_at) VALUES($1,$2,$3,'noc_engineer','synthetic-r972-reviewer',clock_timestamp()+interval '1 day')",
            &[&tenant,&issuer,&noc_subject],
        ).await.unwrap();
        admin.execute(
            "INSERT INTO ipat_platform.identity_pop_grants(tenant_id,issuer,subject,role,pop_id) VALUES($1,$2,$3,'noc_engineer','POP-R970')",
            &[&tenant,&issuer,&noc_subject],
        ).await.unwrap();
        let noc_cookie = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
        let noc_csrf = "fffffffffffffffffffffffffffffffffffffffffff";
        let noc_session = Uuid::parse_str("72727272-7272-4272-8272-727272727272").unwrap();
        let noc_issued=issuer_db.query_one(
            "SELECT ipat_platform.issue_tenant_browser_session($1,$2,$3::uuid,$4::uuid,$5::uuid,$6,$7,clock_timestamp()+interval '10 minutes')",
            &[&issuer,&noc_subject,&tenant,&domain,&noc_session,&hex(noc_cookie),&hex(noc_csrf)]
        ).await.unwrap();
        assert_eq!(noc_issued.get::<_, Option<Uuid>>(0), Some(noc_session));
        let noc_cap = app
            .clone()
            .oneshot(req(
                "GET",
                "/api/v1/capabilities",
                host,
                Some(noc_cookie),
                None,
                None,
            ))
            .await
            .unwrap();
        assert_eq!(noc_cap.status(), StatusCode::OK);
        let noc_json = body_json(noc_cap).await;
        assert_eq!(noc_json["can_manage_sites"], false);
        assert_eq!(noc_json["can_manage_pops"], false);
        assert_eq!(noc_json["noc_pops"], json!(["POP-R970"]));
        assert_eq!(noc_json["real_noc_pops"], json!([]));
        let noc_direct_pops = app
            .clone()
            .oneshot(req(
                "GET",
                "/api/v1/pops",
                host,
                Some(noc_cookie),
                None,
                None,
            ))
            .await
            .unwrap();
        assert_eq!(noc_direct_pops.status(), StatusCode::FORBIDDEN);
        let noc_sites = app
            .clone()
            .oneshot(req(
                "GET",
                "/api/v1/noc/sites?pop=POP-R970",
                host,
                Some(noc_cookie),
                None,
                None,
            ))
            .await
            .unwrap();
        assert_eq!(noc_sites.status(), StatusCode::OK);
        assert_eq!(body_json(noc_sites).await["sites"][0]["code"], "POP-R970");
        // R9.77 typed real POP scope is explicit and does not inherit legacy Site grants.
        admin.execute(
            "INSERT INTO ipat_platform.noc_real_pop_grants(tenant_id,issuer,subject,role,pop_code,requested_by_issuer,requested_by_subject,approved_by_issuer,approved_by_subject,created_at,expires_at) VALUES($1,$2,$3,'noc_engineer','DC-R976',$2,'synthetic-requester',$2,'synthetic-independent-reviewer',clock_timestamp()-interval '1 minute',clock_timestamp()+interval '1 day')",
            &[&tenant,&issuer,&noc_subject],
        ).await.unwrap();
        let real_cap = app
            .clone()
            .oneshot(req(
                "GET",
                "/api/v1/capabilities",
                host,
                Some(noc_cookie),
                None,
                None,
            ))
            .await
            .unwrap();
        assert_eq!(real_cap.status(), StatusCode::OK);
        let real_cap_json = body_json(real_cap).await;
        assert_eq!(real_cap_json["noc_pops"], json!(["POP-R970"]));
        assert_eq!(real_cap_json["real_noc_pops"], json!(["DC-R976"]));
        let real_sites = app
            .clone()
            .oneshot(req(
                "GET",
                "/api/v1/noc/real/sites?pop=DC-R976",
                host,
                Some(noc_cookie),
                None,
                None,
            ))
            .await
            .unwrap();
        assert_eq!(real_sites.status(), StatusCode::OK);
        let real_sites_json = body_json(real_sites).await;
        assert_eq!(real_sites_json["scope_type"], "real_pop");
        assert_eq!(real_sites_json["sites"][0]["code"], "POP-R970");
        let real_wrong = app
            .clone()
            .oneshot(req(
                "GET",
                "/api/v1/noc/real/sites?pop=OTHER-POP",
                host,
                Some(noc_cookie),
                None,
                None,
            ))
            .await
            .unwrap();
        assert_eq!(real_wrong.status(), StatusCode::FORBIDDEN);
        let legacy_never_real = app
            .clone()
            .oneshot(req(
                "GET",
                "/api/v1/noc/real/sites?pop=POP-R970",
                host,
                Some(noc_cookie),
                None,
                None,
            ))
            .await
            .unwrap();
        assert_eq!(legacy_never_real.status(), StatusCode::FORBIDDEN);
        let noc_wrong = app
            .clone()
            .oneshot(req(
                "GET",
                "/api/v1/noc/sites?pop=OTHER-POP",
                host,
                Some(noc_cookie),
                None,
                None,
            ))
            .await
            .unwrap();
        assert_eq!(noc_wrong.status(), StatusCode::FORBIDDEN);
        let noc_devices = app
            .clone()
            .oneshot(req(
                "GET",
                "/api/v1/noc/devices?pop=POP-R970",
                host,
                Some(noc_cookie),
                None,
                None,
            ))
            .await
            .unwrap();
        assert_eq!(noc_devices.status(), StatusCode::OK);
        assert_eq!(body_json(noc_devices).await["read_only"], true);
        let noc_post = app
            .clone()
            .oneshot(req(
                "POST",
                "/api/v1/sites",
                host,
                Some(noc_cookie),
                Some(noc_csrf),
                Some(r#"{"code":"NOC-CANNOT-CREATE","display_name":"Not allowed"}"#),
            ))
            .await
            .unwrap();
        assert_eq!(noc_post.status(), StatusCode::CONFLICT);
        let noc_host_replay = app
            .clone()
            .oneshot(req(
                "GET",
                "/api/v1/noc/sites?pop=POP-R970",
                "other-r970.example.net",
                Some(noc_cookie),
                None,
                None,
            ))
            .await
            .unwrap();
        assert_eq!(noc_host_replay.status(), StatusCode::UNAUTHORIZED);
        admin.execute(
            "UPDATE ipat_platform.identity_memberships SET revoked_at=clock_timestamp() WHERE tenant_id=$1 AND issuer=$2 AND subject=$3",
            &[&tenant,&issuer,&noc_subject]
        ).await.unwrap();
        let noc_revoked = app
            .clone()
            .oneshot(req(
                "GET",
                "/api/v1/noc/sites?pop=POP-R970",
                host,
                Some(noc_cookie),
                None,
                None,
            ))
            .await
            .unwrap();
        assert_eq!(noc_revoked.status(), StatusCode::UNAUTHORIZED);
        // R9.78: real Axum + disposable PostgreSQL two-distinct-admin
        // authorization, bounded exact replay, immediate typed POP revocation.
        admin.execute(
            "INSERT INTO ipat_platform.identity_memberships(tenant_id,issuer,subject,role,approved_by,expires_at) VALUES($1,$2,'r978-checker','tenant_admin','r978-independent-fixture',clock_timestamp()+interval '1 day'),($1,$2,'r978-target','noc_engineer','r978-independent-fixture',clock_timestamp()+interval '1 day')",
            &[&tenant,&issuer],
        ).await.unwrap();
        let checker_cookie = "hhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhh";
        let checker_csrf = "iiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiii";
        let checker_session = Uuid::parse_str("78787878-7878-4787-8787-787878787870").unwrap();
        let checker_issued=issuer_db.query_one(
            "SELECT ipat_platform.issue_tenant_browser_session($1,'r978-checker',$2::uuid,$3::uuid,$4::uuid,$5,$6,clock_timestamp()+interval '10 minutes')",
            &[&issuer,&tenant,&domain,&checker_session,&hex(checker_cookie),&hex(checker_csrf)]
        ).await.unwrap();
        assert_eq!(
            checker_issued.get::<_, Option<Uuid>>(0),
            Some(checker_session)
        );
        let target_cookie = "jjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjjj";
        let target_session = Uuid::parse_str("78787878-7878-4787-8787-787878787871").unwrap();
        let target_issued=issuer_db.query_one(
            "SELECT ipat_platform.issue_tenant_browser_session($1,'r978-target',$2::uuid,$3::uuid,$4::uuid,$5,$6,clock_timestamp()+interval '10 minutes')",
            &[&issuer,&tenant,&domain,&target_session,&hex(target_cookie),&hex("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk")]
        ).await.unwrap();
        assert_eq!(
            target_issued.get::<_, Option<Uuid>>(0),
            Some(target_session)
        );
        let requested_expires: String = admin.query_one(
            "SELECT to_char((clock_timestamp()+interval '2 hours') AT TIME ZONE 'UTC','YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"')", &[]
        ).await.unwrap().get(0);
        let grant_request_id = Uuid::parse_str("78787878-7878-4787-8787-787878787879").unwrap();
        let grant_body = json!({
            "request_id":grant_request_id.to_string(),"target_issuer":issuer,
            "target_subject":"r978-target","pop_code":"DC-R976",
            "expires_at":requested_expires
        })
        .to_string();
        let missing_grant_csrf = app
            .clone()
            .oneshot(req(
                "POST",
                "/api/v1/noc-access/requests",
                host,
                Some(cookie),
                None,
                Some(&grant_body),
            ))
            .await
            .unwrap();
        assert_eq!(missing_grant_csrf.status(), StatusCode::UNAUTHORIZED);
        let wrong_grant_host = app
            .clone()
            .oneshot(req(
                "POST",
                "/api/v1/noc-access/requests",
                "other-r970.example.net",
                Some(cookie),
                Some(csrf),
                Some(&grant_body),
            ))
            .await
            .unwrap();
        assert_eq!(wrong_grant_host.status(), StatusCode::UNAUTHORIZED);
        let wrong_grant_role = app
            .clone()
            .oneshot(req(
                "POST",
                "/api/v1/noc-access/requests",
                host,
                Some(target_cookie),
                Some("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk"),
                Some(&grant_body),
            ))
            .await
            .unwrap();
        assert_eq!(wrong_grant_role.status(), StatusCode::FORBIDDEN);
        for _ in 0..2 {
            let saved = app
                .clone()
                .oneshot(req(
                    "POST",
                    "/api/v1/noc-access/requests",
                    host,
                    Some(cookie),
                    Some(csrf),
                    Some(&grant_body),
                ))
                .await
                .unwrap();
            assert_eq!(saved.status(), StatusCode::CREATED);
            assert_eq!(
                body_json(saved).await["request_id"],
                grant_request_id.to_string()
            );
        }
        let review_path = format!("/api/v1/noc-access/requests/{grant_request_id}/review");
        let self_review = app
            .clone()
            .oneshot(req(
                "POST",
                &review_path,
                host,
                Some(cookie),
                Some(csrf),
                Some(r#"{"approve":true}"#),
            ))
            .await
            .unwrap();
        assert_eq!(self_review.status(), StatusCode::CONFLICT);
        let checker_review = app
            .clone()
            .oneshot(req(
                "POST",
                &review_path,
                host,
                Some(checker_cookie),
                Some(checker_csrf),
                Some(r#"{"approve":true}"#),
            ))
            .await
            .unwrap();
        assert_eq!(checker_review.status(), StatusCode::OK);
        assert_eq!(body_json(checker_review).await["state"], "APPROVED");
        let allowed = app
            .clone()
            .oneshot(req(
                "GET",
                "/api/v1/noc/real/sites?pop=DC-R976",
                host,
                Some(target_cookie),
                None,
                None,
            ))
            .await
            .unwrap();
        assert_eq!(allowed.status(), StatusCode::OK);
        assert_eq!(body_json(allowed).await["scope_type"], "real_pop");
        let revoked_access = app
            .clone()
            .oneshot(req(
                "POST",
                "/api/v1/noc-access/revoke",
                host,
                Some(cookie),
                Some(csrf),
                Some(
                    &json!({
                        "target_issuer":issuer,"target_subject":"r978-target","pop_code":"DC-R976"
                    })
                    .to_string(),
                ),
            ))
            .await
            .unwrap();
        assert_eq!(revoked_access.status(), StatusCode::OK);
        let after_revoke = app
            .clone()
            .oneshot(req(
                "GET",
                "/api/v1/noc/real/sites?pop=DC-R976",
                host,
                Some(target_cookie),
                None,
                None,
            ))
            .await
            .unwrap();
        assert_eq!(after_revoke.status(), StatusCode::FORBIDDEN);
        admin.execute("UPDATE ipat_platform.identity_memberships SET revoked_at=clock_timestamp() WHERE tenant_id=$1 AND issuer=$2 AND subject=$3",&[&tenant,&issuer,&subject]).await.unwrap();
        let revoked = app
            .clone()
            .oneshot(req(
                "GET",
                "/api/v1/capabilities",
                host,
                Some(cookie),
                None,
                None,
            ))
            .await
            .unwrap();
        assert_eq!(revoked.status(), StatusCode::UNAUTHORIZED);
        let revoked_dash = app
            .clone()
            .oneshot(req("GET", "/dashboard", host, Some(cookie), None, None))
            .await
            .unwrap();
        assert_eq!(revoked_dash.status(), StatusCode::UNAUTHORIZED);
        admin.execute("UPDATE ipat_platform.identity_memberships SET revoked_at=NULL WHERE tenant_id=$1 AND issuer=$2 AND subject=$3",&[&tenant,&issuer,&subject]).await.unwrap();
        let logout = app
            .clone()
            .oneshot(req(
                "POST",
                "/api/v1/logout",
                host,
                Some(cookie),
                Some(csrf),
                None,
            ))
            .await
            .unwrap();
        assert_eq!(logout.status(), StatusCode::OK);
        assert_eq!(logout.headers().get_all("set-cookie").iter().count(), 2);
        let replay = app
            .clone()
            .oneshot(req(
                "GET",
                "/api/v1/capabilities",
                host,
                Some(cookie),
                None,
                None,
            ))
            .await
            .unwrap();
        assert_eq!(replay.status(), StatusCode::UNAUTHORIZED);
        let csrf_missing = app
            .clone()
            .oneshot(req(
                "POST",
                "/api/v1/logout",
                host,
                Some(cookie),
                None,
                None,
            ))
            .await
            .unwrap();
        assert_eq!(csrf_missing.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn r968_real_pg_durable_host_session_to_site_device_domain_http() {
        if std::env::var("IPAT_PG_EPHEMERAL_TEST").as_deref() != Ok("1")
            || std::env::var("PGDATABASE").as_deref() != Ok("ipat_synthetic")
        {
            return;
        }
        let admin = connect_admin().await;
        let tenant = Uuid::parse_str("78787878-7878-4878-8878-787878787871").unwrap();
        let domain = Uuid::parse_str("79797979-7979-4979-8979-797979797971").unwrap();
        let issuer = "https://id.r968.synthetic.invalid/realms/ipat";
        let subject = "r968-admin";
        let host = "tenant-r968.example.net";
        admin.execute("INSERT INTO ipat_platform.tenants(id,tenant_slug,state) VALUES($1,'r968-tenant','active') ON CONFLICT(id) DO UPDATE SET state='active'",&[&tenant]).await.unwrap();
        admin.execute("INSERT INTO ipat_platform.identity_memberships(tenant_id,issuer,subject,role,approved_by,expires_at) VALUES($1,$2,$3,'tenant_admin','r968-test-review',clock_timestamp()+interval '1 day') ON CONFLICT(tenant_id,issuer,subject,role) DO UPDATE SET revoked_at=NULL,expires_at=clock_timestamp()+interval '1 day'",&[&tenant,&issuer,&subject]).await.unwrap();
        admin.execute("INSERT INTO ipat_platform.tenant_domains(id,tenant_id,hostname,domain_type,verification_state,verification_method,verified_at,routing_mode,verification_name,verification_value,requested_by_issuer,requested_by_subject,requested_at,activation_state,ownership_verified_at,routing_ready_at,tls_ready_at,activated_at) VALUES($1,$2,$3,'custom_domain','verified','dns_txt',clock_timestamp(),'a_record',$4,$5,$6,$7,clock_timestamp(),'active',clock_timestamp(),clock_timestamp(),clock_timestamp(),clock_timestamp()) ON CONFLICT(id) DO UPDATE SET verification_state='verified',disabled_at=NULL,activation_state='active',ownership_verified_at=clock_timestamp(),routing_ready_at=clock_timestamp(),tls_ready_at=clock_timestamp(),activated_at=clock_timestamp()",&[&domain,&tenant,&host,&format!("_ipat-verify.{host}"),&format!("ipat-domain={domain}"),&issuer,&subject]).await.unwrap();
        let cookie = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let csrf = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        let issuer_db = connect_admin().await;
        issuer_db
            .batch_execute("SET ROLE ipat_oidc_session_issuer_login")
            .await
            .unwrap();
        let sid = Uuid::parse_str("80808080-8080-4080-8080-808080808081").unwrap();
        let row=issuer_db.query_one("SELECT ipat_platform.issue_tenant_browser_session($1,$2,$3::uuid,$4::uuid,$5::uuid,$6,$7,clock_timestamp()+interval '10 minutes')",&[&issuer,&subject,&tenant,&domain,&sid,&hex(cookie),&hex(csrf)]).await.unwrap();
        assert_eq!(row.get::<_, Option<Uuid>>(0), Some(sid));
        let api_client = connect_admin().await;
        api_client
            .batch_execute("SET ROLE ipat_tenant_api_login")
            .await
            .unwrap();
        let app = router(Arc::new(api_client));

        let no_csrf = app
            .clone()
            .oneshot(req(
                "POST",
                "/api/v1/sites",
                host,
                Some(cookie),
                None,
                Some(r#"{"code":"POP-R968","display_name":"R968 Core"}"#),
            ))
            .await
            .unwrap();
        assert_eq!(no_csrf.status(), StatusCode::UNAUTHORIZED);
        let wrong_host = app
            .clone()
            .oneshot(req(
                "GET",
                "/api/v1/sites",
                "other-r968.example.net",
                Some(cookie),
                None,
                None,
            ))
            .await
            .unwrap();
        assert_eq!(wrong_host.status(), StatusCode::UNAUTHORIZED);
        let create = app
            .clone()
            .oneshot(req(
                "POST",
                "/api/v1/sites",
                host,
                Some(cookie),
                Some(csrf),
                Some(r#"{"code":"POP-R968","display_name":"R968 Core"}"#),
            ))
            .await
            .unwrap();
        assert_eq!(create.status(), StatusCode::CREATED);
        let sites = app
            .clone()
            .oneshot(req("GET", "/api/v1/sites", host, Some(cookie), None, None))
            .await
            .unwrap();
        assert_eq!(sites.status(), StatusCode::OK);
        assert!(body_json(sites).await["sites"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["code"] == "POP-R968"));

        let request_id = "81818181-8181-4181-8181-818181818181";
        let create_dev=app.clone().oneshot(req("POST","/api/v1/devices",host,Some(cookie),Some(csrf),Some(&format!(r#"{{"request_id":"{request_id}","pop_id":"POP-R968","display_name":"Synthetic R968 OLT","device_kind":"olt","vendor":"ZTE","intended_model":"C320","management_transport":"ssh","management_host":"olt.r968.invalid","management_port":22}}"#)))).await.unwrap();
        assert_eq!(create_dev.status(), StatusCode::CREATED);
        let created = body_json(create_dev).await;
        let device_id = created["id"].as_str().unwrap().to_string();
        let list = app
            .clone()
            .oneshot(req(
                "GET",
                "/api/v1/devices",
                host,
                Some(cookie),
                None,
                None,
            ))
            .await
            .unwrap();
        assert_eq!(list.status(), StatusCode::OK);
        assert!(body_json(list).await["devices"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["id"] == device_id));
        let detail = app
            .clone()
            .oneshot(req(
                "GET",
                &format!("/api/v1/devices/{device_id}"),
                host,
                Some(cookie),
                None,
                None,
            ))
            .await
            .unwrap();
        assert_eq!(detail.status(), StatusCode::OK);
        assert_eq!(
            body_json(detail).await["device"]["display_name"],
            "Synthetic R968 OLT"
        );
        let edit=app.clone().oneshot(req("PATCH",&format!("/api/v1/devices/{device_id}"),host,Some(cookie),Some(csrf),Some(r#"{"expected_revision":1,"pop_id":"POP-R968","display_name":"Synthetic R968 OLT Renamed","intended_model":"C320","management_host":"olt.r968.invalid","management_port":22}"#))).await.unwrap();
        assert_eq!(edit.status(), StatusCode::OK);

        let domain_create = app
            .clone()
            .oneshot(req(
                "POST",
                "/api/v1/domains",
                host,
                Some(cookie),
                Some(csrf),
                Some(r#"{"hostname":"customer-r968.example.net","routing_mode":"a_record"}"#),
            ))
            .await
            .unwrap();
        assert_eq!(domain_create.status(), StatusCode::CREATED);
        let domains = app
            .clone()
            .oneshot(req(
                "GET",
                "/api/v1/domains",
                host,
                Some(cookie),
                None,
                None,
            ))
            .await
            .unwrap();
        assert_eq!(domains.status(), StatusCode::OK);
        assert!(body_json(domains).await["domains"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["hostname"] == "customer-r968.example.net"));
        // R9.82: the customer's saved domain, never the unowned IPAT
        // brand, is the authority for unique TXT + conditional A instructions.
        let list_domains = app
            .clone()
            .oneshot(req(
                "GET",
                "/api/v1/domains",
                host,
                Some(cookie),
                None,
                None,
            ))
            .await
            .unwrap();
        let saved = body_json(list_domains).await;
        let saved_domain = saved["domains"]
            .as_array()
            .unwrap()
            .iter()
            .find(|d| d["hostname"] == "customer-r968.example.net")
            .unwrap();
        let saved_id = saved_domain["id"].as_str().unwrap().to_string();
        let saved_txt = saved_domain["verification_value"]
            .as_str()
            .unwrap()
            .to_string();
        let dns_path = format!("/api/v1/domains/{saved_id}/dns");
        let no_session = app
            .clone()
            .oneshot(req("GET", &dns_path, host, None, None, None))
            .await
            .unwrap();
        assert_eq!(no_session.status(), StatusCode::UNAUTHORIZED);
        let other_host = app
            .clone()
            .oneshot(req(
                "GET",
                &dns_path,
                "other-r968.example.net",
                Some(cookie),
                None,
                None,
            ))
            .await
            .unwrap();
        assert_eq!(other_host.status(), StatusCode::UNAUTHORIZED);
        let foreign = app
            .clone()
            .oneshot(req(
                "GET",
                "/api/v1/domains/78787878-7878-4878-8878-787878787899/dns",
                host,
                Some(cookie),
                None,
                None,
            ))
            .await
            .unwrap();
        assert_eq!(foreign.status(), StatusCode::NOT_FOUND);
        std::env::set_var("IPAT_CUSTOM_DOMAIN_DNS_MODE", "a_record");
        std::env::set_var("IPAT_CUSTOM_DOMAIN_IPV4", "202.162.204.121");
        std::env::set_var("IPAT_CUSTOM_DOMAIN_ROUTING_READY", "YES");
        std::env::set_var("IPAT_R982_CUSTOM_DOMAIN_TLS_EDGE_READY", "YES");
        let pending = app
            .clone()
            .oneshot(req("GET", &dns_path, host, Some(cookie), None, None))
            .await
            .unwrap();
        assert_eq!(pending.status(), StatusCode::OK);
        let pending_body = body_json(pending).await;
        assert_eq!(pending_body["guide"]["verification_value"], saved_txt);
        assert_eq!(
            pending_body["guide"]["routing_records"][0]["value"],
            "202.162.204.121"
        );
        assert_eq!(pending_body["guide"]["safe_to_point_now"], false);
        assert_eq!(pending_body["guide"]["activation_state"], "pending_dns");
        // Synthetic approved ownership lifecycle: do not imply actual DNS
        // or TLS readiness in the live customer environment.
        let saved_uuid = Uuid::parse_str(&saved_id).unwrap();
        admin.execute("UPDATE ipat_platform.tenant_domains SET verification_state='verified',verified_at=clock_timestamp(),activation_state='ownership_verified',ownership_verified_at=clock_timestamp() WHERE id=$1 AND tenant_id=$2",
            &[&saved_uuid,&tenant]).await.unwrap();
        std::env::remove_var("IPAT_R982_CUSTOM_DOMAIN_TLS_EDGE_READY");
        let not_reviewed = app
            .clone()
            .oneshot(req("GET", &dns_path, host, Some(cookie), None, None))
            .await
            .unwrap();
        assert_eq!(not_reviewed.status(), StatusCode::OK);
        assert_eq!(
            body_json(not_reviewed).await["guide"]["safe_to_point_now"],
            false
        );
        std::env::set_var("IPAT_R982_CUSTOM_DOMAIN_TLS_EDGE_READY", "YES");
        let ready = app
            .clone()
            .oneshot(req("GET", &dns_path, host, Some(cookie), None, None))
            .await
            .unwrap();
        assert_eq!(ready.status(), StatusCode::OK);
        assert_eq!(body_json(ready).await["guide"]["safe_to_point_now"], true);
        for key in [
            "IPAT_CUSTOM_DOMAIN_DNS_MODE",
            "IPAT_CUSTOM_DOMAIN_IPV4",
            "IPAT_CUSTOM_DOMAIN_ROUTING_READY",
            "IPAT_R982_CUSTOM_DOMAIN_TLS_EDGE_READY",
        ] {
            std::env::remove_var(key);
        }
        let archived = app
            .clone()
            .oneshot(req(
                "DELETE",
                &format!("/api/v1/devices/{device_id}?revision=2"),
                host,
                Some(cookie),
                Some(csrf),
                None,
            ))
            .await
            .unwrap();
        assert_eq!(archived.status(), StatusCode::OK);

        admin
            .execute(
                "DELETE FROM ipat_platform.tenant_browser_sessions WHERE tenant_id=$1",
                &[&tenant],
            )
            .await
            .unwrap();
        admin
            .execute(
                "DELETE FROM ipat_ops.managed_device_events WHERE tenant_id=$1",
                &[&tenant],
            )
            .await
            .unwrap();
        admin
            .execute(
                "DELETE FROM ipat_ops.managed_device_audit WHERE tenant_id=$1",
                &[&tenant],
            )
            .await
            .unwrap();
        admin
            .execute(
                "DELETE FROM ipat_ops.managed_devices WHERE tenant_id=$1",
                &[&tenant],
            )
            .await
            .unwrap();
        admin
            .execute(
                "DELETE FROM ipat_ops.tenant_site_events WHERE tenant_id=$1",
                &[&tenant],
            )
            .await
            .unwrap();
        admin
            .execute(
                "DELETE FROM ipat_ops.tenant_sites WHERE tenant_id=$1",
                &[&tenant],
            )
            .await
            .unwrap();
        admin
            .execute(
                "DELETE FROM ipat_platform.tenant_domains WHERE tenant_id=$1",
                &[&tenant],
            )
            .await
            .unwrap();
        admin
            .execute(
                "DELETE FROM ipat_platform.identity_memberships WHERE tenant_id=$1",
                &[&tenant],
            )
            .await
            .unwrap();
        admin
            .execute("DELETE FROM ipat_platform.tenants WHERE id=$1", &[&tenant])
            .await
            .unwrap();
    }
}
