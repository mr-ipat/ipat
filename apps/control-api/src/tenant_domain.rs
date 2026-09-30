//! Production-path tenant hostname resolution.
//! A verified Host selects routing context only. Authorization still requires
//! independently verified identity + DB membership + RBAC/ABAC on every API.
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
    fs::OpenOptions,
    io::Read,
    net::{Ipv4Addr, Ipv6Addr},
    os::unix::{fs::MetadataExt, fs::OpenOptionsExt},
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};
use tokio_postgres::{config::Host, Config, NoTls};

const EXPECTED_DOMAIN_READER: &str = "ipat_domain_reader";
const MAX_CONNFILE_BYTES: u64 = 4096;

pub(crate) struct Store {
    db: Config,
}

fn response_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    headers
}
fn canonical_hostname(headers: &HeaderMap) -> Result<String, StatusCode> {
    if headers.get_all(header::HOST).iter().count() != 1 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let raw = headers
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
        .ok_or(StatusCode::BAD_REQUEST)?;
    if raw.is_empty()
        || raw.len() > 259
        || !raw.is_ascii()
        || raw.chars().any(|c| matches!(c, '/' | '\\' | '@' | ' '))
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let host = match raw.rsplit_once(':') {
        Some((name, port))
            if !name.contains(':')
                && !port.is_empty()
                && port.bytes().all(|b| b.is_ascii_digit()) =>
        {
            name
        }
        Some(_) if raw.contains(':') => return Err(StatusCode::BAD_REQUEST),
        _ => raw,
    };
    let host = host.to_ascii_lowercase();
    if host.len() < 3
        || host.len() > 253
        || host.starts_with('.')
        || host.ends_with('.')
        || host.contains("..")
        || !host.contains('.')
        || !host.bytes().any(|b| b.is_ascii_lowercase())
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    for label in host.split('.') {
        if label.is_empty()
            || label.len() > 63
            || label.starts_with('-')
            || label.ends_with('-')
            || !label
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        {
            return Err(StatusCode::BAD_REQUEST);
        }
    }
    Ok(host)
}

fn canonical_requested_domain(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty()
        || raw.len() > 253
        || !raw.is_ascii()
        || raw
            .chars()
            .any(|c| matches!(c, '/' | '\\' | '@' | ' ' | '\t' | '\r' | '\n' | ':'))
    {
        return None;
    }
    let fqdn = raw.to_ascii_lowercase();
    if fqdn.len() < 3
        || fqdn.starts_with('.')
        || fqdn.ends_with('.')
        || fqdn.contains("..")
        || !fqdn.contains('.')
        || [
            ".local",
            ".localhost",
            ".invalid",
            ".test",
            ".example",
        ]
        .iter()
        .any(|suffix| fqdn.ends_with(suffix))
    {
        return None;
    }
    for label in fqdn.split('.') {
        if label.is_empty()
            || label.len() > 63
            || label.starts_with('-')
            || label.ends_with('-')
            || !label
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        {
            return None;
        }
    }
    Some(fqdn)
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
    cname_target: Option<String>,
    nameservers: Vec<String>,
    auto_allow_cname: bool,
    routing_ready: bool,
    authoritative_dns_ready: bool,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
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
    verification_record_type: &'static str,
    verification_record_name: String,
    verification_value_issued_after_save: bool,
    activation_requires_verification: bool,
    routing_target_known: bool,
    routing_ready: bool,
    authoritative_dns_ready: bool,
    safe_to_point_now: bool,
    authorization_granted: bool,
}

fn env_yes(name: &str) -> bool {
    std::env::var(name).as_deref() == Ok("YES")
}

