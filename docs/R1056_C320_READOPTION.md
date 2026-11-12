# R10.56 C320 lab re-adoption reset

Scope: the single owner-private DEV-01 C320 linked reference only, not an actual factory reset, tenant production adoption, firmware change, or physical device configuration write.

Verified prior inventory from actual VPS: one linked DEV-01 entry, model C320, disconnected, connector credentials still enrolled, draft saved, no additional OWN-* inventory devices, production_adopted=false, physical_writes_enabled=false. Normal owner inventory Delete rejects DEV-01 because deleting metadata cannot revoke separately retained encrypted credentials.

Reset sequence after CI and exact-native deployed build:
1. Review exact service binary and source hash, stop the two user-systemd owner preview and C320 connector services.
2. Verify all private files with 0600 owner-only access and no other saved device entries. Back up encrypted credential, draft and owner inventory metadata to a unique 0700 archive.
3. Remove active encrypted credential and old draft only, retain encryption key in protected storage, record RESET_LINKED_READ_ADOPTION in durable audit, hide old DEV-01 reference until a new draft/enrollment is started, and create a fresh 0600 owner bootstrap token and its hash. Do not output any secret.
4. Restart and verify services and owner inventory: count=0; reenrollment required; connector_online=true; credentials_enrolled=false; draft_saved=false; production_adopted=false; physical_writes_enabled=false. If restart fails, restore original files and restart.
5. Re-enroll through the dedicated C320 lab Add Device connection form, using a one-time code accessed locally on authorized VPS and device password entered only in private form. Saving metadata is not connected/authenticated. A genuine read-only card verification is required before reporting CONNECTED.
6. Keep all physical writes disabled. This reset does not satisfy SaaS OIDC/MFA, other adapters, ONU provisioning or complete PRD.

Script: deploy/scripts/production/r1056/reset_c320_owner_adoption.py
Tests: deploy/scripts/production/r1056/test_reset_c320_owner_adoption.py and Rust owner registry tests.
Owner protected archive supports rollback. Never delete the overall IPAT data directory.
