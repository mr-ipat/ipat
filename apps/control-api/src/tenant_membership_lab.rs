//! R7.8 PRIVATE, EXPLICIT-OPT-IN, read-only JWT->restricted DB->menu proof.
//! Never exposes real business records, creates membership or asserts MFA.
//! The ONLY runtime DB connection uses operator-owned private config and
//! PostgreSQL Unix socket, and must use a separate minimal reader role.
use authz_core::dashboard::{DashboardRole, DashboardSection};
use authz_core::verified_menu::{visible_for_verified_candidate, CandidateMembershipRow};
use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    routing::get,
    Json, Router,
};
use identity_core::{PinnedIssuer, VerifiedSubject};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    fs::OpenOptions,
    io::Read,
    os::unix::{fs::MetadataExt, fs::OpenOptionsExt},
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tenant_core::TenantId;
use tokio_postgres::{config::Host, Config, NoTls};
use uuid::Uuid;

const EXPECTED_DB_READER: &str = "ipat_lab_identity_reader";
const MAX_CONNFILE_BYTES: u64 = 4096;

pub(super) struct Store {
    verifier: Arc<PinnedIssuer>,
    db: Config,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ScopeRequest {
    tenant_id: String,
    role: String,
    pop_id: Option<String>,
}
fn no_store() -> HeaderMap {
    let mut h = HeaderMap::new();
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    h.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    h
}
fn reject(status: StatusCode) -> (StatusCode, HeaderMap, Json<Value>) {
    (status, no_store(), Json(json!({"access":false})))
}
fn role_from_exact(value: &str) -> Option<DashboardRole> {
    match value {
        "tenant_admin" => Some(DashboardRole::TenantAdmin),
        "noc_engineer" => Some(DashboardRole::NocEngineer),
        "helpdesk" => Some(DashboardRole::Helpdesk),
        "auditor" => Some(DashboardRole::Auditor),
        _ => None,
    }
}
fn valid_pop(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
}
fn verified_bearer<'a>(verifier: &PinnedIssuer, headers: &'a HeaderMap) -> Option<VerifiedSubject> {
    if headers.get_all(header::AUTHORIZATION).iter().count() != 1 {
        return None;
    }
    let token = headers
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")?;
    if token.contains([' ', '\t', ',']) {
        return None;
    }
    verifier.verify_access_token(token).ok()
}
fn now() -> Option<u64> {
    Some(SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs())
}
async fn sections(
    State(store): State<Arc<Store>>,
    headers: HeaderMap,
    Query(query): Query<ScopeRequest>,
) -> (StatusCode, HeaderMap, Json<Value>) {
    let Some(subject) = verified_bearer(&store.verifier, &headers) else {
        return reject(StatusCode::UNAUTHORIZED);
    };
    // Source of requested scope is untrusted; database must approve the exact
    // token identity and exact requested tenant, role and POP EVERY request.
    let Some(role) = role_from_exact(&query.role) else {
        return reject(StatusCode::FORBIDDEN);
    };
    let Ok(uuid) = Uuid::parse_str(&query.tenant_id) else {
        return reject(StatusCode::BAD_REQUEST);
    };
    if uuid.hyphenated().to_string() != query.tenant_id {
        return reject(StatusCode::BAD_REQUEST);
    }
    if query.pop_id.as_deref().is_some_and(|p| !valid_pop(p)) {
        return reject(StatusCode::BAD_REQUEST);
    }
    let pop = query.pop_id.as_deref();
    if (role == DashboardRole::TenantAdmin && pop.is_some())
        || (role != DashboardRole::TenantAdmin && pop.is_none())
    {
        return reject(StatusCode::FORBIDDEN);
    }
    let Ok((client, connection)) = store.db.connect(NoTls).await else {
        return reject(StatusCode::SERVICE_UNAVAILABLE);
    };
    let conn_task = tokio::spawn(async move {
        // No DB errors or identity information should enter HTTP/log output.
        let _ = connection.await;
    });
    // Never SET ROLE or SET LOCAL from HTTP claims: this dedicated service
    // credential may only EXECUTE a narrow static function on the DB server.
    let row = client
        .query_opt(
            "SELECT approved_by,EXTRACT(EPOCH FROM expires_at)::bigint,tenant_slug \
         FROM ipat_platform.lookup_active_membership($1,$2,$3::uuid,$4,$5)",
            &[
                &subject.issuer(),
                &subject.subject(),
                &uuid,
                &query.role,
                &pop,
            ],
        )
        .await;
    drop(client);
    conn_task.abort();
    let Ok(Some(row)) = row else {
        return match row {
            Ok(None) => reject(StatusCode::FORBIDDEN),
            _ => reject(StatusCode::SERVICE_UNAVAILABLE),
        };
    };
    let approved_by: String = row.get(0);
    let expires: i64 = row.get(1);
    let slug: String = row.get(2);
    let (Ok(tenant), Ok(exp)) = (TenantId::parse(&slug), u64::try_from(expires)) else {
        return reject(StatusCode::SERVICE_UNAVAILABLE);
    };
    let allowed_pop = pop.into_iter().collect::<Vec<_>>();
    let record = CandidateMembershipRow {
        issuer: subject.issuer(),
        subject: subject.subject(),
        tenant: &tenant,
        role,
        approved_by: &approved_by,
        authorized_pops: &allowed_pop,
        expires_at: exp,
        revoked: false,
    };
    let Some(current) = now() else {
        return reject(StatusCode::SERVICE_UNAVAILABLE);
    };
    let visible = visible_for_verified_candidate(&subject, &tenant, pop, &record, current);
    // An expired JWT or race with revocation/expiry must never produce
    // a positive result: lookups execute once per request with no cache.
    if visible.is_empty() {
        return reject(StatusCode::FORBIDDEN);
    }
    let names: Vec<String> = visible
        .iter()
        .map(|v: &DashboardSection| format!("{v:?}"))
        .collect();
    (
        StatusCode::OK,
        no_store(),
        Json(json!({
            "jwt_signature_verified":true,
            "db_membership_candidate":true,
            "lab_only":true,
            "mfa_verified":false,
            "real_business_access_enabled":false,
            "tenant_slug":tenant.as_str(),
            "sections":names
        })),
    )
}
pub(super) fn router(store: Arc<Store>) -> Router {
    Router::new()
        .route("/lab/auth/sections", get(sections))
        .with_state(store)
}
fn read_owner_file(path: &Path) -> Result<String, &'static str> {
    let parent = path.parent().ok_or("missing private directory")?;
    if !path.is_absolute()
        || path.components().any(|p| {
            matches!(
                p,
                std::path::Component::ParentDir | std::path::Component::CurDir
            )
        })
    {
        return Err("invalid owner config path");
    }
    for (point, required) in [(parent, 0o700), (path, 0o600)] {
        let m = std::fs::symlink_metadata(point).map_err(|_| "missing private config")?;
        if m.file_type().is_symlink()
            || m.uid() != unsafe { libc::geteuid() }
            || m.mode() & 0o777 != required
        {
            return Err("unsafe config ownership/mode");
        }
    }
    let f = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .map_err(|_| "invalid owner config")?;
    let m = f.metadata().map_err(|_| "invalid owner config")?;
    if !m.is_file()
        || m.uid() != unsafe { libc::geteuid() }
        || m.nlink() != 1
        || m.mode() & 0o777 != 0o600
        || m.len() == 0
        || m.len() > MAX_CONNFILE_BYTES
    {
        return Err("unsafe config file");
    }
    let mut content = String::new();
    f.take(MAX_CONNFILE_BYTES + 1)
        .read_to_string(&mut content)
        .map_err(|_| "bad config encoding")?;
    if content.len() as u64 > MAX_CONNFILE_BYTES {
        return Err("oversized config");
    }
    Ok(content)
}
fn valid_private_db_config(config: &Config) -> bool {
    config.get_user() == Some(EXPECTED_DB_READER)
        && config.get_dbname().is_some()
        && config.get_hosts().len() == 1
        && config.get_hostaddrs().is_empty()
        && config.get_options().is_none()
        && matches!(config.get_hosts()[0], Host::Unix(ref path) if path.is_absolute())
}

