//! Synthetic strict C320 unconfigured-ONU parser; firmware compatibility UNTESTED.
//! No transport or customer data logging; reject ambiguity rather than infer free IDs.
use std::collections::HashSet;

pub const MAX_UNCONFIGURED_OUTPUT: usize = 16_384;
const MAX_UNCONFIGURED_ONUS: usize = 128;
#[derive(Debug, PartialEq, Eq)]
pub struct UnconfiguredOnu {
    pub pon_slot: String,
    pub pon_port: u8,
    pub reported_index: u8,
    pub serial: String,
}
#[derive(Debug, PartialEq, Eq)]
pub enum DiscoveryError {
    Empty,
    Unsafe,
    TooLarge,
    UnknownFormat,
    Duplicate,
    Excessive,
}

fn serial_ok(s: &str) -> bool {
    s.len() == 12
        && s.bytes().take(4).all(|b| b.is_ascii_uppercase())
        && s.bytes()
            .skip(4)
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_lowercase())
}

/// The `reported_index` in an unconfigured listing is NOT an available ONU ID.
/// Always cross-check fresh registered ONU inventory before proposing any ID.
pub fn parse_unconfigured(input: &str) -> Result<Vec<UnconfiguredOnu>, DiscoveryError> {
    if input.is_empty() {
        return Err(DiscoveryError::Empty);
    }
    if input.len() > MAX_UNCONFIGURED_OUTPUT {
        return Err(DiscoveryError::TooLarge);
    }
    if input
        .bytes()
        .any(|b| b == 0 || b == 27 || (b < 32 && b != 10 && b != 13))
    {
        return Err(DiscoveryError::Unsafe);
    }
    let mut rows = Vec::new();
    let mut header = false;
    let mut seen = HashSet::new();
    for line in input.lines().map(str::trim).filter(|s| !s.is_empty()) {
        let cols: Vec<_> = line.split_whitespace().collect();
        if cols == ["OnuIndex", "Sn", "State"] {
            header = true;
            continue;
        }
        if !header || line.bytes().all(|b| b == b'-') {
            continue;
        }
        if cols.len() != 3 || cols[2] != "unknown" || !serial_ok(cols[1]) {
            return Err(DiscoveryError::UnknownFormat);
        }
        let path = cols[0]
            .strip_prefix("gpon-onu_")
            .ok_or(DiscoveryError::UnknownFormat)?;
        let (locator, reported) = path.split_once(':').ok_or(DiscoveryError::UnknownFormat)?;
        let parts: Vec<_> = locator.split('/').collect();
        if parts.len() != 3 || parts[0] != "1" || parts[1] != "1" {
            return Err(DiscoveryError::UnknownFormat);
        }
        let port: u8 = parts[2]
            .parse()
            .map_err(|_| DiscoveryError::UnknownFormat)?;
        let index: u8 = reported
            .parse()
            .map_err(|_| DiscoveryError::UnknownFormat)?;
        if !(1..=16).contains(&port) || !(1..=128).contains(&index) || !seen.insert(cols[1]) {
            return Err(DiscoveryError::Duplicate);
        }
        rows.push(UnconfiguredOnu {
            pon_slot: "1/1/1".into(),
            pon_port: port,
            reported_index: index,
            serial: cols[1].into(),
        });
        if rows.len() > MAX_UNCONFIGURED_ONUS {
            return Err(DiscoveryError::Excessive);
        }
    }
    if !header {
        return Err(DiscoveryError::UnknownFormat);
    }
    Ok(rows)
}
#[cfg(test)]
mod tests {
    use super::*;
    const HEADER: &str = "OnuIndex Sn State\n---------------------------------\n";
    #[test]
    fn parses_synthetic_undiscovered_but_does_not_assign_id() {
        let data = format!("{HEADER}gpon-onu_1/1/1:1 ZTEGABCDEF12 unknown\n");
        let v = parse_unconfigured(&data).unwrap();
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].pon_port, 1);
        assert_eq!(v[0].reported_index, 1); // Observed, NEVER proposed allocation.
    }
    #[test]
    fn header_only_is_empty_not_a_verified_complete_discovery() {
        assert_eq!(parse_unconfigured(HEADER).unwrap().len(), 0);
        assert_eq!(parse_unconfigured("").unwrap_err(), DiscoveryError::Empty);
    }
    #[test]
    fn rejects_ambiguity_and_injections() {
        for raw in [
            "gpon-onu_1/1/1:1 ZTEGABCDEF12 unknown\n".to_string(),
            format!("{HEADER}gpon-onu_1/1/1:1 ZTEGABCDEF12 unknown;reboot\n"),
            format!("{HEADER}gpon-onu_1/1/1:1 ztegabcdef12 unknown\n"),
            format!("{HEADER}gpon-onu_1/1/17:1 ZTEGABCDEF12 unknown\n"),
            format!("{HEADER}gpon-onu_1/1/1:1 ZTEGABCDEF12 unknown\n\u{1b}[0m"),
        ] {
            assert!(parse_unconfigured(&raw).is_err());
        }
    }
    #[test]
    fn rejects_duplicate_serial_even_if_index_differs() {
        let raw=format!("{HEADER}gpon-onu_1/1/1:1 ZTEGABCDEF12 unknown\ngpon-onu_1/1/2:2 ZTEGABCDEF12 unknown\n");
        assert_eq!(parse_unconfigured(&raw), Err(DiscoveryError::Duplicate));
    }
}
