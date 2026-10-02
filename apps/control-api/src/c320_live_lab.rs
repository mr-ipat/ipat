//! Owner-supervised REAL C320 READ-ONLY panel bridge, PRIVATE 127.0.0.1:3002 ONLY.
//! Opt-in at startup and a separate interactive nonroot owner Unix agent are
//! BOTH required. Not persistent SaaS adoption and NEVER a write-capable API.
use axum::{
    http::{header, HeaderMap, StatusCode},
    routing::post,
    Json, Router,
};
use serde_json::{json, Value};
use std::time::Duration;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

const SOCKET: &str = "/home/openai/.local/share/ipat/r940-live-agent/live.sock";
const MAX_REPLY: usize = 2048;

fn denied(code: StatusCode, reason: &'static str) -> (StatusCode, HeaderMap, Json<Value>) {
    (
        code,
        super::private_lab_headers("application/json; charset=utf-8"),
        Json(json!({"error":reason,"physical_writes_enabled":false,"device_adopted":false})),
    )
}
fn strict_private(headers: &HeaderMap, mutating: bool) -> bool {
    let host = headers.get_all(header::HOST);
    if host.iter().count() != 1
        || headers.get(header::HOST).and_then(|x| x.to_str().ok()) != Some("127.0.0.1:3002")
    {
        return false;
    }
    !mutating || super::device_workbench_lab::demo_csrf(headers)
}
fn sanitize_agent(v: &Value) -> Option<Value> {
    let n = |key: &str| v.get(key)?.as_u64().filter(|n| *n <= 128);
    let (u, c, o, f, configured) = (
        n("unconfigured")?,
        n("configured")?,
        n("online")?,
        n("offline")?,
        n("configuration_rows")?,
    );
    if v.get("mode")?.as_str()? != "OWNER_SUPERVISED_REAL_C320_READ_ONLY"
        || v.get("snapshot_is_live")?.as_bool() != Some(true)
        || v.get("port")?.as_str()? != "1/1/1"
        || v.get("serials_returned")?.as_bool() != Some(false)
        || v.get("device_adopted")?.as_bool() != Some(false)
        || v.get("provisioning_enabled")?.as_bool() != Some(false)
        || v.get("device_writes")?.as_u64() != Some(0)
        || o + f != c
        || configured != c
    {
        return None;
    }
    let t = v.get("read_at_utc")?.as_str()?;
    if t.len() < 19
        || t.len() > 40
        || !t
            .bytes()
            .all(|x| x.is_ascii_digit() || b"-:TZ+.".contains(&x))
    {
        return None;
    }
    Some(
        json!({"mode":"OWNER_SUPERVISED_REAL_C320_READ_ONLY","read_at_utc":t,
      "pon":"1/1/1","unconfigured":u,"configured":c,"online":o,"offline":f,
      "source":"VERIFIED_LOCAL_OWNER_AGENT_LAB_ONLY","serials_returned":false,
      "physical_writes_enabled":false,"device_adopted":false}),
    )
}
async fn refresh(
    headers: HeaderMap,
) -> Result<(HeaderMap, Json<Value>), (StatusCode, HeaderMap, Json<Value>)> {
    if !strict_private(&headers, true) {
        return Err(denied(StatusCode::FORBIDDEN, "OWNER_PRIVATE_PANEL_ONLY"));
    }
    // Bounded direct IPC. Never accept device address, serial, CLI or profile from browser.
    let read = async {
        let mut stream = UnixStream::connect(SOCKET).await?;
        stream.write_all(b"REFRESH\n").await?;
        let mut data = Vec::new();
        stream
            .take((MAX_REPLY + 1) as u64)
            .read_to_end(&mut data)
            .await?;
        Ok::<Vec<u8>, std::io::Error>(data)
    };
    let Ok(Ok(raw)) = tokio::time::timeout(Duration::from_secs(58), read).await else {
        return Err(denied(
            StatusCode::SERVICE_UNAVAILABLE,
            "OWNER_AGENT_OFFLINE_OR_TIMEOUT",
        ));
    };
    if raw.len() > MAX_REPLY {
        return Err(denied(StatusCode::BAD_GATEWAY, "AGENT_REPLY_REJECTED"));
    }
    let Ok(v) = serde_json::from_slice::<Value>(&raw) else {
        return Err(denied(StatusCode::BAD_GATEWAY, "AGENT_REPLY_REJECTED"));
    };
    let Some(response) = sanitize_agent(&v) else {
        return Err(denied(
            StatusCode::SERVICE_UNAVAILABLE,
            "OWNER_READ_FAILED_OR_INVALID",
        ));
    };
    Ok((
        super::private_lab_headers("application/json; charset=utf-8"),
        Json(response),
    ))
}

