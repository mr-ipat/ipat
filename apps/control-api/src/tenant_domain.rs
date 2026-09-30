//! Production-shaped tenant-domain routing and DNS onboarding instructions.
//!
//! A verified hostname may select public tenant bootstrap context, but NEVER
//! grants user membership, role, POP scope, device access, or secrets.

use axum::{
    extract::State,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashSet,
    net::{Ipv4Addr, Ipv6Addr},
    path::PathBuf,
    sync::Arc,
};
use tokio_postgres::{Config, NoTls};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalHost(String);

impl CanonicalHost {
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

pub(super) fn canonical_dns_name(raw: &str) -> Option<CanonicalHost> {
    let raw = raw.trim();
    if raw.is_empty() || raw.len() > 255 || raw.contains(['/', '\\', ' ', '\t', '\r', '\n']) {
        return None;
    }
    if raw.starts_with('[') {
        return None;
    }
    let host = raw.strip_suffix('.').unwrap_or(raw);
    if host.len() < 3
        || host.len() > 253
        || host.starts_with('.')
        || host.ends_with('.')
        || host.contains("..")
    {
        return None;
    }
    let lower = host.to_ascii_lowercase();
    if !lower
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'.')
    {
        return None;
    }
    for label in lower.split('.') {
        if label.is_empty() || label.len() > 63 || label.starts_with('-') || label.ends_with('-') {
            return None;
        }
    }
    Some(CanonicalHost(lower))
}

pub(crate) fn canonical_host(value: &HeaderValue) -> Option<CanonicalHost> {
    let raw = value.to_str().ok()?.trim();
    if raw.starts_with('[') {
        return None;
    }
    let host = match raw.rsplit_once(':') {
        Some((left, port))
            if !left.is_empty() && !port.is_empty() && port.bytes().all(|b| b.is_ascii_digit()) =>
        {
            left
        }
        Some(_) => raw,
        None => raw,
    };
    if raw.contains(':') && host == raw {
        return None;
    }
    canonical_dns_name(host)
}

pub(super) fn valid_custom_domain(host: &CanonicalHost) -> bool {
    let name = host.as_str();
    name.contains('.')
        && !name.ends_with(".local")
        && !name.ends_with(".localhost")
        && !name.ends_with(".invalid")
        && !name.ends_with(".test")
        && !name.ends_with(".example")
}

pub(crate) struct DomainStore {
    db: Config,
}

fn safe_name(value: &str, max: usize) -> bool {
    !value.is_empty()
        && value.len() <= max
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
}