pub(crate) fn dns_profile_from_environment() -> Result<Arc<DnsInstructionProfile>, &'static str> {
    if !env_yes("IPAT_CUSTOM_DOMAIN_INSTRUCTIONS") || unsafe { libc::geteuid() } == 0 {
        return Err("custom-domain instructions require explicit nonroot opt-in");
    }
    let mode = match std::env::var("IPAT_CUSTOM_DOMAIN_DNS_MODE")
        .map_err(|_| "missing custom-domain DNS mode")?
        .as_str()
    {
        "auto" => DnsRoutingMode::Auto,
        "a_record" => DnsRoutingMode::ARecord,
        "cname" => DnsRoutingMode::Cname,
        "nameserver" => DnsRoutingMode::Nameserver,
        _ => return Err("unsupported custom-domain DNS mode"),
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
        .map(|value| canonical_requested_domain(&value).ok_or("invalid custom-domain CNAME target"))
        .transpose()?;
    let nameservers_configured = std::env::var("IPAT_CUSTOM_DOMAIN_NAMESERVERS").is_ok();
    let nameservers = std::env::var("IPAT_CUSTOM_DOMAIN_NAMESERVERS")
        .ok()
        .map(|value| {
            value
                .split(',')
                .map(|name| canonical_requested_domain(name).ok_or("invalid custom-domain nameserver"))
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?
        .unwrap_or_default();
    let auto_allow_cname = env_yes("IPAT_CUSTOM_DOMAIN_AUTO_ALLOW_CNAME");
    let routing_ready = env_yes("IPAT_CUSTOM_DOMAIN_ROUTING_READY");
    let authoritative_dns_ready = env_yes("IPAT_CUSTOM_DOMAIN_AUTHORITATIVE_DNS");

    let unique_nameservers = nameservers.iter().collect::<HashSet<_>>();
    if nameservers_configured
        && (unique_nameservers.len() < 2 || unique_nameservers.len() != nameservers.len())
    {
        return Err("nameserver routing requires at least two unique nameservers");
    }
    match mode {
        DnsRoutingMode::Auto
            if ipv4.is_none()
                && ipv6.is_none()
                && cname_target.is_none()
                && nameservers.is_empty() =>
        {
            return Err("auto routing requires at least one usable DNS target");
        }
        DnsRoutingMode::ARecord if ipv4.is_none() && ipv6.is_none() => {
            return Err("A/AAAA routing requires at least one IP target");
        }
        DnsRoutingMode::Cname if cname_target.is_none() => {
            return Err("CNAME routing requires canonical target");
        }
        DnsRoutingMode::Nameserver if unique_nameservers.len() < 2 => {
            return Err("nameserver routing requires at least two unique nameservers");
        }
        DnsRoutingMode::Nameserver if routing_ready && !authoritative_dns_ready => {
            return Err("ready NS routing requires authoritative DNS service evidence");
        }
        _ => {}
    }
    Ok(Arc::new(DnsInstructionProfile {
        mode,
        ipv4,
        ipv6,
        cname_target,
        nameservers,
        auto_allow_cname,
        routing_ready,
        authoritative_dns_ready,
    }))
}

fn available_routing_modes(
    profile: &DnsInstructionProfile,
    hostname: &str,
) -> Vec<&'static str> {
    let mut modes = Vec::new();
    if profile.ipv4.is_some() || profile.ipv6.is_some() {
        modes.push("a_record");
    }
    if profile.nameservers.len() >= 2 {
        modes.push("nameserver");
    }
    if (profile.mode != DnsRoutingMode::Auto || profile.auto_allow_cname)
        && profile.cname_target.as_deref().is_some_and(|target| target != hostname)
    {
        modes.push("cname");
    }
    modes
}

fn select_routing_mode(
    profile: &DnsInstructionProfile,
    hostname: &str,
) -> Option<(DnsRoutingMode, &'static str)> {
    match profile.mode {
        DnsRoutingMode::ARecord => Some((DnsRoutingMode::ARecord, "DEPLOYMENT_FORCED_A_RECORD")),
        DnsRoutingMode::Cname => profile
            .cname_target
            .as_deref()
            .filter(|target| *target != hostname)
            .map(|_| (DnsRoutingMode::Cname, "DEPLOYMENT_FORCED_CNAME")),
        DnsRoutingMode::Nameserver => (profile.nameservers.len() >= 2)
            .then_some((DnsRoutingMode::Nameserver, "DEPLOYMENT_FORCED_NAMESERVER")),
        DnsRoutingMode::Auto => {
            if profile.ipv4.is_some() || profile.ipv6.is_some() {
                Some((DnsRoutingMode::ARecord, "AUTO_STABLE_INGRESS_ADDRESS"))
            } else if profile.nameservers.len() >= 2 {
                Some((DnsRoutingMode::Nameserver, "AUTO_AUTHORITATIVE_NAMESERVERS"))
            } else if profile.auto_allow_cname
                && profile.cname_target.as_deref().is_some_and(|target| target != hostname)
            {
                Some((DnsRoutingMode::Cname, "AUTO_CANONICAL_INGRESS_HOSTNAME"))
            } else {
                None
            }
        }
    }
}

fn build_dns_instructions(
    profile: &DnsInstructionProfile,
    hostname: String,
) -> Option<DomainInstructionResponse> {
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
                    name: hostname.clone(),
                    value: ipv4.to_string(),
                    stage: "after_ownership_verification",
                });
            }
            if let Some(ipv6) = profile.ipv6 {
                routing_records.push(DnsRecordInstruction {
                    record_type: "AAAA",
                    name: hostname.clone(),
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
                name: hostname.clone(),
                value: target.clone(),
                stage: "after_ownership_verification",
            });
        }
        DnsRoutingMode::Nameserver => {
            for target in &profile.nameservers {
                routing_records.push(DnsRecordInstruction {
                    record_type: "NS",
                    name: hostname.clone(),
                    value: target.clone(),
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
        verification_record_name: format!("_ipat-verify.{hostname}"),
        hostname,
        routing_mode: selected_mode.as_str(),
        available_routing_modes,
        selection_reason,
        customer_action,
        routing_target_known: !routing_records.is_empty(),
        routing_records,
        verification_record_type: "TXT",
        verification_value_issued_after_save: true,
        activation_requires_verification: true,
        routing_ready: profile.routing_ready,
        authoritative_dns_ready,
        safe_to_point_now,
        authorization_granted: false,
    })
}

async fn dns_instructions(
    State(profile): State<Arc<DnsInstructionProfile>>,
    Json(input): Json<DomainInstructionRequest>,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let Some(hostname) = canonical_requested_domain(&input.hostname) else {
        return (
            StatusCode::BAD_REQUEST,
            response_headers(),
            Json(json!({"error":"INVALID_DOMAIN"})),
        );
    };
    let Some(result) = build_dns_instructions(&profile, hostname) else {
        return (
            StatusCode::BAD_REQUEST,
            response_headers(),
            Json(json!({"error":"UNSUPPORTED_DOMAIN"})),
        );
    };
    (
        StatusCode::OK,
        response_headers(),
        Json(serde_json::to_value(result).expect("serializable DNS instructions")),
    )
}

pub(crate) fn instruction_router(profile: Arc<DnsInstructionProfile>) -> Router {
    Router::new()
        .route("/v1/domains/instructions", post(dns_instructions))
        .with_state(profile)
}

async fn context(
    State(store): State<Arc<Store>>,
    headers: HeaderMap,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let host = match canonical_hostname(&headers) {
        Ok(host) => host,
        Err(status) => {
            return (
                status,
                response_headers(),
                Json(json!({"tenant_context":false})),
            )
        }
    };
    let Ok((client, connection)) = store.db.connect(NoTls).await else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            response_headers(),
            Json(json!({"tenant_context":false})),
        );
    };
    let task = tokio::spawn(async move {
        let _ = connection.await;
    });
    let row = client
        .query_opt(
            "SELECT tenant_slug,fqdn FROM ipat_platform.resolve_verified_tenant_domain($1)",
            &[&host],
        )
        .await;
    drop(client);
    task.abort();

    let Ok(Some(row)) = row else {
        return match row {
            Ok(None) => (
                StatusCode::NOT_FOUND,
                response_headers(),
                Json(json!({"tenant_context":false})),
            ),
            Err(_) => (
                StatusCode::SERVICE_UNAVAILABLE,
                response_headers(),
                Json(json!({"tenant_context":false})),
            ),
            _ => unreachable!(),
        };
    };
    let tenant_slug: String = row.get(0);
    let fqdn: String = row.get(1);
    if fqdn != host {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            response_headers(),
            Json(json!({"tenant_context":false})),
        );
    }
    (
        StatusCode::OK,
        response_headers(),
        Json(json!({
            "tenant_context":true,
            "tenant_slug":tenant_slug,
            "hostname":fqdn,
            "authentication_required":true,
            "business_access_enabled":false
        })),
    )
}

