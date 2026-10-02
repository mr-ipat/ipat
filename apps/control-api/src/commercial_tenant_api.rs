//! R9.68 commercial tenant JSON API over R9.67 durable Host-bound sessions.
//!
//! Source-complete router boundary, NOT publicly mounted until deployment has
//! trusted HTTPS + real confidential OIDC callback + production PostgreSQL.
//! Tenant and actor identity come only from durable session authentication.
use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderName, HeaderValue, StatusCode},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{fs::File, io::Read, sync::Arc};
use tokio_postgres::Client;
use uuid::Uuid;

use crate::{
    durable_tenant_session,
    tenant_domain::{canonical_dns_name, canonical_host, valid_custom_domain},
};

const CSRF_HEADER: HeaderName = HeaderName::from_static("x-ipat-csrf");

#[derive(Clone)]
pub(super) struct StateData {
    pub db: Arc<Client>,
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
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SiteEdit {
    display_name: String,
    expected_revision: i64,
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
    let rows=s.db.query("SELECT code,display_name,revision,assigned_devices FROM ipat_platform.list_tenant_sites($1,$2,$3::uuid,$4,$5)",&[&a.issuer,&a.subject,&a.tenant_id,&q.pop.as_deref(),&q.after.as_deref()]).await;
    let Ok(rows) = rows else {
        return denied(StatusCode::SERVICE_UNAVAILABLE);
    };
    if rows.len() > 101 {
        return denied(StatusCode::SERVICE_UNAVAILABLE);
    }
    let has_more = rows.len() == 101;
    let mut out = Vec::new();
    for r in rows.into_iter().take(100) {
        out.push(json!({"code":r.get::<_,String>(0),"display_name":r.get::<_,String>(1),"revision":r.get::<_,i64>(2),"assigned_devices":r.get::<_,i64>(3)}));
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
    if !valid_code(&q.code) || !valid_name(&q.display_name) {
        return denied(StatusCode::UNPROCESSABLE_ENTITY);
    }
    let r =
        s.db.query_one(
            "SELECT ipat_platform.create_tenant_site($1,$2,$3::uuid,$4,$5)",
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