/// Initial runtime adapter uses a local Unix socket with peer/local credential
/// isolation. It intentionally refuses TCP without a reviewed TLS DB adapter.
pub(crate) fn from_environment() -> Result<Arc<DomainStore>, String> {
    let socket =
        std::env::var("IPAT_TENANT_DOMAIN_DB_SOCKET").map_err(|_| "missing domain DB socket")?;
    let database =
        std::env::var("IPAT_TENANT_DOMAIN_DB_NAME").map_err(|_| "missing domain DB name")?;
    let user = std::env::var("IPAT_TENANT_DOMAIN_DB_USER").map_err(|_| "missing domain DB user")?;
    if user != "ipat_domain_reader_login" || !safe_name(&database, 63) {
        return Err("unexpected domain DB identity".into());
    }
    let path = PathBuf::from(&socket);
    if !path.is_absolute() || socket.len() > 200 || socket.contains("..") {
        return Err("domain DB socket must be absolute and canonical".into());
    }
    let mut db = Config::new();
    db.host_path(path);
    db.user(&user);
    db.dbname(&database);
    Ok(Arc::new(DomainStore { db }))
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DnsRoutingMode {
    Auto,
    ARecord,
    Cname,
    Nameserver,
}

impl DnsRoutingMode {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::ARecord => "a_record",
            Self::Cname => "cname",
            Self::Nameserver => "nameserver",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct DnsInstructionProfile {
    mode: DnsRoutingMode,
    ipv4: Option<Ipv4Addr>,
    ipv6: Option<Ipv6Addr>,
    cname_target: Option<CanonicalHost>,
    nameservers: Vec<CanonicalHost>,
    routing_ready: bool,
    authoritative_dns_ready: bool,
}

pub(crate) fn dns_profile_from_environment() -> Result<Arc<DnsInstructionProfile>, String> {
    let mode = match std::env::var("IPAT_CUSTOM_DOMAIN_DNS_MODE")
        .map_err(|_| "missing custom-domain DNS mode")?
        .as_str()
    {
        "auto" => DnsRoutingMode::Auto,
        "a_record" => DnsRoutingMode::ARecord,
        "cname" => DnsRoutingMode::Cname,
        "nameserver" => DnsRoutingMode::Nameserver,
        _ => return Err("unsupported custom-domain DNS mode".into()),
    };
    let ipv4 = std::env::var("IPAT_CUSTOM_DOMAIN_IPV4")
        .ok()
        .map(|value| value.parse::<Ipv4Addr>())
        .transpose()
        .map_err(|_| "invalid custom-domain IPv4 target")?;
    let ipv6 = std::env::var("IPAT_CUSTOM_DOMAIN_IPV6")
        .ok()
        .map(|value| value.parse::<Ipv6Addr>())
        .transpose()
        .map_err(|_| "invalid custom-domain IPv6 target")?;
    let cname_target = std::env::var("IPAT_CUSTOM_DOMAIN_CNAME_TARGET")
        .ok()
        .map(|value| canonical_dns_name(&value).ok_or("invalid custom-domain CNAME target"))
        .transpose()?;
    let nameservers_configured = std::env::var("IPAT_CUSTOM_DOMAIN_NAMESERVERS").is_ok();
    let nameservers = std::env::var("IPAT_CUSTOM_DOMAIN_NAMESERVERS")
        .ok()
        .map(|value| {
            value
                .split(',')
                .map(|name| {
                    canonical_dns_name(name)
                        .ok_or_else(|| "invalid custom-domain nameserver".to_string())
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?
        .unwrap_or_default();
    let routing_ready = std::env::var("IPAT_CUSTOM_DOMAIN_ROUTING_READY").as_deref() == Ok("YES");
    let authoritative_dns_ready =
        std::env::var("IPAT_CUSTOM_DOMAIN_AUTHORITATIVE_DNS_READY").as_deref() == Ok("YES");

    let unique_nameservers = nameservers
        .iter()
        .map(|n| n.as_str())
        .collect::<HashSet<_>>();
    if nameservers_configured
        && (unique_nameservers.len() < 2 || unique_nameservers.len() != nameservers.len())
    {
        return Err("nameserver routing requires at least two unique nameservers".into());
    }

    match mode {
        DnsRoutingMode::Auto
            if ipv4.is_none()
                && ipv6.is_none()
                && cname_target.is_none()
                && nameservers.is_empty() =>
        {
            return Err("auto routing requires at least one usable DNS target".into());
        }
        DnsRoutingMode::ARecord if ipv4.is_none() && ipv6.is_none() => {
            return Err("A/AAAA routing requires at least one IP target".into());
        }
        DnsRoutingMode::Cname if cname_target.is_none() => {
            return Err("CNAME routing requires canonical target".into());
        }
        DnsRoutingMode::Nameserver if unique_nameservers.len() < 2 => {
            return Err("nameserver routing requires at least two unique nameservers".into());
        }
        _ => {}
    }

    Ok(Arc::new(DnsInstructionProfile {
        mode,
        ipv4,
        ipv6,
        cname_target,
        nameservers,
        routing_ready,
        authoritative_dns_ready,
    }))
}

#[derive(Debug, Deserialize)]
struct DomainInstructionRequest {
    hostname: String,
}

#[derive(Debug, Serialize)]
struct DnsRecordInstruction {
    record_type: &'static str,
    name: String,
    value: String,
    stage: &'static str,
}

#[derive(Debug, Serialize)]
struct DomainInstructionResponse {
    hostname: String,
    routing_mode: &'static str,
    available_routing_modes: Vec<&'static str>,
    selection_reason: &'static str,
    customer_action: &'static str,
    routing_records: Vec<DnsRecordInstruction>,
    routing_target_known: bool,
    routing_ready: bool,
    authoritative_dns_ready: bool,
    safe_to_point_now: bool,
    verification_record_type: &'static str,
    verification_record_name: String,
    verification_value_issued_after_save: bool,
    activation_requires_verification: bool,
    authorization_granted: bool,
}

fn available_routing_modes(
    profile: &DnsInstructionProfile,
    hostname: &CanonicalHost,
) -> Vec<&'static str> {
    let mut modes = Vec::new();
    if profile.ipv4.is_some() || profile.ipv6.is_some() {
        modes.push("a_record");
    }
    if profile
        .cname_target
        .as_ref()
        .is_some_and(|target| target != hostname)
    {
        modes.push("cname");
    }
    if profile.nameservers.len() >= 2 {
        modes.push("nameserver");
    }
    modes
}

fn select_routing_mode(
    profile: &DnsInstructionProfile,
    hostname: &CanonicalHost,
) -> Option<(DnsRoutingMode, &'static str)> {
    match profile.mode {
        DnsRoutingMode::ARecord => {
            Some((DnsRoutingMode::ARecord, "DEPLOYMENT_FORCED_A_RECORD"))
        }
        DnsRoutingMode::Cname => {
            let target = profile.cname_target.as_ref()?;
            if target == hostname {
                None
            } else {
                Some((DnsRoutingMode::Cname, "DEPLOYMENT_FORCED_CNAME"))
            }
        }
        DnsRoutingMode::Nameserver => (profile.nameservers.len() >= 2).then_some((
            DnsRoutingMode::Nameserver,
            "DEPLOYMENT_FORCED_NAMESERVER",
        )),
        DnsRoutingMode::Auto => {
            if profile.ipv4.is_some() || profile.ipv6.is_some() {
                Some((DnsRoutingMode::ARecord, "AUTO_STABLE_INGRESS_ADDRESS"))
            } else if profile
                .cname_target
                .as_ref()
                .is_some_and(|target| target != hostname)
            {
                Some((
                    DnsRoutingMode::Cname,
                    "AUTO_CANONICAL_INGRESS_HOSTNAME",
                ))
            } else if profile.nameservers.len() >= 2 {
                Some((
                    DnsRoutingMode::Nameserver,
                    "AUTO_AUTHORITATIVE_NAMESERVERS",
                ))
            } else {
                None
            }
        }
    }
}

fn build_dns_instructions(
    profile: &DnsInstructionProfile,
    hostname: CanonicalHost,
) -> Option<DomainInstructionResponse> {
    if !valid_custom_domain(&hostname) {
        return None;
    }
    let available_routing_modes = available_routing_modes(profile, &hostname);
    let (selected_mode, selection_reason) = select_routing_mode(profile, &hostname)?;
    let customer_action = match selected_mode {
        DnsRoutingMode::ARecord => "CREATE_ADDRESS_RECORDS",
        DnsRoutingMode::Cname => "CREATE_CNAME_RECORD",
        DnsRoutingMode::Nameserver => "DELEGATE_NAMESERVERS",
        DnsRoutingMode::Auto => return None,
    };
    let mut routing_records = Vec::new();
    match selected_mode {
        DnsRoutingMode::ARecord => {
            if let Some(ipv4) = profile.ipv4 {
                routing_records.push(DnsRecordInstruction {
                    record_type: "A",
                    name: hostname.as_str().to_string(),
                    value: ipv4.to_string(),
                    stage: "after_ownership_verification",
                });
            }
            if let Some(ipv6) = profile.ipv6 {
                routing_records.push(DnsRecordInstruction {
                    record_type: "AAAA",
                    name: hostname.as_str().to_string(),
                    value: ipv6.to_string(),
                    stage: "after_ownership_verification",
                });
            }
        }
        DnsRoutingMode::Cname => {
            let target = profile.cname_target.as_ref()?;
            if target == &hostname {
                return None;
            }
            routing_records.push(DnsRecordInstruction {
                record_type: "CNAME",
                name: hostname.as_str().to_string(),
                value: target.as_str().to_string(),
                stage: "after_ownership_verification",
            });
        }
        DnsRoutingMode::Nameserver => {
            for target in &profile.nameservers {
                routing_records.push(DnsRecordInstruction {
                    record_type: "NS",
                    name: hostname.as_str().to_string(),
                    value: target.as_str().to_string(),
                    stage: "after_ownership_verification",
                });
            }
        }
        DnsRoutingMode::Auto => return None,
    }

    let authoritative_dns_ready = match selected_mode {
        DnsRoutingMode::Nameserver => profile.authoritative_dns_ready,
        _ => true,
    };
    let safe_to_point_now = profile.routing_ready && authoritative_dns_ready;

    Some(DomainInstructionResponse {
        verification_record_name: format!("_ipat-verify.{}", hostname.as_str()),
        hostname: hostname.as_str().to_string(),
        routing_mode: selected_mode.as_str(),
        available_routing_modes,
        selection_reason,
        customer_action,
        routing_target_known: !routing_records.is_empty(),
        routing_records,
        routing_ready: profile.routing_ready,
        authoritative_dns_ready,
        safe_to_point_now,
        verification_record_type: "TXT",
        verification_value_issued_after_save: true,
        activation_requires_verification: true,
        authorization_granted: false,
    })
}

fn headers() -> HeaderMap {
    let mut h = HeaderMap::new();
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    h.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    h
}

async fn dns_instructions(
    State(profile): State<Arc<DnsInstructionProfile>>,
    Json(input): Json<DomainInstructionRequest>,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let Some(hostname) = canonical_dns_name(&input.hostname) else {
        return (
            StatusCode::BAD_REQUEST,
            headers(),
            Json(json!({"error":"INVALID_DOMAIN"})),
        );
    };
    let Some(result) = build_dns_instructions(&profile, hostname) else {
        return (
            StatusCode::BAD_REQUEST,
            headers(),
            Json(json!({"error":"UNSUPPORTED_DOMAIN"})),
        );
    };
    (
        StatusCode::OK,
        headers(),
        Json(serde_json::to_value(result).expect("serializable DNS instructions")),
    )
}

async fn bootstrap(
    State(store): State<Arc<DomainStore>>,
    request_headers: HeaderMap,
) -> (StatusCode, HeaderMap, Json<Value>) {
    if request_headers.get_all(header::HOST).iter().count() != 1 {
        return (
            StatusCode::BAD_REQUEST,
            headers(),
            Json(json!({"tenant":null})),
        );
    }
    let Some(host) = request_headers.get(header::HOST).and_then(canonical_host) else {
        return (
            StatusCode::BAD_REQUEST,
            headers(),
            Json(json!({"tenant":null})),
        );
    };

    let Ok((client, connection)) = store.db.connect(NoTls).await else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            headers(),
            Json(json!({"tenant":null})),
        );
    };
    let task = tokio::spawn(async move {
        let _ = connection.await;
    });
    let row = client
        .query_opt(
            "SELECT tenant_id::text,tenant_slug,hostname,domain_type
             FROM ipat_platform.resolve_active_tenant_domain($1)",
            &[&host.as_str()],
        )
        .await;
    drop(client);
    task.abort();

    let Ok(Some(row)) = row else {
        return match row {
            Ok(None) => (
                StatusCode::NOT_FOUND,
                headers(),
                Json(json!({"tenant":null})),
            ),
            Err(_) => (
                StatusCode::SERVICE_UNAVAILABLE,
                headers(),
                Json(json!({"tenant":null})),
            ),
            _ => unreachable!(),
        };
    };
    let tenant_id: String = row.get(0);
    let tenant_slug: String = row.get(1);
    let hostname: String = row.get(2);
    let domain_type: String = row.get(3);
    if hostname != host.as_str() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            headers(),
            Json(json!({"tenant":null})),
        );
    }
    (
        StatusCode::OK,
        headers(),
        Json(json!({
            "tenant":{
                "id":tenant_id,
                "slug":tenant_slug,
                "hostname":hostname,
                "domain_type":domain_type
            },
            "authorization_granted":false,
            "membership_required":true
        })),
    )
}

pub(crate) fn router(store: Arc<DomainStore>) -> Router {
    Router::new()
        .route("/v1/bootstrap/tenant", get(bootstrap))
        .with_state(store)
}

pub(crate) fn instruction_router(profile: Arc<DnsInstructionProfile>) -> Router {
    Router::new()
        .route("/v1/domains/instructions", post(dns_instructions))
        .with_state(profile)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::Request,
    };
    use tower::ServiceExt;

    fn profile_a() -> Arc<DnsInstructionProfile> {
        Arc::new(DnsInstructionProfile {
            mode: DnsRoutingMode::ARecord,
            ipv4: Some("203.0.113.20".parse().unwrap()),
            ipv6: None,
            cname_target: None,
            nameservers: Vec::new(),
            routing_ready: false,
            authoritative_dns_ready: false,
        })
    }

    fn profile_auto() -> DnsInstructionProfile {
        DnsInstructionProfile {
            mode: DnsRoutingMode::Auto,
            ipv4: Some("203.0.113.20".parse().unwrap()),
            ipv6: None,
            cname_target: canonical_dns_name("edge.ipat.id"),
            nameservers: vec![
                canonical_dns_name("ns1.ipat.id").unwrap(),
                canonical_dns_name("ns2.ipat.id").unwrap(),
            ],
            routing_ready: true,
            authoritative_dns_ready: true,
        }
    }

    #[test]
    fn canonicalizes_dns_host_without_conferring_authority() {
        let h = HeaderValue::from_static("IPAT.FADLY.ID:443");
        assert_eq!(canonical_host(&h).unwrap().as_str(), "ipat.fadly.id");
    }

    #[test]
    fn accepts_managed_customer_subdomain() {
        let h = HeaderValue::from_static("kangnet.ipat.id");
        assert_eq!(canonical_host(&h).unwrap().as_str(), "kangnet.ipat.id");
    }

    #[test]
    fn rejects_ambiguous_or_non_dns_authorities() {
        for raw in [
            "ipat.fadly.id:notaport",
            "ipat..fadly.id",
            "-bad.ipat.id",
            "bad-.ipat.id",
            "[::1]:443",
            "ipat.fadly.id/path",
            "ipat.fadly.id evil.example",
        ] {
            let value = HeaderValue::from_bytes(raw.as_bytes()).unwrap();
            assert!(canonical_host(&value).is_none(), "{raw}");
        }
    }

    #[test]
    fn a_record_profile_builds_customer_specific_instructions() {
        let response = build_dns_instructions(
            &profile_a(),
            canonical_dns_name("portal.customer.id").unwrap(),
        )
        .unwrap();
        assert_eq!(response.hostname, "portal.customer.id");
        assert_eq!(response.routing_mode, "a_record");
        assert_eq!(response.routing_records.len(), 1);
        assert_eq!(response.routing_records[0].record_type, "A");
        assert_eq!(response.routing_records[0].value, "203.0.113.20");
        assert_eq!(
            response.verification_record_name,
            "_ipat-verify.portal.customer.id"
        );
        assert!(response.routing_target_known);
        assert!(!response.routing_ready);
        assert!(!response.safe_to_point_now);
        assert!(!response.authorization_granted);
    }

    #[test]
    fn auto_profile_prefers_stable_address_and_reports_all_available_methods() {
        let profile = profile_auto();
        let response = build_dns_instructions(
            &profile,
            canonical_dns_name("portal.customer.id").unwrap(),
        )
        .unwrap();
        assert_eq!(response.routing_mode, "a_record");
        assert_eq!(response.customer_action, "CREATE_ADDRESS_RECORDS");
        assert_eq!(response.selection_reason, "AUTO_STABLE_INGRESS_ADDRESS");
        assert_eq!(
            response.available_routing_modes,
            vec!["a_record", "cname", "nameserver"]
        );
        assert_eq!(response.routing_records[0].value, "203.0.113.20");
        assert!(response.safe_to_point_now);
    }

    #[test]
    fn auto_profile_falls_back_to_cname_when_address_is_unavailable() {
        let mut profile = profile_auto();
        profile.ipv4 = None;
        let response =
            build_dns_instructions(&profile, canonical_dns_name("portal.customer.id").unwrap())
                .unwrap();
        assert_eq!(response.routing_mode, "cname");
        assert_eq!(response.customer_action, "CREATE_CNAME_RECORD");
        assert_eq!(response.routing_records[0].value, "edge.ipat.id");
    }

    #[test]
    fn auto_profile_falls_back_to_nameservers_when_other_targets_are_unavailable() {
        let mut profile = profile_auto();
        profile.ipv4 = None;
        profile.cname_target = None;
        let response =
            build_dns_instructions(&profile, canonical_dns_name("customer.co.id").unwrap())
                .unwrap();
        assert_eq!(response.routing_mode, "nameserver");
        assert_eq!(response.customer_action, "DELEGATE_NAMESERVERS");
        assert_eq!(response.routing_records.len(), 2);
        assert!(response.safe_to_point_now);
    }

    #[test]
    fn nameserver_profile_requires_two_unique_nameservers() {
        let profile = DnsInstructionProfile {
            mode: DnsRoutingMode::Nameserver,
            ipv4: None,
            ipv6: None,
            cname_target: None,
            nameservers: vec![
                canonical_dns_name("ns1.ipat.id").unwrap(),
                canonical_dns_name("ns2.ipat.id").unwrap(),
            ],
            routing_ready: true,
            authoritative_dns_ready: true,
        };
        let response =
            build_dns_instructions(&profile, canonical_dns_name("customer.co.id").unwrap())
                .unwrap();
        assert_eq!(response.routing_mode, "nameserver");
        assert_eq!(response.routing_records.len(), 2);
        assert!(response
            .routing_records
            .iter()
            .all(|record| record.record_type == "NS"));
        assert!(response.authoritative_dns_ready);
        assert!(response.safe_to_point_now);
    }

    #[test]
    fn nameserver_target_is_not_safe_until_authoritative_dns_is_ready() {
        let profile = DnsInstructionProfile {
            mode: DnsRoutingMode::Nameserver,
            ipv4: None,
            ipv6: None,
            cname_target: None,
            nameservers: vec![
                canonical_dns_name("ns1.ipat.id").unwrap(),
                canonical_dns_name("ns2.ipat.id").unwrap(),
            ],
            routing_ready: true,
            authoritative_dns_ready: false,
        };
        let response =
            build_dns_instructions(&profile, canonical_dns_name("customer.co.id").unwrap())
                .unwrap();
        assert!(response.routing_target_known);
        assert!(response.routing_ready);
        assert!(!response.authoritative_dns_ready);
        assert!(!response.safe_to_point_now);
    }

    #[tokio::test]
    async fn instruction_endpoint_rejects_reserved_or_malformed_domains() {
        let app = instruction_router(profile_a());
        for hostname in ["localhost", "tenant.invalid", "bad domain.id"] {
            let body = serde_json::to_vec(&json!({"hostname":hostname})).unwrap();
            let response = app
                .clone()
                .oneshot(
                    Request::post("/v1/domains/instructions")
                        .header("content-type", "application/json")
                        .body(Body::from(body))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        }
    }

    #[tokio::test]
    async fn instruction_endpoint_returns_no_store_and_no_authorization() {
        let app = instruction_router(profile_a());
        let body = serde_json::to_vec(&json!({"hostname":"portal.customer.id"})).unwrap();
        let response = app
            .oneshot(
                Request::post("/v1/domains/instructions")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CACHE_CONTROL).unwrap(),
            "no-store"
        );
        let body = to_bytes(response.into_body(), 32 * 1024).await.unwrap();
        let value: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(value["routing_mode"], "a_record");
        assert_eq!(value["authorization_granted"], false);
        assert_eq!(value["activation_requires_verification"], true);
        assert_eq!(value["routing_target_known"], true);
        assert_eq!(value["routing_ready"], false);
        assert_eq!(value["safe_to_point_now"], false);
    }

    #[test]
    fn runtime_db_adapter_refuses_unexpected_identity_and_relative_socket() {
        std::env::set_var("IPAT_TENANT_DOMAIN_DB_SOCKET", "relative/socket");
        std::env::set_var("IPAT_TENANT_DOMAIN_DB_NAME", "ipat");
        std::env::set_var("IPAT_TENANT_DOMAIN_DB_USER", "postgres");
        assert!(from_environment().is_err());
        std::env::remove_var("IPAT_TENANT_DOMAIN_DB_SOCKET");
        std::env::remove_var("IPAT_TENANT_DOMAIN_DB_NAME");
        std::env::remove_var("IPAT_TENANT_DOMAIN_DB_USER");
    }
}