/// Only two additional verified read-command families; no arbitrary CLI.
fn sanitize_extra(v: &Value, kind: &str) -> Option<Value> {
    if v.get("mode")?.as_str()? != "OWNER_SUPERVISED_REAL_C320_READ_ONLY"
        || v.get("read_kind")?.as_str()? != kind
        || v.get("snapshot_is_live")?.as_bool() != Some(true)
        || v.get("serials_returned")?.as_bool() != Some(false)
        || v.get("device_adopted")?.as_bool() != Some(false)
        || v.get("provisioning_enabled")?.as_bool() != Some(false)
        || v.get("device_writes")?.as_u64() != Some(0)
    {
        return None;
    }
    let timestamp = v.get("read_at_utc")?.as_str()?;
    if timestamp.len() < 19
        || timestamp.len() > 40
        || !timestamp
            .bytes()
            .all(|x| x.is_ascii_digit() || b"-:TZ+.".contains(&x))
    {
        return None;
    }
    match kind {
        "CARDS" => {
            let n = v.get("cards_in_service")?.as_u64()?;
            if !(1..=22).contains(&n) {
                return None;
            }
            Some(json!({"read_kind":"CARDS","cards_in_service":n,
                "read_at_utc":timestamp,"snapshot_is_live":true,
                "device_adopted":false,"physical_writes_enabled":false}))
        }
        "FIRMWARE" => {
            let n = v.get("firmware_rows")?.as_u64()?;
            if !(1..=66).contains(&n) || v.get("firmware_reconciled")?.as_bool() != Some(false) {
                return None;
            }
            Some(json!({"read_kind":"FIRMWARE","firmware_rows":n,
                "firmware_reconciled":false,"read_at_utc":timestamp,
                "snapshot_is_live":true,"device_adopted":false,
                "physical_writes_enabled":false}))
        }
        _ => None,
    }
}
async fn extra_read(
    headers: HeaderMap,
    request: &'static [u8],
    kind: &'static str,
) -> Result<(HeaderMap, Json<Value>), (StatusCode, HeaderMap, Json<Value>)> {
    if !strict_private(&headers, true) {
        return Err(denied(StatusCode::FORBIDDEN, "OWNER_PRIVATE_PANEL_ONLY"));
    }
    let attempt = async {
        let mut stream = UnixStream::connect(SOCKET).await?;
        stream.write_all(request).await?;
        let mut data = Vec::new();
        stream
            .take((MAX_REPLY + 1) as u64)
            .read_to_end(&mut data)
            .await?;
        Ok::<Vec<u8>, std::io::Error>(data)
    };
    let Ok(Ok(raw)) = tokio::time::timeout(Duration::from_secs(58), attempt).await else {
        return Err(denied(
            StatusCode::SERVICE_UNAVAILABLE,
            "OWNER_AGENT_OFFLINE_OR_TIMEOUT",
        ));
    };
    if raw.len() > MAX_REPLY {
        return Err(denied(StatusCode::BAD_GATEWAY, "AGENT_REPLY_REJECTED"));
    }
    let Ok(v) = serde_json::from_slice::<Value>(&raw) else {
        return Err(denied(StatusCode::BAD_GATEWAY, "AGENT_REPLY_REJECTED"));
    };
    let Some(result) = sanitize_extra(&v, kind) else {
        return Err(denied(
            StatusCode::SERVICE_UNAVAILABLE,
            "OWNER_READ_FAILED_OR_INVALID",
        ));
    };
    Ok((
        super::private_lab_headers("application/json; charset=utf-8"),
        Json(result),
    ))
}
async fn cards(
    headers: HeaderMap,
) -> Result<(HeaderMap, Json<Value>), (StatusCode, HeaderMap, Json<Value>)> {
    extra_read(headers, b"CARDS\n", "CARDS").await
}
async fn firmware(
    headers: HeaderMap,
) -> Result<(HeaderMap, Json<Value>), (StatusCode, HeaderMap, Json<Value>)> {
    extra_read(headers, b"FIRMWARE\n", "FIRMWARE").await
}

// Presence is NOT physical OLT connectivity: distinguish interactive agent
// readiness from a successful independently timestamped physical CLI response.
async fn agent_status(
    headers: HeaderMap,
) -> Result<(HeaderMap, Json<Value>), (StatusCode, HeaderMap, Json<Value>)> {
    if !strict_private(&headers, false) {
        return Err(denied(StatusCode::FORBIDDEN, "OWNER_PRIVATE_PANEL_ONLY"));
    }
    let empty = json!({"agent_ready":false,"seconds_left":0,"requests_left":0,
      "actual_olt_connectivity_verified":false,"device_adopted":false,"physical_writes_enabled":false});
    let query = async {
        let mut stream = UnixStream::connect(SOCKET).await?;
        stream.write_all(b"STATUS\n").await?;
        let mut data = Vec::new();
        stream.take(513).read_to_end(&mut data).await?;
        Ok::<Vec<u8>, std::io::Error>(data)
    };
    let agent = match tokio::time::timeout(Duration::from_secs(2), query).await {
        Ok(Ok(bytes)) if bytes.len() <= 512 => serde_json::from_slice::<Value>(&bytes).ok(),
        _ => None,
    };
    let result=agent.and_then(|v| {
        if v.get("mode")?.as_str()?!="OWNER_SUPERVISED_REAL_C320_READ_ONLY"
            || v.get("device_adopted")?.as_bool()!=Some(false)
            || v.get("device_writes")?.as_u64()!=Some(0) {return None;}
        let left=v.get("seconds_left")?.as_u64()?;
        let quota=v.get("requests_left")?.as_u64()?;
        let ready=v.get("agent_ready")?.as_bool()?;
        let busy=v.get("read_in_progress")?.as_bool()?;
        if left>900||quota>5||ready!=(left>0&&quota>0) {return None;}
        Some(json!({"agent_ready":ready,"read_in_progress":busy,"seconds_left":left,"requests_left":quota,
          "actual_olt_connectivity_verified":false,"device_adopted":false,"physical_writes_enabled":false}))
    }).unwrap_or(empty);
    Ok((
        super::private_lab_headers("application/json; charset=utf-8"),
        Json(result),
    ))
}