pub(crate) fn router(store: Arc<Store>) -> Router {
    Router::new()
        .route("/v1/tenant-context", get(context))
        .with_state(store)
}
fn read_private_file(path: &Path) -> Result<String, &'static str> {
    let parent = path.parent().ok_or("missing private config directory")?;
    if !path.is_absolute() {
        return Err("domain DB config path must be absolute");
    }
    for (point, mode) in [(parent, 0o700), (path, 0o600)] {
        let metadata = std::fs::symlink_metadata(point).map_err(|_| "missing domain DB config")?;
        if metadata.file_type().is_symlink()
            || metadata.uid() != unsafe { libc::geteuid() }
            || metadata.mode() & 0o777 != mode
        {
            return Err("unsafe domain DB config ownership/mode");
        }
    }
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .map_err(|_| "invalid domain DB config")?;
    let metadata = file.metadata().map_err(|_| "invalid domain DB config")?;
    if !metadata.is_file()
        || metadata.nlink() != 1
        || metadata.len() == 0
        || metadata.len() > MAX_CONNFILE_BYTES
    {
        return Err("unsafe domain DB config file");
    }
    let mut value = String::new();
    file.take(MAX_CONNFILE_BYTES + 1)
        .read_to_string(&mut value)
        .map_err(|_| "invalid domain DB config encoding")?;
    if value.len() as u64 > MAX_CONNFILE_BYTES {
        return Err("oversized domain DB config");
    }
    Ok(value)
}
fn valid_db_config(config: &Config) -> bool {
    config.get_user() == Some(EXPECTED_DOMAIN_READER)
        && config.get_dbname().is_some()
        && config.get_hosts().len() == 1
        && config.get_hostaddrs().is_empty()
        && config.get_options().is_none()
        && matches!(config.get_hosts()[0], Host::Unix(ref path) if path.is_absolute())
}

