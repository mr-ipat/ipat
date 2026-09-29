//! Strict offline ONT registration draft validator. No CLI command generator or network.
//! Only an authenticated tenant-bound, audited production workflow may eventually
//! translate an independently approved draft to verified firmware-specific CLI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OntDraft {
    pub device_id: String,
    pub pon_slot: String,
    pub pon_port: u8,
    pub onu_id: u8,
    pub serial: String,
    pub profile_ref: String,
    pub idempotency_key: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DraftError { InvalidField, UnsupportedSlot, DuplicateSerial, DuplicatePosition }

fn safe_ref(s: &str) -> bool {
    !s.is_empty() && s.len() <= 64 && s.bytes().all(|b| {
        b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_'
    })
}

fn valid_serial(s: &str) -> bool {
    s.len() == 12 && s.as_bytes()[..4].iter().all(|b| b.is_ascii_uppercase())
        && s.as_bytes()[4..].iter().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_lowercase())
}
/// Review of syntax and collisions against a separately verified inventory.
/// Never asserts inventory freshness, host trust or authorization by itself.
pub fn validate_draft(
    draft: &OntDraft,
    observed_serials: &[String],
    observed_positions: &[(String, u8, u8)],
) -> Result<(), DraftError> {
    if draft.device_id != "dev-01" || !safe_ref(&draft.profile_ref)
        || !safe_ref(&draft.idempotency_key) || !valid_serial(&draft.serial)
        || draft.pon_port == 0 || draft.pon_port > 16
        || draft.onu_id == 0 || draft.onu_id > 128
    { return Err(DraftError::InvalidField); }
    // Only slot actually observed with GPON card; this still does NOT
    // establish physical PON port population or available ONU IDs.
    if draft.pon_slot != "1/1/1" { return Err(DraftError::UnsupportedSlot); }
    if observed_serials.iter().any(|s| s == &draft.serial) {
        return Err(DraftError::DuplicateSerial);
    }
    if observed_positions.iter().any(|(slot, port, id)| {
        slot == &draft.pon_slot && *port == draft.pon_port && *id == draft.onu_id
    }) { return Err(DraftError::DuplicatePosition); }
    Ok(())
}

/// Even syntactically valid drafts are REVIEW_ONLY until trusted live
/// inventory, tenant ownership, separate approval and read-back exist.
pub const ONT_EXECUTION_ENABLED: bool = false;
#[cfg(test)]
mod tests {
    use super::*;
    fn draft() -> OntDraft {
        OntDraft { device_id: "dev-01".into(), pon_slot: "1/1/1".into(),
            pon_port: 1, onu_id: 1, serial: "ZTEGABCDEF12".into(),
            profile_ref: "review-only".into(), idempotency_key: "draft-1".into() }
    }
    #[test]
    fn validates_synthetic_draft_but_never_enables_execution() {
        assert_eq!(validate_draft(&draft(), &[], &[]), Ok(()));
        assert!(!ONT_EXECUTION_ENABLED);
    }
    #[test]
    fn blocks_duplicate_serial_or_position() {
        let d = draft();
        assert_eq!(validate_draft(&d, &[d.serial.clone()], &[]), Err(DraftError::DuplicateSerial));
        assert_eq!(validate_draft(&d, &[], &[(d.pon_slot.clone(),1,1)]), Err(DraftError::DuplicatePosition));
    }
    #[test]
    fn rejects_unknown_slot_and_unsafe_serial() {
        let mut d = draft();
        d.pon_slot = "1/1/4".into();
        assert_eq!(validate_draft(&d, &[], &[]), Err(DraftError::UnsupportedSlot));
        d.pon_slot = "1/1/1".into(); d.serial = "ZTEG;config!!".into();
        assert_eq!(validate_draft(&d, &[], &[]), Err(DraftError::InvalidField));
    }
    #[test]
    fn denies_untrusted_fields_and_out_of_range() {
        let mut d = draft();
        d.device_id = "dev-02".into();
        assert_eq!(validate_draft(&d, &[], &[]), Err(DraftError::InvalidField));
        d = draft(); d.pon_port = 17;
        assert_eq!(validate_draft(&d, &[], &[]), Err(DraftError::InvalidField));
        d = draft(); d.onu_id = 0;
        assert_eq!(validate_draft(&d, &[], &[]), Err(DraftError::InvalidField));
        d = draft(); d.profile_ref = "x;reboot".into();
        assert_eq!(validate_draft(&d, &[], &[]), Err(DraftError::InvalidField));
    }
}
