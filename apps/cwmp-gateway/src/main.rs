//! Original IPAT CWMP Rust HTTP boundary, deliberately LAB ONLY.
//! Only accepts synthetic parser exercise on *Mac/VPS loopback*; real
//! /cwmp always denies until verified mTLS + tenant + durable sessions exist.
use axum::{
    body::Bytes,
    extract::DefaultBodyLimit,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    routing::{any, get, post},
    Router,
};
const LAB_BIND: &str = "127.0.0.1:3300";
const SAFE_LAB_REPLY: &str = r#"{"parser":"valid","peer_authenticated":false,"tenant_bound":false,"device_enrolled":false,"cwmp_response_sent":false}"#;
fn app() -> Router {
    Router::new()
        .route("/healthz", get(|| async { "cwmp-parser-lab-only" }))
        .route("/lab/parse-inform", post(parse_only))
        .route("/cwmp", any(|| async { StatusCode::SERVICE_UNAVAILABLE }))
        .fallback(|| async { StatusCode::SERVICE_UNAVAILABLE })
        .layer(DefaultBodyLimit::max(cwmp_protocol::MAX_XML_BYTES))
}
fn protected_lab_headers() -> HeaderMap {
    let mut h = HeaderMap::new();
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    h.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    h.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json; charset=utf-8"),
    );
    h
}
async fn parse_only(
    headers: HeaderMap,
    body: Bytes,
) -> Result<(HeaderMap, &'static str), StatusCode> {
    // SOAPAction is deliberately ignored because validation MUST NOT
    // authorize or schedule any device session, RPC or enrollment.
    if headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        != Some("text/xml")
    {
        return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }
    let xml = std::str::from_utf8(&body).map_err(|_| StatusCode::BAD_REQUEST)?;
    cwmp_protocol::parse_inform(xml).map_err(|_| StatusCode::BAD_REQUEST)?;
    // No serial/OUI or supplied XML is ever echoed or logged.
    Ok((protected_lab_headers(), SAFE_LAB_REPLY))
}
#[tokio::main]
async fn main() {
    if std::env::var("IPAT_RUN_OFFLINE_CWMP_LAB").as_deref() != Ok("1") {
        eprintln!("Real ACS HTTPS/mTLS not enabled; opt-in local parser lab only.");
        std::process::exit(2);
    }
    // Never allow 0.0.0.0 or configurable public binds for this binary.
    let listener = tokio::net::TcpListener::bind(LAB_BIND)
        .await
        .expect("bind loopback-only parser laboratory");
    axum::serve(listener, app())
        .await
        .expect("serve loopback-only synthetic CWMP parser lab");
}
#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;
    const INFORM: &str = r#"<soap:Envelope
     xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/"
     xmlns:cwmp="urn:dslforum-org:cwmp-1-0">
     <soap:Header><cwmp:ID soap:mustUnderstand="1">synthetic-only</cwmp:ID></soap:Header>
     <soap:Body><cwmp:Inform>
       <DeviceId><Manufacturer>SYNTHETIC</Manufacturer><OUI>001122</OUI>
       <ProductClass>FAKE-ONT</ProductClass><SerialNumber>FAKE-NOT-PHYSICAL</SerialNumber></DeviceId>
       <Event><EventStruct><EventCode>0 BOOTSTRAP</EventCode><CommandKey/></EventStruct></Event>
       <MaxEnvelopes>1</MaxEnvelopes><CurrentTime>2026-09-26T12:00:00Z</CurrentTime>
       <RetryCount>0</RetryCount><ParameterList/>
     </cwmp:Inform></soap:Body></soap:Envelope>"#;
    async fn call(path: &str, content_type: Option<&str>, body: &str) -> StatusCode {
        let mut b = Request::builder().method("POST").uri(path);
        if let Some(ct) = content_type {
            b = b.header("content-type", ct);
        }
        app()
            .oneshot(b.body(Body::from(body.to_owned())).unwrap())
            .await
            .unwrap()
            .status()
    }
    #[test]
    fn fixed_loopback_only_no_public_bind_parameter() {
        assert_eq!(LAB_BIND, "127.0.0.1:3300");
    }
    #[tokio::test]
    async fn real_cwmp_denies_even_valid_inform_and_forged_headers() {
        let req = Request::builder()
            .method("POST")
            .uri("/cwmp")
            .header("x-tenant-id", "kangnet")
            .header("x-client-cert-verified", "true")
            .header("content-type", "text/xml")
            .body(Body::from(INFORM))
            .unwrap();
        assert_eq!(
            app().oneshot(req).await.unwrap().status(),
            StatusCode::SERVICE_UNAVAILABLE
        );
    }
    #[tokio::test]
    async fn lab_parser_accepts_but_does_not_enroll_or_issue_soap_response() {
        let req = Request::builder()
            .method("POST")
            .uri("/lab/parse-inform")
            .header("content-type", "text/xml")
            .header("x-tenant-id", "kangnet")
            .body(Body::from(INFORM))
            .unwrap();
        let res = app().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let data = axum::body::to_bytes(res.into_body(), 1024).await.unwrap();
        let text = std::str::from_utf8(&data).unwrap();
        assert_eq!(text, SAFE_LAB_REPLY);
        assert!(!text.contains("FAKE-NOT-PHYSICAL"));
        assert!(text.contains("\"peer_authenticated\":false"));
    }
    #[tokio::test]
    async fn refuses_wrong_type_malformed_xml_unsafe_xml_and_rpc_write() {
        assert_eq!(
            call("/lab/parse-inform", None, INFORM).await,
            StatusCode::UNSUPPORTED_MEDIA_TYPE
        );
        assert_eq!(
            call("/lab/parse-inform", Some("application/xml"), INFORM).await,
            StatusCode::UNSUPPORTED_MEDIA_TYPE
        );
        assert_eq!(
            call("/lab/parse-inform", Some("text/xml"), "<wrong/>").await,
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            call(
                "/lab/parse-inform",
                Some("text/xml"),
                &INFORM.replace("<soap:Envelope", "<!DOCTYPE x []><soap:Envelope")
            )
            .await,
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            call(
                "/lab/parse-inform",
                Some("text/xml"),
                &INFORM.replace("cwmp:Inform", "cwmp:SetParameterValues")
            )
            .await,
            StatusCode::BAD_REQUEST
        );
    }
    #[tokio::test]
    async fn strict_body_bound_and_unrecognized_paths() {
        assert_eq!(
            call("/lab/parse-inform", Some("text/xml"), &"x".repeat(65537)).await,
            StatusCode::PAYLOAD_TOO_LARGE
        );
        assert_eq!(
            call("/v1/tenants", Some("text/xml"), INFORM).await,
            StatusCode::SERVICE_UNAVAILABLE
        );
    }
}