pub(crate) fn from_environment() -> Result<Arc<Store>, &'static str> {
    if std::env::var("IPAT_TENANT_DOMAIN_ROUTING").as_deref() != Ok("YES")
        || unsafe { libc::geteuid() } == 0
    {
        return Err("tenant domain routing requires explicit nonroot opt-in");
    }
    let path = PathBuf::from(
        std::env::var("IPAT_TENANT_DOMAIN_DB_CONNINFO_FILE")
            .map_err(|_| "missing tenant domain DB conninfo file")?,
    );
    let value = read_private_file(&path)?;
    let db = Config::from_str(value.trim()).map_err(|_| "invalid tenant domain DB config")?;
    if !valid_db_config(&db) {
        return Err("tenant domain reader must use the dedicated local Unix-socket role");
    }
    Ok(Arc::new(Store { db }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::Request,
    };
    use tower::ServiceExt;

    fn host(value: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(header::HOST, HeaderValue::from_str(value).unwrap());
        headers
    }

    #[test]
    fn canonicalizes_dns_case_and_numeric_port_only() {
        assert_eq!(
            canonical_hostname(&host("IPAT.FADLY.ID:443")).unwrap(),
            "ipat.fadly.id"
        );
        assert_eq!(
            canonical_hostname(&host("kangnet.ipat.id")).unwrap(),
            "kangnet.ipat.id"
        );
    }
    #[test]
    fn rejects_ip_literals_malformed_labels_and_injection() {
        for value in [
            "127.0.0.1",
            "[::1]",
            "bad",
            "-bad.example",
            "bad-.example",
            "bad..example",
            "bad.example.",
            "bad.example:abc",
            "bad.example/tenant",
            "bad.example tenant",
            "user@bad.example",
        ] {
            assert!(canonical_hostname(&host(value)).is_err(), "{value}");
        }
    }

    #[test]
    fn duplicate_host_is_rejected() {
        let mut headers = HeaderMap::new();
        headers.append(header::HOST, HeaderValue::from_static("ipat.fadly.id"));
        headers.append(header::HOST, HeaderValue::from_static("other.example"));
        assert_eq!(canonical_hostname(&headers), Err(StatusCode::BAD_REQUEST));
    }

    fn a_profile(ready: bool) -> Arc<DnsInstructionProfile> {
        Arc::new(DnsInstructionProfile {
            mode: DnsRoutingMode::ARecord,
            ipv4: Some("203.0.113.20".parse().unwrap()),
            ipv6: None,
            cname_target: None,
            nameservers: Vec::new(),
            auto_allow_cname: false,
            routing_ready: ready,
            authoritative_dns_ready: false,
        })
    }

    #[test]
    fn r953_dns_instruction_builder_is_domain_specific_and_non_authorizing() {
        let response = build_dns_instructions(
            &a_profile(true),
            canonical_requested_domain("Portal.Customer.Co.Id").unwrap(),
        )
        .unwrap();
        assert_eq!(response.hostname, "portal.customer.co.id");
        assert_eq!(response.routing_mode, "a_record");
        assert_eq!(response.routing_records.len(), 1);
        assert_eq!(response.routing_records[0].record_type, "A");
        assert_eq!(response.routing_records[0].value, "203.0.113.20");
        assert_eq!(
            response.verification_record_name,
            "_ipat-verify.portal.customer.co.id"
        );
        assert!(response.routing_target_known);
        assert!(response.routing_ready);
        assert!(response.safe_to_point_now);
        assert!(!response.authorization_granted);
    }

    fn auto_profile() -> DnsInstructionProfile {
        DnsInstructionProfile {
            mode: DnsRoutingMode::Auto,
            ipv4: Some("203.0.113.20".parse().unwrap()),
            ipv6: None,
            cname_target: Some("edge.ipat.id".into()),
            nameservers: vec!["ns1.ipat.id".into(), "ns2.ipat.id".into()],
            auto_allow_cname: true,
            routing_ready: true,
            authoritative_dns_ready: true,
        }
    }

    #[test]
    fn r954_auto_prefers_stable_address_and_reports_customer_action() {
        let profile = auto_profile();
        let response = build_dns_instructions(
            &profile,
            canonical_requested_domain("portal.customer.co.id").unwrap(),
        )
        .unwrap();
        assert_eq!(response.routing_mode, "a_record");
        assert_eq!(response.selection_reason, "AUTO_STABLE_INGRESS_ADDRESS");
        assert_eq!(response.customer_action, "CREATE_ADDRESS_RECORDS");
        assert_eq!(
            response.available_routing_modes,
            vec!["a_record", "nameserver", "cname"]
        );
        assert_eq!(response.routing_records[0].value, "203.0.113.20");
        assert!(response.safe_to_point_now);
    }

    #[test]
    fn r954_auto_prefers_authoritative_ns_when_address_is_unavailable() {
        let mut profile = auto_profile();
        profile.ipv4 = None;
        let response = build_dns_instructions(
            &profile,
            canonical_requested_domain("customer.co.id").unwrap(),
        )
        .unwrap();
        assert_eq!(response.routing_mode, "nameserver");
        assert_eq!(response.selection_reason, "AUTO_AUTHORITATIVE_NAMESERVERS");
        assert_eq!(response.customer_action, "DELEGATE_NAMESERVERS");
        assert_eq!(response.routing_records.len(), 2);
    }

    #[test]
    fn r954_auto_cname_requires_explicit_deployment_opt_in() {
        let mut profile = auto_profile();
        profile.ipv4 = None;
        profile.nameservers.clear();
        let response = build_dns_instructions(
            &profile,
            canonical_requested_domain("portal.customer.co.id").unwrap(),
        )
        .unwrap();
        assert_eq!(response.routing_mode, "cname");
        assert_eq!(response.customer_action, "CREATE_CNAME_RECORD");

        profile.auto_allow_cname = false;
        assert!(build_dns_instructions(
            &profile,
            canonical_requested_domain("portal.customer.co.id").unwrap(),
        )
        .is_none());
    }

    #[test]
    fn r953_nameserver_profile_requires_real_authoritative_readiness_before_ready() {
        let profile = DnsInstructionProfile {
            mode: DnsRoutingMode::Nameserver,
            ipv4: None,
            ipv6: None,
            cname_target: None,
            nameservers: vec!["ns1.ipat.id".into(), "ns2.ipat.id".into()],
            auto_allow_cname: false,
            routing_ready: true,
            authoritative_dns_ready: false,
        };
        let response = build_dns_instructions(
            &profile,
            canonical_requested_domain("customer.co.id").unwrap(),
        )
        .unwrap();
        assert!(response.routing_target_known);
        assert!(response.routing_ready);
        assert!(!response.authoritative_dns_ready);
        assert!(!response.safe_to_point_now);
    }

    #[tokio::test]
    async fn r953_instruction_endpoint_shows_target_but_blocks_pointing_until_ready() {
        let app = instruction_router(a_profile(false));
        let body =
            serde_json::to_vec(&json!({"hostname":"portal.customer.co.id"})).unwrap();
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
        let body = to_bytes(response.into_body(), 16 * 1024).await.unwrap();
        let value: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(value["routing_records"][0]["value"], "203.0.113.20");
        assert_eq!(value["routing_target_known"], true);
        assert_eq!(value["routing_ready"], false);
        assert_eq!(value["safe_to_point_now"], false);
        assert_eq!(value["authorization_granted"], false);
    }

    #[tokio::test]
    async fn r953_instruction_endpoint_returns_configured_record_when_ready() {
        let app = instruction_router(a_profile(true));
        let body =
            serde_json::to_vec(&json!({"hostname":"portal.customer.co.id"})).unwrap();
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
        let body = to_bytes(response.into_body(), 16 * 1024).await.unwrap();
        let value: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(value["routing_mode"], "a_record");
        assert_eq!(value["routing_records"][0]["value"], "203.0.113.20");
        assert_eq!(value["activation_requires_verification"], true);
        assert_eq!(value["routing_target_known"], true);
        assert_eq!(value["routing_ready"], true);
        assert_eq!(value["safe_to_point_now"], true);
        assert_eq!(value["authorization_granted"], false);
    }

    #[test]
    fn direct_tcp_or_unrestricted_database_config_is_not_allowed() {
        for dsn in [
            "host=127.0.0.1 user=ipat_domain_reader dbname=ipat",
            "host=/var/run/postgresql user=postgres dbname=ipat",
            "host=/var/run/postgresql user=ipat_domain_reader",
            "host=/var/run/postgresql user=ipat_domain_reader dbname=ipat options='-c role=postgres'",
        ] {
            let config = Config::from_str(dsn).unwrap();
            assert!(!valid_db_config(&config), "{dsn}");
        }
        let config =
            Config::from_str("host=/var/run/postgresql user=ipat_domain_reader dbname=ipat")
                .unwrap();
        assert!(valid_db_config(&config));
    }

    #[tokio::test]
    async fn r952_real_disposable_postgres_host_to_tenant_context_is_fail_closed() {
        if std::env::var("IPAT_PG_EPHEMERAL_TEST").as_deref() != Ok("1") {
            return;
        }
        assert_eq!(std::env::var("PGHOST").as_deref(), Ok("127.0.0.1"));
        let db = Config::from_str(
            "host=127.0.0.1 port=5432 user=ipat_domain_reader password=local_ci_synthetic_only dbname=ipat_synthetic",
        )
        .unwrap();
        let app = router(Arc::new(Store { db }));

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/v1/tenant-context")
                    .header("Host", "ipat.fadly.id")
                    .header("X-Forwarded-Host", "forged.other.invalid")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 4096).await.unwrap();
        let value: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(value["tenant_context"], true);
        assert_eq!(value["tenant_slug"], "tenant-beta");
        assert_eq!(value["hostname"], "ipat.fadly.id");
        assert_eq!(value["authentication_required"], true);
        assert_eq!(value["business_access_enabled"], false);
        assert!(value.get("tenant_id").is_none());

        let unknown = app
            .oneshot(
                Request::builder()
                    .uri("/v1/tenant-context")
                    .header("Host", "unknown.ipat.id")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(unknown.status(), StatusCode::NOT_FOUND);
    }
}