// R9.45 private one-time device credential enrollment.
// This route is intentionally NOT a general SaaS tenancy/admin API.
// A separate owner bootstrap code is verified by the persistent Unix daemon.
// Browser origin is SSH-forwarded local 127.0.0.1:3002; no public enrollment.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct C320OwnerEnrollment {
    device_profile: String,
    device_type: String,
    device_name: String,
    management_ip: String,
    ssh_port: u16,
    username: String,
    bootstrap_code: String,
    password: String,
}
async fn enroll_c320(
    headers: HeaderMap,
    Json(input): Json<C320OwnerEnrollment>,
) -> Result<(HeaderMap, Json<Value>), (StatusCode, HeaderMap, Json<Value>)> {
    if !strict_private(&headers, true) {
        return Err(denied(StatusCode::FORBIDDEN, "OWNER_PRIVATE_PANEL_ONLY"));
    }
    if input.device_type != "olt"
        || input.device_name.trim().is_empty()
        || input.device_name.len() > 64
        || input.device_name.chars().any(|c| c.is_control())
        || input.ssh_port != 321
        || input.username != "zte"
        || input.management_ip != "10.10.13.233"
    {
        return Err(denied(StatusCode::BAD_REQUEST, "TARGET_NOT_ALLOWLISTED"));
    }
    if input.device_profile != "zte_c320_lab"
        || !(32..=128).contains(&input.bootstrap_code.len())
        || !(1..=128).contains(&input.password.len())
        || !input
            .bootstrap_code
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
        || input.password.bytes().any(|c| c == 0 || c == 10 || c == 13)
    {
        return Err(denied(StatusCode::BAD_REQUEST, "ENROLLMENT_INPUT_REJECTED"));
    }
    // Device address/port/username are fixed in this first physical C320
    // adapter, not user-controlled SSRF/SSH targets. Secret is never logged.
    let payload = json!({"device_profile":"zte_c320_lab",
       "bootstrap_code":input.bootstrap_code,"password":input.password});
    let request = format!("ENROLL {}\n", payload);
    if request.len() > 1024 {
        return Err(denied(
            StatusCode::BAD_REQUEST,
            "ENROLLMENT_INPUT_TOO_LARGE",
        ));
    }
    let query = async {
        let mut stream = UnixStream::connect(SOCKET).await?;
        stream.write_all(request.as_bytes()).await?;
        let mut data = Vec::new();
        stream.take(1025).read_to_end(&mut data).await?;
        Ok::<Vec<u8>, std::io::Error>(data)
    };
    let Ok(Ok(raw)) = tokio::time::timeout(Duration::from_secs(58), query).await else {
        return Err(denied(
            StatusCode::SERVICE_UNAVAILABLE,
            "ENROLLMENT_CONNECTOR_UNAVAILABLE",
        ));
    };
    if raw.len() > 1024 {
        return Err(denied(
            StatusCode::BAD_GATEWAY,
            "ENROLLMENT_RESPONSE_REJECTED",
        ));
    }
    let Ok(v) = serde_json::from_slice::<Value>(&raw) else {
        return Err(denied(
            StatusCode::BAD_GATEWAY,
            "ENROLLMENT_RESPONSE_REJECTED",
        ));
    };
    if let Some(stage) = v.get("error").and_then(Value::as_str) {
        let safe = match stage {
            "OWNER_VERIFICATION_FAILED" => "OWNER_VERIFICATION_FAILED",
            "SSH_HANDSHAKE_OR_AUTH_METHOD" => "SSH_HANDSHAKE_OR_AUTH_METHOD",
            "SSH_AUTH_FAILED_OR_UNKNOWN_PROMPT" => "SSH_AUTH_FAILED_OR_UNKNOWN_PROMPT",
            "DEVICE_CLI_RESPONSE_UNSUPPORTED" => "DEVICE_CLI_RESPONSE_UNSUPPORTED",
            "ALREADY_ENROLLED" => "ALREADY_ENROLLED",
            _ => "DEVICE_CONNECTION_FAILED",
        };
        return Err(denied(StatusCode::CONFLICT, safe));
    }
    if v.get("enrolled_for_read").and_then(Value::as_bool) != Some(true)
        || v.get("commercial_production_adopted")
            .and_then(Value::as_bool)
            != Some(false)
        || v.get("physical_writes_enabled").and_then(Value::as_bool) != Some(false)
    {
        return Err(denied(
            StatusCode::FORBIDDEN,
            "ENROLLMENT_VERIFICATION_FAILED",
        ));
    }
    let cards = v
        .get("physical_card_count")
        .and_then(Value::as_u64)
        .filter(|n| (1..=22).contains(n))
        .ok_or_else(|| denied(StatusCode::BAD_GATEWAY, "CARD_PROOF_REJECTED"))?;
    let timestamp = v
        .get("verified_at_utc")
        .and_then(Value::as_str)
        .filter(|x| {
            x.len() >= 19
                && x.len() <= 40
                && x.bytes()
                    .all(|c| c.is_ascii_digit() || b"-:TZ+.".contains(&c))
        })
        .ok_or_else(|| denied(StatusCode::BAD_GATEWAY, "TIMESTAMP_REJECTED"))?;
    Ok((
        super::private_lab_headers("application/json; charset=utf-8"),
        Json(json!({"enrolled_for_read":true,"card_count":cards,
           "verified_at_utc":timestamp,"network_path":"DIRECT_SSH_PINNED",
           "adoption_state":"READ_ONLY_CONNECTED_LAB",
           "production_adopted":false,"physical_writes_enabled":false,
           "host_identity_level":"NETWORK_OBSERVED_SSH_PIN"})),
    ))
}

