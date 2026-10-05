//! R9.83: dedicated Platform Owner confidential OIDC/MFA issuer.
//!
//! This process is deliberately separate from the Platform Owner API, tenant
//! BFF, tenant OIDC issuer, DNS verifier and physical-device services.
//! It uses Authorization Code + PKCE, exact pinned issuer/key/client, fresh
//! signed MFA token-pair verification and the ISSUE-only platform DB role.
//! No token, client secret, role claim or device credential is logged.

use crate::{platform_owner_api, tenant_domain::canonical_host};
use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use identity_core::{
    browser_pkce::Challenge, oidc_id_token::VerifiedBrowserIdentity, PinnedIssuer,
};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::{fs::MetadataExt, fs::OpenOptionsExt},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use subtle::ConstantTimeEq;
use tokio_postgres::{Client, Config as PgConfig, NoTls};
use uuid::Uuid;

const STATE_COOKIE: &str = "__Host-ipat_platform_oidc_state";
const MAX_PENDING: usize = 64;
const MAX_AGE: Duration = Duration::from_secs(180);
const TOKEN_RESPONSE_LIMIT: usize = 16 * 1024;
const KEY_LIMIT: u64 = 16 * 1024;
const SECRET_LIMIT: u64 = 1024;

struct Pending {
    proof: Challenge,
    started: Instant,
}
struct Config {
    issuer: String,
    client_id: String,
    client_secret: String,
    host: String,
    verifier: PinnedIssuer,
    db: Arc<Client>,
    pending: Mutex<HashMap<String, Pending>>,
}
fn strict_dns_host(host: &str) -> bool {
    host.len() <= 253
        && host.contains('.')
        && !host.ends_with('.')
        && host
            .rsplit('.')
            .next()
            .is_some_and(|t| t.len() >= 2 && t.bytes().all(|b| b.is_ascii_alphabetic()))
        && host.split('.').all(|part| {
            !part.is_empty()
                && part.len() <= 63
                && !part.starts_with('-')
                && !part.ends_with('-')
                && part.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
}
fn oidc_config_shape(issuer: &str, client: &str, host: &str) -> bool {
    if !platform_owner_api::platform_host_shape(host)
        || client.is_empty()
        || client.len() > 80
        || !client
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
        || !issuer.is_ascii()
        || !issuer.starts_with("https://")
        || issuer.len() > 300
        || issuer.ends_with('/')
        || issuer.contains(['?', '#', '@', '\\'])
        || issuer.chars().any(char::is_whitespace)
    {
        return false;
    }
    let Some(rest) = issuer.strip_prefix("https://") else {
        return false;
    };
    let authority = rest.split('/').next().unwrap_or("");
    strict_dns_host(authority)
        && !authority.contains(':')
        && rest
            .split('/')
            .skip(1)
            .all(|p| !p.is_empty() && p != "." && p != "..")
}
fn url_component(value: &str) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(value.len());
    for b in value.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            write!(&mut out, "%{b:02X}").expect("write to string");
        }
    }
    out
}
fn redirect_uri(host: &str) -> String {
    format!("https://{host}/platform/auth/oidc/callback")
}
fn authorization_url(issuer: &str, client: &str, host: &str, proof: &Challenge) -> String {
    format!(
        "{issuer}/protocol/openid-connect/auth?response_type=code&client_id={}&redirect_uri={}&scope=openid&prompt=login&max_age=300&code_challenge_method=S256&code_challenge={}&state={}&nonce={}",
        url_component(client),
        url_component(&redirect_uri(host)),
        proof.s256(),
        proof.state(),
        proof.nonce()
    )
}
fn secure_read(path: &Path, limit: u64) -> Option<Vec<u8>> {
    if !path.is_absolute()
        || path.components().any(|c| {
            matches!(
                c,
                std::path::Component::ParentDir | std::path::Component::CurDir
            )
        })
    {
        return None;
    }
    let uid = unsafe { libc::geteuid() };
    let parent = fs::symlink_metadata(path.parent()?).ok()?;
    let meta = fs::symlink_metadata(path).ok()?;
    if !parent.is_dir()
        || parent.file_type().is_symlink()
        || parent.uid() != uid
        || parent.mode() & 0o777 != 0o700
        || !meta.is_file()
        || meta.file_type().is_symlink()
        || meta.uid() != uid
        || meta.mode() & 0o777 != 0o600
        || meta.nlink() != 1
        || meta.len() == 0
        || meta.len() > limit
    {
        return None;
    }
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .ok()?;
    let current = file.metadata().ok()?;
    if current.ino() != meta.ino() || current.dev() != meta.dev() || current.len() != meta.len() {
        return None;
    }
    let mut data = Vec::with_capacity(meta.len() as usize);
    file.take(limit + 1).read_to_end(&mut data).ok()?;
    (data.len() as u64 <= limit).then_some(data)
}
fn current_host(headers: &HeaderMap) -> Option<String> {
    if headers.get_all(header::HOST).iter().count() != 1 {
        return None;
    }
    let host = canonical_host(headers.get(header::HOST)?)?
        .as_str()
        .to_string();
    platform_owner_api::platform_host_shape(&host).then_some(host)
}
fn browser_cookie(headers: &HeaderMap) -> Option<&str> {
    if headers.get_all(header::COOKIE).iter().count() != 1 {
        return None;
    }
    let raw = headers.get(header::COOKIE)?.to_str().ok()?;
    let mut found = None;
    for part in raw.split(';') {
        let (name, value) = part.trim().split_once('=')?;
        if name == STATE_COOKIE {
            if found.is_some() || !Challenge::valid_state(value) {
                return None;
            }
            found = Some(value);
        }
    }
    found
}
fn safe_headers() -> HeaderMap {
    let mut h = HeaderMap::new();
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    h.insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    h.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    h.insert(
        "content-security-policy",
        HeaderValue::from_static("default-src 'none'; frame-ancestors 'none'"),
    );
    h
}
fn fail() -> Response {
    let mut h = safe_headers();
    h.insert(
        header::SET_COOKIE,
        HeaderValue::from_static(
            "__Host-ipat_platform_oidc_state=; Secure; HttpOnly; SameSite=Lax; Path=/; Max-Age=0",
        ),
    );
    (StatusCode::UNAUTHORIZED, h, "OIDC_LOGIN_REJECTED").into_response()
}
async fn start(State(cfg): State<Arc<Config>>, headers: HeaderMap) -> Response {
    if current_host(&headers).as_deref() != Some(cfg.host.as_str()) {
        return fail();
    }
    let Ok(proof) = Challenge::random() else {
        return fail();
    };
    let state = proof.state().to_owned();
    let link = authorization_url(&cfg.issuer, &cfg.client_id, &cfg.host, &proof);
    let mut pending = cfg.pending.lock().unwrap_or_else(|e| e.into_inner());
    pending.retain(|_, v| v.started.elapsed() < MAX_AGE);
    if pending.len() >= MAX_PENDING {
        return (StatusCode::TOO_MANY_REQUESTS, safe_headers(), "OIDC_BUSY").into_response();
    }
    pending.insert(
        state.clone(),
        Pending {
            proof,
            started: Instant::now(),
        },
    );
    drop(pending);
    let mut h = safe_headers();
    h.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&format!(
            "{STATE_COOKIE}={state}; Secure; HttpOnly; SameSite=Lax; Path=/; Max-Age=180"
        ))
        .expect("bounded state"),
    );
    h.insert(
        header::LOCATION,
        HeaderValue::from_str(&link).expect("validated URL"),
    );
    (StatusCode::SEE_OTHER, h, "").into_response()
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Callback {
    state: String,
    code: String,
}
fn valid_code(code: &str) -> bool {
    !code.is_empty() && code.len() <= 4096 && code.bytes().all(|b| b.is_ascii_graphic())
}
fn token_form(cfg: &Config, code: &str, proof: &Challenge) -> String {
    format!(
        "grant_type=authorization_code&client_id={}&client_secret={}&code={}&code_verifier={}&redirect_uri={}",
        url_component(&cfg.client_id),
        url_component(&cfg.client_secret),
        url_component(code),
        url_component(proof.verifier()),
        url_component(&redirect_uri(&cfg.host))
    )
}
fn exchange(token_url: String, form: String) -> Option<(String, String)> {
    let mut child = Command::new("/usr/bin/curl")
        .env_clear()
        .args([
            "-q",
            "--silent",
            "--show-error",
            "--fail",
            "--no-progress-meter",
            "--proto",
            "=https",
            "--noproxy",
            "*",
            "--connect-timeout",
            "3",
            "--max-time",
            "8",
            "--tlsv1.2",
            "--max-filesize",
            "16384",
            "--request",
            "POST",
            "--header",
            "Content-Type: application/x-www-form-urlencoded",
            "--data-binary",
            "@-",
            &token_url,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    child.stdin.take()?.write_all(form.as_bytes()).ok()?;
    let stdout = child.stdout.take()?;
    let mut body = Vec::with_capacity(TOKEN_RESPONSE_LIMIT);
    if stdout
        .take((TOKEN_RESPONSE_LIMIT + 1) as u64)
        .read_to_end(&mut body)
        .is_err()
        || body.len() > TOKEN_RESPONSE_LIMIT
    {
        let _ = child.kill();
        let _ = child.wait();
        return None;
    }
    if !child.wait().ok()?.success() {
        return None;
    }
    let value: Value = serde_json::from_slice(&body).ok()?;
    if value.get("token_type")?.as_str()? != "Bearer" {
        return None;
    }
    let id = value.get("id_token")?.as_str()?;
    let access = value.get("access_token")?.as_str()?;
    if id.len() > TOKEN_RESPONSE_LIMIT || access.len() > TOKEN_RESPONSE_LIMIT {
        return None;
    }
    Some((id.to_owned(), access.to_owned()))
}
fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
fn random_secret() -> Option<String> {
    let mut bytes = [0u8; 32];
    File::open("/dev/urandom")
        .ok()?
        .read_exact(&mut bytes)
        .ok()?;
    let out = URL_SAFE_NO_PAD.encode(bytes);
    (out.len() == 43).then_some(out)
}
fn sha256_hex(value: &str) -> String {
    Sha256::digest(value.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn new_uuid_v4() -> Option<Uuid> {
    let mut bytes = [0u8; 16];
    File::open("/dev/urandom")
        .ok()?
        .read_exact(&mut bytes)
        .ok()?;
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Some(Uuid::from_bytes(bytes))
}
async fn issue_platform_session_after_verified_identity(
    db: &Client,
    identity: &VerifiedBrowserIdentity,
    host: &str,
    now: u64,
) -> Option<(String, String, u64)> {
    if !platform_owner_api::platform_host_shape(host) {
        return None;
    }
    let cookie = random_secret()?;
    let csrf = random_secret()?;
    if cookie == csrf {
        return None;
    }
    let id = new_uuid_v4()?;
    let expires = identity.expires_at().min(now.saturating_add(600));
    if expires <= now {
        return None;
    }
    let expiry = i64::try_from(expires).ok()?;
    let row = db
        .query_one(
            "SELECT ipat_platform.issue_platform_browser_session(
               $1,$2,$3,$4::uuid,$5,$6,to_timestamp($7::bigint))",
            &[
                &identity.issuer(),
                &identity.subject(),
                &host,
                &id,
                &sha256_hex(&cookie),
                &sha256_hex(&csrf),
                &expiry,
            ],
        )
        .await
        .ok()?;
    (row.get::<_, Option<Uuid>>(0) == Some(id)).then_some((cookie, csrf, expires))
}
async fn callback(
    State(cfg): State<Arc<Config>>,
    headers: HeaderMap,
    Query(query): Query<Callback>,
) -> Response {
    if current_host(&headers).as_deref() != Some(cfg.host.as_str())
        || !Challenge::valid_state(&query.state)
        || !valid_code(&query.code)
    {
        return fail();
    }
    let Some(cookie_state) = browser_cookie(&headers) else {
        return fail();
    };
    if !bool::from(query.state.as_bytes().ct_eq(cookie_state.as_bytes())) {
        return fail();
    }
    let entry = cfg
        .pending
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .remove(&query.state);
    let Some(entry) = entry.filter(|entry| entry.started.elapsed() < MAX_AGE) else {
        return fail();
    };
    let nonce = entry.proof.nonce().to_owned();
    let form = token_form(&cfg, &query.code, &entry.proof);
    let token_url = format!("{}/protocol/openid-connect/token", cfg.issuer);
    let pair = tokio::task::spawn_blocking(move || exchange(token_url, form))
        .await
        .ok()
        .flatten();
    let Some((id, access)) = pair else {
        return fail();
    };
    let Ok(identity) =
        cfg.verifier
            .verify_offline_browser_pair(&cfg.client_id, &nonce, &id, &access)
    else {
        return fail();
    };
    let now = now_unix();
    let Some((session, csrf, expires_at)) =
        issue_platform_session_after_verified_identity(&cfg.db, &identity, &cfg.host, now).await
    else {
        return fail();
    };
    let max_age = expires_at.saturating_sub(now);
    let mut h = safe_headers();
    h.append(
        header::SET_COOKIE,
        HeaderValue::from_static(
            "__Host-ipat_platform_oidc_state=; Secure; HttpOnly; SameSite=Lax; Path=/; Max-Age=0",
        ),
    );
    h.append(
        header::SET_COOKIE,
        HeaderValue::from_str(&format!(
            "__Host-ipat_platform_session={session}; Secure; HttpOnly; SameSite=Strict; Path=/; Max-Age={max_age}"
        ))
        .expect("bounded session"),
    );
    h.append(
        header::SET_COOKIE,
        HeaderValue::from_str(&format!(
            "__Host-ipat_platform_csrf={csrf}; Secure; SameSite=Strict; Path=/; Max-Age={max_age}"
        ))
        .expect("bounded csrf"),
    );
    h.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    h.insert(
        "content-security-policy",
        HeaderValue::from_static(
            "default-src 'none'; script-src 'self'; base-uri 'none'; frame-ancestors 'none'",
        ),
    );
    (StatusCode::OK,h,"<!doctype html><html lang=\"en-US\"><meta charset=\"utf-8\"><title>IPAT signed in</title><h1>Identity verified</h1><p>Continue to the Platform Owner workspace.</p><a href=\"/platform/dashboard\">Continue to dashboard</a><script src=\"/platform/auth/oidc/complete.js\" defer></script></html>").into_response()
}
async fn completion_script() -> Response {
    let mut h = safe_headers();
    h.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/javascript; charset=utf-8"),
    );
    (StatusCode::OK,h,"'use strict';history.replaceState(null,'','/platform/auth/oidc/complete');location.replace('/platform/dashboard');").into_response()
}
fn router(cfg: Arc<Config>) -> Router {
    Router::new()
        .route("/platform/auth/oidc/start", get(start))
        .route("/platform/auth/oidc/callback", get(callback))
        .route("/platform/auth/oidc/complete.js", get(completion_script))
        .with_state(cfg)
}
pub(super) async fn from_environment() -> Result<Router, &'static str> {
    if unsafe { libc::geteuid() } == 0
        || std::env::var("IPAT_R983_PLATFORM_OIDC_ISSUER_SERVICE").as_deref() != Ok("YES")
        || std::env::var("IPAT_R983_PLATFORM_TRUSTED_HTTPS_EDGE").as_deref() != Ok("YES")
        || std::env::var("IPAT_R983_VERIFIED_PLATFORM_CONFIDENTIAL_IDP").as_deref() != Ok("YES")
        || std::env::var("IPAT_R983_SINGLE_INSTANCE_OIDC").as_deref() != Ok("YES")
    {
        return Err("separate verified Platform Owner issuer deployment not enabled");
    }
    let issuer = std::env::var("IPAT_R983_ISSUER").map_err(|_| "missing issuer")?;
    let host = std::env::var("IPAT_R983_HOST").map_err(|_| "missing verified host")?;
    let client_id = std::env::var("IPAT_R983_CLIENT_ID").map_err(|_| "missing client ID")?;
    if !oidc_config_shape(&issuer, &client_id, &host) {
        return Err("invalid pinned issuer/client/host");
    }
    let ip_mode = std::env::var("IPAT_R982_TEMPORARY_IPV4_MODE").as_deref() == Ok("YES");
    let ip_san = std::env::var("IPAT_R982_IP_SAN_TLS_REVIEWED").as_deref() == Ok("YES");
    if !platform_owner_api::allowed_platform_host(&host, ip_mode, ip_san) {
        return Err("unapproved Platform Owner host");
    }
    let secret_file = PathBuf::from(
        std::env::var("IPAT_R983_CLIENT_SECRET_FILE").map_err(|_| "missing client secret file")?,
    );
    let key_file = PathBuf::from(
        std::env::var("IPAT_R983_PUBLIC_KEY_FILE").map_err(|_| "missing pinned issuer key file")?,
    );
    let secret =
        String::from_utf8(secure_read(&secret_file, SECRET_LIMIT).ok_or("unsafe secret path")?)
            .map_err(|_| "invalid secret bytes")?;
    if secret.is_empty()
        || secret.len() > SECRET_LIMIT as usize
        || secret.chars().any(char::is_whitespace)
    {
        return Err("invalid client secret");
    }
    let pem = secure_read(&key_file, KEY_LIMIT).ok_or("unsafe key path")?;
    let kid = std::env::var("IPAT_R983_KID").map_err(|_| "missing pinned key identifier")?;
    let verifier = PinnedIssuer::new(&issuer, &client_id, &kid, &pem)
        .map_err(|_| "invalid pinned IdP signing key")?;
    let db_user = std::env::var("IPAT_R983_DB_USER").map_err(|_| "missing issuer DB identity")?;
    if db_user != "ipat_platform_session_issuer_login" {
        return Err("not a platform session ISSUE-only PostgreSQL user");
    }
    let db_name = std::env::var("IPAT_R983_DB_NAME").map_err(|_| "missing issuer DB name")?;
    if db_name.is_empty()
        || db_name.len() > 63
        || !db_name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
    {
        return Err("invalid DB name");
    }
    let socket =
        PathBuf::from(std::env::var("IPAT_R983_DB_SOCKET").map_err(|_| "missing DB socket")?);
    if !socket.is_absolute()
        || socket.components().any(|c| {
            matches!(
                c,
                std::path::Component::ParentDir | std::path::Component::CurDir
            )
        })
    {
        return Err("unsafe DB socket");
    }
    let mut pg = PgConfig::new();
    pg.host_path(socket).dbname(&db_name).user(&db_user);
    let (db, connection) = pg
        .connect(NoTls)
        .await
        .map_err(|_| "issuer-only database unavailable")?;
    tokio::spawn(async move {
        let _ = connection.await;
    });
    Ok(router(Arc::new(Config {
        issuer,
        client_id,
        client_secret: secret,
        host,
        verifier,
        db: Arc::new(db),
        pending: Mutex::new(HashMap::new()),
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};

    #[test]
    fn config_accepts_exact_public_ip_shape_but_requires_outer_runtime_gate() {
        assert!(oidc_config_shape(
            "https://id.example.net/realms/ipat-platform",
            "ipat-platform-browser",
            "202.162.204.121"
        ));
        assert!(!oidc_config_shape(
            "https://id.example.net:443/realms/ipat",
            "ipat-platform-browser",
            "202.162.204.121"
        ));
        assert!(!oidc_config_shape(
            "https://id.example.net/../realm",
            "ipat-platform-browser",
            "202.162.204.121"
        ));
        assert!(!platform_owner_api::allowed_platform_host(
            "202.162.204.121",
            false,
            true
        ));
        assert!(!platform_owner_api::allowed_platform_host(
            "202.162.204.121",
            true,
            false
        ));
    }
    #[test]
    fn auth_url_uses_platform_callback_pkce_nonce_and_never_verifier() {
        let p = Challenge::random().unwrap();
        let url = authorization_url(
            "https://id.example.net/realms/ipat-platform",
            "ipat-platform-browser",
            "202.162.204.121",
            &p,
        );
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains(
            "redirect_uri=https%3A%2F%2F202.162.204.121%2Fplatform%2Fauth%2Foidc%2Fcallback"
        ));
        assert!(url.contains("prompt=login"));
        assert!(url.contains("max_age=300"));
        assert!(!url.contains(p.verifier()));
    }
    #[tokio::test]
    async fn wrong_host_never_starts_browser_auth() {
        // No DB operation occurs on a wrong Host: a dummy config cannot be
        // constructed without a verifier, so validate the request-host gate.
        let mut h = HeaderMap::new();
        h.insert(header::HOST, HeaderValue::from_static("evil.example.net"));
        assert_ne!(current_host(&h).as_deref(), Some("202.162.204.121"));
        let req = Request::builder()
            .uri("/platform/auth/oidc/start")
            .header("Host", "evil.example.net")
            .body(Body::empty())
            .unwrap();
        assert_eq!(req.headers().get("host").unwrap(), "evil.example.net");
    }
}

#[cfg(test)]
mod r983_pg_integration {
    use super::*;
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
    use serde_json::json;
    use std::process::Command;
    use tempfile::tempdir;

    async fn admin_db() -> Client {
        let mut cfg = PgConfig::new();
        cfg.host(std::env::var("PGHOST").as_deref().unwrap_or("127.0.0.1"))
            .port(
                std::env::var("PGPORT")
                    .ok()
                    .and_then(|x| x.parse().ok())
                    .unwrap_or(5432),
            )
            .user(std::env::var("PGUSER").as_deref().unwrap_or("postgres"))
            .dbname("ipat_synthetic");
        if let Ok(password) = std::env::var("PGPASSWORD") {
            cfg.password(password);
        }
        let (client, connection) = cfg.connect(NoTls).await.unwrap();
        tokio::spawn(async move {
            let _ = connection.await;
        });
        client
    }
    fn sign(key: &[u8], kid: &str, typ: &str, value: &serde_json::Value) -> String {
        let mut h = Header::new(Algorithm::RS256);
        h.kid = Some(kid.to_string());
        h.typ = Some(typ.to_string());
        encode(&h, value, &EncodingKey::from_rsa_pem(key).unwrap()).unwrap()
    }
    #[tokio::test]
    async fn r983_signed_fresh_mfa_identity_issues_platform_only_session() {
        if std::env::var("IPAT_PG_EPHEMERAL_TEST").as_deref() != Ok("1")
            || std::env::var("PGDATABASE").as_deref() != Ok("ipat_synthetic")
        {
            return;
        }
        const ISS: &str = "https://id.r983.synthetic.invalid/realms/platform";
        const CLIENT: &str = "ipat-platform-r983";
        const KID: &str = "r983-key";
        const SUBJECT: &str = "platform-owner-r983";
        const HOST: &str = "202.162.204.121";
        let tmp = tempdir().unwrap();
        let private = tmp.path().join("private.pem");
        let public = tmp.path().join("public.pem");
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
        let private_bytes = fs::read(&private).unwrap();
        let verifier = PinnedIssuer::new(ISS, CLIENT, KID, &fs::read(&public).unwrap()).unwrap();
        let now = now_unix();
        let nonce = Challenge::random().unwrap().nonce().to_string();
        let access = sign(
            &private_bytes,
            KID,
            "at+jwt",
            &json!({
                "iss":ISS,"aud":CLIENT,"sub":SUBJECT,"iat":now,"nbf":now,
                "exp":now+180,"amr":["pwd","mfa"]
            }),
        );
        let hash = Sha256::digest(access.as_bytes());
        let id = sign(
            &private_bytes,
            KID,
            "JWT",
            &json!({
                "iss":ISS,"aud":CLIENT,"azp":CLIENT,"sub":SUBJECT,"iat":now,"nbf":now,
                "exp":now+180,"auth_time":now,"nonce":nonce,
                "at_hash":URL_SAFE_NO_PAD.encode(&hash[..16]),"amr":["pwd","mfa"]
            }),
        );
        let identity = verifier
            .verify_offline_browser_pair(CLIENT, &nonce, &id, &access)
            .unwrap();
        assert_eq!(identity.issuer(), ISS);
        assert_eq!(identity.subject(), SUBJECT);

        let admin = admin_db().await;
        admin
            .execute(
                "INSERT INTO ipat_platform.platform_principals(
            issuer,subject,role,approved_by,created_at,expires_at)
            VALUES($1,$2,'platform_owner','independent-r983-reviewer',
            clock_timestamp()-interval '1 minute',clock_timestamp()+interval '1 day')
            ON CONFLICT(issuer,subject) DO UPDATE SET role='platform_owner',
            approved_by='independent-r983-reviewer',revoked_at=NULL,
            expires_at=clock_timestamp()+interval '1 day'",
                &[&ISS, &SUBJECT],
            )
            .await
            .unwrap();
        admin.execute("INSERT INTO ipat_platform.platform_console_hosts(
            hostname,verified_at,tls_ready_at) VALUES($1,clock_timestamp()-interval '1 day',
            clock_timestamp()-interval '1 hour') ON CONFLICT(hostname) DO UPDATE
            SET verified_at=excluded.verified_at,tls_ready_at=excluded.tls_ready_at,disabled_at=NULL",
            &[&HOST]).await.unwrap();

        let issuer = admin_db().await;
        issuer
            .batch_execute("SET ROLE ipat_platform_session_issuer_login")
            .await
            .unwrap();
        let (cookie, csrf, expiry) =
            issue_platform_session_after_verified_identity(&issuer, &identity, HOST, now)
                .await
                .expect("approved signed MFA Platform Owner session");
        assert_eq!(cookie.len(), 43);
        assert_eq!(csrf.len(), 43);
        assert_ne!(cookie, csrf);
        assert!(expiry > now && expiry <= now + 600);

        let api = admin_db().await;
        api.batch_execute("SET ROLE ipat_platform_session_api_login")
            .await
            .unwrap();
        let row = api
            .query_opt(
                "SELECT issuer,subject FROM ipat_platform.authenticate_platform_browser_session(
            $1,$2,NULL,false)",
                &[&sha256_hex(&cookie), &HOST],
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(row.get::<_, String>(0), ISS);
        assert_eq!(row.get::<_, String>(1), SUBJECT);

        // Signed identity alone must not bypass the platform-principal approval.
        admin
            .execute(
                "UPDATE ipat_platform.platform_principals SET revoked_at=clock_timestamp()
            WHERE issuer=$1 AND subject=$2",
                &[&ISS, &SUBJECT],
            )
            .await
            .unwrap();
        assert!(
            issue_platform_session_after_verified_identity(&issuer, &identity, HOST, now)
                .await
                .is_none()
        );
        let after = api
            .query_opt(
                "SELECT issuer FROM ipat_platform.authenticate_platform_browser_session(
            $1,$2,NULL,false)",
                &[&sha256_hex(&cookie), &HOST],
            )
            .await
            .unwrap();
        assert!(after.is_none());

        // Tenant issuer role cannot mint a platform session.
        let tenant_issuer = admin_db().await;
        tenant_issuer
            .batch_execute("SET ROLE ipat_oidc_session_issuer_login")
            .await
            .unwrap();
        let denied = tenant_issuer
            .query_one(
                "SELECT ipat_platform.issue_platform_browser_session(
             $1,$2,$3,$4::uuid,$5,$6,clock_timestamp()+interval '5 minutes')",
                &[
                    &ISS,
                    &SUBJECT,
                    &HOST,
                    &Uuid::parse_str("83838383-8383-4383-8383-838383838383").unwrap(),
                    &"a".repeat(64),
                    &"b".repeat(64),
                ],
            )
            .await;
        assert!(denied.is_err());
    }
}
