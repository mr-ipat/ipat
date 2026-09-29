//! Evidence-constrained strict parser for owner-observed C320 actual CLI shapes.
//! This is a PURE parser: no device access, credentials, persistence or CLI writes.
//! Never log raw running-config or real ONT serials; do not infer current free IDs
//! from historical snapshots. A parsed state is evidence, NOT adopted status.
use std::collections::BTreeSet;

pub const MAX_REAL_CLI: usize = 32768;
const MAX_ONUS: usize = 128;
#[derive(Debug, PartialEq, Eq)]
pub enum RealInventoryError {
    Empty, Size, Unsafe, Layout, Duplicate, CountMismatch, InventoryMismatch,
}
#[derive(Debug, PartialEq, Eq)]
pub struct StateSummary {
    pub pon: String,
    pub configured: usize,
    pub online: usize,
    pub offline: usize,
    pub enabled: usize,
    pub disabled: usize,
    /// Ephemeral only: don't use this set to allocate from a stale read.
    pub observed_ids: BTreeSet<u8>,
}
#[derive(Debug, PartialEq, Eq)]
pub struct ConfigSummary {
    pub pon: String,
    pub declarations: usize,
    pub observed_ids: BTreeSet<u8>,
    /// Real serials are validated inside parser, then discarded before return.
}
fn bounded(input: &str) -> Result<(), RealInventoryError> {
    if input.is_empty() { return Err(RealInventoryError::Empty); }
    if input.len()>MAX_REAL_CLI { return Err(RealInventoryError::Size); }
    if input.bytes().any(|b|b==0||b==27||(b<32&&b!=10&&b!=13)) {
        return Err(RealInventoryError::Unsafe);
    }
    Ok(())
}
fn id_for(s: &str, pon: &str) -> Result<u8,RealInventoryError> {
    let (prefix,id)=s.split_once(':').ok_or(RealInventoryError::Layout)?;
    if prefix!=pon { return Err(RealInventoryError::Layout); }
    let n=id.parse::<u8>().map_err(|_|RealInventoryError::Layout)?;
    if !(1..=128).contains(&n) || id!=n.to_string() { return Err(RealInventoryError::Layout); }
    Ok(n)
}
/// Matches the exact state table layout observed by the owner on 2026-09-29.
/// Accepts an optional echoed fixed command / prompt around the table.
pub fn parse_state(input: &str, pon: &str) -> Result<StateSummary,RealInventoryError> {
    bounded(input)?;
    if pon!="1/1/1" { return Err(RealInventoryError::Layout); }
    let mut header=false;
    let mut footer=None;
    let mut ids=BTreeSet::new();
    let mut online=0;
    let mut offline=0;
    let mut enabled=0;
    let mut disabled=0;
    for line in input.lines().map(str::trim).filter(|s|!s.is_empty()) {
        if line.starts_with("OnuIndex") && line.contains("Admin State")
            && line.contains("OMCC State") && line.contains("Phase State")
            && line.contains("Channel") { header=true; continue; }
        if !header || line.bytes().all(|b|b==b'-') { continue; }
        if line.starts_with("ONU Number:") {
            let n=line["ONU Number:".len()..].trim();
            let (on,all)=n.split_once('/').ok_or(RealInventoryError::Layout)?;
            footer=Some((on.trim().parse::<usize>().map_err(|_|RealInventoryError::Layout)?,
                all.trim().parse::<usize>().map_err(|_|RealInventoryError::Layout)?));
            continue;
        }
        if footer.is_some() { continue; }
        let cols: Vec<_>=line.split_whitespace().collect();
        if cols.len()!=5 || !cols[0].starts_with("1/1/1:") || cols[4]!="1(GPON)" {
            return Err(RealInventoryError::Layout);
        }
        let id=id_for(cols[0],pon)?;
        if !ids.insert(id) { return Err(RealInventoryError::Duplicate); }
        match cols[1] { "enable"=>enabled+=1, "disable"=>disabled+=1,_=>return Err(RealInventoryError::Layout) }
        if cols[2]!="disable" && cols[2]!="enable" {return Err(RealInventoryError::Layout);}
        match cols[3] {"OffLine"=>offline+=1,"working"|"Online"=>online+=1,_=>return Err(RealInventoryError::Layout)}
        if ids.len()>MAX_ONUS {return Err(RealInventoryError::Size);}
    }
    let (reported_online,reported_total)=footer.ok_or(RealInventoryError::Layout)?;
    if !header||reported_online!=online||reported_total!=ids.len() {return Err(RealInventoryError::CountMismatch);}
    Ok(StateSummary {pon:pon.into(),configured:ids.len(),online,offline,enabled,disabled,observed_ids:ids})
}
/// Strict owner-observed response: %Code 62310-GPONSRV no unconfigured ONUs.
/// This is a report at observation time, not proof of attached ONTs.
pub fn parse_empty_unconfigured(input: &str) -> Result<bool,RealInventoryError> {
    bounded(input)?;
    let lines:Vec<_>=input.lines().map(str::trim).filter(|s|!s.is_empty()).collect();
    if lines.iter().any(|s|*s=="%Code 62310-GPONSRV : No related information to show.") {
        if lines.iter().filter(|s|s.starts_with("%Code")).count()!=1 ||
            lines.iter().any(|s|s.starts_with("gpon-onu_")||s.starts_with("OnuIndex")) {
            return Err(RealInventoryError::Layout);
        }
        Ok(true)
    } else {Err(RealInventoryError::Layout)}
}