async fn save_draft(
    headers: HeaderMap,
    Json(input): Json<Value>,
) -> Result<(HeaderMap, Json<Value>), (StatusCode, HeaderMap, Json<Value>)> {
    if !strict_private(&headers, true) {
        return Err(denied(StatusCode::FORBIDDEN, "PRIVATE_PANEL_ONLY"));
    }
    if !input.as_object().is_some_and(|m| {
        m.len() == 6
            && [
                "device_profile",
                "device_type",
                "device_name",
                "management_ip",
                "ssh_port",
                "username",
            ]
            .iter()
            .all(|key| m.contains_key(*key))
    }) {
        return Err(denied(StatusCode::BAD_REQUEST, "DRAFT_METADATA_ONLY"));
    }
    let name = input
        .get("device_name")
        .and_then(Value::as_str)
        .unwrap_or("");
    if name.trim().is_empty()
        || name.len() > 64
        || name.chars().any(|c| c.is_control())
        || input.get("device_profile").and_then(Value::as_str) != Some("zte_c320_lab")
        || input.get("device_type").and_then(Value::as_str) != Some("olt")
        || input.get("management_ip").and_then(Value::as_str) != Some("10.10.13.233")
        || input.get("ssh_port").and_then(Value::as_u64) != Some(321)
        || input.get("username").and_then(Value::as_str) != Some("zte")
    {
        return Err(denied(
            StatusCode::BAD_REQUEST,
            "UNSUPPORTED_DEVICE_PROFILE",
        ));
    }
    let request = format!(
        "DRAFT {}\n",
        json!({"device_profile":"zte_c320_lab","device_name":name.trim()})
    );
    let query = async {
        let mut stream = UnixStream::connect(SOCKET).await?;
        stream.write_all(request.as_bytes()).await?;
        let mut raw = Vec::new();
        stream.take(512).read_to_end(&mut raw).await?;
        Ok::<Vec<u8>, std::io::Error>(raw)
    };
    let Ok(Ok(data)) = tokio::time::timeout(Duration::from_secs(4), query).await else {
        return Err(denied(
            StatusCode::SERVICE_UNAVAILABLE,
            "DRAFT_STORAGE_UNAVAILABLE",
        ));
    };
    let Ok(result) = serde_json::from_slice::<Value>(&data) else {
        return Err(denied(StatusCode::BAD_GATEWAY, "INVALID_DRAFT_RESPONSE"));
    };
    if result.get("draft_saved").and_then(Value::as_bool) != Some(true) {
        return Err(denied(StatusCode::CONFLICT, "DRAFT_REJECTED"));
    }
    Ok((
        super::private_lab_headers("application/json; charset=utf-8"),
        Json(json!({"saved":true,"target":"DEV-01",
           "adoption_state":"DRAFT_SAVED_AWAITING_AUTH","physical_writes_enabled":false})),
    ))
}

