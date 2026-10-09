//! First **read-only** RouterOS 7 REST HTTPS transport, intentionally separate
//! from API-SSL (binary port 8729), PPPoE credentials and ANY device write.
//! The caller must independently approve exact physical target, firmware,
//! owner, TLS CA, and pinned server certificate. A valid network response
//! remains UNREVIEWED inventory, never device adoption/compatibility evidence.
use crate::{normalize_resource, ApprovedReadProfile, UnreviewedInventory};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use rustls::{
    pki_types::{CertificateDer, ServerName},
    ClientConfig, RootCertStore,
};
use sha2::{Digest, Sha256};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    time::timeout,
};
use tokio_rustls::TlsConnector;

const MAX_HEADERS: usize = 8192;
const MAX_BODY: usize = 32768;
const MAX_WIRE: usize = 65536;
const TIMEOUT: Duration = Duration::from_secs(5);

// Never derive Debug from credentials. No secret or raw device reply in errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReadError {
    InvalidTarget,
    MissingApproval,
    UntrustedCertificate,
    UnsafeCredentials,
    TransportUnavailable,
    UnsafeHttpResponse,
    UnqualifiedDevice,
}

#[derive(Clone, Copy)]
pub struct ApprovedTarget {
    ip: Ipv4Addr,
    port: u16,
}
impl ApprovedTarget {
    /// This is only a target restriction, not approval or a physical identity.
    pub fn private_ipv4(ip: Ipv4Addr, port: u16) -> Result<Self, ReadError> {
        if !(ip.is_private() && !ip.is_loopback() && !ip.is_link_local() && !ip.is_broadcast())
            || port != 443
        {
            return Err(ReadError::InvalidTarget);
        }
        Ok(Self { ip, port })
    }
}
pub struct LocalTrust<'a> {
    pub ca_der: &'a [u8],
    pub expected_leaf_sha256: [u8; 32],
    pub username: &'a str,
    pub password: &'a str,
    /// Explicit narrowly scoped owner approval; not a substitute for OOB proof.
    pub approved_read_only: bool,
}
fn safe_secret(s: &str, max: usize) -> bool {
    !s.is_empty()
        && s.len() <= max
        && s.bytes()
            .all(|c| c.is_ascii_graphic() && c != b':' && c != b'\r' && c != b'\n')
}
fn request(ip: Ipv4Addr, user: &str, password: &str) -> Result<Vec<u8>, ReadError> {
    if !safe_secret(user, 64) || !safe_secret(password, 256) {
        return Err(ReadError::UnsafeCredentials);
    }
    let basic = BASE64.encode(format!("{user}:{password}"));
    // Fixed GET resource: no URL/body/header/query/body supplied by a browser.
    // Disable connection reuse and compression to bound memory and decode.
    let req=format!(
        "GET /rest/system/resource HTTP/1.1\r\nHost: {ip}\r\nAuthorization: Basic {basic}\r\nAccept: application/json\r\nAccept-Encoding: identity\r\nConnection: close\r\n\r\n"
    );
    Ok(req.into_bytes())
}
fn split_http(reply: &[u8]) -> Result<&[u8], ReadError> {
    if reply.len() > MAX_WIRE {
        return Err(ReadError::UnsafeHttpResponse);
    }
    let Some(sep) = reply.windows(4).position(|v| v == b"\r\n\r\n") else {
        return Err(ReadError::UnsafeHttpResponse);
    };
    if sep > MAX_HEADERS {
        return Err(ReadError::UnsafeHttpResponse);
    }
    let hdr = std::str::from_utf8(&reply[..sep]).map_err(|_| ReadError::UnsafeHttpResponse)?;
    let mut lines = hdr.split("\r\n");
    let status = lines.next().ok_or(ReadError::UnsafeHttpResponse)?;
    // Never follow redirects or accept auth errors returned with identifying bodies.
    if status != "HTTP/1.1 200 OK" && status != "HTTP/1.0 200 OK" {
        return Err(ReadError::UnsafeHttpResponse);
    }
    let mut len = None;
    let mut chunked = false;
    let mut json = false;
    let mut hcount = 0;
    for line in lines {
        hcount += 1;
        if hcount > 48 || line.len() > 1024 {
            return Err(ReadError::UnsafeHttpResponse);
        }
        let (key, value) = line.split_once(':').ok_or(ReadError::UnsafeHttpResponse)?;
        let key = key.trim();
        let value = value.trim();
        if value.as_bytes().contains(&0) {
            return Err(ReadError::UnsafeHttpResponse);
        }
        if key.eq_ignore_ascii_case("content-length") {
            if len.is_some() || chunked || !value.bytes().all(|b| b.is_ascii_digit()) {
                return Err(ReadError::UnsafeHttpResponse);
            }
            len = Some(
                value
                    .parse::<usize>()
                    .map_err(|_| ReadError::UnsafeHttpResponse)?,
            );
        }
        if key.eq_ignore_ascii_case("transfer-encoding") {
            if chunked || len.is_some() || !value.eq_ignore_ascii_case("chunked") {
                return Err(ReadError::UnsafeHttpResponse);
            }
            chunked = true;
        }
        if key.eq_ignore_ascii_case("content-type") {
            if json
                || !(value.eq_ignore_ascii_case("application/json")
                    || value.to_ascii_lowercase().starts_with("application/json;"))
            {
                return Err(ReadError::UnsafeHttpResponse);
            }
            json = true;
        }
        if key.eq_ignore_ascii_case("content-encoding") && !value.eq_ignore_ascii_case("identity") {
            return Err(ReadError::UnsafeHttpResponse);
        }
    }
    if !json || len.is_some_and(|n| n > MAX_BODY) || (len.is_none() && !chunked) {
        return Err(ReadError::UnsafeHttpResponse);
    }
    let body = &reply[sep + 4..];
    if chunked {
        // RouterOS should prefer Content-Length. Never silently consume
        // untrusted chunked bytes without an explicit bounded decoder.
        return Err(ReadError::UnsafeHttpResponse);
    }
    if len != Some(body.len()) || body.is_empty() {
        return Err(ReadError::UnsafeHttpResponse);
    }
    Ok(body)
}

