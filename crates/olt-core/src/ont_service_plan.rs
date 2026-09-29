//! Offline service-profile review for a *single* C320 ONT. NEVER emits CLI.
//! VLAN/TCONT/GEM support and exact ONT capability require actual firmware tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WanMode {
    TransparentBridge,
    TaggedBridge,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServicePlan {
    pub ont_draft_key: String,
    pub wan_mode: WanMode,
    pub service_vlan: u16,
    pub customer_vlan: Option<u16>,
    pub ethernet_port: u8,
    pub tcont_ref: String,
    pub gem_ref: String,
    pub upstream_profile_ref: String,
    pub downstream_profile_ref: String,
}
#[derive(Debug, PartialEq, Eq)]
pub enum ServiceError {
    Invalid,
    Unsupported,
    NeedsVerifiedDeviceProfile,
}

fn reference(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 48
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
}
/// Only formal/synthetic review; the external capability catalog must itself
/// be sourced from independently verified exact model+firmware inventory.
pub fn validate_service(
    plan: &ServicePlan,
    verified_device_profile: bool,
    discovered_ethernet_ports: &[u8],
    verified_tcont_refs: &[String],
    verified_gem_refs: &[String],
) -> Result<(), ServiceError> {
    if !reference(&plan.ont_draft_key)
        || !reference(&plan.tcont_ref)
        || !reference(&plan.gem_ref)
        || !reference(&plan.upstream_profile_ref)
        || !reference(&plan.downstream_profile_ref)
        || !(1..=4094).contains(&plan.service_vlan)
        || plan.customer_vlan.is_some_and(|v| !(1..=4094).contains(&v))
        || !(1..=8).contains(&plan.ethernet_port)
    {
        return Err(ServiceError::Invalid);
    }
    if plan.wan_mode == WanMode::TransparentBridge && plan.customer_vlan.is_some() {
        return Err(ServiceError::Invalid);
    }
    if plan.wan_mode == WanMode::TaggedBridge && plan.customer_vlan.is_none() {
        return Err(ServiceError::Invalid);
    }
    if !verified_device_profile {
        return Err(ServiceError::NeedsVerifiedDeviceProfile);
    }
    if !discovered_ethernet_ports.contains(&plan.ethernet_port)
        || !verified_tcont_refs.contains(&plan.tcont_ref)
        || !verified_gem_refs.contains(&plan.gem_ref)
    {
        return Err(ServiceError::Unsupported);
    }
    Ok(())
}

pub const ONT_SERVICE_WRITES_ENABLED: bool = false;
#[cfg(test)]
mod tests {
    use super::*;
    fn p() -> ServicePlan {
        ServicePlan {
            ont_draft_key: "draft-1".into(),
            wan_mode: WanMode::TaggedBridge,
            service_vlan: 600,
            customer_vlan: Some(600),
            ethernet_port: 1,
            tcont_ref: "tcont-a".into(),
            gem_ref: "gem-a".into(),
            upstream_profile_ref: "up-a".into(),
            downstream_profile_ref: "down-a".into(),
        }
    }
    fn check(p: &ServicePlan, verified: bool) -> Result<(), ServiceError> {
        validate_service(p, verified, &[1, 2], &["tcont-a".into()], &["gem-a".into()])
    }
    #[test]
    fn requires_verified_model_and_never_enables_olt_writes() {
        assert_eq!(
            check(&p(), false),
            Err(ServiceError::NeedsVerifiedDeviceProfile)
        );
        assert_eq!(check(&p(), true), Ok(()));
        assert!(!ONT_SERVICE_WRITES_ENABLED);
    }
    #[test]
    fn rejects_unsupported_and_invalid_service_fields() {
        let mut v = p();
        v.ethernet_port = 3;
        assert_eq!(check(&v, true), Err(ServiceError::Unsupported));
        v = p();
        v.service_vlan = 4095;
        assert_eq!(check(&v, true), Err(ServiceError::Invalid));
        v = p();
        v.customer_vlan = None;
        assert_eq!(check(&v, true), Err(ServiceError::Invalid));
        v = p();
        v.tcont_ref = "tcont-unknown".into();
        assert_eq!(check(&v, true), Err(ServiceError::Unsupported));
    }
    #[test]
    fn bridge_modes_are_separate_from_routed_ont_wan_configuration() {
        let mut v = p();
        v.wan_mode = WanMode::TransparentBridge;
        assert_eq!(check(&v, true), Err(ServiceError::Invalid));
        v.customer_vlan = None;
        assert_eq!(check(&v, true), Ok(()));
        v.upstream_profile_ref = "bad;reboot".into();
        assert_eq!(check(&v, true), Err(ServiceError::Invalid));
    }
}