async fn fixed_network_probe(
    headers: HeaderMap,
) -> Result<(HeaderMap, Json<Value>), (StatusCode, HeaderMap, Json<Value>)> {
    if !strict_private(&headers, false) {
        return Err(denied(StatusCode::FORBIDDEN, "PRIVATE_PANEL_ONLY"));
    }
    // A deliberately fixed lab target, not a browser-controlled network scanner.
    // A successful TCP handshake does NOT prove SSH login or physical adoption.
    let stage = match tokio::time::timeout(
        Duration::from_secs(3),
        tokio::net::TcpStream::connect("10.10.13.233:321"),
    )
    .await
    {
        Ok(Ok(_)) => "TCP_REACHABLE_AUTH_NOT_TESTED",
        Ok(Err(e)) if e.kind() == std::io::ErrorKind::ConnectionRefused => "SSH_PORT_REFUSED",
        Ok(Err(e))
            if matches!(
                e.kind(),
                std::io::ErrorKind::HostUnreachable | std::io::ErrorKind::NetworkUnreachable
            ) =>
        {
            "NETWORK_UNREACHABLE"
        }
        Ok(Err(_)) => "NETWORK_OR_PORT_ERROR",
        Err(_) => "TCP_TIMEOUT_NETWORK_OR_PORT",
    };
    Ok((
        super::private_lab_headers("application/json; charset=utf-8"),
        Json(json!({"target":"DEV-01","probe_stage":stage,
        "ssh_authentication_verified":false,"production_adopted":false,
        "physical_writes_enabled":false})),
    ))
}

async fn connection_status(
    headers: HeaderMap,
) -> Result<(HeaderMap, Json<Value>), (StatusCode, HeaderMap, Json<Value>)> {
    if !strict_private(&headers, false) {
        return Err(denied(StatusCode::FORBIDDEN, "OWNER_PRIVATE_PANEL_ONLY"));
    }
    let query = async {
        let mut stream = UnixStream::connect(SOCKET).await?;
        stream.write_all(b"STATUS\n").await?;
        let mut data = Vec::new();
        stream.take(513).read_to_end(&mut data).await?;
        Ok::<Vec<u8>, std::io::Error>(data)
    };
    let response = match tokio::time::timeout(Duration::from_secs(3), query).await {
        Ok(Ok(raw)) if raw.len() <= 512 => serde_json::from_slice::<Value>(&raw).ok(),
        _ => None,
    };
    let configured = response.as_ref().is_some_and(|v| {
        v.get("persistent_connector").and_then(Value::as_bool) == Some(true)
            && v.get("credentials_enrolled").and_then(Value::as_bool) == Some(true)
            && v.get("device_adopted").and_then(Value::as_bool) == Some(false)
            && v.get("device_writes").and_then(Value::as_u64) == Some(0)
    });
    let last = response
        .as_ref()
        .and_then(|v| v.get("last_verified_at_utc"))
        .and_then(Value::as_str)
        .filter(|s| {
            s.len() >= 19
                && s.len() <= 40
                && s.bytes()
                    .all(|c| c.is_ascii_digit() || b"-:TZ+.".contains(&c))
        })
        .unwrap_or("");
    let physically_fresh = configured
        && !last.is_empty()
        && response
            .as_ref()
            .and_then(|v| v.get("actual_olt_connectivity_verified"))
            .and_then(Value::as_bool)
            == Some(true);
    let previously_verified = configured
        && response
            .as_ref()
            .and_then(|v| v.get("ever_verified_since_start"))
            .and_then(Value::as_bool)
            == Some(true);
    let status = if response.is_none() {
        "UNKNOWN"
    } else if !configured {
        "PENDING"
    } else if physically_fresh {
        "CONNECTED"
    } else if previously_verified {
        "DISCONNECTED"
    } else {
        "PENDING"
    };
    Ok((
        super::private_lab_headers("application/json; charset=utf-8"),
        Json(json!({"target":"DEV-01","vendor":"ZTE","model":"C320",
        "management_transport":"DIRECT_SSH_PINNED","pon_scope":"1/1/1",
        "connector_online":response.is_some(),"credentials_enrolled":configured,
        "device_status":status,
        "draft_saved":response.as_ref().and_then(|v|v.get("draft_saved")).and_then(Value::as_bool)==Some(true),
        "device_name":response.as_ref().and_then(|v|v.get("device_name")).and_then(Value::as_str).filter(|v|!v.is_empty()&&v.len()<=64).unwrap_or("ZTE C320 Lab"),
        "last_verified_at_utc":if configured {last} else {""},
        "adoption_state":if physically_fresh {"READ_ONLY_CONNECTED_LAB"}
            else if configured {"CONFIGURED_AWAITING_READ"}
            else if response.as_ref().and_then(|v|v.get("draft_saved")).and_then(Value::as_bool)==Some(true)
                {"DRAFT_SAVED_AWAITING_AUTH"} else {"NOT_ENROLLED"},
        "host_identity_level":"NETWORK_OBSERVED_SSH_PIN",
        "production_adopted":false,"physical_writes_enabled":false})),
    ))
}

