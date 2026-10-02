//! R9.58 shared firmware upgrade adapter contract; NO live vendor driver.
//! Only the future restricted worker may translate a fresh, signed, database-
//! verified operator execution intent into a ticket. Never call directly
//! from an HTTP form or infer firmware compatibility from model text alone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Target {
    pub tenant: String,
    pub device_id: String,
    pub vendor: String,
    pub exact_model: String,
    pub running_version: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Image {
    pub object_ref: String,
    pub sha256: String,
    pub vendor: String,
    pub exact_model: String,
    pub target_version: String,
}
#[derive(Clone, Debug)]
pub struct VerifiedTicket {
    pub target: Target,
    pub image: Image,
    pub explicit_operator_request: bool,
    pub independent_approval_current: bool,
    pub exact_hardware_evidence_fresh: bool,
    pub independently_authenticated_vendor_image: bool,
    pub backup_and_restore_tested: bool,
    pub impact_and_maintenance_reviewed: bool,
    pub independent_recovery_available: bool,
    pub inside_approved_window: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FirmwareError {
    MissingOperatorRequest,
    MissingIndependentApproval,
    StaleOrIncompleteEvidence,
    UnsupportedExactVendorTuple,
    TransportFailure,
    ActivationFailure,
    VerificationFailedRecoveryRequired,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    VerifiedTargetVersion(String),
}
/// Exact-vendor adapters MUST implement a per-(model,board,firmware) support
/// check. This crate deliberately contains no real driver or firmware bytes.
/// Attacker-controlled text in Target/Image can never be an allowlist entry.
pub trait FirmwareAdapter {
    fn is_physically_qualified(&self, target: &Target, image: &Image) -> bool;
    fn transfer_verified_image(
        &mut self,
        target: &Target,
        image: &Image,
    ) -> Result<(), FirmwareError>;
    fn activate_in_maintenance_window(&mut self, target: &Target) -> Result<(), FirmwareError>;
    fn read_back_running_version(&mut self, target: &Target) -> Result<String, FirmwareError>;
}

pub fn execute_verified_firmware<A: FirmwareAdapter>(
    adapter: &mut A,
    ticket: &VerifiedTicket,
) -> Result<Outcome, FirmwareError> {
    if !ticket.explicit_operator_request {
        return Err(FirmwareError::MissingOperatorRequest);
    }
    if !ticket.independent_approval_current {
        return Err(FirmwareError::MissingIndependentApproval);
    }
    if !ticket.exact_hardware_evidence_fresh
        || !ticket.independently_authenticated_vendor_image
        || !ticket.backup_and_restore_tested
        || !ticket.impact_and_maintenance_reviewed
        || !ticket.independent_recovery_available
        || !ticket.inside_approved_window
        || ticket.target.vendor != ticket.image.vendor
        || ticket.target.exact_model != ticket.image.exact_model
    {
        return Err(FirmwareError::StaleOrIncompleteEvidence);
    }
    if !adapter.is_physically_qualified(&ticket.target, &ticket.image) {
        return Err(FirmwareError::UnsupportedExactVendorTuple);
    }
    adapter.transfer_verified_image(&ticket.target, &ticket.image)?;
    adapter.activate_in_maintenance_window(&ticket.target)?;
    let observed = adapter
        .read_back_running_version(&ticket.target)
        .map_err(|_| FirmwareError::VerificationFailedRecoveryRequired)?;
    if observed != ticket.image.target_version {
        return Err(FirmwareError::VerificationFailedRecoveryRequired);
    }
    Ok(Outcome::VerifiedTargetVersion(observed))
}

/// Deny-by-default until the exact vendor/board/image compatibility and
/// recovery procedure are physically tested; this is the sole default.
pub struct UnsupportedFirmwareAdapter;
impl FirmwareAdapter for UnsupportedFirmwareAdapter {
    fn is_physically_qualified(&self, _: &Target, _: &Image) -> bool {
        false
    }
    fn transfer_verified_image(&mut self, _: &Target, _: &Image) -> Result<(), FirmwareError> {
        Err(FirmwareError::UnsupportedExactVendorTuple)
    }
    fn activate_in_maintenance_window(&mut self, _: &Target) -> Result<(), FirmwareError> {
        Err(FirmwareError::UnsupportedExactVendorTuple)
    }
    fn read_back_running_version(&mut self, _: &Target) -> Result<String, FirmwareError> {
        Err(FirmwareError::UnsupportedExactVendorTuple)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ticket() -> VerifiedTicket {
        VerifiedTicket {
            target: Target {
                tenant: "synthetic-a".into(),
                device_id: "lab-1".into(),
                vendor: "SYNTHETIC".into(),
                exact_model: "TEST-BOARD".into(),
                running_version: "0.1".into(),
            },
            image: Image {
                object_ref: "synthetic://test-only".into(),
                sha256: "a".repeat(64),
                vendor: "SYNTHETIC".into(),
                exact_model: "TEST-BOARD".into(),
                target_version: "0.2".into(),
            },
            explicit_operator_request: true,
            independent_approval_current: true,
            exact_hardware_evidence_fresh: true,
            independently_authenticated_vendor_image: true,
            backup_and_restore_tested: true,
            impact_and_maintenance_reviewed: true,
            independent_recovery_available: true,
            inside_approved_window: true,
        }
    }
    struct SyntheticAdapter {
        calls: Vec<&'static str>,
        verified_version: String,
    }
    impl FirmwareAdapter for SyntheticAdapter {
        fn is_physically_qualified(&self, t: &Target, i: &Image) -> bool {
            t.vendor == "SYNTHETIC" && i.exact_model == "TEST-BOARD"
        }
        fn transfer_verified_image(&mut self, _: &Target, _: &Image) -> Result<(), FirmwareError> {
            self.calls.push("transfer");
            Ok(())
        }
        fn activate_in_maintenance_window(&mut self, _: &Target) -> Result<(), FirmwareError> {
            self.calls.push("activate");
            Ok(())
        }
        fn read_back_running_version(&mut self, _: &Target) -> Result<String, FirmwareError> {
            self.calls.push("readback");
            Ok(self.verified_version.clone())
        }
    }
    #[test]
    fn default_rejects_even_all_green_claims_without_vendor_driver() {
        let mut a = UnsupportedFirmwareAdapter;
        assert_eq!(
            execute_verified_firmware(&mut a, &ticket()),
            Err(FirmwareError::UnsupportedExactVendorTuple)
        );
    }
    #[test]
    fn missing_operator_click_never_reaches_mock_driver() {
        let mut t = ticket();
        t.explicit_operator_request = false;
        let mut a = SyntheticAdapter {
            calls: vec![],
            verified_version: "0.2".into(),
        };
        assert_eq!(
            execute_verified_firmware(&mut a, &t),
            Err(FirmwareError::MissingOperatorRequest)
        );
        assert!(a.calls.is_empty());
    }
    #[test]
    fn missing_approval_or_recovery_never_reaches_driver() {
        let mut t = ticket();
        t.independent_approval_current = false;
        let mut a = SyntheticAdapter {
            calls: vec![],
            verified_version: "0.2".into(),
        };
        assert_eq!(
            execute_verified_firmware(&mut a, &t),
            Err(FirmwareError::MissingIndependentApproval)
        );
        t.independent_approval_current = true;
        t.backup_and_restore_tested = false;
        assert_eq!(
            execute_verified_firmware(&mut a, &t),
            Err(FirmwareError::StaleOrIncompleteEvidence)
        );
        assert!(a.calls.is_empty());
    }
    #[test]
    fn synthetic_driver_orders_transfer_activate_readback() {
        let mut a = SyntheticAdapter {
            calls: vec![],
            verified_version: "0.2".into(),
        };
        assert_eq!(
            execute_verified_firmware(&mut a, &ticket()),
            Ok(Outcome::VerifiedTargetVersion("0.2".into()))
        );
        assert_eq!(a.calls, vec!["transfer", "activate", "readback"]);
    }
    #[test]
    fn synthetic_wrong_post_version_returns_recovery_required() {
        let mut a = SyntheticAdapter {
            calls: vec![],
            verified_version: "0.1".into(),
        };
        assert_eq!(
            execute_verified_firmware(&mut a, &ticket()),
            Err(FirmwareError::VerificationFailedRecoveryRequired)
        );
    }
}
