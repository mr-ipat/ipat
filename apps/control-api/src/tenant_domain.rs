//! Tenant-domain routing primitives.
//!
//! A verified hostname may select public tenant bootstrap context, but NEVER
//! grants user membership, role, POP scope, device access, or secrets.

use axum::http::HeaderValue;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalHost(String);

impl CanonicalHost {
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

pub(crate) fn canonical_host(value: &HeaderValue) -> Option<CanonicalHost> {
    let raw = value.to_str().ok()?.trim();
    if raw.is_empty() || raw.len() > 255 || raw.contains(['/', '\\', ' ', '\t', '\r', '\n']) {
        return None;
    }

    // IPv6 literals are intentionally unsupported for tenant-domain selection.
    if raw.starts_with('[') {
        return None;
    }

    let host = match raw.rsplit_once(':') {
        Some((left, port))
            if !left.is_empty()
                && !port.is_empty()
                && port.bytes().all(|b| b.is_ascii_digit()) =>
        {
            left
        }
        Some(_) => return None,
        None => raw,
    };

    let host = host.strip_suffix('.').unwrap_or(host);
    if host.len() < 3 || host.len() > 253 {
        return None;
    }
    if host.starts_with('.') || host.ends_with('.') || host.contains("..") {
        return None;
    }

    let lower = host.to_ascii_lowercase();
    if !lower.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'.') {
        return None;
    }

    for label in lower.split('.') {
        if label.is_empty()
            || label.len() > 63
            || label.starts_with('-')
            || label.ends_with('-')
        {
            return None;
        }
    }

    Some(CanonicalHost(lower))
}

#[cfg(test)]
mod tests {
    use super::*;

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
            let value = HeaderValue::from_str(raw).unwrap();
            assert!(canonical_host(&value).is_none(), "{raw}");
        }
    }
}