// Owner-private, server-derived capability catalog: NEVER an executable CLI.
// Each enabled operation maps to an existing fixed read-only POST handler.
// An observed live connection alone does not authorize an unqualified write.
async fn owner_action_catalog(
    headers: HeaderMap,
) -> Result<(HeaderMap, Json<Value>), (StatusCode, HeaderMap, Json<Value>)> {
    let (_, Json(connection)) = connection_status(headers).await?;
    Ok((
        super::private_lab_headers("application/json; charset=utf-8"),
        Json(owner_catalog_response(&connection)),
    ))
}

fn owner_catalog_response(connection: &Value) -> Value {
    let connected = connection.get("connector_online").and_then(Value::as_bool) == Some(true)
        && connection.get("credentials_enrolled").and_then(Value::as_bool) == Some(true)
        && connection.get("device_status").and_then(Value::as_str) == Some("CONNECTED")
        && connection.get("physical_writes_enabled").and_then(Value::as_bool) == Some(false)
        && connection.get("production_adopted").and_then(Value::as_bool) == Some(false);
    let read = if connected { "AVAILABLE_READ_ONLY" } else { "CONNECTION_REQUIRED" };
    let catalog = json!([
        {"id":"cards","group":"Chassis & Hardware","label":"Read active line cards",
         "state":read,"operation":"READ","endpoint":"/lab/c320-owner-live-cards",
         "scope":"Live chassis card count only; no serial or unqualified board assertions."},
        {"id":"firmware","group":"Chassis & Hardware","label":"Read firmware inventory summary",
         "state":read,"operation":"READ","endpoint":"/lab/c320-owner-live-firmware",
         "scope":"Live firmware row count; board/version mapping remains unverified."},
        {"id":"onu_counts","group":"PON & ONU","label":"Read PON 1/1/1 ONU counts",
         "state":"DEGRADED","operation":"READ","endpoint":null,
         "scope":"The current live test returned HTTP 503; command/parser requires repair and fresh acceptance."},
        {"id":"onu_details","group":"PON & ONU","label":"Read per-ONU operational status",
         "state":"NOT_QUALIFIED","operation":"READ","endpoint":null,
         "scope":"Only historical sanitized ONU IDs exist; no current per-ONU CLI test."},
        {"id":"optical_levels","group":"PON & ONU","label":"Read optical diagnostics",
         "state":"NOT_QUALIFIED","operation":"READ","endpoint":null,
         "scope":"Exact model/firmware and bounded optical command not verified."},
        {"id":"alarms","group":"Diagnostics","label":"Read current alarms",
         "state":"NOT_QUALIFIED","operation":"READ","endpoint":null,
         "scope":"Alarm command/format not yet observed on the connected hardware."},
        {"id":"traffic","group":"Diagnostics","label":"Read PON/port traffic and utilization",
         "state":"NOT_QUALIFIED","operation":"READ","endpoint":null,
         "scope":"Counter widths, poll interval and firmware-specific parsing need testing."},
        {"id":"config_backup","group":"Configuration & Recovery","label":"Create device-native configuration backup",
         "state":"NOT_IMPLEMENTED","operation":"CONTROLLED","endpoint":null,
         "scope":"Encrypted external CLI reference backup exists; vendor-native restore has not been tested."},
        {"id":"onu_provision","group":"Provisioning","label":"Provision an ONU / bind service profile",
         "state":"APPROVAL_AND_DRIVER_REQUIRED","operation":"WRITE","endpoint":null,
         "scope":"Exact ONU, PON, VLAN, bandwidth, service profile; validate dry run, backup, rollback and approvals."},
        {"id":"onu_deprovision","group":"Provisioning","label":"Remove or migrate an ONU",
         "state":"APPROVAL_AND_DRIVER_REQUIRED","operation":"WRITE","endpoint":null,
         "scope":"Subscriber impact, tenant/POP rights, immutable audit and tested native recovery required."},
        {"id":"vlan_service","group":"Provisioning","label":"Configure PON VLAN and service profiles",
         "state":"APPROVAL_AND_DRIVER_REQUIRED","operation":"WRITE","endpoint":null,
         "scope":"Typed, vendor-tested parameters; diff/approval and rollback; no arbitrary CLI input."},
        {"id":"ont_cwmp","group":"ONT Management","label":"Manage ONT via TR-069/CWMP",
         "state":"AGENT_INTEROP_REQUIRED","operation":"CONTROLLED","endpoint":null,
         "scope":"Requires exact ONT agent model, authenticated Inform and independently tested RPC/data model."},
        {"id":"ont_usp","group":"ONT Management","label":"Manage ONT via TR-369/USP",
         "state":"AGENT_INTEROP_REQUIRED","operation":"CONTROLLED","endpoint":null,
         "scope":"Requires a verified native USP Agent and authenticated controller MTP on that ONT."},
        {"id":"reboot","group":"Device Operations","label":"Operator-initiated controlled reboot",
         "state":"APPROVAL_AND_DRIVER_REQUIRED","operation":"WRITE","endpoint":null,
         "scope":"Maintenance window, service-impact estimation, independent approval and recovery."},
        {"id":"firmware_upgrade","group":"Device Operations","label":"Operator-initiated firmware upgrade",
         "state":"APPROVAL_AND_DRIVER_REQUIRED","operation":"WRITE","endpoint":null,
         "scope":"R9.58 workflow intent exists in source; vendor image attestation, exact board match and physical worker not ready."},
        {"id":"config_restore","group":"Configuration & Recovery","label":"Restore native configuration",
         "state":"APPROVAL_AND_DRIVER_REQUIRED","operation":"WRITE","endpoint":null,
         "scope":"Full native backup/restore drill, independent approver, out-of-band recovery and post-readback required."}
    ]);
    json!({"target":"DEV-01","vendor":"ZTE","model":"C320",
            "mode":"OWNER_PRIVATE_LAB_CAPABILITIES","connected":connected,
            "last_verified_at_utc":connection.get("last_verified_at_utc").and_then(Value::as_str).unwrap_or(""),
            "production_adopted":false,"physical_writes_enabled":false,
            "entries":catalog})
}

