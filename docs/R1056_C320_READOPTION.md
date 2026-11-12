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


## Verified Mac access and new enrollment (R10.57)

On the approved owner Mac, bind exactly the local port 3002 because the owner-private API requires Host and Origin 127.0.0.1:3002 (the older port13002 instructions support viewing HTML but not protected owner inventory actions):

    ssh -o ExitOnForwardFailure=yes -N -L 127.0.0.1:3002:127.0.0.1:3002 ipat-lab

Then open:

    http://127.0.0.1:3002/lab/device-workbench

The actual Mac Chrome browser and owner API were tested over this tunnel, showing 0 managed records and C320 reset complete. The tunnel is owner-private only; do not expose 3002 on a public interface. In the dedicated C320 connection form, use its Copy Command to fetch the new one-time owner code into your local clipboard (never chat or Git), and enter the lab OLT SSH password in the private browser. Saving a draft alone is not proof of connection. On successful enrollment, the linked DEV-01 reappears without a duplicate; verified physical CLI read results still need separate acceptance. Generic owner device CRUD now rejects any duplicate C320 host (HTTP 409). Real SaaS login and broader device-adapter coverage remain incomplete.
