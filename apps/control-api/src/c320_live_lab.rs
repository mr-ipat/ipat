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
    if input.device_profile != "zte_c320_lab"
        || !(32..=128).contains(&input.bootstrap_code.len())
        || !(1..=128).contains(&input.password.len())
        || !input.bootstrap_code.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
        || input.password.bytes().any(|c| c == 0 || c == 10 || c == 13)
    {
        return Err(denied(StatusCode::BAD_REQUEST, "ENROLLMENT_INPUT_REJECTED"));
    }
    // Device address/port/username are fixed in this first physical C320
    // adapter, not user-controlled SSRF/SSH targets. Secret is never logged.
    let payload = json!({"device_profile":"zte_c320_lab",
       "bootstrap_code":input.bootstrap_code,"password":input.password});
    let request = format!("ENROLL {}\n",payload);
    if request.len() > 1024 {
        return Err(denied(StatusCode::BAD_REQUEST, "ENROLLMENT_INPUT_TOO_LARGE"));
    }
    let query = async {
        let mut stream = UnixStream::connect(SOCKET).await?;
        stream.write_all(request.as_bytes()).await?;
        let mut data = Vec::new();
        stream.take(1025).read_to_end(&mut data).await?;
        Ok::<Vec<u8>,std::io::Error>(data)
    };
    let Ok(Ok(raw))=tokio::time::timeout(Duration::from_secs(58),query).await else {
        return Err(denied(StatusCode::SERVICE_UNAVAILABLE,"ENROLLMENT_CONNECTOR_UNAVAILABLE"));
    };
    if raw.len()>1024 {
        return Err(denied(StatusCode::BAD_GATEWAY,"ENROLLMENT_RESPONSE_REJECTED"));
    }
    let Ok(v)=serde_json::from_slice::<Value>(&raw) else {
        return Err(denied(StatusCode::BAD_GATEWAY,"ENROLLMENT_RESPONSE_REJECTED"));
    };
    if v.get("enrolled_for_read").and_then(Value::as_bool)!=Some(true)
        || v.get("commercial_production_adopted").and_then(Value::as_bool)!=Some(false)
        || v.get("physical_writes_enabled").and_then(Value::as_bool)!=Some(false)
    {
        return Err(denied(StatusCode::FORBIDDEN,"ENROLLMENT_VERIFICATION_FAILED"));
    }
    let cards=v.get("physical_card_count").and_then(Value::as_u64)
        .filter(|n| (1..=22).contains(n)).ok_or_else(||
            denied(StatusCode::BAD_GATEWAY,"CARD_PROOF_REJECTED"))?;
    let timestamp=v.get("verified_at_utc").and_then(Value::as_str)
        .filter(|x| x.len()>=19 && x.len()<=40
            && x.bytes().all(|c| c.is_ascii_digit() || b"-:TZ+.".contains(&c)))
        .ok_or_else(||denied(StatusCode::BAD_GATEWAY,"TIMESTAMP_REJECTED"))?;
    Ok((super::private_lab_headers("application/json; charset=utf-8"),
        Json(json!({"enrolled_for_read":true,"card_count":cards,
           "verified_at_utc":timestamp,"network_path":"DIRECT_SSH_PINNED",
           "adoption_state":"READ_ONLY_CONNECTED_LAB",
           "production_adopted":false,"physical_writes_enabled":false,
           "host_identity_level":"NETWORK_OBSERVED_SSH_PIN"}))))
}

