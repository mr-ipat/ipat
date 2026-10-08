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
use diagnostic_core::{diagnose, Hypothesis, Observation, Scope, Signal};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{fs::File, io::Read, path::PathBuf, sync::Arc};
use tenant_core::TenantId;
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
    // Rolling migration safety: until 0035 exists, the new modules remain
    // invisible and their routes fail closed. Never turn a missing schema into
    // an authorization fallback.
    let subscriber_schema_ready = s.db.query_one(
        "SELECT to_regprocedure('ipat_platform.subscriber_diagnostic_ui_capabilities(text,text,uuid)') IS NOT NULL",
        &[],
    ).await;
    let subscriber_schema_ready = subscriber_schema_ready
        .ok()
        .is_some_and(|row| row.get::<_, bool>(0));
    let (can_read_subscribers, can_manage_subscribers) = if subscriber_schema_ready {
        let caps=s.db.query_one(
            "SELECT can_read,can_manage FROM ipat_platform.subscriber_diagnostic_ui_capabilities($1,$2,$3::uuid)",
            &[&a.issuer,&a.subject,&a.tenant_id],
        ).await;
        let Ok(caps) = caps else {
            return denied(StatusCode::SERVICE_UNAVAILABLE);
        };
        (caps.get::<_, bool>(0), caps.get::<_, bool>(1))
    } else {
        (false, false)
    };
    let (can_create_pppoe_plans, can_review_pppoe_plans, can_arm_pppoe_plans) =
        match pppoe_role_capabilities(&s.db, &a).await {
            Ok(caps) => caps,
            Err(status) => return denied(status),
        };
    let can_manage_pppoe_plans =
        can_create_pppoe_plans || can_review_pppoe_plans || can_arm_pppoe_plans;
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
        "can_read_subscribers":can_read_subscribers,
        "can_manage_subscribers":can_manage_subscribers,
        "can_read_diagnostics":can_read_subscribers,
        "can_manage_pppoe_plans":can_manage_pppoe_plans,
        "can_create_pppoe_plans":can_create_pppoe_plans,
        "can_review_pppoe_plans":can_review_pppoe_plans,
        "can_arm_pppoe_plans":can_arm_pppoe_plans,
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
async fn high_risk_actor(
    db: &Client,
    headers: &HeaderMap,
) -> Option<durable_tenant_session::HighRiskSessionContext> {
    let h = host(headers)?;
    let c = cookie(headers)?;
    let csrf_token = csrf(headers)?;
    if !mutation_origin(headers, &h) {
        return None;
    }
    durable_tenant_session::authenticate_high_risk(db, c, csrf_token, &h).await
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

async fn pppoe_role_capabilities(
    db: &Client,
    actor: &durable_tenant_session::DurableSessionContext,
) -> Result<(bool, bool, bool), StatusCode> {
    let row = db
        .query_one(
            "SELECT
              to_regprocedure('ipat_platform.pppoe_batch_create_capability(text,text,uuid)') IS NOT NULL
              AND to_regprocedure('ipat_platform.pppoe_batch_review_capability(text,text,uuid)') IS NOT NULL,
              to_regprocedure('ipat_platform.pppoe_execution_arm_capability(text,text,uuid)') IS NOT NULL",
            &[],
        )
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let base_ready: bool = row.get(0);
    let arm_ready: bool = row.get(1);
    if !base_ready {
        return Ok((false, false, false));
    }
    if !arm_ready {
        let row = db
            .query_one(
                "SELECT
                   ipat_platform.pppoe_batch_create_capability($1,$2,$3::uuid),
                   ipat_platform.pppoe_batch_review_capability($1,$2,$3::uuid)",
                &[&actor.issuer, &actor.subject, &actor.tenant_id],
            )
            .await
            .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        return Ok((row.get(0), row.get(1), false));
    }
    let row = db
        .query_one(
            "SELECT
               ipat_platform.pppoe_batch_create_capability($1,$2,$3::uuid),
               ipat_platform.pppoe_batch_review_capability($1,$2,$3::uuid),
               ipat_platform.pppoe_execution_arm_capability($1,$2,$3::uuid)",
            &[&actor.issuer, &actor.subject, &actor.tenant_id],
        )
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok((row.get(0), row.get(1), row.get(2)))
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Subscriber360Upsert {
    subscriber_id: String,
    display_name: String,
    pppoe_username: Option<String>,
    pop_code: String,
    site_code: String,
    distribution_device_id: String,
    access_device_id: Option<String>,
    ont_reference: Option<String>,
    expected_revision: i64,
}
async fn list_subscribers360(
    State(s): State<Arc<StateData>>,
    headers: HeaderMap,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let Some(a) = actor(&s.db, &headers, false).await else {
        return denied(StatusCode::UNAUTHORIZED);
    };
    let rows =
        s.db.query(
            "SELECT subscriber_id,display_name,pppoe_username,pop_code,site_code,
                distribution_device_id,access_device_id,ont_reference,revision,
                topology_state,topology_source_id,topology_verified_epoch
         FROM ipat_platform.list_subscriber360($1,$2,$3::uuid)",
            &[&a.issuer, &a.subject, &a.tenant_id],
        )
        .await;
    let Ok(rows) = rows else {
        return denied(StatusCode::SERVICE_UNAVAILABLE);
    };
    let subscribers = rows
        .into_iter()
        .map(|r| {
            json!({
                "subscriber_id":r.get::<_,String>(0),
                "display_name":r.get::<_,String>(1),
                "pppoe_username":r.get::<_,Option<String>>(2),
                "pop_code":r.get::<_,String>(3),
                "site_code":r.get::<_,String>(4),
                "distribution_device_id":r.get::<_,Uuid>(5).to_string(),
                "access_device_id":r.get::<_,Option<Uuid>>(6).map(|x|x.to_string()),
                "ont_reference":r.get::<_,Option<String>>(7),
                "revision":r.get::<_,i64>(8),
                "topology_state":r.get::<_,String>(9),
                "topology_source_id":r.get::<_,Option<String>>(10),
                "topology_verified_epoch":r.get::<_,Option<i64>>(11),
            })
        })
        .collect::<Vec<_>>();
    response(StatusCode::OK, json!({"ok":true,"subscribers":subscribers}))
}
async fn upsert_subscriber360(
    State(s): State<Arc<StateData>>,
    headers: HeaderMap,
    Json(q): Json<Subscriber360Upsert>,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let Some(a) = actor(&s.db, &headers, true).await else {
        return denied(StatusCode::UNAUTHORIZED);
    };
    if !valid_code(&q.subscriber_id)
        || !valid_name(&q.display_name)
        || !valid_code(&q.pop_code)
        || !valid_code(&q.site_code)
        || q.expected_revision < 0
        || q.pppoe_username.as_deref().is_some_and(|x| {
            x.is_empty()
                || x.len() > 128
                || x.trim() != x
                || x.bytes()
                    .any(|b| b.is_ascii_control() || b.is_ascii_whitespace())
        })
        || q.ont_reference.as_deref().is_some_and(|x| {
            x.is_empty()
                || x.len() > 128
                || !x
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_.:/-".contains(&b))
        })
    {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let Ok(distribution) = Uuid::parse_str(&q.distribution_device_id) else {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    };
    let access = match q.access_device_id.as_deref() {
        None => None,
        Some(v) => match Uuid::parse_str(v) {
            Ok(x) => Some(x),
            Err(_) => return denied(StatusCode::UNPROCESSABLE_ENTITY),
        },
    };
    let row =
        s.db.query_one(
            "SELECT ipat_platform.upsert_subscriber360(
           $1,$2,$3::uuid,$4,$5,$6,$7,$8,$9::uuid,$10::uuid,$11,$12::bigint)",
            &[
                &a.issuer,
                &a.subject,
                &a.tenant_id,
                &q.subscriber_id,
                &q.display_name,
                &q.pppoe_username,
                &q.pop_code,
                &q.site_code,
                &distribution,
                &access,
                &q.ont_reference,
                &q.expected_revision,
            ],
        )
        .await;
    let revision = row.ok().and_then(|r| r.get::<_, Option<i64>>(0));
    match revision {
        Some(revision) => response(
            if q.expected_revision == 0 {
                StatusCode::CREATED
            } else {
                StatusCode::OK
            },
            json!({"ok":true,"subscriber_id":q.subscriber_id,"revision":revision,
            "remediation_permitted":false}),
        ),
        None => denied(StatusCode::CONFLICT),
    }
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PppoePlanItem {
    subscriber_id: String,
    action: String,
    username: Option<String>,
    profile: Option<String>,
    secret_ref: Option<String>,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PppoePlanCreate {
    request_id: String,
    router_id: String,
    items: Vec<PppoePlanItem>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PppoePlanReview {
    approve: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PppoeExecutionArm {
    confirm: String,
}
fn pppoe_plan_digest(router: Uuid, items: &[PppoePlanItem]) -> Option<String> {
    if items.is_empty() || items.len() > 128 {
        return None;
    }
    let canonical = json!({"router_id":router.to_string(),"items":items});
    let bytes = serde_json::to_vec(&canonical).ok()?;
    let mut digest = String::with_capacity(64);
    for byte in Sha256::digest(bytes) {
        use std::fmt::Write as _;
        write!(&mut digest, "{byte:02x}").ok()?;
    }
    Some(digest)
}
async fn list_pppoe_plans(
    State(s): State<Arc<StateData>>,
    headers: HeaderMap,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let Some(a) = actor(&s.db, &headers, false).await else {
        return denied(StatusCode::UNAUTHORIZED);
    };
    let (can_create, can_review, can_arm) = match pppoe_role_capabilities(&s.db, &a).await {
        Ok(caps) => caps,
        Err(status) => return denied(status),
    };
    if !(can_create || can_review || can_arm) {
        return denied(StatusCode::FORBIDDEN);
    }
    let rows =
        s.db.query(
            "SELECT id,router_id,site_code,pop_code,idempotency_key,plan_digest,item_count,
                state,requested_by,reviewed_by,requested_at::text,reviewed_at::text,
                approval_expires_at::text,rate_limit_per_minute,execution_allowed,
                physical_readback_verified,basis,can_review
         FROM ipat_platform.list_pppoe_batch_dry_runs($1,$2,$3::uuid)",
            &[&a.issuer, &a.subject, &a.tenant_id],
        )
        .await;
    let Ok(rows) = rows else {
        return denied(StatusCode::SERVICE_UNAVAILABLE);
    };
    let plans = rows
        .into_iter()
        .map(|r| {
            json!({
                "id":r.get::<_,Uuid>(0).to_string(),"router_id":r.get::<_,Uuid>(1).to_string(),
                "site_code":r.get::<_,String>(2),"pop_code":r.get::<_,String>(3),
                "idempotency_key":r.get::<_,String>(4),"plan_digest":r.get::<_,String>(5),
                "item_count":r.get::<_,i32>(6),"state":r.get::<_,String>(7),
                "requested_by":r.get::<_,String>(8),"reviewed_by":r.get::<_,Option<String>>(9),
                "requested_at":r.get::<_,String>(10),"reviewed_at":r.get::<_,Option<String>>(11),
                "approval_expires_at":r.get::<_,Option<String>>(12),
                "rate_limit_per_minute":r.get::<_,i32>(13),
                "execution_allowed":r.get::<_,bool>(14),
                "physical_readback_verified":r.get::<_,bool>(15),
                "basis":r.get::<_,String>(16),"can_review":r.get::<_,bool>(17),
                "can_arm":can_arm && r.get::<_,String>(7)=="approved"
                    && !r.get::<_,bool>(14),
            })
        })
        .collect::<Vec<_>>();
    response(
        StatusCode::OK,
        json!({"ok":true,"plans":plans,
        "physical_execution_available":false}),
    )
}
async fn pppoe_plan_items(
    State(s): State<Arc<StateData>>,
    headers: HeaderMap,
    Path(id_raw): Path<String>,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let Some(a) = actor(&s.db, &headers, false).await else {
        return denied(StatusCode::UNAUTHORIZED);
    };
    let (can_create, can_review, can_arm) = match pppoe_role_capabilities(&s.db, &a).await {
        Ok(caps) => caps,
        Err(status) => return denied(status),
    };
    if !(can_create || can_review || can_arm) {
        return denied(StatusCode::FORBIDDEN);
    }
    let Ok(id) = Uuid::parse_str(&id_raw) else {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    };
    let rows =
        s.db.query(
            "SELECT ordinal,subscriber_id,action,before_username,desired_username,
                profile_name,has_secret_ref
         FROM ipat_platform.list_pppoe_batch_items($1,$2,$3::uuid,$4::uuid)",
            &[&a.issuer, &a.subject, &a.tenant_id, &id],
        )
        .await;
    let Ok(rows) = rows else {
        return denied(StatusCode::SERVICE_UNAVAILABLE);
    };
    let items = rows
        .into_iter()
        .map(|r| {
            json!({
                "ordinal":r.get::<_,i32>(0),"subscriber_id":r.get::<_,String>(1),
                "action":r.get::<_,String>(2),"before_username":r.get::<_,Option<String>>(3),
                "desired_username":r.get::<_,Option<String>>(4),
                "profile":r.get::<_,Option<String>>(5),"has_secret_ref":r.get::<_,bool>(6)
            })
        })
        .collect::<Vec<_>>();
    if items.is_empty() {
        return denied(StatusCode::NOT_FOUND);
    }
    response(
        StatusCode::OK,
        json!({"ok":true,"items":items,
        "secret_values_returned":false,"physical_execution_available":false}),
    )
}
async fn create_pppoe_plan(
    State(s): State<Arc<StateData>>,
    headers: HeaderMap,
    Json(q): Json<PppoePlanCreate>,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let Some(a) = actor(&s.db, &headers, true).await else {
        return denied(StatusCode::UNAUTHORIZED);
    };
    let (can_create, _, _) = match pppoe_role_capabilities(&s.db, &a).await {
        Ok(caps) => caps,
        Err(status) => return denied(status),
    };
    if !can_create {
        return denied(StatusCode::FORBIDDEN);
    }
    let (Ok(request), Ok(router)) = (
        Uuid::parse_str(&q.request_id),
        Uuid::parse_str(&q.router_id),
    ) else {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    };
    let Some(digest) = pppoe_plan_digest(router, &q.items) else {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    };
    if q.items.iter().any(|i| {
        !valid_code(&i.subscriber_id)
            || !matches!(i.action.as_str(), "create" | "update" | "disable")
            || i.username.as_deref().is_some_and(|x| {
                x.is_empty()
                    || x.len() > 128
                    || x.trim() != x
                    || x.bytes()
                        .any(|b| b.is_ascii_whitespace() || b.is_ascii_control())
            })
            || i.profile.as_deref().is_some_and(|x| {
                x.is_empty()
                    || x.len() > 64
                    || !x
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"_.:-".contains(&b))
            })
            || i.secret_ref
                .as_deref()
                .is_some_and(|x| x.len() > 255 || !x.starts_with("vault://tenant/"))
    }) {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let items = serde_json::to_string(&q.items).expect("serializable PPPoE plan items");
    let key = format!("request:{request}");
    let row =
        s.db.query_one(
            "SELECT ipat_platform.create_pppoe_batch_dry_run(
          $1,$2,$3::uuid,$4::uuid,$5::uuid,$6::uuid,$7,$8,$9::text::jsonb)",
            &[
                &a.issuer,
                &a.subject,
                &a.tenant_id,
                &request,
                &request,
                &router,
                &key,
                &digest,
                &items,
            ],
        )
        .await;
    match row.ok().and_then(|r| r.get::<_, Option<Uuid>>(0)) {
        Some(id) => response(
            StatusCode::CREATED,
            json!({"ok":true,
          "id":id.to_string(),"state":"awaiting_approval","plan_digest":digest,
          "execution_allowed":false,"physical_readback_verified":false,
          "next":"A current Security Admin must review this Provisioning Officer dry-run. Approval still does not execute RouterOS."}),
        ),
        None => denied(StatusCode::CONFLICT),
    }
}
async fn review_pppoe_plan(
    State(s): State<Arc<StateData>>,
    headers: HeaderMap,
    Path(id_raw): Path<String>,
    Json(q): Json<PppoePlanReview>,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let Some(a) = actor(&s.db, &headers, true).await else {
        return denied(StatusCode::UNAUTHORIZED);
    };
    let (_, can_review, _) = match pppoe_role_capabilities(&s.db, &a).await {
        Ok(caps) => caps,
        Err(status) => return denied(status),
    };
    if !can_review {
        return denied(StatusCode::FORBIDDEN);
    }
    let Ok(id) = Uuid::parse_str(&id_raw) else {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    };
    let row =
        s.db.query_one(
            "SELECT ipat_platform.review_pppoe_batch_dry_run($1,$2,$3::uuid,$4::uuid,$5)",
            &[&a.issuer, &a.subject, &a.tenant_id, &id, &q.approve],
        )
        .await;
    if row.ok().is_some_and(|r| r.get::<_, bool>(0)) {
        response(
            StatusCode::OK,
            json!({"ok":true,
          "state":if q.approve{"approved"}else{"rejected"},
          "execution_allowed":false,"physical_execution_available":false}),
        )
    } else {
        denied(StatusCode::CONFLICT)
    }
}

async fn arm_pppoe_execution(
    State(s): State<Arc<StateData>>,
    headers: HeaderMap,
    Path(id_raw): Path<String>,
    Json(q): Json<PppoeExecutionArm>,
) -> (StatusCode, HeaderMap, Json<Value>) {
    if q.confirm != "ARM_REVIEWED_PPPOE_EXECUTION" {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let Some(high) = high_risk_actor(&s.db, &headers).await else {
        return denied(StatusCode::UNAUTHORIZED);
    };
    let (_, _, can_arm) = match pppoe_role_capabilities(&s.db, &high.actor).await {
        Ok(caps) => caps,
        Err(status) => return denied(status),
    };
    if !can_arm {
        return denied(StatusCode::FORBIDDEN);
    }
    let Ok(plan) = Uuid::parse_str(&id_raw) else {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    };
    let Some(attempt) = uuid_v4() else {
        return denied(StatusCode::SERVICE_UNAVAILABLE);
    };
    let ready =
        s.db.query_one(
            "SELECT to_regprocedure(
              'ipat_platform.arm_pppoe_batch_execution(text,text,text,uuid,uuid)'
             ) IS NOT NULL",
            &[],
        )
        .await
        .ok()
        .is_some_and(|r| r.get::<_, bool>(0));
    if !ready {
        return denied(StatusCode::SERVICE_UNAVAILABLE);
    }
    let row =
        s.db.query_one(
            "SELECT ipat_platform.arm_pppoe_batch_execution(
              $1,$2,$3,$4::uuid,$5::uuid)",
            &[
                &high.cookie_sha256,
                &high.hostname,
                &high.csrf_sha256,
                &plan,
                &attempt,
            ],
        )
        .await;
    match row.ok().and_then(|r| r.get::<_, Option<Uuid>>(0)) {
        Some(id) if id == attempt => response(
            StatusCode::CREATED,
            json!({
                "ok":true,
                "attempt_id":id.to_string(),
                "state":"armed",
                "physical_execution_adapter_enabled":false,
                "next":"A restricted RouterOS worker must independently claim one item at a time. No generic command path exists."
            }),
        ),
        _ => denied(StatusCode::CONFLICT),
    }
}

async fn pppoe_execution_status(
    State(s): State<Arc<StateData>>,
    headers: HeaderMap,
    Path(id_raw): Path<String>,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let Some(a) = actor(&s.db, &headers, false).await else {
        return denied(StatusCode::UNAUTHORIZED);
    };
    let (can_create, can_review, can_arm) = match pppoe_role_capabilities(&s.db, &a).await {
        Ok(v) => v,
        Err(status) => return denied(status),
    };
    if !(can_create || can_review || can_arm) {
        return denied(StatusCode::FORBIDDEN);
    }
    let Ok(plan) = Uuid::parse_str(&id_raw) else {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    };
    let available =
        s.db.query_one(
            "SELECT to_regprocedure(
              'ipat_platform.get_pppoe_execution_status(text,text,uuid,uuid)'
             ) IS NOT NULL",
            &[],
        )
        .await
        .ok()
        .is_some_and(|r| r.get::<_, bool>(0));
    if !available {
        return response(
            StatusCode::OK,
            json!({"ok":true,"attempt":null,"physical_execution_adapter_enabled":false}),
        );
    }
    let row =
        s.db.query_opt(
            "SELECT attempt_id,state,armed_at::text,updated_at::text,item_count,
                    pending_count,unknown_count,verified_count,
                    physical_execution_adapter_enabled
             FROM ipat_platform.get_pppoe_execution_status($1,$2,$3::uuid,$4::uuid)",
            &[&a.issuer, &a.subject, &a.tenant_id, &plan],
        )
        .await;
    match row {
        Ok(Some(r)) => response(
            StatusCode::OK,
            json!({"ok":true,"attempt":{
                "id":r.get::<_,Uuid>(0).to_string(),
                "state":r.get::<_,String>(1),
                "armed_at":r.get::<_,String>(2),
                "updated_at":r.get::<_,String>(3),
                "item_count":r.get::<_,i32>(4),
                "pending_count":r.get::<_,i32>(5),
                "unknown_count":r.get::<_,i32>(6),
                "verified_count":r.get::<_,i32>(7),
                "physical_execution_adapter_enabled":r.get::<_,bool>(8)
            }}),
        ),
        Ok(None) => response(
            StatusCode::OK,
            json!({"ok":true,"attempt":null,"physical_execution_adapter_enabled":false}),
        ),
        Err(_) => denied(StatusCode::SERVICE_UNAVAILABLE),
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DiagnosticQuery {
    distribution_device_id: String,
    max_age_seconds: Option<u64>,
}
fn diagnostic_signal(raw: &str) -> Option<Signal> {
    Some(match raw {
        "distribution_uplink_down" => Signal::DistributionUplinkDown,
        "distribution_uplink_healthy" => Signal::DistributionUplinkHealthy,
        "subscriber_unreachable" => Signal::SubscriberUnreachable,
        "ont_optical_los" => Signal::OntOpticalLos,
        "neighbor_ont_healthy" => Signal::NeighborOntHealthy,
        "ont_optical_normal" => Signal::OntOpticalNormal,
        "pppoe_authentication_rejected" => Signal::PppoeAuthenticationRejected,
        "cwmp_inform_missing" => Signal::CwmpInformMissing,
        "access_pon_all_configured_offline" => Signal::AccessPonAllConfiguredOffline,
        _ => return None,
    })
}
fn diagnostic_signal_name(value: Signal) -> &'static str {
    match value {
        Signal::DistributionUplinkDown => "distribution_uplink_down",
        Signal::DistributionUplinkHealthy => "distribution_uplink_healthy",
        Signal::SubscriberUnreachable => "subscriber_unreachable",
        Signal::OntOpticalLos => "ont_optical_los",
        Signal::NeighborOntHealthy => "neighbor_ont_healthy",
        Signal::OntOpticalNormal => "ont_optical_normal",
        Signal::PppoeAuthenticationRejected => "pppoe_authentication_rejected",
        Signal::CwmpInformMissing => "cwmp_inform_missing",
        Signal::AccessPonAllConfiguredOffline => "access_pon_all_configured_offline",
    }
}
fn hypothesis_name(value: Hypothesis) -> &'static str {
    match value {
        Hypothesis::DistributionPath => "distribution_path",
        Hypothesis::OntAccess => "ont_access",
        Hypothesis::PppoeAuthentication => "pppoe_authentication",
        Hypothesis::AccessPonSegment => "access_pon_segment",
        Hypothesis::InsufficientEvidence => "insufficient_evidence",
        Hypothesis::ConflictingEvidence => "conflicting_evidence",
    }
}
async fn diagnostic_snapshot(
    State(s): State<Arc<StateData>>,
    headers: HeaderMap,
    Query(q): Query<DiagnosticQuery>,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let Some(a) = actor(&s.db, &headers, false).await else {
        return denied(StatusCode::UNAUTHORIZED);
    };
    let Ok(distribution) = Uuid::parse_str(&q.distribution_device_id) else {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    };
    let max_age = q.max_age_seconds.unwrap_or(300);
    if !(1..=3600).contains(&max_age) {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let subscribers =
        s.db.query(
            "SELECT subscriber_id,pop_code,distribution_device_id,access_device_id,
                    topology_state,topology_verified_epoch
         FROM ipat_platform.list_subscriber360($1,$2,$3::uuid)",
            &[&a.issuer, &a.subject, &a.tenant_id],
        )
        .await;
    let Ok(subscribers) = subscribers else {
        return denied(StatusCode::SERVICE_UNAVAILABLE);
    };
    let now_epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|x| x.as_secs())
        .unwrap_or(0);
    let mut pops = std::collections::BTreeSet::new();
    let mut topology_verified = true;
    let mut topology_oldest_verified_epoch: Option<u64> = None;
    let mut distribution_subscribers = 0usize;
    let mut subscriber_access = std::collections::BTreeMap::<String, Option<String>>::new();
    for row in &subscribers {
        if row.get::<_, Uuid>(2) == distribution {
            distribution_subscribers += 1;
            pops.insert(row.get::<_, String>(1));
            let subscriber_id: String = row.get(0);
            let access_id: Option<Uuid> = row.get(3);
            subscriber_access.insert(subscriber_id, access_id.map(|v| v.to_string()));
            let state: String = row.get(4);
            let verified: Option<i64> = row.get(5);
            let current = state == "verified"
                && verified.is_some_and(|epoch| {
                    epoch >= 0
                        && (epoch as u64) <= now_epoch
                        && now_epoch.saturating_sub(epoch as u64) <= 86_400
                });
            topology_verified &= current;
            if current {
                let epoch = verified.expect("checked verified timestamp") as u64;
                topology_oldest_verified_epoch =
                    Some(topology_oldest_verified_epoch.map_or(epoch, |old| old.min(epoch)));
            }
        }
    }
    if pops.len() != 1 || distribution_subscribers == 0 {
        return response(
            StatusCode::OK,
            json!({"ok":true,
          "hypothesis":"insufficient_evidence","affected_subscribers":[],
          "evidence":[],"discarded_stale":0,"uncertainty":"high",
          "reason":"TOPOLOGY_SCOPE_NOT_UNIQUE_OR_EMPTY",
          "requires_operator_review":true,"remediation_permitted":false}),
        );
    }
    let pop = pops.into_iter().next().expect("one pop");
    let rows =
        s.db.query(
            "SELECT subscriber_id,access_device_id,signal,source_id,observed_epoch,evidence_sha256
         FROM ipat_platform.list_diagnostic_observations($1,$2,$3::uuid,$4::uuid)",
            &[&a.issuer, &a.subject, &a.tenant_id, &distribution],
        )
        .await;
    let Ok(rows) = rows else {
        return denied(StatusCode::SERVICE_UNAVAILABLE);
    };
    let tenant = match TenantId::parse(&a.tenant_id.to_string()) {
        Ok(v) => v,
        Err(_) => return denied(StatusCode::SERVICE_UNAVAILABLE),
    };
    let mut observations = Vec::with_capacity(rows.len());
    let mut evidence_hashes = std::collections::BTreeMap::<
        (String, u64, Option<String>, Option<String>, String),
        String,
    >::new();
    for row in rows {
        let signal_raw: String = row.get(2);
        let Some(signal) = diagnostic_signal(&signal_raw) else {
            return denied(StatusCode::SERVICE_UNAVAILABLE);
        };
        let observed: i64 = row.get(4);
        if observed < 0 {
            return denied(StatusCode::SERVICE_UNAVAILABLE);
        }
        let subscriber_id: Option<String> = row.get(0);
        let stored_access_id: Option<Uuid> = row.get(1);
        let access_id = stored_access_id.map(|v| v.to_string()).or_else(|| {
            subscriber_id
                .as_ref()
                .and_then(|id| subscriber_access.get(id).cloned().flatten())
        });
        let source_id: String = row.get(3);
        let evidence_sha: String = row.get(5);
        let key = (
            source_id.clone(),
            observed as u64,
            subscriber_id.clone(),
            access_id.clone(),
            signal_raw.clone(),
        );
        if evidence_hashes.insert(key, evidence_sha).is_some() {
            return denied(StatusCode::SERVICE_UNAVAILABLE);
        }
        observations.push(Observation {
            tenant_id: tenant.clone(),
            pop_id: pop.clone(),
            distribution_id: distribution.to_string(),
            subscriber_id,
            access_id,
            signal,
            source_id,
            observed_at_epoch: observed as u64,
        });
    }
    let scope = Scope {
        tenant_id: tenant,
        pop_id: pop.clone(),
        distribution_id: distribution.to_string(),
        topology_verified,
    };
    let result = match diagnose(&scope, &observations, now_epoch, max_age) {
        Ok(v) => v,
        Err(_) => return denied(StatusCode::SERVICE_UNAVAILABLE),
    };
    let uncertainty = match result.hypothesis {
        Hypothesis::DistributionPath
        | Hypothesis::OntAccess
        | Hypothesis::PppoeAuthentication
        | Hypothesis::AccessPonSegment
            if topology_verified && result.evidence.len() >= 3 =>
        {
            "medium"
        }
        _ => "high",
    };
    let evidence = result
        .evidence
        .iter()
        .map(|o| {
            let key = (
                o.source_id.clone(),
                o.observed_at_epoch,
                o.subscriber_id.clone(),
                o.access_id.clone(),
                diagnostic_signal_name(o.signal).to_string(),
            );
            json!({
                "subscriber_id":o.subscriber_id,
                "access_device_id":o.access_id,
                "signal":diagnostic_signal_name(o.signal),
                "source_id":o.source_id,
                "observed_at_epoch":o.observed_at_epoch,
                "evidence_sha256":evidence_hashes.get(&key),
            })
        })
        .collect::<Vec<_>>();
    response(
        StatusCode::OK,
        json!({"ok":true,
      "hypothesis":hypothesis_name(result.hypothesis),
      "affected_subscribers":result.affected_subscribers,
      "evidence":evidence,"discarded_stale":result.discarded_stale,
      "uncertainty":uncertainty,"pop_code":pop,
      "distribution_device_id":distribution.to_string(),
      "topology_verified":topology_verified,
      "topology_oldest_verified_epoch":topology_oldest_verified_epoch,
      "requires_operator_review":result.requires_operator_review,
      "remediation_permitted":result.remediation_permitted}),
    )
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
        .route(
            "/api/v1/subscribers",
            get(list_subscribers360).post(upsert_subscriber360),
        )
        .route(
            "/api/v1/pppoe-plans",
            get(list_pppoe_plans).post(create_pppoe_plan),
        )
        .route("/api/v1/pppoe-plans/{id}", get(pppoe_plan_items))
        .route(
            "/api/v1/pppoe-plans/{id}/review",
            axum::routing::post(review_pppoe_plan),
        )
        .route(
            "/api/v1/pppoe-plans/{id}/arm",
            axum::routing::post(arm_pppoe_execution),
        )
        .route(
            "/api/v1/pppoe-plans/{id}/execution",
            get(pppoe_execution_status),
        )
        .route("/api/v1/diagnostics", get(diagnostic_snapshot))
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
        c.host(std::env::var("PGHOST").as_deref().unwrap_or("127.0.0.1"));
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
    async fn r1007_real_pg_axum_pppoe_batch_dry_run_never_executes_routeros() {
        if std::env::var("IPAT_PG_EPHEMERAL_TEST").as_deref() != Ok("1")
            || std::env::var("PGDATABASE").as_deref() != Ok("ipat_synthetic")
        {
            return;
        }
        let admin = connect_admin().await;
        let schema_ready: bool = admin
            .query_one(
                "SELECT to_regprocedure(
                 'ipat_platform.create_pppoe_batch_dry_run(text,text,uuid,uuid,uuid,uuid,text,text,jsonb)'
                 ) IS NOT NULL",
                &[],
            )
            .await
            .unwrap()
            .get(0);
        if !schema_ready {
            return;
        }

        let tenant = Uuid::parse_str("a1070000-0000-4000-8000-000000000001").unwrap();
        let domain = Uuid::parse_str("a1070000-0000-4000-8000-000000000002").unwrap();
        let router_id = Uuid::parse_str("a1070000-0000-4000-8000-000000000003").unwrap();
        let maker_session = Uuid::parse_str("a1070000-0000-4000-8000-000000000004").unwrap();
        let checker_session = Uuid::parse_str("a1070000-0000-4000-8000-000000000005").unwrap();
        let request_id = Uuid::parse_str("a1070000-0000-4000-8000-000000000006").unwrap();
        let issuer = "https://identity.r1007.synthetic.invalid/realms/ipat";
        let maker = "r1007-http-maker";
        let checker = "r1007-http-checker";
        let host = "tenant-r1007.example.net";
        let maker_cookie = "mmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm";
        let maker_csrf = "nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn";
        let checker_cookie = "ooooooooooooooooooooooooooooooooooooooooooo";
        let checker_csrf = "ppppppppppppppppppppppppppppppppppppppppppp";

        admin
            .execute(
                "INSERT INTO ipat_platform.tenants(id,tenant_slug,state)
             VALUES($1,'r1007-http','active')",
                &[&tenant],
            )
            .await
            .unwrap();
        for subject in [maker, checker] {
            admin.execute(
                "INSERT INTO ipat_platform.identity_memberships(
                 tenant_id,issuer,subject,role,approved_by,expires_at)
                 VALUES($1,$2,$3,'tenant_admin','independent-r1007-review',clock_timestamp()+interval '1 day')",
                &[&tenant, &issuer, &subject],
            )
            .await
            .unwrap();
        }
        admin
            .execute(
                "INSERT INTO ipat_platform.tenant_domains(
             id,tenant_id,hostname,domain_type,verification_state,verification_method,verified_at,
             routing_mode,verification_name,verification_value,requested_by_issuer,
             requested_by_subject,requested_at,activation_state,ownership_verified_at,
             routing_ready_at,tls_ready_at,activated_at)
             VALUES($1,$2,$3,'custom_domain','verified','dns_txt',clock_timestamp(),'a_record',
             $4,$5,$6,$7,clock_timestamp(),'active',clock_timestamp(),clock_timestamp(),
             clock_timestamp(),clock_timestamp())",
                &[
                    &domain,
                    &tenant,
                    &host,
                    &format!("_ipat-verify.{host}"),
                    &format!("ipat-domain={domain}"),
                    &issuer,
                    &maker,
                ],
            )
            .await
            .unwrap();

        assert_eq!(
            admin.query_one(
                "SELECT ipat_platform.create_tenant_pop($1,$2,$3::uuid,'POP-R1007','R1007 POP')",
                &[&issuer,&maker,&tenant]
            ).await.unwrap().get::<_,Option<String>>(0).as_deref(),
            Some("POP-R1007")
        );
        assert_eq!(
            admin.query_one(
                "SELECT ipat_platform.create_tenant_site($1,$2,$3::uuid,'SITE-R1007','R1007 Site')",
                &[&issuer,&maker,&tenant]
            ).await.unwrap().get::<_,Option<String>>(0).as_deref(),
            Some("SITE-R1007")
        );
        assert_eq!(
            admin
                .query_one(
                    "SELECT ipat_platform.assign_tenant_site_to_pop(
                 $1,$2,$3::uuid,'SITE-R1007','POP-R1007',1)",
                    &[&issuer, &maker, &tenant]
                )
                .await
                .unwrap()
                .get::<_, Option<i64>>(0),
            Some(2)
        );
        let router_request = Uuid::parse_str("a1070000-0000-4000-8000-000000000007").unwrap();
        assert_eq!(
            admin
                .query_one(
                    "SELECT ipat_platform.register_managed_device(
                 $1,$2,$3::uuid,$4::uuid,$5::uuid,'SITE-R1007',
                 'R1007 synthetic MikroTik','router','MikroTik','TEST-ONLY',
                 'routeros_api_ssl','router.invalid',8729,NULL)",
                    &[&issuer, &maker, &tenant, &router_id, &router_request]
                )
                .await
                .unwrap()
                .get::<_, Option<Uuid>>(0),
            Some(router_id)
        );
        for (sid, current) in [
            ("SUB-R1007-CREATE", None),
            ("SUB-R1007-UPDATE", Some("old-r1007")),
        ] {
            let revision: Option<i64> = admin
                .query_one(
                    "SELECT ipat_platform.upsert_subscriber360(
                 $1,$2,$3::uuid,$4,$4,$5,'POP-R1007','SITE-R1007',$6::uuid,NULL,NULL,0)",
                    &[&issuer, &maker, &tenant, &sid, &current, &router_id],
                )
                .await
                .unwrap()
                .get(0);
            assert_eq!(revision, Some(1));
        }

        let issuer_db = connect_admin().await;
        issuer_db
            .batch_execute("SET ROLE ipat_oidc_session_issuer_login")
            .await
            .unwrap();
        let issued_maker: Option<Uuid> = issuer_db
            .query_one(
                "SELECT ipat_platform.issue_tenant_browser_session(
             $1,$2,$3::uuid,$4::uuid,$5::uuid,$6,$7,clock_timestamp()+interval '10 minutes')",
                &[
                    &issuer,
                    &maker,
                    &tenant,
                    &domain,
                    &maker_session,
                    &hex(maker_cookie),
                    &hex(maker_csrf),
                ],
            )
            .await
            .unwrap()
            .get(0);
        assert_eq!(issued_maker, Some(maker_session));
        let issued_checker: Option<Uuid> = issuer_db
            .query_one(
                "SELECT ipat_platform.issue_tenant_browser_session(
             $1,$2,$3::uuid,$4::uuid,$5::uuid,$6,$7,clock_timestamp()+interval '10 minutes')",
                &[
                    &issuer,
                    &checker,
                    &tenant,
                    &domain,
                    &checker_session,
                    &hex(checker_cookie),
                    &hex(checker_csrf),
                ],
            )
            .await
            .unwrap()
            .get(0);
        assert_eq!(issued_checker, Some(checker_session));

        let api_db = connect_admin().await;
        api_db
            .batch_execute("SET ROLE ipat_tenant_api_login")
            .await
            .unwrap();
        let app = router(Arc::new(api_db));
        let vault_create = format!("vault://tenant/{tenant}/pppoe/SUB-R1007-CREATE");
        let vault_update = format!("vault://tenant/{tenant}/pppoe/SUB-R1007-UPDATE");
        let body = json!({
            "request_id":request_id.to_string(),
            "router_id":router_id.to_string(),
            "items":[
                {"subscriber_id":"SUB-R1007-CREATE","action":"create",
                 "username":"new-r1007-create","profile":"default","secret_ref":vault_create},
                {"subscriber_id":"SUB-R1007-UPDATE","action":"update",
                 "username":"new-r1007-update","profile":"premium","secret_ref":vault_update}
            ]
        })
        .to_string();

        let no_session = app
            .clone()
            .oneshot(req(
                "POST",
                "/api/v1/pppoe-plans",
                host,
                None,
                Some(maker_csrf),
                Some(&body),
            ))
            .await
            .unwrap();
        assert_eq!(no_session.status(), StatusCode::UNAUTHORIZED);
        let wrong_host = app
            .clone()
            .oneshot(req(
                "POST",
                "/api/v1/pppoe-plans",
                "wrong-r1007.example.net",
                Some(maker_cookie),
                Some(maker_csrf),
                Some(&body),
            ))
            .await
            .unwrap();
        assert_eq!(wrong_host.status(), StatusCode::UNAUTHORIZED);

        let role_split_ready: bool = admin
            .query_one(
                "SELECT to_regprocedure(
                   'ipat_platform.pppoe_batch_create_capability(text,text,uuid)'
                 ) IS NOT NULL",
                &[],
            )
            .await
            .unwrap()
            .get(0);

        let created = app
            .clone()
            .oneshot(req(
                "POST",
                "/api/v1/pppoe-plans",
                host,
                Some(maker_cookie),
                Some(maker_csrf),
                Some(&body),
            ))
            .await
            .unwrap();

        // R10.07 historically used two distinct Tenant Admins because the
        // commercial vocabulary did not yet contain the final maker/checker
        // roles. Once append-only R10.09 is present in a shared CI database,
        // retaining that old right would be a privilege-regression. The
        // dedicated R10.09 integration below covers the full create/review
        // flow with Provisioning Officer + Security Admin. This older test
        // still proves that the dry-run never gains an execution function.
        // The R10.09 BFF source is fail-closed both before and after the
        // append-only 0039 migration: before 0039 the role-specific functions
        // do not exist, after 0039 a Tenant Admin is intentionally not either
        // PPPoE workflow role. Never preserve the legacy Tenant Admin success
        // just to keep an older integration fixture green.
        assert_eq!(created.status(), StatusCode::FORBIDDEN);
        let no_execute: bool = admin
            .query_one(
                "SELECT to_regprocedure('ipat_platform.execute_pppoe_batch(uuid)') IS NULL",
                &[],
            )
            .await
            .unwrap()
            .get(0);
        assert!(no_execute);
        if !role_split_ready {
            // Expected rolling-deploy safety state: new BFF with old schema
            // denies the high-risk module until 0039 is applied.
            return;
        }
        return;
        let created_json = body_json(created).await;
        assert_eq!(created_json["id"], request_id.to_string());
        assert_eq!(created_json["execution_allowed"], false);
        assert_eq!(created_json["physical_readback_verified"], false);

        let exact_replay = app
            .clone()
            .oneshot(req(
                "POST",
                "/api/v1/pppoe-plans",
                host,
                Some(maker_cookie),
                Some(maker_csrf),
                Some(&body),
            ))
            .await
            .unwrap();
        assert_eq!(exact_replay.status(), StatusCode::CREATED);

        let list = app
            .clone()
            .oneshot(req(
                "GET",
                "/api/v1/pppoe-plans",
                host,
                Some(maker_cookie),
                None,
                None,
            ))
            .await
            .unwrap();
        assert_eq!(list.status(), StatusCode::OK);
        let list_json = body_json(list).await;
        assert_eq!(list_json["physical_execution_available"], false);
        assert!(list_json["plans"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["id"] == request_id.to_string()
                && p["state"] == "awaiting_approval"
                && p["execution_allowed"] == false));

        let detail = app
            .clone()
            .oneshot(req(
                "GET",
                &format!("/api/v1/pppoe-plans/{request_id}"),
                host,
                Some(maker_cookie),
                None,
                None,
            ))
            .await
            .unwrap();
        assert_eq!(detail.status(), StatusCode::OK);
        let detail_json = body_json(detail).await;
        assert_eq!(detail_json["secret_values_returned"], false);
        assert_eq!(detail_json["physical_execution_available"], false);
        let detail_text = detail_json.to_string();
        assert!(!detail_text.contains("vault://"));
        assert!(!detail_text.contains("password"));

        let maker_approve = app
            .clone()
            .oneshot(req(
                "POST",
                &format!("/api/v1/pppoe-plans/{request_id}/review"),
                host,
                Some(maker_cookie),
                Some(maker_csrf),
                Some(r#"{"approve":true}"#),
            ))
            .await
            .unwrap();
        assert_eq!(maker_approve.status(), StatusCode::CONFLICT);

        let checker_approve = app
            .clone()
            .oneshot(req(
                "POST",
                &format!("/api/v1/pppoe-plans/{request_id}/review"),
                host,
                Some(checker_cookie),
                Some(checker_csrf),
                Some(r#"{"approve":true}"#),
            ))
            .await
            .unwrap();
        assert_eq!(checker_approve.status(), StatusCode::OK);
        let approved = body_json(checker_approve).await;
        assert_eq!(approved["state"], "approved");
        assert_eq!(approved["execution_allowed"], false);
        assert_eq!(approved["physical_execution_available"], false);

        let row = admin
            .query_one(
                "SELECT state,execution_allowed,physical_readback_verified,
             approval_expires_at>reviewed_at,
             approval_expires_at<=reviewed_at+interval '30 minutes'
             FROM ipat_ops.pppoe_batch_plans WHERE tenant_id=$1 AND id=$2",
                &[&tenant, &request_id],
            )
            .await
            .unwrap();
        assert_eq!(row.get::<_, String>(0), "approved");
        assert!(!row.get::<_, bool>(1));
        assert!(!row.get::<_, bool>(2));
        assert!(row.get::<_, bool>(3));
        assert!(row.get::<_, bool>(4));
        let no_execute: bool = admin
            .query_one(
                "SELECT to_regprocedure('ipat_platform.execute_pppoe_batch(uuid)') IS NULL",
                &[],
            )
            .await
            .unwrap()
            .get(0);
        assert!(no_execute);
    }

    #[tokio::test]
    async fn r1009_real_pg_axum_pppoe_role_separation_hides_and_denies_wrong_actions() {
        if std::env::var("IPAT_PG_EPHEMERAL_TEST").as_deref() != Ok("1")
            || std::env::var("PGDATABASE").as_deref() != Ok("ipat_synthetic")
        {
            return;
        }
        let admin = connect_admin().await;
        let ready: bool = admin
            .query_one(
                "SELECT to_regprocedure(
                 'ipat_platform.pppoe_batch_create_capability(text,text,uuid)'
                 ) IS NOT NULL",
                &[],
            )
            .await
            .unwrap()
            .get(0);
        if !ready {
            return;
        }

        let tenant = Uuid::parse_str("a1090000-0000-4000-8000-000000000001").unwrap();
        let domain = Uuid::parse_str("a1090000-0000-4000-8000-000000000002").unwrap();
        let router_id = Uuid::parse_str("a1090000-0000-4000-8000-000000000003").unwrap();
        let admin_session = Uuid::parse_str("a1090000-0000-4000-8000-000000000004").unwrap();
        let maker_session = Uuid::parse_str("a1090000-0000-4000-8000-000000000005").unwrap();
        let checker_session = Uuid::parse_str("a1090000-0000-4000-8000-000000000006").unwrap();
        let request_id = Uuid::parse_str("a1090000-0000-4000-8000-000000000007").unwrap();
        let issuer = "https://identity.r1009.synthetic.invalid/realms/ipat";
        let admin_subject = "r1009-http-admin";
        let maker = "r1009-http-provisioner";
        let checker = "r1009-http-security";
        let host = "tenant-r1009.example.net";
        let admin_cookie = "r1009ar1009ar1009ar1009ar1009ar1009ar1009ax";
        let admin_csrf = "r1009br1009br1009br1009br1009br1009br1009bx";
        let maker_cookie = "r1009cr1009cr1009cr1009cr1009cr1009cr1009cx";
        let maker_csrf = "r1009dr1009dr1009dr1009dr1009dr1009dr1009dx";
        let checker_cookie = "r1009er1009er1009er1009er1009er1009er1009ex";
        let checker_csrf = "r1009fr1009fr1009fr1009fr1009fr1009fr1009fx";

        admin
            .execute(
                "INSERT INTO ipat_platform.tenants(id,tenant_slug,state)
                 VALUES($1,'r1009-http','active')",
                &[&tenant],
            )
            .await
            .unwrap();
        admin
            .execute(
                "INSERT INTO ipat_platform.identity_memberships(
                 tenant_id,issuer,subject,role,approved_by,expires_at) VALUES
                 ($1,$2,$3,'tenant_admin','independent-admin',clock_timestamp()+interval '1 day'),
                 ($1,$2,$4,'provisioning_officer','independent-provisioning',clock_timestamp()+interval '1 day'),
                 ($1,$2,$5,'security_admin','independent-security',clock_timestamp()+interval '1 day')",
                &[&tenant, &issuer, &admin_subject, &maker, &checker],
            )
            .await
            .unwrap();
        admin
            .execute(
                "INSERT INTO ipat_platform.tenant_domains(
             id,tenant_id,hostname,domain_type,verification_state,verification_method,verified_at,
             routing_mode,verification_name,verification_value,requested_by_issuer,
             requested_by_subject,requested_at,activation_state,ownership_verified_at,
             routing_ready_at,tls_ready_at,activated_at)
             VALUES($1,$2,$3,'custom_domain','verified','dns_txt',clock_timestamp(),'a_record',
             $4,$5,$6,$7,clock_timestamp(),'active',clock_timestamp(),clock_timestamp(),
             clock_timestamp(),clock_timestamp())",
                &[
                    &domain,
                    &tenant,
                    &host,
                    &format!("_ipat-verify.{host}"),
                    &format!("ipat-domain={domain}"),
                    &issuer,
                    &admin_subject,
                ],
            )
            .await
            .unwrap();
        assert_eq!(
            admin.query_one(
                "SELECT ipat_platform.create_tenant_pop($1,$2,$3::uuid,'POP-R1009','R1009 POP')",
                &[&issuer,&admin_subject,&tenant]
            ).await.unwrap().get::<_,Option<String>>(0).as_deref(),
            Some("POP-R1009")
        );
        assert_eq!(
            admin.query_one(
                "SELECT ipat_platform.create_tenant_site($1,$2,$3::uuid,'SITE-R1009','R1009 Site')",
                &[&issuer,&admin_subject,&tenant]
            ).await.unwrap().get::<_,Option<String>>(0).as_deref(),
            Some("SITE-R1009")
        );
        assert_eq!(
            admin
                .query_one(
                    "SELECT ipat_platform.assign_tenant_site_to_pop(
                 $1,$2,$3::uuid,'SITE-R1009','POP-R1009',1)",
                    &[&issuer, &admin_subject, &tenant]
                )
                .await
                .unwrap()
                .get::<_, Option<i64>>(0),
            Some(2)
        );
        let router_request = Uuid::parse_str("a1090000-0000-4000-8000-000000000008").unwrap();
        assert_eq!(
            admin
                .query_one(
                    "SELECT ipat_platform.register_managed_device(
                 $1,$2,$3::uuid,$4::uuid,$5::uuid,'SITE-R1009',
                 'R1009 MikroTik','router','MikroTik','TEST-ONLY',
                 'routeros_api_ssl','router.invalid',8729,NULL)",
                    &[
                        &issuer,
                        &admin_subject,
                        &tenant,
                        &router_id,
                        &router_request
                    ]
                )
                .await
                .unwrap()
                .get::<_, Option<Uuid>>(0),
            Some(router_id)
        );
        assert_eq!(
            admin
                .query_one(
                    "SELECT ipat_platform.upsert_subscriber360(
                 $1,$2,$3::uuid,'SUB-R1009','SUB-R1009',NULL,
                 'POP-R1009','SITE-R1009',$4::uuid,NULL,NULL,0)",
                    &[&issuer, &admin_subject, &tenant, &router_id]
                )
                .await
                .unwrap()
                .get::<_, Option<i64>>(0),
            Some(1)
        );

        let issuer_db = connect_admin().await;
        issuer_db
            .batch_execute("SET ROLE ipat_oidc_session_issuer_login")
            .await
            .unwrap();
        for (subject, sid, cookie, csrf) in [
            (admin_subject, admin_session, admin_cookie, admin_csrf),
            (maker, maker_session, maker_cookie, maker_csrf),
            (checker, checker_session, checker_cookie, checker_csrf),
        ] {
            let issued: Option<Uuid> = issuer_db
                .query_one(
                    "SELECT ipat_platform.issue_tenant_browser_session(
                 $1,$2,$3::uuid,$4::uuid,$5::uuid,$6,$7,clock_timestamp()+interval '10 minutes')",
                    &[
                        &issuer,
                        &subject,
                        &tenant,
                        &domain,
                        &sid,
                        &hex(cookie),
                        &hex(csrf),
                    ],
                )
                .await
                .unwrap()
                .get(0);
            assert_eq!(issued, Some(sid));
        }

        let api_db = connect_admin().await;
        api_db
            .batch_execute("SET ROLE ipat_tenant_api_login")
            .await
            .unwrap();
        let app = router(Arc::new(api_db));

        let admin_caps = app
            .clone()
            .oneshot(req(
                "GET",
                "/api/v1/capabilities",
                host,
                Some(admin_cookie),
                None,
                None,
            ))
            .await
            .unwrap();
        assert_eq!(admin_caps.status(), StatusCode::OK);
        let admin_caps = body_json(admin_caps).await;
        assert_eq!(admin_caps["can_manage_pppoe_plans"], false);
        assert_eq!(admin_caps["can_create_pppoe_plans"], false);
        assert_eq!(admin_caps["can_review_pppoe_plans"], false);

        let maker_caps = body_json(
            app.clone()
                .oneshot(req(
                    "GET",
                    "/api/v1/capabilities",
                    host,
                    Some(maker_cookie),
                    None,
                    None,
                ))
                .await
                .unwrap(),
        )
        .await;
        assert_eq!(maker_caps["can_manage_pppoe_plans"], true);
        assert_eq!(maker_caps["can_create_pppoe_plans"], true);
        assert_eq!(maker_caps["can_review_pppoe_plans"], false);

        let checker_caps = body_json(
            app.clone()
                .oneshot(req(
                    "GET",
                    "/api/v1/capabilities",
                    host,
                    Some(checker_cookie),
                    None,
                    None,
                ))
                .await
                .unwrap(),
        )
        .await;
        assert_eq!(checker_caps["can_manage_pppoe_plans"], true);
        assert_eq!(checker_caps["can_create_pppoe_plans"], false);
        assert_eq!(checker_caps["can_review_pppoe_plans"], true);

        let vault = format!("vault://tenant/{tenant}/pppoe/SUB-R1009");
        let body = json!({
            "request_id":request_id.to_string(),
            "router_id":router_id.to_string(),
            "items":[{"subscriber_id":"SUB-R1009","action":"create",
                      "username":"r1009-user","profile":"default","secret_ref":vault}]
        })
        .to_string();

        let admin_create = app
            .clone()
            .oneshot(req(
                "POST",
                "/api/v1/pppoe-plans",
                host,
                Some(admin_cookie),
                Some(admin_csrf),
                Some(&body),
            ))
            .await
            .unwrap();
        assert_eq!(admin_create.status(), StatusCode::FORBIDDEN);
        let checker_create = app
            .clone()
            .oneshot(req(
                "POST",
                "/api/v1/pppoe-plans",
                host,
                Some(checker_cookie),
                Some(checker_csrf),
                Some(&body),
            ))
            .await
            .unwrap();
        assert_eq!(checker_create.status(), StatusCode::FORBIDDEN);
        let created = app
            .clone()
            .oneshot(req(
                "POST",
                "/api/v1/pppoe-plans",
                host,
                Some(maker_cookie),
                Some(maker_csrf),
                Some(&body),
            ))
            .await
            .unwrap();
        assert_eq!(created.status(), StatusCode::CREATED);
        assert_eq!(body_json(created).await["execution_allowed"], false);

        let admin_list = app
            .clone()
            .oneshot(req(
                "GET",
                "/api/v1/pppoe-plans",
                host,
                Some(admin_cookie),
                None,
                None,
            ))
            .await
            .unwrap();
        assert_eq!(admin_list.status(), StatusCode::FORBIDDEN);
        let maker_list = body_json(
            app.clone()
                .oneshot(req(
                    "GET",
                    "/api/v1/pppoe-plans",
                    host,
                    Some(maker_cookie),
                    None,
                    None,
                ))
                .await
                .unwrap(),
        )
        .await;
        assert_eq!(maker_list["plans"][0]["can_review"], false);
        let checker_list = body_json(
            app.clone()
                .oneshot(req(
                    "GET",
                    "/api/v1/pppoe-plans",
                    host,
                    Some(checker_cookie),
                    None,
                    None,
                ))
                .await
                .unwrap(),
        )
        .await;
        assert_eq!(checker_list["plans"][0]["can_review"], true);

        let maker_review = app
            .clone()
            .oneshot(req(
                "POST",
                &format!("/api/v1/pppoe-plans/{request_id}/review"),
                host,
                Some(maker_cookie),
                Some(maker_csrf),
                Some(r#"{"approve":true}"#),
            ))
            .await
            .unwrap();
        assert_eq!(maker_review.status(), StatusCode::FORBIDDEN);
        let admin_review = app
            .clone()
            .oneshot(req(
                "POST",
                &format!("/api/v1/pppoe-plans/{request_id}/review"),
                host,
                Some(admin_cookie),
                Some(admin_csrf),
                Some(r#"{"approve":true}"#),
            ))
            .await
            .unwrap();
        assert_eq!(admin_review.status(), StatusCode::FORBIDDEN);
        let checker_review = app
            .clone()
            .oneshot(req(
                "POST",
                &format!("/api/v1/pppoe-plans/{request_id}/review"),
                host,
                Some(checker_cookie),
                Some(checker_csrf),
                Some(r#"{"approve":true}"#),
            ))
            .await
            .unwrap();
        assert_eq!(checker_review.status(), StatusCode::OK);
        let approved = body_json(checker_review).await;
        assert_eq!(approved["state"], "approved");
        assert_eq!(approved["execution_allowed"], false);
        assert_eq!(approved["physical_execution_available"], false);
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

    #[tokio::test]
    async fn r1005_real_pg_axum_scoped_subscriber360_and_verified_diagnostics() {
        if std::env::var("IPAT_PG_EPHEMERAL_TEST").as_deref() != Ok("1")
            || std::env::var("PGDATABASE").as_deref() != Ok("ipat_synthetic")
        {
            return;
        }
        let admin = connect_admin().await;
        let has_0035: bool = admin
            .query_one(
                "SELECT to_regprocedure('ipat_platform.upsert_subscriber360(text,text,uuid,text,text,text,text,text,uuid,uuid,text,bigint)') IS NOT NULL",
                &[],
            )
            .await
            .unwrap()
            .get(0);
        if !has_0035 {
            return;
        }

        let tenant = Uuid::parse_str("a5a5a5a5-a5a5-45a5-85a5-a5a5a5a5a5a1").unwrap();
        let domain = Uuid::parse_str("a5a5a5a5-a5a5-45a5-85a5-a5a5a5a5a5a2").unwrap();
        let router_id = Uuid::parse_str("a5a5a5a5-a5a5-45a5-85a5-a5a5a5a5a5a3").unwrap();
        let olt_id = Uuid::parse_str("a5a5a5a5-a5a5-45a5-85a5-a5a5a5a5a5a4").unwrap();
        let issuer = "https://id.r1005.synthetic.invalid/realms/ipat";
        let admin_subject = "r1005-http-admin";
        let help_subject = "r1005-http-helpdesk";
        let host = "tenant-r1005.example.net";

        admin.execute("INSERT INTO ipat_platform.tenants(id,tenant_slug,state) VALUES($1,'r1005-http','active')",&[&tenant]).await.unwrap();
        admin.execute(
            "INSERT INTO ipat_platform.identity_memberships(tenant_id,issuer,subject,role,approved_by,expires_at) VALUES
             ($1,$2,$3,'tenant_admin','independent-http',clock_timestamp()+interval '1 day'),
             ($1,$2,$4,'helpdesk','independent-http',clock_timestamp()+interval '1 day')",
            &[&tenant,&issuer,&admin_subject,&help_subject],
        ).await.unwrap();
        admin.execute(
            "INSERT INTO ipat_platform.tenant_domains(id,tenant_id,hostname,domain_type,verification_state,
             verification_method,verified_at,routing_mode,verification_name,verification_value,
             requested_by_issuer,requested_by_subject,requested_at,activation_state,
             ownership_verified_at,routing_ready_at,tls_ready_at,activated_at)
             VALUES($1,$2,$3,'custom_domain','verified','dns_txt',clock_timestamp(),'a_record',
             $4,$5,$6,$7,clock_timestamp(),'active',clock_timestamp(),clock_timestamp(),
             clock_timestamp(),clock_timestamp())",
            &[&domain,&tenant,&host,&format!("_ipat-verify.{host}"),
              &format!("ipat-domain={domain}"),&issuer,&admin_subject],
        ).await.unwrap();
        assert_eq!(admin.query_one(
            "SELECT ipat_platform.create_tenant_pop($1,$2,$3::uuid,'POP-R1005','POP R1005')",
            &[&issuer,&admin_subject,&tenant]).await.unwrap().get::<_,Option<String>>(0).as_deref(),
            Some("POP-R1005"));
        assert_eq!(admin.query_one(
            "SELECT ipat_platform.create_tenant_site($1,$2,$3::uuid,'SITE-R1005','Site R1005')",
            &[&issuer,&admin_subject,&tenant]).await.unwrap().get::<_,Option<String>>(0).as_deref(),
            Some("SITE-R1005"));
        assert_eq!(admin.query_one(
            "SELECT ipat_platform.assign_tenant_site_to_pop($1,$2,$3::uuid,'SITE-R1005','POP-R1005',1)",
            &[&issuer,&admin_subject,&tenant]).await.unwrap().get::<_,Option<i64>>(0),Some(2));
        admin.execute(
            "INSERT INTO ipat_platform.identity_pop_grants(tenant_id,issuer,subject,role,pop_id)
             VALUES($1,$2,$3,'helpdesk','SITE-R1005')",
            &[&tenant,&issuer,&help_subject],
        ).await.unwrap();

        async fn register(
            db: &Client,
            issuer: &str,
            subject: &str,
            tenant: Uuid,
            id: Uuid,
            site: &str,
            name: &str,
            kind: &str,
            vendor: &str,
            transport: &str,
            host: &str,
            port: i32,
        ) {
            let request = Uuid::new_v4();
            let row = db
                .query_one(
                    "SELECT ipat_platform.register_managed_device(
                 $1,$2,$3::uuid,$4::uuid,$5::uuid,$6,$7,$8,$9,NULL,$10,$11,$12,NULL)",
                    &[
                        &issuer, &subject, &tenant, &id, &request, &site, &name, &kind, &vendor,
                        &transport, &host, &port,
                    ],
                )
                .await
                .unwrap();
            assert_eq!(row.get::<_, Option<Uuid>>(0), Some(id));
        }
        register(
            &admin,
            issuer,
            admin_subject,
            tenant,
            router_id,
            "SITE-R1005",
            "Distribution R1005",
            "router",
            "MikroTik",
            "routeros_api_ssl",
            "router.r1005.invalid",
            8729,
        )
        .await;
        register(
            &admin,
            issuer,
            admin_subject,
            tenant,
            olt_id,
            "SITE-R1005",
            "OLT R1005",
            "olt",
            "ZTE",
            "ssh",
            "olt.r1005.invalid",
            22,
        )
        .await;

        let admin_cookie = "mmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm";
        let admin_csrf = "nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn";
        let help_cookie = "ooooooooooooooooooooooooooooooooooooooooooo";
        let help_csrf = "ppppppppppppppppppppppppppppppppppppppppppp";
        let issuer_db = connect_admin().await;
        issuer_db
            .batch_execute("SET ROLE ipat_oidc_session_issuer_login")
            .await
            .unwrap();
        for (subject, cookie, csrf, sid) in [
            (
                admin_subject,
                admin_cookie,
                admin_csrf,
                Uuid::parse_str("a5a5a5a5-a5a5-45a5-85a5-a5a5a5a5a5b1").unwrap(),
            ),
            (
                help_subject,
                help_cookie,
                help_csrf,
                Uuid::parse_str("a5a5a5a5-a5a5-45a5-85a5-a5a5a5a5a5b2").unwrap(),
            ),
        ] {
            let row = issuer_db
                .query_one(
                    "SELECT ipat_platform.issue_tenant_browser_session(
                 $1,$2,$3::uuid,$4::uuid,$5::uuid,$6,$7,clock_timestamp()+interval '10 minutes')",
                    &[
                        &issuer,
                        &subject,
                        &tenant,
                        &domain,
                        &sid,
                        &hex(cookie),
                        &hex(csrf),
                    ],
                )
                .await
                .unwrap();
            assert_eq!(row.get::<_, Option<Uuid>>(0), Some(sid));
        }
        let api_db = connect_admin().await;
        api_db
            .batch_execute("SET ROLE ipat_tenant_api_login")
            .await
            .unwrap();
        let app = router(Arc::new(api_db));

        let admin_caps = app
            .clone()
            .oneshot(req(
                "GET",
                "/api/v1/capabilities",
                host,
                Some(admin_cookie),
                None,
                None,
            ))
            .await
            .unwrap();
        assert_eq!(admin_caps.status(), StatusCode::OK);
        let admin_caps = body_json(admin_caps).await;
        assert_eq!(admin_caps["can_read_subscribers"], true);
        assert_eq!(admin_caps["can_manage_subscribers"], true);

        let help_caps = app
            .clone()
            .oneshot(req(
                "GET",
                "/api/v1/capabilities",
                host,
                Some(help_cookie),
                None,
                None,
            ))
            .await
            .unwrap();
        assert_eq!(help_caps.status(), StatusCode::OK);
        let help_caps = body_json(help_caps).await;
        assert_eq!(help_caps["can_read_subscribers"], true);
        assert_eq!(help_caps["can_manage_subscribers"], false);

        let subscriber_body = |id: &str, name: &str, ont: &str| {
            json!({
                "subscriber_id":id,"display_name":name,"pppoe_username":Value::Null,
                "pop_code":"POP-R1005","site_code":"SITE-R1005",
                "distribution_device_id":router_id.to_string(),
                "access_device_id":olt_id.to_string(),"ont_reference":ont,
                "expected_revision":0
            })
            .to_string()
        };
        for (id, name, ont) in [
            ("SUB-R1005-A", "Customer A", "1/1/1:1"),
            ("SUB-R1005-B", "Customer B", "1/1/1:2"),
        ] {
            let created = app
                .clone()
                .oneshot(req(
                    "POST",
                    "/api/v1/subscribers",
                    host,
                    Some(admin_cookie),
                    Some(admin_csrf),
                    Some(&subscriber_body(id, name, ont)),
                ))
                .await
                .unwrap();
            assert_eq!(created.status(), StatusCode::CREATED);
        }
        let help_write = app
            .clone()
            .oneshot(req(
                "POST",
                "/api/v1/subscribers",
                host,
                Some(help_cookie),
                Some(help_csrf),
                Some(&subscriber_body("SUB-R1005-X", "Forbidden", "1/1/1:9")),
            ))
            .await
            .unwrap();
        assert_eq!(help_write.status(), StatusCode::CONFLICT);

        let help_list = app
            .clone()
            .oneshot(req(
                "GET",
                "/api/v1/subscribers",
                host,
                Some(help_cookie),
                None,
                None,
            ))
            .await
            .unwrap();
        assert_eq!(help_list.status(), StatusCode::OK);
        let help_list = body_json(help_list).await;
        assert_eq!(help_list["subscribers"].as_array().unwrap().len(), 2);
        assert!(help_list["subscribers"]
            .as_array()
            .unwrap()
            .iter()
            .all(|row| row["topology_state"] == "declared"));

        let diag = connect_admin().await;
        diag.batch_execute("SET ROLE ipat_diag_ingest_exec")
            .await
            .unwrap();
        let evidence = "2".repeat(64);
        for (subscriber, signal) in [
            (Some("SUB-R1005-A"), "subscriber_unreachable"),
            (Some("SUB-R1005-B"), "subscriber_unreachable"),
        ] {
            let row = diag
                .query_one(
                    "SELECT ipat_platform.record_diagnostic_observation(
                 $1::uuid,$2::uuid,$3,$4,'telemetry.http:01',clock_timestamp(),$5)",
                    &[&tenant, &router_id, &subscriber, &signal, &evidence],
                )
                .await
                .unwrap();
            assert!(row.get::<_, Option<i64>>(0).is_some());
        }

        let path =
            format!("/api/v1/diagnostics?distribution_device_id={router_id}&max_age_seconds=300");
        let before = app
            .clone()
            .oneshot(req("GET", &path, host, Some(help_cookie), None, None))
            .await
            .unwrap();
        assert_eq!(before.status(), StatusCode::OK);
        let before = body_json(before).await;
        assert_eq!(before["hypothesis"], "insufficient_evidence");
        assert_eq!(before["topology_verified"], false);
        assert_eq!(before["remediation_permitted"], false);

        let topology = connect_admin().await;
        topology
            .batch_execute("SET ROLE ipat_topology_verify_exec")
            .await
            .unwrap();
        for (subscriber, ont) in [("SUB-R1005-A", "1/1/1:1"), ("SUB-R1005-B", "1/1/1:2")] {
            let row = topology
                .query_one(
                    "SELECT ipat_platform.record_subscriber_topology_verification(
                 $1::uuid,$2,'POP-R1005','SITE-R1005',$3::uuid,$4::uuid,$5,
                 'topology.http:01',clock_timestamp(),$6)",
                    &[&tenant, &subscriber, &router_id, &olt_id, &ont, &evidence],
                )
                .await
                .unwrap();
            assert!(row.get::<_, bool>(0));
        }
        let verified_without_access = app
            .clone()
            .oneshot(req("GET", &path, host, Some(help_cookie), None, None))
            .await
            .unwrap();
        assert_eq!(verified_without_access.status(), StatusCode::OK);
        let verified_without_access = body_json(verified_without_access).await;
        assert_eq!(
            verified_without_access["hypothesis"],
            "insufficient_evidence"
        );
        assert_eq!(verified_without_access["topology_verified"], true);

        let pon = diag
            .query_one(
                "SELECT ipat_platform.record_access_pon_aggregate_observation(
                 $1::uuid,$2::uuid,$3::uuid,'c320.http:dev01',
                 clock_timestamp(),$4,72,0,72)",
                &[&tenant, &router_id, &olt_id, &evidence],
            )
            .await
            .unwrap();
        assert!(pon.get::<_, Option<i64>>(0).is_some());

        let access = app
            .clone()
            .oneshot(req("GET", &path, host, Some(help_cookie), None, None))
            .await
            .unwrap();
        assert_eq!(access.status(), StatusCode::OK);
        let access = body_json(access).await;
        assert_eq!(access["hypothesis"], "access_pon_segment");
        assert_eq!(access["topology_verified"], true);
        assert_eq!(access["affected_subscribers"].as_array().unwrap().len(), 2);
        assert_eq!(access["remediation_permitted"], false);
        assert!(access["evidence"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["signal"] == "access_pon_all_configured_offline"
                && e["access_device_id"] == olt_id.to_string()));
        assert!(access["evidence"]
            .as_array()
            .unwrap()
            .iter()
            .all(|e| e["evidence_sha256"] == evidence));

        let distribution = diag
            .query_one(
                "SELECT ipat_platform.record_diagnostic_observation(
                 $1::uuid,$2::uuid,NULL,'distribution_uplink_down',
                 'telemetry.http:01',clock_timestamp(),$3)",
                &[&tenant, &router_id, &evidence],
            )
            .await
            .unwrap();
        assert!(distribution.get::<_, Option<i64>>(0).is_some());
        let conflict = app
            .clone()
            .oneshot(req("GET", &path, host, Some(help_cookie), None, None))
            .await
            .unwrap();
        assert_eq!(conflict.status(), StatusCode::OK);
        let conflict = body_json(conflict).await;
        assert_eq!(conflict["hypothesis"], "conflicting_evidence");
        assert!(conflict["affected_subscribers"]
            .as_array()
            .unwrap()
            .is_empty());
        assert_eq!(conflict["remediation_permitted"], false);

        let wrong_host = app
            .clone()
            .oneshot(req(
                "GET",
                "/api/v1/subscribers",
                "other-r1005.example.net",
                Some(help_cookie),
                None,
                None,
            ))
            .await
            .unwrap();
        assert_eq!(wrong_host.status(), StatusCode::UNAUTHORIZED);

        // Remove synthetic fixtures in FK-safe order.
        admin
            .execute(
                "DELETE FROM ipat_ops.diagnostic_observations WHERE tenant_id=$1",
                &[&tenant],
            )
            .await
            .unwrap();
        admin
            .execute(
                "DELETE FROM ipat_ops.subscribers WHERE tenant_id=$1",
                &[&tenant],
            )
            .await
            .unwrap();
        admin
            .execute(
                "DELETE FROM ipat_platform.tenant_browser_sessions WHERE tenant_id=$1",
                &[&tenant],
            )
            .await
            .unwrap();
        admin
            .execute(
                "DELETE FROM ipat_platform.identity_pop_grants WHERE tenant_id=$1",
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
                "DELETE FROM ipat_ops.tenant_pop_events WHERE tenant_id=$1",
                &[&tenant],
            )
            .await
            .unwrap();
        admin
            .execute(
                "DELETE FROM ipat_ops.tenant_pops WHERE tenant_id=$1",
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
