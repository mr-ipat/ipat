//! R9.70 confidential Keycloak OIDC authorization-code issuer.
//!
//! NOT automatically Internet-facing. A separate nonroot singleton listener
//! must sit behind a separately approved trusted HTTPS edge. In-memory PKCE
//! pending proofs deliberately REFUSE multi-replica deployment; the durable
//! browser session itself is in PostgreSQL via a dedicated ISSUE-only role.
//! No physical/tenant business routes or access-token debug logging here.
use crate::{durable_tenant_session, tenant_domain::canonical_host};
use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use identity_core::{browser_pkce::Challenge, PinnedIssuer};
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

const STATE_COOKIE: &str = "__Host-ipat_oidc_state";
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
    durable_pending: bool,
}
fn strict_host(host: &str) -> bool {
    host.len() <= 253
        && host.contains('.')
        && !host.ends_with('.')
        && host
            .rsplit('.')
            .next()
            .is_some_and(|tld| tld.len() >= 2 && tld.bytes().all(|b| b.is_ascii_alphabetic()))
        && host.split('.').all(|part| {
            !part.is_empty()
                && part.len() <= 63
                && !part.starts_with('-')
                && !part.ends_with('-')
                && part.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
}
fn oidc_config_shape(issuer: &str, client: &str, host: &str) -> bool {
    if !strict_host(host)
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
    // No runtime issuer discovery/redirects; exact pre-reviewed HTTPS issuer.
    strict_host(authority)
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
            out.push(b as char)
        } else {
            write!(&mut out, "%{b:02X}").expect("write to string")
        }
    }
    out
}
fn redirect_uri(host: &str) -> String {
    format!("https://{host}/auth/oidc/callback")
}
fn authorization_url(issuer: &str, client: &str, host: &str, proof: &Challenge) -> String {
    format!("{issuer}/protocol/openid-connect/auth?response_type=code&client_id={}&redirect_uri={}&scope=openid&prompt=login&max_age=300&code_challenge_method=S256&code_challenge={}&state={}&nonce={}",
        url_component(client),url_component(&redirect_uri(host)),proof.s256(),proof.state(),proof.nonce())
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
    strict_host(&host).then_some(host)
}
fn valid_browser_state_cookie(value: &str) -> bool {
    if Challenge::valid_state(value) {
        return true;
    }
    value.split_once('.').is_some_and(|(state, verifier)| {
        Challenge::valid_state(state) && Challenge::valid_state(verifier)
    })
}
fn state_digest(state: &str) -> String {
    Sha256::digest(state.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn browser_cookie(headers: &HeaderMap) -> Option<&str> {
    if headers.get_all(header::COOKIE).iter().count() != 1 {
        return None;
    }
    let header = headers.get(header::COOKIE)?.to_str().ok()?;
    let mut found = None;
    for part in header.split(';') {
        let Some((name, value)) = part.trim().split_once('=') else {
            return None;
        };
        if name == STATE_COOKIE {
            if found.is_some() || !valid_browser_state_cookie(value) {
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
            "__Host-ipat_oidc_state=; Secure; HttpOnly; SameSite=Lax; Path=/; Max-Age=0",
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
    let cookie = if cfg.durable_pending {
        // PKCE verifier lives ONLY in the browser HttpOnly/Lax cookie. The
        // shared DB stores its S256 hash, state SHA256 and nonsecret nonce.
        // Any future replica can reconstruct proof AFTER atomic consumption.
        let digest = state_digest(&state);
        let result = cfg
            .db
            .query_one(
                "SELECT ipat_platform.begin_oidc_pending($1,$2,$3,$4)",
                &[&digest, &proof.s256(), &proof.nonce(), &cfg.host],
            )
            .await;
        if !result.ok().is_some_and(|row| row.get::<_, bool>(0)) {
            return (StatusCode::TOO_MANY_REQUESTS, safe_headers(), "OIDC_BUSY").into_response();
        }
        format!("{state}.{}", proof.verifier())
    } else {
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
        state.clone()
    };
    let mut h = safe_headers();
    // Lax (not Strict): IdP→app top-level callback is a cross-site GET.
    h.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&format!(
            "{STATE_COOKIE}={cookie}; Secure; HttpOnly; SameSite=Lax; Path=/; Max-Age=180"
        ))
        .expect("bounded state"),
    );
    h.insert(
        header::LOCATION,
        HeaderValue::from_str(&link).expect("validated issuer and url component"),
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
    format!("grant_type=authorization_code&client_id={}&client_secret={}&code={}&code_verifier={}&redirect_uri={}",
        url_component(&cfg.client_id),url_component(&cfg.client_secret),url_component(code),
        url_component(proof.verifier()),url_component(&redirect_uri(&cfg.host)))
}
fn exchange(token_url: String, form: String) -> Option<(String, String)> {
    // The code, PKCE verifier and confidential client secret are ONLY piped to
    // curl stdin; none appears in argv, environment, logs or a temp file.
    // curl defaults to NO redirects; fixed HTTPS endpoint + system CA verify.
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
    let mut stdin = child.stdin.take()?;
    if stdin.write_all(form.as_bytes()).is_err() {
        let _ = child.kill();
        let _ = child.wait();
        return None;
    }
    drop(stdin);
    // Do NOT wait_with_output(): chunked/malicious IdP responses may omit
    // Content-Length, and curl's max-filesize is not a reliable heap bound.
    // Read at most 16KiB + 1 byte; terminate/reap any oversized child.
    let Some(stdout) = child.stdout.take() else {
        let _ = child.kill();
        let _ = child.wait();
        return None;
    };
    let mut body = Vec::with_capacity(TOKEN_RESPONSE_LIMIT);
    let read = stdout
        .take((TOKEN_RESPONSE_LIMIT + 1) as u64)
        .read_to_end(&mut body);
    if read.is_err() || body.len() > TOKEN_RESPONSE_LIMIT {
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
    if id.len() > 16 * 1024 || access.len() > 16 * 1024 {
        return None;
    }
    Some((id.to_owned(), access.to_owned()))
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
    let Some(cookie) = browser_cookie(&headers) else {
        return fail();
    };
    let proof = if cfg.durable_pending {
        // The untrusted cookie cannot forge a verifier because a digest of
        // the ORIGINAL server-generated verifier was inserted in PostgreSQL.
        let Some((state, verifier)) = cookie.split_once('.') else {
            return fail();
        };
        if !Challenge::valid_state(state)
            || !Challenge::valid_state(verifier)
            || !bool::from(query.state.as_bytes().ct_eq(state.as_bytes()))
        {
            return fail();
        }
        let result = cfg
            .db
            .query_opt(
                "SELECT nonce FROM ipat_platform.consume_oidc_pending($1,$2,$3)",
                &[
                    &state_digest(state),
                    &format!("{}", {
                        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
                        URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
                    }),
                    &cfg.host,
                ],
            )
            .await;
        let Some(nonce) = result.ok().flatten().map(|row| row.get::<_, String>(0)) else {
            return fail();
        };
        let Some(proof) = Challenge::reconstruct_after_consumption(state, &nonce, verifier) else {
            return fail();
        };
        proof
    } else {
        if !bool::from(query.state.as_bytes().ct_eq(cookie.as_bytes())) {
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
        entry.proof
    };
    let nonce = proof.nonce().to_owned();
    let form = token_form(&cfg, &query.code, &proof);
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
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let Some(session) =
        durable_tenant_session::issue_after_verified_identity(&cfg.db, &identity, &cfg.host, now)
            .await
    else {
        return fail();
    };
    let mut h = safe_headers();
    h.append(
        header::SET_COOKIE,
        HeaderValue::from_static(
            "__Host-ipat_oidc_state=; Secure; HttpOnly; SameSite=Lax; Path=/; Max-Age=0",
        ),
    );
    h.append(
        header::SET_COOKIE,
        HeaderValue::from_str(&session.secure_cookie_header()).expect("bounded secure cookie"),
    );
    // JS reads only the CSRF cookie. Session cookie is HttpOnly and never
    // returned in a response body, URL, browser-visible storage or logs.
    h.append(
        header::SET_COOKIE,
        HeaderValue::from_str(&format!(
            "__Host-ipat_csrf={}; Secure; SameSite=Strict; Path=/; Max-Age={}",
            session.csrf_secret(),
            session.expires_at().saturating_sub(now)
        ))
        .expect("bounded csrf"),
    );
    // Avoid a 303 immediately after a cross-site IdP callback: browsers may
    // withhold newly-set SameSite=Strict cookies for the entire redirect
    // chain. Land on THIS origin first; same-origin script starts navigation.
    // The fallback link works with scripts disabled. No token is exposed.
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
    (StatusCode::OK,h,"<!doctype html><html lang=\"en-US\"><meta charset=\"utf-8\"><title>IPAT signed in</title><h1>Identity verified</h1><p>Continue to your company workspace.</p><a href=\"/dashboard\">Continue to dashboard</a><script src=\"/auth/oidc/complete.js\" defer></script></html>").into_response()
}
async fn completion_script() -> Response {
    let mut h = safe_headers();
    h.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/javascript; charset=utf-8"),
    );
    (StatusCode::OK,h,"'use strict';history.replaceState(null,'','/auth/oidc/complete');location.replace('/dashboard');").into_response()
}
pub(super) fn router(cfg: Arc<Config>) -> Router {
    Router::new()
        .route("/auth/oidc/start", get(start))
        .route("/auth/oidc/callback", get(callback))
        .route("/auth/oidc/complete.js", get(completion_script))
        .with_state(cfg)
}
pub(super) async fn from_environment() -> Result<Router, &'static str> {
    if unsafe { libc::geteuid() } == 0
        || std::env::var("IPAT_R970_OIDC_ISSUER_SERVICE").as_deref() != Ok("YES")
        || std::env::var("IPAT_TRUSTED_HTTPS_EDGE").as_deref() != Ok("YES")
        || std::env::var("IPAT_R970_VERIFIED_CONFIDENTIAL_IDP").as_deref() != Ok("YES")
    {
        return Err("separate verified issuer deployment not enabled");
    }
    let durable_pending = std::env::var("IPAT_R971_DURABLE_PENDING").as_deref() == Ok("YES");
    let singleton = std::env::var("IPAT_R970_SINGLE_INSTANCE_OIDC").as_deref() == Ok("YES");
    if durable_pending == singleton {
        return Err("select exactly one OIDC pending-state mode");
    }
    let issuer = std::env::var("IPAT_R970_ISSUER").map_err(|_| "missing issuer")?;
    let host = std::env::var("IPAT_R970_HOST").map_err(|_| "missing verified host")?;
    let client_id = std::env::var("IPAT_R970_CLIENT_ID").map_err(|_| "missing client ID")?;
    if !oidc_config_shape(&issuer, &client_id, &host) {
        return Err("invalid pinned issuer or host");
    }
    let secret_file = PathBuf::from(
        std::env::var("IPAT_R970_CLIENT_SECRET_FILE").map_err(|_| "missing client secret file")?,
    );
    let key_file = PathBuf::from(
        std::env::var("IPAT_R970_PUBLIC_KEY_FILE").map_err(|_| "missing pinned issuer key file")?,
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
    let kid = std::env::var("IPAT_R970_KID").map_err(|_| "missing pinned key identifier")?;
    let verifier = PinnedIssuer::new(&issuer, &client_id, &kid, &pem)
        .map_err(|_| "invalid pinned IdP signing key")?;
    let db_user =
        std::env::var("IPAT_R970_DB_USER").map_err(|_| "missing issuer-only DB identity")?;
    if db_user != "ipat_oidc_session_issuer_login" {
        return Err("not an issuer-only PostgreSQL user");
    }
    let db_name = std::env::var("IPAT_R970_DB_NAME").map_err(|_| "missing issuer DB name")?;
    if db_name.is_empty()
        || db_name.len() > 63
        || !db_name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
    {
        return Err("invalid DB name");
    }
    let socket = PathBuf::from(
        std::env::var("IPAT_R970_DB_SOCKET").map_err(|_| "missing private issuer DB socket")?,
    );
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
        durable_pending,
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn durable_pending_cookie_has_strict_bounded_host_cookie_parts() {
        let p = Challenge::random().unwrap();
        let c = format!("{}.{}", p.state(), p.verifier());
        assert!(valid_browser_state_cookie(&c));
        assert_eq!(state_digest(p.state()).len(), 64);
        assert!(!valid_browser_state_cookie(&format!(
            "{}..{}",
            p.state(),
            p.verifier()
        )));
        assert!(!valid_browser_state_cookie("short.invalid"));
    }
    #[test]
    fn reject_issuer_endpoint_host_injection_and_encode_form() {
        for (issuer, host) in [
            ("http://evil.invalid", "tenant.example.net"),
            ("https://evil.invalid@safe.invalid", "tenant.example.net"),
            ("https://evil.invalid:443", "tenant.example.net"),
            ("https://id.example.net", "127.0.0.1"),
            ("https://id.example.net", "other.example.net:443"),
            ("https://id.example.net/../realm", "tenant.example.net"),
        ] {
            assert!(!oidc_config_shape(issuer, "ipat-confidential", host))
        }
        assert!(oidc_config_shape(
            "https://id.example.net/realms/ipat",
            "ipat-confidential",
            "tenant.example.net"
        ));
        let p = Challenge::random().unwrap();
        let url = authorization_url(
            "https://id.example.net/realms/ipat",
            "ipat-confidential",
            "tenant.example.net",
            &p,
        );
        assert!(url.contains("code_challenge_method=S256"));
        assert!(
            url.contains("redirect_uri=https%3A%2F%2Ftenant.example.net%2Fauth%2Foidc%2Fcallback")
        );
        assert!(!url.contains(p.verifier()));
        assert_eq!(url_component("a&b=+ "), "a%26b%3D%2B%20");
        assert!("/auth/oidc/complete.js".starts_with("/auth/oidc/"));
        assert!(!valid_code("\nsecret"));
    }
    #[test]
    fn reject_missing_duplicate_or_malformed_state_cookie() {
        let mut h = HeaderMap::new();
        assert!(browser_cookie(&h).is_none());
        let v = "a".repeat(43);
        h.insert(
            header::COOKIE,
            HeaderValue::from_str(&format!("{STATE_COOKIE}={v}; other=1")).unwrap(),
        );
        assert_eq!(browser_cookie(&h), Some(v.as_str()));
        h.insert(
            header::COOKIE,
            HeaderValue::from_str(&format!("{STATE_COOKIE}={v}; {STATE_COOKIE}={v}")).unwrap(),
        );
        assert!(browser_cookie(&h).is_none());
        h.append(header::COOKIE, HeaderValue::from_static("other=2"));
        assert!(browser_cookie(&h).is_none());
    }
}

// The disposable integration verifies that a pending login minted by one
// service/database connection can be consumed by a DIFFERENT instance. This
// is the HA-specific property that an in-memory HashMap cannot provide.
#[cfg(test)]
mod r971_pg_integration {
    use super::*;
    #[tokio::test]
    async fn r971_durable_oidc_cross_instance_exact_host_one_use() {
        if std::env::var("IPAT_PG_EPHEMERAL_TEST").as_deref() != Ok("1")
            || std::env::var("PGDATABASE").as_deref() != Ok("ipat_synthetic")
            || std::env::var("PGHOST").as_deref() != Ok("127.0.0.1")
        {
            return;
        }
        let mut config = PgConfig::new();
        config
            .host("127.0.0.1")
            .port(
                std::env::var("PGPORT")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(5432),
            )
            .dbname("ipat_synthetic")
            .user("postgres");
        if let Ok(p) = std::env::var("PGPASSWORD") {
            config.password(p);
        }
        let (a, ca) = config
            .connect(NoTls)
            .await
            .expect("fresh isolated synthetic database A");
        let (b, cb) = config
            .connect(NoTls)
            .await
            .expect("fresh isolated synthetic database B");
        tokio::spawn(async move {
            let _ = ca.await;
        });
        tokio::spawn(async move {
            let _ = cb.await;
        });
        a.batch_execute("SET ROLE ipat_oidc_session_issuer_login")
            .await
            .expect("restricted issuer A");
        b.batch_execute("SET ROLE ipat_oidc_session_issuer_login")
            .await
            .expect("restricted issuer B");
        let proof = Challenge::random().unwrap();
        let digest = state_digest(proof.state());
        let host = "tenant-r971.synthetic.invalid";
        assert!(a
            .query_one(
                "SELECT ipat_platform.begin_oidc_pending($1,$2,$3,$4)",
                &[&digest, &proof.s256(), &proof.nonce(), &host]
            )
            .await
            .unwrap()
            .get::<_, bool>(0));
        assert!(b
            .query_opt(
                "SELECT nonce FROM ipat_platform.consume_oidc_pending($1,$2,$3)",
                &[&digest, &proof.s256(), &"other.synthetic.invalid"]
            )
            .await
            .unwrap()
            .is_none());
        let issued = b
            .query_one(
                "SELECT nonce FROM ipat_platform.consume_oidc_pending($1,$2,$3)",
                &[&digest, &proof.s256(), &host],
            )
            .await
            .unwrap()
            .get::<_, String>(0);
        let reconstructed =
            Challenge::reconstruct_after_consumption(proof.state(), &issued, proof.verifier())
                .expect("bounded consumed proof");
        assert_eq!(reconstructed.s256(), proof.s256());
        assert!(a
            .query_opt(
                "SELECT nonce FROM ipat_platform.consume_oidc_pending($1,$2,$3)",
                &[&digest, &proof.s256(), &host]
            )
            .await
            .unwrap()
            .is_none());
    }
}