/// A successful read validates only the exact R6.1 owner-reported target tuple,
/// NOT its source provenance, operator approval, compatibility, or write rights.
/// Raw JSON/credentials are never logged or returned from this function.
pub async fn read_routeros_identity(
    target: ApprovedTarget,
    trust: LocalTrust<'_>,
) -> Result<UnreviewedInventory, ReadError> {
    // One global deadline prevents a malicious or stalled router from
    // extending total elapsed time via a byte-per-timeout trickle.
    timeout(Duration::from_secs(12), read_once(target, trust))
        .await
        .map_err(|_| ReadError::TransportUnavailable)?
}
async fn read_once(
    target: ApprovedTarget,
    trust: LocalTrust<'_>,
) -> Result<UnreviewedInventory, ReadError> {
    if !trust.approved_read_only {
        return Err(ReadError::MissingApproval);
    }
    let outgoing = request(target.ip, trust.username, trust.password)?;
    if trust.ca_der.len() < 128 || trust.ca_der.len() > 8192 {
        return Err(ReadError::UntrustedCertificate);
    }
    let mut roots = RootCertStore::empty();
    roots
        .add(CertificateDer::from(trust.ca_der.to_vec()))
        .map_err(|_| ReadError::UntrustedCertificate)?;
    let cfg = ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    let connector = TlsConnector::from(Arc::new(cfg));
    let address = SocketAddr::new(IpAddr::V4(target.ip), target.port);
    let tcp = timeout(TIMEOUT, TcpStream::connect(address))
        .await
        .map_err(|_| ReadError::TransportUnavailable)?
        .map_err(|_| ReadError::TransportUnavailable)?;
    let name = ServerName::try_from(IpAddr::V4(target.ip)).map_err(|_| ReadError::InvalidTarget)?;
    let mut stream = timeout(TIMEOUT, connector.connect(name, tcp))
        .await
        .map_err(|_| ReadError::TransportUnavailable)?
        .map_err(|_| ReadError::UntrustedCertificate)?;
    let cert = stream
        .get_ref()
        .1
        .peer_certificates()
        .and_then(|c| c.first())
        .ok_or(ReadError::UntrustedCertificate)?;
    let got = Sha256::digest(cert.as_ref());
    use std::ops::Deref as _;
    if got.deref() != trust.expected_leaf_sha256.as_slice() {
        return Err(ReadError::UntrustedCertificate);
    }
    timeout(TIMEOUT, stream.write_all(&outgoing))
        .await
        .map_err(|_| ReadError::TransportUnavailable)?
        .map_err(|_| ReadError::TransportUnavailable)?;
    let mut raw = Vec::with_capacity(4096);
    let mut buf = [0u8; 4096];
    loop {
        let n = timeout(TIMEOUT, stream.read(&mut buf))
            .await
            .map_err(|_| ReadError::TransportUnavailable)?
            .map_err(|_| ReadError::TransportUnavailable)?;
        if n == 0 {
            break;
        }
        if raw.len() + n > MAX_WIRE {
            return Err(ReadError::UnsafeHttpResponse);
        }
        raw.extend_from_slice(&buf[..n]);
    }
    let data = split_http(&raw)?;
    normalize_resource(data, ApprovedReadProfile::Dev08OwnerReportedRb951)
        .map_err(|_| ReadError::UnqualifiedDevice)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn r1012_request_is_exact_read_only_and_rejects_headers_and_secrets() {
        let ip = "192.168.50.9".parse().unwrap();
        let req = request(ip, "read_only", "synthetic-not-real").unwrap();
        let text = String::from_utf8(req).unwrap();
        assert!(text.starts_with("GET /rest/system/resource HTTP/1.1\r\n"));
        assert!(text.contains("Connection: close\r\n"));
        assert!(!text.contains("POST ") && !text.contains("DELETE "));
        for wrong in ["user\r\nHost: evil", "", "user:admin"] {
            assert_eq!(
                request(ip, wrong, "password"),
                Err(ReadError::UnsafeCredentials)
            );
        }
        assert_eq!(
            request(ip, "user", "pw\n"),
            Err(ReadError::UnsafeCredentials)
        );
    }
    #[test]
    fn r1012_private_exact_rest_target_only() {
        assert!(ApprovedTarget::private_ipv4("192.168.50.9".parse().unwrap(), 443).is_ok());
        assert!(ApprovedTarget::private_ipv4("10.10.13.233".parse().unwrap(), 443).is_ok());
        for address in [
            "127.0.0.1",
            "1.1.1.1",
            "169.254.1.1",
            "0.0.0.0",
            "203.0.113.20",
        ] {
            assert_eq!(
                ApprovedTarget::private_ipv4(address.parse().unwrap(), 443).err(),
                Some(ReadError::InvalidTarget)
            );
        }
        assert_eq!(
            ApprovedTarget::private_ipv4("192.168.50.9".parse().unwrap(), 8729).err(),
            Some(ReadError::InvalidTarget)
        );
    }
    #[test]
    fn r1012_bounded_http_json_framing() {
        let body =
            br#"[{"board-name":"RB951Ui-2HnD","architecture-name":"mipsbe","version":"7.23.7"}]"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
            body.len()
        );
        let mut wire = response.into_bytes();
        wire.extend_from_slice(body);
        let parsed = split_http(&wire).unwrap();
        let normalized =
            normalize_resource(parsed, ApprovedReadProfile::Dev08OwnerReportedRb951).unwrap();
        assert_eq!(normalized.board(), "RB951Ui-2HnD");
        let bad=[
           b"HTTP/1.1 302 Found\r\nLocation: /other\r\n\r\n".as_slice(),
           b"HTTP/1.1 401 Unauthorized\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n{}".as_slice(),
           b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 12\r\nContent-Length: 12\r\n\r\n{}".as_slice(),
           b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nTransfer-Encoding: chunked\r\n\r\n2\r\n{}\r\n0\r\n\r\n".as_slice(),
           b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Encoding: gzip\r\nContent-Length: 2\r\n\r\n{}".as_slice(),
        ];
        for sample in bad {
            assert_eq!(split_http(sample), Err(ReadError::UnsafeHttpResponse));
        }
        let oversized = vec![b'A'; MAX_WIRE + 1];
        assert_eq!(split_http(&oversized), Err(ReadError::UnsafeHttpResponse));
    }
}
