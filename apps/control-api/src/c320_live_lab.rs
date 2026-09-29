//! Owner-supervised REAL C320 READ-ONLY panel bridge, PRIVATE 127.0.0.1:3002 ONLY.
//! Opt-in at startup and a separate interactive nonroot owner Unix agent are
//! BOTH required. Not persistent SaaS adoption and NEVER a write-capable API.
use axum::{http::{header, HeaderMap, StatusCode}, routing::post, Json, Router};
use serde_json::{json, Value};
use std::time::Duration;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::UnixStream};

const SOCKET: &str = "/home/openai/.local/share/ipat/r940-live-agent/live.sock";
const MAX_REPLY: usize = 2048;

fn denied(code: StatusCode, reason: &'static str) -> (StatusCode,HeaderMap,Json<Value>) {
    (code, super::private_lab_headers("application/json; charset=utf-8"),
      Json(json!({"error":reason,"physical_writes_enabled":false,"device_adopted":false})))
}
fn strict_private(headers:&HeaderMap, mutating:bool)->bool {
    let host=headers.get_all(header::HOST);
    if host.iter().count()!=1 || headers.get(header::HOST).and_then(|x|x.to_str().ok())!=Some("127.0.0.1:3002") {
        return false;
    }
    !mutating || super::device_workbench_lab::demo_csrf(headers)
}
fn sanitize_agent(v: &Value) -> Option<Value> {
    let n=|key:&str| v.get(key)?.as_u64().filter(|n|*n<=128);
    let (u,c,o,f,configured)=(n("unconfigured")?,n("configured")?,n("online")?,n("offline")?,n("configuration_rows")?);
    if v.get("mode")?.as_str()?!="OWNER_SUPERVISED_REAL_C320_READ_ONLY"
       || v.get("snapshot_is_live")?.as_bool()!=true
       || v.get("port")?.as_str()?!="1/1/1"
       || v.get("serials_returned")?.as_bool()!=false
       || v.get("device_adopted")?.as_bool()!=false
       || v.get("provisioning_enabled")?.as_bool()!=false
       || v.get("device_writes")?.as_u64()!=0
       || o+f!=c || configured!=c {return None;}
    let t=v.get("read_at_utc")?.as_str()?;
    if t.len()<19||t.len()>40||!t.bytes().all(|x|x.is_ascii_digit()||b"-:TZ+.".contains(&x)) {return None;}
    Some(json!({"mode":"OWNER_SUPERVISED_REAL_C320_READ_ONLY","read_at_utc":t,
      "pon":"1/1/1","unconfigured":u,"configured":c,"online":o,"offline":f,
      "source":"VERIFIED_LOCAL_OWNER_AGENT_LAB_ONLY","serials_returned":false,
      "physical_writes_enabled":false,"device_adopted":false}))
}
async fn refresh(headers:HeaderMap)->Result<(HeaderMap,Json<Value>),(StatusCode,HeaderMap,Json<Value>)> {
    if !strict_private(&headers,true) { return Err(denied(StatusCode::FORBIDDEN,"OWNER_PRIVATE_PANEL_ONLY")); }
    // Bounded direct IPC. Never accept device address, serial, CLI or profile from browser.
    let read=async {
        let mut stream=UnixStream::connect(SOCKET).await?;
        stream.write_all(b"REFRESH\n").await?;
        let mut data=Vec::new();
        stream.take((MAX_REPLY+1) as u64).read_to_end(&mut data).await?;
        Ok::<Vec<u8>,std::io::Error>(data)
    };
    let Ok(Ok(raw))=tokio::time::timeout(Duration::from_secs(58),read).await else {
        return Err(denied(StatusCode::SERVICE_UNAVAILABLE,"OWNER_AGENT_OFFLINE_OR_TIMEOUT"));
    };
    if raw.len()>MAX_REPLY {return Err(denied(StatusCode::BAD_GATEWAY,"AGENT_REPLY_REJECTED"));}
    let Ok(v)=serde_json::from_slice::<Value>(&raw) else {
        return Err(denied(StatusCode::BAD_GATEWAY,"AGENT_REPLY_REJECTED"));
    };
    let Some(response)=sanitize_agent(&v) else {
        return Err(denied(StatusCode::SERVICE_UNAVAILABLE,"OWNER_READ_FAILED_OR_INVALID"));
    };
    Ok((super::private_lab_headers("application/json; charset=utf-8"),Json(response)))
}

pub(super) fn router()->Router {
    Router::new().route("/lab/c320-owner-live-refresh",post(refresh))
}

#[cfg(test)]
mod tests {
 use super::*;
 use axum::{body::Body,http::Request};
 use tower::ServiceExt;
 #[test]
 fn rejects_non_numeric_and_untrusted_agent_data() {
     let mut v=json!({"mode":"OWNER_SUPERVISED_REAL_C320_READ_ONLY","snapshot_is_live":true,
       "port":"1/1/1","unconfigured":0,"configured":72,"online":0,"offline":72,
       "configuration_rows":72,"serials_returned":false,"device_adopted":false,
       "provisioning_enabled":false,"device_writes":0,"read_at_utc":"2026-09-29T11:00:00+00:00",
       "raw_serial":"NEVER_RELAY_THIS"});
     let safe=sanitize_agent(&v).unwrap();
     assert!(safe.get("raw_serial").is_none());
     assert_eq!(safe["physical_writes_enabled"],false);
     v["online"]=json!(1);
     assert_eq!(sanitize_agent(&v),None);
     v["online"]=json!(0);v["device_writes"]=json!(1);
     assert_eq!(sanitize_agent(&v),None);
 }
 #[tokio::test]
 async fn denies_csrf_and_missing_supervised_agent() {
     let app=router();
     let forged=app.clone().oneshot(Request::builder().method("POST")
       .uri("/lab/c320-owner-live-refresh").header("host","127.0.0.1:3002")
       .header("Origin","https://attacker.invalid")
       .header("X-IPAT-Demo-Only","1").body(Body::empty()).unwrap()).await.unwrap();
     assert_eq!(forged.status(),StatusCode::FORBIDDEN);
     let forged_host=app.oneshot(Request::builder().method("POST")
       .uri("/lab/c320-owner-live-refresh").header("host","other.invalid")
       .header("Origin","http://127.0.0.1:3002")
       .header("X-IPAT-Demo-Only","1").body(Body::empty()).unwrap()).await.unwrap();
     assert_eq!(forged_host.status(),StatusCode::FORBIDDEN);
 }
}