pub(super) fn router() -> Router {
    Router::new()
        .route(
            "/lab/c320-owner-enroll",
            post(enroll_c320).layer(axum::extract::DefaultBodyLimit::max(512)),
        )
        .route(
            "/lab/c320-owner-connection",
            axum::routing::get(connection_status),
        )
        .route(
            "/lab/c320-owner-action-catalog",
            axum::routing::get(owner_action_catalog),
        )
        .route(
            "/lab/c320-owner-save-draft",
            post(save_draft).layer(axum::extract::DefaultBodyLimit::max(512)),
        )
        .route(
            "/lab/c320-owner-network-probe",
            axum::routing::get(fixed_network_probe),
        )
        .route("/lab/c320-owner-live-refresh", post(refresh))
        .route("/lab/c320-owner-live-cards", post(cards))
        .route("/lab/c320-owner-live-firmware", post(firmware))
        .route(
            "/lab/c320-owner-agent-state",
            axum::routing::get(agent_status),
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;
    #[test]
    fn owner_catalog_never_authorizes_an_unqualified_command() {
        let connected = json!({"connector_online":true,"credentials_enrolled":true,
            "device_status":"CONNECTED","physical_writes_enabled":false,
            "production_adopted":false,"last_verified_at_utc":"2026-10-02T02:49:33Z"});
        let catalog = owner_catalog_response(&connected);
        assert_eq!(catalog["connected"], true);
        assert_eq!(catalog["physical_writes_enabled"], false);
        let entries = catalog["entries"].as_array().unwrap();
        assert!(entries.len() >= 15);
        let runnable: Vec<_> = entries.iter()
            .filter(|entry| entry["state"] == "AVAILABLE_READ_ONLY").collect();
        assert_eq!(runnable.len(), 2);
        assert_eq!(runnable[0]["endpoint"], "/lab/c320-owner-live-cards");
        assert_eq!(runnable[1]["endpoint"], "/lab/c320-owner-live-firmware");
        for entry in entries {
            if entry["operation"] != "READ" {
                assert!(entry["endpoint"].is_null());
                assert_ne!(entry["state"], "AVAILABLE_READ_ONLY");
            }
        }
        let mut not_connected = connected.clone();
        not_connected["device_status"] = json!("DISCONNECTED");
        let denied = owner_catalog_response(&not_connected);
        assert_eq!(denied["connected"], false);
        assert!(denied["entries"].as_array().unwrap().iter()
            .all(|entry| entry["state"] != "AVAILABLE_READ_ONLY"));
        let mut writes = connected.clone();
        writes["physical_writes_enabled"] = json!(true);
        assert_eq!(owner_catalog_response(&writes)["connected"], false);
    }

    #[test]
    fn rejects_non_numeric_and_untrusted_agent_data() {
        let mut v = json!({"mode":"OWNER_SUPERVISED_REAL_C320_READ_ONLY","snapshot_is_live":true,
       "port":"1/1/1","unconfigured":0,"configured":72,"online":0,"offline":72,
       "configuration_rows":72,"serials_returned":false,"device_adopted":false,
       "provisioning_enabled":false,"device_writes":0,"read_at_utc":"2026-09-29T11:00:00+00:00",
       "raw_serial":"NEVER_RELAY_THIS"});
        let safe = sanitize_agent(&v).unwrap();
        assert!(safe.get("raw_serial").is_none());
        assert_eq!(safe["physical_writes_enabled"], false);
        v["online"] = json!(1);
        assert_eq!(sanitize_agent(&v), None);
        v["online"] = json!(0);
        v["device_writes"] = json!(1);
        assert_eq!(sanitize_agent(&v), None);
    }
    #[test]
    fn extra_read_types_are_allowlisted_and_redacted() {
        let mut v = json!({"mode":"OWNER_SUPERVISED_REAL_C320_READ_ONLY",
           "read_kind":"CARDS","cards_in_service":3,
           "snapshot_is_live":true,"serials_returned":false,
           "device_adopted":false,"provisioning_enabled":false,
           "device_writes":0,"read_at_utc":"2026-09-29T11:00:00+00:00",
           "raw_password":"SYNTHETIC_SHOULD_NOT_RETURN"});
        let clean = sanitize_extra(&v, "CARDS").unwrap();
        assert_eq!(clean["cards_in_service"], 3);
        assert!(clean.get("raw_password").is_none());
        assert_eq!(clean["device_adopted"], false);
        assert!(sanitize_extra(&v, "FIRMWARE").is_none());
        v["cards_in_service"] = json!(0);
        assert!(sanitize_extra(&v, "CARDS").is_none());
        v["read_kind"] = json!("FIRMWARE");
        v["firmware_rows"] = json!(5);
        v["firmware_reconciled"] = json!(false);
        let fw = sanitize_extra(&v, "FIRMWARE").unwrap();
        assert_eq!(fw["firmware_reconciled"], false);
        assert!(fw.get("raw_password").is_none());
        v["device_writes"] = json!(1);
        assert!(sanitize_extra(&v, "FIRMWARE").is_none());
    }
    #[tokio::test]
    async fn persistent_enrollment_rejects_cross_origin_and_bad_device_target() {
        let forged=router().oneshot(Request::builder()
            .method("POST").uri("/lab/c320-owner-enroll")
            .header("Host","127.0.0.1:3002")
            .header("Origin","http://untrusted.invalid")
            .header("X-IPAT-Demo-Only","1")
            .header("Content-Type","application/json")
            .body(Body::from(r#"{"device_profile":"zte_c320_lab","device_type":"olt","device_name":"Synthetic Device","management_ip":"192.0.2.20","ssh_port":2222,"username":"synthetic","bootstrap_code":"synthetic_1234567890123456789012345678","password":"synthetic"}"#))
            .unwrap()).await.unwrap();
        assert_eq!(forged.status(), StatusCode::FORBIDDEN);
        let bad=router().oneshot(Request::builder()
            .method("POST").uri("/lab/c320-owner-enroll")
            .header("Host","127.0.0.1:3002")
            .header("Origin","http://127.0.0.1:3002")
            .header("X-IPAT-Demo-Only","1")
            .header("Content-Type","application/json")
            .body(Body::from(r#"{"device_profile":"other_olt","device_type":"olt","device_name":"Synthetic Lab","management_ip":"192.0.2.30","ssh_port":2222,"username":"synthetic","bootstrap_code":"synthetic_1234567890123456789012345678","password":"synthetic"}"#))
            .unwrap()).await.unwrap();
        assert_eq!(bad.status(), StatusCode::BAD_REQUEST);
        let read = router()
            .oneshot(
                Request::builder()
                    .uri("/lab/c320-owner-connection")
                    .header("Host", "127.0.0.1:3002")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(read.status(), StatusCode::OK);
        let body = axum::body::to_bytes(read.into_body(), 2048).await.unwrap();
        let value: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(value["production_adopted"], false);
        assert_eq!(value["physical_writes_enabled"], false);
    }
    #[tokio::test]
    async fn status_route_rejects_external_host_and_is_not_device_health() {
        let unauthorized = router()
            .oneshot(
                Request::builder()
                    .uri("/lab/c320-owner-agent-state")
                    .header("Host", "public.invalid")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(unauthorized.status(), StatusCode::FORBIDDEN);
        let private = router()
            .oneshot(
                Request::builder()
                    .uri("/lab/c320-owner-agent-state")
                    .header("Host", "127.0.0.1:3002")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(private.status(), StatusCode::OK);
        let b = axum::body::to_bytes(private.into_body(), 512)
            .await
            .unwrap();
        let d: Value = serde_json::from_slice(&b).unwrap();
        assert_eq!(d["actual_olt_connectivity_verified"], false);
        assert_eq!(d["device_adopted"], false);
        assert_eq!(d["physical_writes_enabled"], false);
    }
    #[tokio::test]
    async fn extra_routes_deny_cross_origin() {
        for path in [
            "/lab/c320-owner-live-cards",
            "/lab/c320-owner-live-firmware",
        ] {
            let r = router()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri(path)
                        .header("Host", "127.0.0.1:3002")
                        .header("Origin", "http://untrusted.invalid")
                        .header("X-IPAT-Demo-Only", "1")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(r.status(), StatusCode::FORBIDDEN);
        }
    }
    #[tokio::test]
    async fn denies_csrf_and_missing_supervised_agent() {
        let app = router();
        let forged = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/lab/c320-owner-live-refresh")
                    .header("host", "127.0.0.1:3002")
                    .header("Origin", "https://attacker.invalid")
                    .header("X-IPAT-Demo-Only", "1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(forged.status(), StatusCode::FORBIDDEN);
        let forged_host = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/lab/c320-owner-live-refresh")
                    .header("host", "other.invalid")
                    .header("Origin", "http://127.0.0.1:3002")
                    .header("X-IPAT-Demo-Only", "1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(forged_host.status(), StatusCode::FORBIDDEN);
    }
}