async fn connection_status(headers: HeaderMap)
    -> Result<(HeaderMap, Json<Value>), (StatusCode, HeaderMap, Json<Value>)> {
    if !strict_private(&headers,false) {
        return Err(denied(StatusCode::FORBIDDEN,"OWNER_PRIVATE_PANEL_ONLY"));
    }
    let query=async {
        let mut stream=UnixStream::connect(SOCKET).await?;
        stream.write_all(b"STATUS\n").await?;
        let mut data=Vec::new();
        stream.take(513).read_to_end(&mut data).await?;
        Ok::<Vec<u8>,std::io::Error>(data)
    };
    let response=match tokio::time::timeout(Duration::from_secs(3),query).await {
        Ok(Ok(raw)) if raw.len()<=512 => serde_json::from_slice::<Value>(&raw).ok(),
        _ => None,
    };
    let configured=response.as_ref().is_some_and(|v|
        v.get("persistent_connector").and_then(Value::as_bool)==Some(true)
        && v.get("credentials_enrolled").and_then(Value::as_bool)==Some(true)
        && v.get("device_adopted").and_then(Value::as_bool)==Some(false)
        && v.get("device_writes").and_then(Value::as_u64)==Some(0));
    let last=response.as_ref().and_then(|v|v.get("last_verified_at_utc"))
        .and_then(Value::as_str).filter(|s|
            s.len()>=19 && s.len()<=40
            && s.bytes().all(|c|c.is_ascii_digit()||b"-:TZ+.".contains(&c)))
        .unwrap_or("");
    let physically_fresh=configured && !last.is_empty() && response.as_ref()
        .and_then(|v|v.get("actual_olt_connectivity_verified"))
        .and_then(Value::as_bool)==Some(true);
    let previously_verified=configured && response.as_ref()
        .and_then(|v|v.get("ever_verified_since_start"))
        .and_then(Value::as_bool)==Some(true);
    let status=if response.is_none() {"UNKNOWN"} else if !configured {"PENDING"}
        else if physically_fresh {"CONNECTED"}
        else if previously_verified {"DISCONNECTED"} else {"PENDING"};
    Ok((super::private_lab_headers("application/json; charset=utf-8"),
      Json(json!({"target":"DEV-01","vendor":"ZTE","model":"C320",
        "management_transport":"DIRECT_SSH_PINNED","pon_scope":"1/1/1",
        "connector_online":response.is_some(),"credentials_enrolled":configured,
        "device_status":status,
        "last_verified_at_utc":if configured {last} else {""},
        "adoption_state":if physically_fresh {"READ_ONLY_CONNECTED_LAB"}
            else if configured {"CONFIGURED_AWAITING_READ"} else {"NOT_ENROLLED"},
        "host_identity_level":"NETWORK_OBSERVED_SSH_PIN",
        "production_adopted":false,"physical_writes_enabled":false}))))
}

pub(super) fn router() -> Router {
    Router::new()
        .route("/lab/c320-owner-enroll", post(enroll_c320).layer(axum::extract::DefaultBodyLimit::max(512)))
        .route("/lab/c320-owner-connection", axum::routing::get(connection_status))
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
            .body(Body::from(r#"{"device_profile":"zte_c320_lab","bootstrap_code":"synthetic_1234567890123456789012345678","password":"synthetic"}"#))
            .unwrap()).await.unwrap();
        assert_eq!(forged.status(),StatusCode::FORBIDDEN);
        let bad=router().oneshot(Request::builder()
            .method("POST").uri("/lab/c320-owner-enroll")
            .header("Host","127.0.0.1:3002")
            .header("Origin","http://127.0.0.1:3002")
            .header("X-IPAT-Demo-Only","1")
            .header("Content-Type","application/json")
            .body(Body::from(r#"{"device_profile":"other_olt","bootstrap_code":"synthetic_1234567890123456789012345678","password":"synthetic"}"#))
            .unwrap()).await.unwrap();
        assert_eq!(bad.status(),StatusCode::BAD_REQUEST);
        let read=router().oneshot(Request::builder()
            .uri("/lab/c320-owner-connection")
            .header("Host","127.0.0.1:3002")
            .body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(read.status(),StatusCode::OK);
        let body=axum::body::to_bytes(read.into_body(),2048).await.unwrap();
        let value:Value=serde_json::from_slice(&body).unwrap();
        assert_eq!(value["production_adopted"],false);
        assert_eq!(value["physical_writes_enabled"],false);
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