/// Parses only bounded per-PON `show run interface` responses.
/// SN is checked for shape/duplicate but NEVER returned or logged.
pub fn parse_pon_config(input: &str, pon:&str) -> Result<ConfigSummary,RealInventoryError> {
    bounded(input)?;
    if pon!="1/1/1" {return Err(RealInventoryError::Layout);}
    let mut started=false;
    let mut ended=false;
    let mut ids=BTreeSet::new();
    let mut serials=BTreeSet::new();
    let mut interfaces=0;
    for line in input.lines().map(str::trim).filter(|s|!s.is_empty()) {
        if line=="interface gpon-olt_1/1/1" {
            if started { return Err(RealInventoryError::Duplicate); }
            started=true;interfaces+=1;continue;
        }
        if !started { continue; }
        if line=="end" { ended=true;break; }
        if line=="!"||line=="no shutdown"||line=="linktrap disable" {continue;}
        if !line.starts_with("onu ") {return Err(RealInventoryError::Layout);}
        let parts:Vec<_>=line.split_whitespace().collect();
        if parts.len()!=6 || parts[0]!="onu" || parts[2]!="type" || parts[4]!="sn" {
            return Err(RealInventoryError::Layout);
        }
        let id=parts[1].parse::<u8>().map_err(|_|RealInventoryError::Layout)?;
        if !(1..=128).contains(&id)||parts[1]!=id.to_string()||
            parts[3].is_empty()||parts[3].len()>48||
            !parts[3].bytes().all(|b|b.is_ascii_alphanumeric()||b==b'-') {
            return Err(RealInventoryError::Layout);
        }
        let sn=parts[5];
        if sn.len()!=12||!sn.bytes().take(4).all(|b|b.is_ascii_uppercase())||
            !sn.bytes().skip(4).all(|b|b.is_ascii_hexdigit()&&!b.is_ascii_lowercase()) {
            return Err(RealInventoryError::Layout);
        }
        if !ids.insert(id)||!serials.insert(sn) {return Err(RealInventoryError::Duplicate);}
        if ids.len()>MAX_ONUS {return Err(RealInventoryError::Size);}
    }
    if !started||!ended||interfaces!=1 {return Err(RealInventoryError::Layout);}
    Ok(ConfigSummary {pon:pon.into(),declarations:ids.len(),observed_ids:ids})
}
/// Reconciled inventory is STILL read-only, never a production adopt token.
pub fn reconcile(state:&StateSummary, config:&ConfigSummary) -> Result<(),RealInventoryError> {
    if state.pon!=config.pon||state.observed_ids!=config.observed_ids||
        state.configured!=config.declarations {return Err(RealInventoryError::InventoryMismatch);}
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn sample_state()->String {
        "OnuIndex Admin State OMCC State Phase State Channel\n\
         -----------------------------------------------------\n\
         1/1/1:2 enable disable OffLine 1(GPON)\n\
         1/1/1:8 enable disable OffLine 1(GPON)\n\
         ONU Number: 0/2\n".into()
    }
    fn sample_config()->String {
        "Building configuration...\ninterface gpon-olt_1/1/1\n\
         no shutdown\n linktrap disable\n\
         onu 2 type TEST-ONT sn ZTEGABCDEF12\n\
         onu 8 type TEST-ONT sn ZTEGABCDEF13\n!\nend\n".into()
    }
    #[test]
    fn actual_firmware_response_formats_reconcile_with_only_synthetic_serials() {
        let state=parse_state(&sample_state(),"1/1/1").unwrap();
        let config=parse_pon_config(&sample_config(),"1/1/1").unwrap();
        assert_eq!((state.configured,state.online,state.offline), (2,0,2));
        assert_eq!(reconcile(&state,&config),Ok(()));
        assert_eq!(parse_empty_unconfigured("%Code 62310-GPONSRV : No related information to show.\n"),Ok(true));
    }
    #[test]
    fn denies_truncated_or_contradictory_responses() {
        for x in ["", "ONU Number: 0/2", "OnuIndex Admin State OMCC State Phase State Channel\n1/1/1:2 enable disable OffLine 1(GPON)\nONU Number: 1/1", "OnuIndex Admin State OMCC State Phase State Channel\n1/1/1:2 enable disable OffLine 1(GPON)\n1/1/1:2 enable disable OffLine 1(GPON)\nONU Number: 0/2"] {
            assert!(parse_state(x,"1/1/1").is_err());
        }
        assert!(parse_pon_config("interface gpon-olt_1/1/1\nonu 2 type TEST-ONT sn ZTEGABCDEF12\n", "1/1/1").is_err());
        assert!(parse_empty_unconfigured("%Code 62310-GPONSRV : No related information to show.\ngpon-onu_1/1/1:1").is_err());
    }
    #[test]
    fn accepts_seventy_two_synthetic_matching_entries_and_reports_zero_online() {
        let mut state=String::from("OnuIndex Admin State OMCC State Phase State Channel\n");
        let mut config=String::from("Building configuration...\ninterface gpon-olt_1/1/1\n no shutdown\n");
        for id in 1..=72 {
            state.push_str(&format!("1/1/1:{id} enable disable OffLine 1(GPON)\n"));
            config.push_str(&format!(" onu {id} type TEST-ONT sn ZTEG{id:08X}\n"));
        }
        state.push_str("ONU Number: 0/72\n");
        config.push_str("!\nend\n");
        let s=parse_state(&state,"1/1/1").unwrap();
        let c=parse_pon_config(&config,"1/1/1").unwrap();
        assert_eq!((s.configured,s.online,s.offline),(72,0,72));
        assert_eq!(reconcile(&s,&c),Ok(()));
    }
    #[test]
    fn rejects_stale_or_inconsistent_live_reads_without_offering_id() {
        let state=parse_state(&sample_state(),"1/1/1").unwrap();
        let config=parse_pon_config("interface gpon-olt_1/1/1\n onu 2 type TEST-ONT sn ZTEGABCDEF12\n!\nend\n","1/1/1").unwrap();
        assert_eq!(reconcile(&state,&config),Err(RealInventoryError::InventoryMismatch));
        let dupe="interface gpon-olt_1/1/1\n onu 2 type TEST-ONT sn ZTEGABCDEF12\n onu 3 type TEST-ONT sn ZTEGABCDEF12\n!\nend\n";
        assert_eq!(parse_pon_config(dupe,"1/1/1"),Err(RealInventoryError::Duplicate));
    }
}
