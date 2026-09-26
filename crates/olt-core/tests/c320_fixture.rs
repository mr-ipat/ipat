use olt_core::{
    parse_cards, parse_running_versions, review_firmware, CardStatus, Disposition, EvidenceError,
    FirmwareReview, READ_COMMANDS, WRITE_ENABLED,
};
const CARDS:&str = "ZXAN#show card\nRack Shelf Slot CfgType RealType Port HardVer SoftVer Status\n-------------------------\n1 1 1 GTGO GTGOG 8 120301 V2.0.0 INSERVICE\n1 1 3 SMXA SMXA 0 110702 V2.0.0 INSERVICE\n1 1 4 SMXA SMXA 0 110702 V2.0.0 STANDBY\n";
const VERSIONS:&str = "ZXAN#show version-running\nPhyLoc FileType VerType VerTag BuildTime VerLength\n--------------------------------------------\n1/1/11 SCXL MVR V1.2.5 2013-01-07 11:18:49 8144910\n1/1/11 SCXL BT V1.0.0 2012-08-01 16:04:46 448032\n";
#[test]
fn exact_commands_are_read_only_and_no_firmware_execution_exists() {
    assert_eq!(READ_COMMANDS, ["show card", "show version-running"]);
    assert!(!WRITE_ENABLED);
}
#[test]
fn extracts_bounded_card_inventory_from_synthetic_vendor_format() {
    let cards = parse_cards(CARDS).unwrap();
    assert_eq!(cards.len(), 3);
    assert_eq!(cards[0].location, "1/1/1");
    assert_eq!(cards[0].card_type, "GTGOG");
    assert_eq!(cards[1].status, CardStatus::InService);
    assert_eq!(cards[2].status, CardStatus::Standby);
}
#[test]
fn rejects_duplicates_unexpected_state_and_injected_shell_characters() {
    let duplicated = CARDS.to_string() + "1 1 4 SMXA SMXA 0 110702 V2.0.0 INSERVICE\n";
    assert_eq!(parse_cards(&duplicated), Err(EvidenceError::Duplicate));
    assert_eq!(
        parse_cards(&CARDS.replace("STANDBY", "FAILED")),
        Err(EvidenceError::Layout)
    );
    assert_eq!(
        parse_cards(&CARDS.replace("GTGOG", "GTGOG;rm")),
        Err(EvidenceError::Layout)
    );
    assert_eq!(
        parse_cards(&format!("{CARDS}\u{1b}[0m")),
        Err(EvidenceError::Unsafe)
    );
    assert_eq!(parse_cards(&"x".repeat(32769)), Err(EvidenceError::Size));
}
#[test]
fn parses_only_minimal_version_metadata_without_raw_transcript() {
    let v = parse_running_versions(VERSIONS).unwrap();
    assert_eq!(v.len(), 2);
    assert_eq!(v[0].location, "1/1/11");
    assert_eq!(v[0].version, "V1.2.5");
    assert_eq!(v[1].file_kind, "BT");
    assert_eq!(
        parse_running_versions(&VERSIONS.replace("V1.0.0", "V1.0.0 &secret")),
        Err(EvidenceError::Layout)
    );
}
#[test]
fn unsafe_malformed_and_duplicate_versions_are_denied() {
    assert_eq!(
        parse_running_versions("unknown"),
        Err(EvidenceError::Layout)
    );
    assert_eq!(
        parse_running_versions(&VERSIONS.replace("MVR V1.2.5", "PASSWORD V1.2.5")),
        Err(EvidenceError::Layout)
    );
    let duplicate = VERSIONS.to_string() + "1/1/11 SCXL MVR V1.2.5 2013-01-07 11:18:49 8144910\n";
    assert_eq!(
        parse_running_versions(&duplicate),
        Err(EvidenceError::Duplicate)
    );
    assert_eq!(
        parse_running_versions(&VERSIONS.replace("11:18:49", "11:\t:49")),
        Err(EvidenceError::Unsafe)
    );
}
#[test]
fn firmware_stays_blocked_until_every_attestation_is_observed_and_approved() {
    let blank = FirmwareReview::default();
    assert_eq!(
        review_firmware(blank),
        Disposition::Blocked("exact physical card/chassis unknown")
    );
    let mut all = FirmwareReview {
        actual_chassis_and_card_identity_verified: true,
        actual_running_versions_recorded: true,
        exact_vendor_firmware_and_checksum_independently_verified: true,
        complete_configuration_backup_restored_and_tested: true,
        alarm_free_and_service_impact_reviewed: true,
        approved_maintenance_window: true,
        separate_maker_checker_approved: true,
        independent_onsite_recovery_console_and_rollback: true,
    };
    assert_eq!(review_firmware(all), Disposition::HumanReviewOnly);
    all.exact_vendor_firmware_and_checksum_independently_verified = false;
    assert_eq!(
        review_firmware(all),
        Disposition::Blocked("vendor image not verified")
    );
    all.exact_vendor_firmware_and_checksum_independently_verified = true;
    all.separate_maker_checker_approved = false;
    assert_eq!(
        review_firmware(all),
        Disposition::Blocked("no independent approval")
    );
    // Even all booleans true never produces firmware commands or executable action.
}