pub(super) fn from_owner_environment(
    verifier: Arc<PinnedIssuer>,
) -> Result<Arc<Store>, &'static str> {
    if std::env::var("IPAT_LAB_SCOPED_MEMBERSHIP").as_deref() != Ok("YES")
        || std::env::var("IPAT_LAB_OIDC_VERIFY").as_deref() != Ok("YES")
        || unsafe { libc::geteuid() } == 0
    {
        return Err("requires explicit nonroot OIDC and membership lab opt-in");
    }
    let file = PathBuf::from(
        std::env::var("IPAT_LAB_DB_CONNINFO_FILE")
            .map_err(|_| "missing owner private conninfo path")?,
    );
    let secret = read_owner_file(&file)?;
    let config = Config::from_str(secret.trim())
        .map_err(|_| "invalid private database connection configuration")?;
    if !valid_private_db_config(&config) {
        return Err("only a dedicated reader over a private Unix socket is allowed");
    }
    Ok(Arc::new(Store {
        verifier,
        db: config,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::Request,
    };
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
    use serde_json::json;
    use std::{fs, process::Command};
    use tower::ServiceExt;
    fn synthetic_token() -> (Arc<PinnedIssuer>, String, tempfile::TempDir) {
        let tmp = tempfile::tempdir().unwrap();
        let private = tmp.path().join("only-test.key");
        let public = tmp.path().join("only-test.pub");
        assert!(Command::new("openssl")
            .args([
                "genpkey",
                "-algorithm",
                "RSA",
                "-pkeyopt",
                "rsa_keygen_bits:2048",
                "-out"
            ])
            .arg(&private)
            .status()
            .unwrap()
            .success());
        assert!(Command::new("openssl")
            .args(["pkey", "-pubout", "-in"])
            .arg(&private)
            .arg("-out")
            .arg(&public)
            .status()
            .unwrap()
            .success());
        let verifier = Arc::new(
            PinnedIssuer::new(
                "https://id.example.invalid/realms/lab",
                "ipat-control-api",
                "private-lab",
                &fs::read(public).unwrap(),
            )
            .unwrap(),
        );
        let clock = now().unwrap();
        let claims = json!({
            "iss":"https://id.example.invalid/realms/lab",
            "aud":"ipat-control-api","sub":"synthetic-operator",
            "iat":clock,"nbf":clock,"exp":clock+300,
            "roles":["platform_owner","super_admin"],
            "tenant_id":"forged-synthetic-other",
            "custom_domain":"ipat.fadly.id"
        });
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some("private-lab".to_owned());
        header.typ = Some("JWT".to_owned());
        let signed = encode(
            &header,
            &claims,
            &EncodingKey::from_rsa_pem(&fs::read(private).unwrap()).unwrap(),
        )
        .unwrap();
        (verifier, format!("Bearer {signed}"), tmp)
    }
    async fn call(
        router: Router,
        path: &str,
        auth: Option<&str>,
        method: &str,
    ) -> axum::response::Response {
        let mut req = Request::builder()
            .uri(path)
            .method(method)
            .header("Host", "ipat.fadly.id")
            .header("X-Tenant-Id", "forged-synthetic-other")
            .header("X-Verified-Role", "platform_owner");
        if let Some(value) = auth {
            req = req.header(header::AUTHORIZATION, value);
        }
        router
            .oneshot(req.body(Body::empty()).unwrap())
            .await
            .unwrap()
    }
    #[test]
    fn pop_and_role_are_exact_allowlist_not_jwt_claims() {
        assert_eq!(role_from_exact("platform_owner"), None);
        assert_eq!(role_from_exact("super_admin"), None);
        assert_eq!(
            role_from_exact("noc_engineer"),
            Some(DashboardRole::NocEngineer)
        );
        assert!(valid_pop("pop-a"));
        for pop in ["", "a/b", "../a", "bad pop", "a%2fb"] {
            assert!(!valid_pop(pop));
        }
    }
    #[test]
    fn blocks_hostaddr_network_override_and_startup_role_options() {
        let good = Config::from_str(
            "host=/var/run/postgresql user=ipat_lab_identity_reader dbname=ipat_synthetic",
        )
        .unwrap();
        assert!(valid_private_db_config(&good));
        for evil in [
            "host=127.0.0.1 user=ipat_lab_identity_reader dbname=ipat_synthetic",
            "host=relative user=ipat_lab_identity_reader dbname=ipat_synthetic",
            "host=/var/run/postgresql hostaddr=127.0.0.1 user=ipat_lab_identity_reader dbname=ipat_synthetic",
            "host=/var/run/postgresql user=postgres dbname=ipat_synthetic",
            "host=/var/run/postgresql user=ipat_lab_identity_reader",
            "host=/var/run/postgresql user=ipat_lab_identity_reader dbname=ipat_synthetic options='-c role=postgres'",
        ] {
            let cfg=Config::from_str(evil).expect("synthetic config syntax");
            assert!(!valid_private_db_config(&cfg),"unsafe database transport/user/options accepted");
        }
    }
    #[tokio::test]
    async fn r78_end_to_end_real_signed_jwt_real_restricted_sql_real_axum_router() {
        // No local Postgres, no implicit run against live VPS. CI explicitly
        // constructs disposable PG fixture AFTER 0001/0002/0003/0004.
        if std::env::var("IPAT_PG_EPHEMERAL_TEST").as_deref() != Ok("1") {
            return;
        }
        assert_eq!(std::env::var("PGHOST").unwrap(), "127.0.0.1");
        assert_eq!(std::env::var("PGDATABASE").unwrap(), "ipat_synthetic");
        assert_eq!(
            std::env::var("IPAT_PG_SYNTHETIC_PASSWORD").unwrap(),
            "local_ci_synthetic_only"
        );
        let (verifier, bearer, _ephemeral_private) = synthetic_token();
        // Synthetic-only localhost CI account; never store a live connection
        // string in Git or allow this test constructor in production.
        let db = Config::from_str(
            "host=127.0.0.1 port=5432 user=ipat_lab_identity_reader \
             password=local_ci_synthetic_only dbname=ipat_synthetic",
        )
        .unwrap();
        let store = Arc::new(Store {
            verifier: verifier.clone(),
            db,
        });
        let router = crate::app_with_lab_identity_and_store(true, Some(verifier), Some(store));
        let a = "11111111-1111-4111-8111-111111111111";
        let b = "22222222-2222-4222-8222-222222222222";
        let good = format!("/lab/auth/sections?tenant_id={a}&role=noc_engineer&pop_id=pop-a");
        let positive = call(router.clone(), &good, Some(&bearer), "GET").await;
        assert_eq!(positive.status(), StatusCode::OK);
        assert_eq!(positive.headers()[header::CACHE_CONTROL], "no-store");
        let body = to_bytes(positive.into_body(), 4096).await.unwrap();
        let output: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(output["tenant_slug"], "tenant-alpha");
        assert_eq!(output["mfa_verified"], false);
        assert_eq!(output["real_business_access_enabled"], false);
        assert_eq!(output["db_membership_candidate"], true);
        assert!(output["sections"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s == "OperationsInventory"));
        assert!(!output["sections"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s == "PlatformTenantCatalog" || s == "BulkPppoeWrite"));
        let other = format!("/lab/auth/sections?tenant_id={b}&role=helpdesk&pop_id=pop-b");
        let other_valid = call(router.clone(), &other, Some(&bearer), "GET").await;
        assert_eq!(other_valid.status(), StatusCode::OK);
        for bad in [
            format!("/lab/auth/sections?tenant_id={b}&role=noc_engineer&pop_id=pop-a"),
            format!("/lab/auth/sections?tenant_id={a}&role=helpdesk&pop_id=pop-b"),
            format!("/lab/auth/sections?tenant_id={a}&role=platform_owner&pop_id=pop-a"),
            format!("/lab/auth/sections?tenant_id={a}&role=noc_engineer&pop_id=pop-b"),
            format!("/lab/auth/sections?tenant_id={a}&role=noc_engineer"),
            format!("/lab/auth/sections?tenant_id={a}&role=tenant_admin&pop_id=pop-a"),
        ] {
            assert_eq!(
                call(router.clone(), &bad, Some(&bearer), "GET")
                    .await
                    .status(),
                StatusCode::FORBIDDEN,
                "{bad}"
            );
        }
        assert_eq!(
            call(router.clone(), &good, None, "GET").await.status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            call(router.clone(), &good, Some("Bearer fake"), "GET")
                .await
                .status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            call(router.clone(), &good, Some(&bearer), "POST")
                .await
                .status(),
            StatusCode::METHOD_NOT_ALLOWED
        );
        for path in [
            "/v1/platform/tenants",
            "/v1/tenant/members",
            "/v1/operations/devices",
            "/v1/devices/111",
        ] {
            assert_eq!(
                call(router.clone(), path, Some(&bearer), "GET")
                    .await
                    .status(),
                StatusCode::UNAUTHORIZED,
                "{path}"
            );
        }
        assert_eq!(
            call(crate::app_with_lab(true), &good, Some(&bearer), "GET")
                .await
                .status(),
            StatusCode::NOT_FOUND
        );
    }
}
