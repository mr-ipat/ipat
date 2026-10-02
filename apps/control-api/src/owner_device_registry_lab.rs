//! R9.61: durable owner-only LAB inventory, never a production tenant API.
//! A linked live C320 is taken from the existing sealed connector; a Saved
//! metadata record is NEVER displayed as authenticated device connectivity.
use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    routing::get,
    extract::DefaultBodyLimit,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    path::{Path as FsPath, PathBuf},
    sync::Arc,
};
use tokio::sync::Mutex;

const MAX_FILE_BYTES: u64 = 512 * 1024;
const MAX_DEVICES: usize = 64;
const MAX_EVENTS: usize = 2048;
const DEFAULT_OWNER_STORE: &str = "/home/openai/.local/share/ipat/r961-device-inventory";

#[derive(Clone)]
struct OwnerState {
    path: PathBuf,
    guard: Arc<Mutex<()>>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Device {
    id: String,
    revision: u64,
    display_name: String,
    pop_id: String,
    device_kind: String,
    vendor: String,
    exact_model: String,
    management_protocol: String,
    management_host: Option<String>,
    management_port: Option<u16>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Audit {
    sequence: u64,
    operation: String,
    device_id: String,
    revision: u64,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    next_id: u64,
    linked_c320_label: Option<String>,
    linked_revision: u64,
    devices: Vec<Device>,
    audit: Vec<Audit>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fields {
    display_name: String,
    pop_id: String,
    device_kind: String,
    vendor: String,
    exact_model: String,
    management_protocol: String,
    management_host: Option<String>,
    management_port: Option<u16>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LinkedRename {
    expected_revision: u64,
    display_name: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Remove {
    expected_revision: u64,
    confirm: String,
}
fn error(status: StatusCode, code: &'static str) -> (StatusCode, HeaderMap, Json<Value>) {
    (
        status,
        super::private_lab_headers("application/json; charset=utf-8"),
        Json(json!({"error":code,"owner_private_lab_only":true,"device_command_sent":false})),
    )
}
fn strict_owner(headers: &HeaderMap, mutating: bool) -> bool {
    if headers.get_all(header::HOST).iter().count() != 1
        || headers.get(header::HOST).and_then(|v| v.to_str().ok()) != Some("127.0.0.1:3002")
    {
        return false;
    }
    !mutating || (headers.get_all(header::ORIGIN).iter().count() == 1
        && headers.get(header::ORIGIN).and_then(|v| v.to_str().ok())
            == Some("http://127.0.0.1:3002")
        && headers.get_all("x-ipat-owner-private").iter().count() == 1
        && headers.get("x-ipat-owner-private").and_then(|v| v.to_str().ok()) == Some("1"))
}
fn valid_label(s: &str, max: usize) -> bool {
    !s.is_empty() && s.trim() == s && s.len() <= max
        && !s.chars().any(char::is_control)
        && !s.contains('<') && !s.contains('>')
}
fn identifier(s: &str, max: usize) -> bool {
    !s.is_empty() && s.len() <= max && s.bytes().all(|b| {
        b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_')
    })
}
fn safe_host(s: &str) -> bool {
    (3..=253).contains(&s.len())
        && s.as_bytes().first().is_some_and(u8::is_ascii_alphanumeric)
        && s.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b':'))
        && !s.contains("..")
}
fn valid(fields: &Fields) -> bool {
    let protocol = fields.management_protocol.as_str();
    let endpoint = match protocol {
        "cwmp" | "usp" => fields.management_host.is_none() && fields.management_port.is_none(),
        "ssh" | "snmp" | "api-ssl" | "https" => fields
            .management_host.as_deref().is_some_and(safe_host)
            && fields.management_port.is_some_and(|p| p > 0),
        _ => false,
    };
    valid_label(&fields.display_name, 80)
        && identifier(&fields.pop_id, 48)
        && identifier(&fields.vendor, 48)
        && valid_label(&fields.exact_model, 80)
        && ["olt", "ont", "router", "switch", "other"].contains(&fields.device_kind.as_str())
        && endpoint
        && !(fields.device_kind == "olt"
            && fields.vendor.eq_ignore_ascii_case("ZTE")
            && fields.exact_model.eq_ignore_ascii_case("C320")
            && fields.management_host.as_deref() == Some("10.10.13.233")
            && fields.management_port == Some(321))
}
fn private_dir(folder: &FsPath) -> std::io::Result<()> {
    if !folder.exists() {
        fs::create_dir(folder)?;
        fs::set_permissions(folder, fs::Permissions::from_mode(0o700))?;
    }
    let metadata = folder.symlink_metadata()?;
    if !metadata.file_type().is_dir()
        || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.permissions().mode() & 0o777 != 0o700
    {
        return Err(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "unsafe owner directory"));
    }
    Ok(())
}
fn private_file(file: &FsPath) -> std::io::Result<()> {
    let m = file.symlink_metadata()?;
    if !m.file_type().is_file()
        || m.uid() != unsafe { libc::geteuid() }
        || m.permissions().mode() & 0o777 != 0o600
        || m.nlink() != 1 || m.len() > MAX_FILE_BYTES
    {
        return Err(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "unsafe owner file"));
    }
    Ok(())
}
fn load(path: &FsPath) -> std::io::Result<Snapshot> {
    let parent = path.parent().ok_or_else(|| std::io::Error::other("no owner dir"))?;
    private_dir(parent)?;
    if !path.exists() {
        // A dangling symlink must not be treated as a fresh empty registry.
        if path.symlink_metadata().is_ok() {
            return Err(std::io::Error::other("unsafe owner registry symlink"));
        }
        return Ok(Snapshot::default());
    }
    private_file(path)?;
    let mut raw = Vec::new();
    File::open(path)?.take(MAX_FILE_BYTES + 1).read_to_end(&mut raw)?;
    let snapshot: Snapshot = serde_json::from_slice(&raw)
        .map_err(|_| std::io::Error::other("unreadable owner registry"))?;
    if snapshot.devices.len() > MAX_DEVICES || snapshot.audit.len() > MAX_EVENTS
        || snapshot.devices.iter().any(|d| !d.id.starts_with("OWN-"))
    {
        return Err(std::io::Error::other("invalid stored inventory"));
    }
    Ok(snapshot)
}
fn save(path: &FsPath, snapshot: &Snapshot) -> std::io::Result<()> {
    let folder = path.parent().ok_or_else(|| std::io::Error::other("no owner folder"))?;
    private_dir(folder)?;
    let raw = serde_json::to_vec(snapshot).map_err(std::io::Error::other)?;
    if raw.len() as u64 > MAX_FILE_BYTES {
        return Err(std::io::Error::other("owner inventory limit"));
    }
    let mut nonce = [0u8; 8];
    File::open("/dev/urandom")?.read_exact(&mut nonce)?;
    let tmp = path.with_extension(format!("json.pending-{:016x}",u64::from_ne_bytes(nonce)));
    let mut stream = OpenOptions::new().write(true).create_new(true).mode(0o600).open(&tmp)?;
    let transaction = (|| {
        stream.write_all(&raw)?;
        stream.sync_all()?;
        if path.symlink_metadata().is_ok() {
            private_file(path)?;
        }
        fs::rename(&tmp, path)?;
        File::open(folder)?.sync_all()?;
        Ok::<(), std::io::Error>(())
    })();
    if transaction.is_err() { let _ = fs::remove_file(&tmp); }
    transaction
}
fn audit(snapshot: &mut Snapshot, op: &str, id: &str, revision: u64) -> bool {
    if snapshot.audit.len() >= MAX_EVENTS { return false; }
    snapshot.audit.push(Audit {
        sequence: snapshot.audit.len() as u64 + 1,
        operation: op.into(), device_id: id.into(), revision,
    });
    true
}
fn presented(d: &Device) -> Value {
    json!({"id":d.id,"revision":d.revision,"display_name":d.display_name,
        "pop_id":d.pop_id,"device_kind":d.device_kind,"vendor":d.vendor,
        "exact_model":d.exact_model,"management_protocol":d.management_protocol,
        "management_host":d.management_host,"management_port":d.management_port,
        "linked_live_connector":false,"connection_state":"SAVED_NOT_CONNECTED",
        "health":"NOT_MEASURED","physical_writes_enabled":false})
}
fn c320_present(snapshot: &Snapshot, live: &Value) -> Value {
    let online = live.get("connector_online").and_then(Value::as_bool) == Some(true);
    let active = live.get("credentials_enrolled").and_then(Value::as_bool) == Some(true);
    let source = live.get("device_status").and_then(Value::as_str).unwrap_or("UNKNOWN");
    let state = if online && active && ["CONNECTED", "DISCONNECTED"].contains(&source) {
        source
    } else if active && online { "PENDING" } else { "UNKNOWN" };
    json!({"id":"DEV-01","revision":snapshot.linked_revision,
        "display_name":snapshot.linked_c320_label.as_deref().unwrap_or_else(||
            live.get("device_name").and_then(Value::as_str).unwrap_or("ZTE C320 Lab")),
        "pop_id":"UNASSIGNED","device_kind":"olt","vendor":"ZTE",
        "exact_model":"C320","management_protocol":"ssh",
        "management_host":null,"management_port":null,
        "linked_live_connector":true,"connection_state":state,
        "last_verified_at_utc":live.get("last_verified_at_utc"),
        "health":"NOT_MEASURED","physical_writes_enabled":false,
        "remove_requires_separate_revocation":true})
}
async fn physical(headers: HeaderMap) -> Value {
    super::c320_live_lab::connection_status(headers)
        .await
        .map(|(_, Json(v))| v)
        .unwrap_or_else(|_| json!({"connector_online":false,"credentials_enrolled":false}))
}
async fn list(
    State(state): State<OwnerState>, headers: HeaderMap,
) -> Result<(HeaderMap, Json<Value>), (StatusCode, HeaderMap, Json<Value>)> {
    if !strict_owner(&headers, false) { return Err(error(StatusCode::FORBIDDEN, "OWNER_TUNNEL_REQUIRED")); }
    let _guard = state.guard.lock().await;
    let snapshot = load(&state.path).map_err(|_| error(StatusCode::SERVICE_UNAVAILABLE, "OWNER_STORE_UNAVAILABLE"))?;
    let c320 = c320_present(&snapshot, &physical(headers).await);
    let mut rows = vec![c320];
    rows.extend(snapshot.devices.iter().map(presented));
    Ok((super::private_lab_headers("application/json; charset=utf-8"),
        Json(json!({"owner_private_lab_only":true,"persistent":true,
            "count":rows.len(),"devices":rows,"audit_count":snapshot.audit.len(),
            "production_tenant_registry":false}))))
}
async fn detail(
    State(state): State<OwnerState>, headers: HeaderMap, Path(id): Path<String>,
) -> Result<(HeaderMap, Json<Value>), (StatusCode, HeaderMap, Json<Value>)> {
    if !strict_owner(&headers, false) { return Err(error(StatusCode::FORBIDDEN, "OWNER_TUNNEL_REQUIRED")); }
    let _guard = state.guard.lock().await;
    let snapshot = load(&state.path).map_err(|_| error(StatusCode::SERVICE_UNAVAILABLE, "OWNER_STORE_UNAVAILABLE"))?;
    let row = if id == "DEV-01" { c320_present(&snapshot, &physical(headers).await) }
        else { snapshot.devices.iter().find(|d| d.id == id).map(presented)
            .ok_or_else(||error(StatusCode::NOT_FOUND,"DEVICE_NOT_FOUND"))? };
    Ok((super::private_lab_headers("application/json; charset=utf-8"),Json(json!({"device":row}))))
}
async fn create(
    State(state): State<OwnerState>, headers: HeaderMap, Json(fields): Json<Fields>,
) -> Result<(StatusCode, HeaderMap, Json<Value>), (StatusCode, HeaderMap, Json<Value>)> {
    if !strict_owner(&headers, true) { return Err(error(StatusCode::FORBIDDEN,"OWNER_TUNNEL_REQUIRED")); }
    if !valid(&fields) { return Err(error(StatusCode::BAD_REQUEST,"INVALID_DEVICE_METADATA")); }
    let _guard = state.guard.lock().await;
    let mut snapshot = load(&state.path).map_err(|_|error(StatusCode::SERVICE_UNAVAILABLE,"OWNER_STORE_UNAVAILABLE"))?;
    if snapshot.devices.len() >= MAX_DEVICES { return Err(error(StatusCode::CONFLICT,"OWNER_DEVICE_LIMIT")); }
    if snapshot.devices.iter().any(|d| d.management_host == fields.management_host
        && d.management_port == fields.management_port
        && d.management_protocol == fields.management_protocol
        && d.pop_id == fields.pop_id && fields.management_host.is_some()) {
        return Err(error(StatusCode::CONFLICT,"DUPLICATE_MANAGEMENT_ENDPOINT"));
    }
    let next = snapshot.next_id.checked_add(1).ok_or_else(||error(StatusCode::CONFLICT,"OWNER_DEVICE_LIMIT"))?;
    let id = format!("OWN-{next:06}");
    let record = Device { id: id.clone(), revision: 1, display_name: fields.display_name,
        pop_id: fields.pop_id, device_kind: fields.device_kind, vendor: fields.vendor,
        exact_model: fields.exact_model, management_protocol: fields.management_protocol,
        management_host: fields.management_host, management_port: fields.management_port };
    if !audit(&mut snapshot,"CREATE",&id,1) { return Err(error(StatusCode::CONFLICT,"OWNER_AUDIT_LIMIT")); }
    snapshot.next_id = next;
    let result = presented(&record);
    snapshot.devices.push(record);
    save(&state.path,&snapshot).map_err(|_|error(StatusCode::SERVICE_UNAVAILABLE,"OWNER_SAVE_FAILED"))?;
    Ok((StatusCode::CREATED,super::private_lab_headers("application/json; charset=utf-8"),
        Json(json!({"device":result,"durable":true,"device_command_sent":false}))))
}
async fn update(
    State(state): State<OwnerState>, headers: HeaderMap, Path(id): Path<String>, Json(input): Json<Value>,
) -> Result<(HeaderMap, Json<Value>), (StatusCode, HeaderMap, Json<Value>)> {
    if !strict_owner(&headers, true) { return Err(error(StatusCode::FORBIDDEN,"OWNER_TUNNEL_REQUIRED")); }
    let _guard = state.guard.lock().await;
    let mut snapshot = load(&state.path).map_err(|_|error(StatusCode::SERVICE_UNAVAILABLE,"OWNER_STORE_UNAVAILABLE"))?;
    if id == "DEV-01" {
        let new: LinkedRename = serde_json::from_value(input).map_err(|_|error(StatusCode::BAD_REQUEST,"LINKED_DEVICE_LABEL_ONLY"))?;
        if expected_revision != snapshot.linked_revision {return Err(error(StatusCode::CONFLICT,"STALE_REVISION"));}
        if !valid_label(&new.display_name, 80) {return Err(error(StatusCode::BAD_REQUEST,"INVALID_DEVICE_LABEL"));}
        let rev = snapshot.linked_revision + 1;
        if !audit(&mut snapshot,"RENAME_LINKED",&id,rev) {return Err(error(StatusCode::CONFLICT,"OWNER_AUDIT_LIMIT"));}
        snapshot.linked_c320_label = Some(new.display_name);
        snapshot.linked_revision = rev;
        save(&state.path,&snapshot).map_err(|_|error(StatusCode::SERVICE_UNAVAILABLE,"OWNER_SAVE_FAILED"))?;
        return Ok((super::private_lab_headers("application/json; charset=utf-8"),
            Json(json!({"updated":true,"id":id,"revision":rev,"connector_unchanged":true}))));
    }
    let mut object = input.as_object().cloned().ok_or_else(||error(StatusCode::BAD_REQUEST,"INVALID_DEVICE_METADATA"))?;
    let expected_revision = object.remove("expected_revision").and_then(|v|v.as_u64())
        .ok_or_else(||error(StatusCode::BAD_REQUEST,"INVALID_DEVICE_METADATA"))?;
    let fields: Fields = serde_json::from_value(Value::Object(object))
        .map_err(|_|error(StatusCode::BAD_REQUEST,"INVALID_DEVICE_METADATA"))?;
    if !valid(&fields) {return Err(error(StatusCode::BAD_REQUEST,"INVALID_DEVICE_METADATA"));}
    let Some(pos) = snapshot.devices.iter().position(|d|d.id == id) else {
        return Err(error(StatusCode::NOT_FOUND,"DEVICE_NOT_FOUND"));
    };
    if snapshot.devices[pos].revision != expected_revision {return Err(error(StatusCode::CONFLICT,"STALE_REVISION"));}
    if snapshot.devices.iter().enumerate().any(|(i,d)|i != pos && fields.management_host.is_some()
        && d.management_host == fields.management_host && d.management_port == fields.management_port
        && d.management_protocol == fields.management_protocol && d.pop_id == fields.pop_id) {
        return Err(error(StatusCode::CONFLICT,"DUPLICATE_MANAGEMENT_ENDPOINT"));
    }
    let rev = expected_revision.checked_add(1).ok_or_else(||error(StatusCode::CONFLICT,"STALE_REVISION"))?;
    if !audit(&mut snapshot,"UPDATE",&id,rev) {return Err(error(StatusCode::CONFLICT,"OWNER_AUDIT_LIMIT"));}
    let item = &mut snapshot.devices[pos];
    item.revision = rev;
    item.display_name = fields.display_name;
    item.pop_id = fields.pop_id;
    item.device_kind = fields.device_kind;
    item.vendor = fields.vendor;
    item.exact_model = fields.exact_model;
    item.management_protocol = fields.management_protocol;
    item.management_host = fields.management_host;
    item.management_port = fields.management_port;
    let result = presented(item);
    save(&state.path,&snapshot).map_err(|_|error(StatusCode::SERVICE_UNAVAILABLE,"OWNER_SAVE_FAILED"))?;
    Ok((super::private_lab_headers("application/json; charset=utf-8"),
        Json(json!({"updated":true,"device":result,"durable":true,"device_command_sent":false}))))
}
async fn remove(
    State(state): State<OwnerState>, headers: HeaderMap, Path(id): Path<String>, Json(input): Json<Remove>,
) -> Result<(HeaderMap, Json<Value>), (StatusCode, HeaderMap, Json<Value>)> {
    if !strict_owner(&headers, true) { return Err(error(StatusCode::FORBIDDEN,"OWNER_TUNNEL_REQUIRED")); }
    if id == "DEV-01" {return Err(error(StatusCode::CONFLICT,"ACTIVE_CONNECTOR_REQUIRES_SEPARATE_REVOCATION"));}
    if input.confirm != format!("REMOVE {id}") {return Err(error(StatusCode::BAD_REQUEST,"EXPLICIT_REMOVE_CONFIRMATION_REQUIRED"));}
    let _guard = state.guard.lock().await;
    let mut snapshot = load(&state.path).map_err(|_|error(StatusCode::SERVICE_UNAVAILABLE,"OWNER_STORE_UNAVAILABLE"))?;
    let Some(pos) = snapshot.devices.iter().position(|d|d.id == id) else {
        return Err(error(StatusCode::NOT_FOUND,"DEVICE_NOT_FOUND"));
    };
    if snapshot.devices[pos].revision != input.expected_revision {return Err(error(StatusCode::CONFLICT,"STALE_REVISION"));}
    if !audit(&mut snapshot,"REMOVE",&id,input.expected_revision) {return Err(error(StatusCode::CONFLICT,"OWNER_AUDIT_LIMIT"));}
    snapshot.devices.remove(pos);
    save(&state.path,&snapshot).map_err(|_|error(StatusCode::SERVICE_UNAVAILABLE,"OWNER_SAVE_FAILED"))?;
    Ok((super::private_lab_headers("application/json; charset=utf-8"),
        Json(json!({"removed":true,"id":id,"durable":true,"device_command_sent":false}))))
}
fn router_at(path: PathBuf) -> Router {
    let state = OwnerState {path, guard:Arc::new(Mutex::new(()))};
    Router::new()
        .route("/lab/owner/devices", get(list).post(create))
        .route("/lab/owner/devices/{id}", get(detail).put(update).delete(remove))
        .layer(DefaultBodyLimit::max(2048))
        .with_state(state)
}
pub(super) fn router() -> Router {
    router_at(PathBuf::from(DEFAULT_OWNER_STORE).join("devices.json"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::{Body,to_bytes},http::Request};
    use tower::ServiceExt;
    async fn call(router:&Router,method:&str,uri:&str,payload:&str,authorized:bool)->(StatusCode,Value) {
        let mut req=Request::builder().method(method).uri(uri).header("host","127.0.0.1:3002");
        if authorized {
            req=req.header("origin","http://127.0.0.1:3002").header("x-ipat-owner-private","1");
        }
        let resp=router.clone().oneshot(req.header("content-type","application/json")
            .body(Body::from(payload.to_owned())).unwrap()).await.unwrap();
        let code=resp.status();
        let body=to_bytes(resp.into_body(),65536).await.unwrap();
        (code,serde_json::from_slice(&body).unwrap())
    }
    fn example()->String {
        json!({"display_name":"OLT Test POP A","pop_id":"POP-A","device_kind":"olt",
        "vendor":"C-DATA","exact_model":"MODEL-TO-VERIFY","management_protocol":"ssh",
        "management_host":"192.0.2.20","management_port":22}).to_string()
    }
    #[tokio::test]
    async fn persists_crud_and_immutable_audit_across_router_restarts() {
        let dir=tempfile::tempdir().unwrap();
        let path=dir.path().join("owner").join("devices.json");
        let router=router_at(path.clone());
        let (code, created)=call(&router,"POST","/lab/owner/devices",&example(),true).await;
        assert_eq!(code,StatusCode::CREATED);
        assert_eq!(created["device"]["connection_state"],"SAVED_NOT_CONNECTED");
        assert_eq!(created["device"]["id"],"OWN-000001");
        let (code, duplicate)=call(&router,"POST","/lab/owner/devices",&example(),true).await;
        assert_eq!(code,StatusCode::CONFLICT);assert_eq!(duplicate["error"],"DUPLICATE_MANAGEMENT_ENDPOINT");
        let mut edited:Value=serde_json::from_str(&example()).unwrap();
        edited["display_name"]=json!("New Name"); edited["expected_revision"]=json!(1);
        let (code, edited_result)=call(&router,"PUT","/lab/owner/devices/OWN-000001",&edited.to_string(),true).await;
        assert_eq!(code,StatusCode::OK);assert_eq!(edited_result["device"]["revision"],2);
        let (code,_)=call(&router,"PUT","/lab/owner/devices/OWN-000001",&edited.to_string(),true).await;
        assert_eq!(code,StatusCode::CONFLICT);
        let (code, detail)=call(&router_at(path.clone()),"GET","/lab/owner/devices/OWN-000001","",false).await;
        assert_eq!(code,StatusCode::OK);assert_eq!(detail["device"]["display_name"],"New Name");
        let (code, _)=call(&router,"DELETE","/lab/owner/devices/OWN-000001",
            r#"{"expected_revision":2,"confirm":"REMOVE OTHER"}"#,true).await;
        assert_eq!(code,StatusCode::BAD_REQUEST);
        let (code, removed)=call(&router,"DELETE","/lab/owner/devices/OWN-000001",
            r#"{"expected_revision":2,"confirm":"REMOVE OWN-000001"}"#,true).await;
        assert_eq!(code,StatusCode::OK);assert_eq!(removed["removed"],true);
        let snapshot=load(&path).unwrap();
        assert_eq!(snapshot.audit.iter().map(|a|a.operation.as_str()).collect::<Vec<_>>(),vec!["CREATE","UPDATE","REMOVE"]);
        assert!(snapshot.devices.is_empty());
        assert_eq!(path.symlink_metadata().unwrap().permissions().mode()&0o777,0o600);
        assert_eq!(path.parent().unwrap().symlink_metadata().unwrap().permissions().mode()&0o777,0o700);
        let (code, _)=call(&router_at(path),"GET","/lab/owner/devices/OWN-000001","",false).await;
        assert_eq!(code,StatusCode::NOT_FOUND);
    }
    #[tokio::test]
    async fn denies_cross_origin_secrets_and_live_connector_removal() {
        let dir=tempfile::tempdir().unwrap();let path=dir.path().join("reg").join("devices.json");
        let router=router_at(path.clone());
        assert_eq!(call(&router,"POST","/lab/owner/devices",&example(),false).await.0,StatusCode::FORBIDDEN);
        let mut secret:Value=serde_json::from_str(&example()).unwrap();secret["password"]=json!("NEVERSTORE");
        assert_eq!(call(&router,"POST","/lab/owner/devices",&secret.to_string(),true).await.0,StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(call(&router,"DELETE","/lab/owner/devices/DEV-01",
            r#"{"expected_revision":0,"confirm":"REMOVE DEV-01"}"#,true).await.0,StatusCode::CONFLICT);
        let (code,_)=call(&router,"PUT","/lab/owner/devices/DEV-01",
            r#"{"expected_revision":0,"display_name":"Production-Like Lab OLT"}"#,true).await;
        assert_eq!(code,StatusCode::OK);
        assert_eq!(load(&path).unwrap().linked_c320_label.as_deref(),Some("Production-Like Lab OLT"));
        assert_eq!(call(&router,"PUT","/lab/owner/devices/DEV-01",
            r#"{"expected_revision":1,"display_name":"Lab OLT","management_host":"1.2.3.4"}"#,true).await.0,StatusCode::BAD_REQUEST);
        let (code,_)=call(&router,"PUT","/lab/owner/devices/DEV-01",
            r#"{"expected_revision":0,"display_name":"Stale rename"}"#,true).await;
        assert_eq!(code,StatusCode::CONFLICT);
    }
    #[test]
    fn refuses_symlinks_and_corrupt_owner_registry_instead_of_resetting() {
        let dir=tempfile::tempdir().unwrap();let folder=dir.path().join("owner");
        fs::create_dir(&folder).unwrap();fs::set_permissions(&folder,fs::Permissions::from_mode(0o700)).unwrap();
        let path=folder.join("devices.json");
        fs::write(&path,"not json").unwrap();fs::set_permissions(&path,fs::Permissions::from_mode(0o600)).unwrap();
        assert!(load(&path).is_err());fs::remove_file(&path).unwrap();
        std::os::unix::fs::symlink("missing.json",&path).unwrap();
        assert!(load(&path).is_err());
    }
}
