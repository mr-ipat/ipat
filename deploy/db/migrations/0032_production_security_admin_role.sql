-- R9.79: production-shaped security_admin identity role support.
-- This replaces only the restrictive role CHECK. It does NOT import R8.4 lab
-- reviewer tables/roles and does NOT expand ordinary tenant membership lookup.
BEGIN;
ALTER TABLE ipat_platform.identity_memberships
 ADD CONSTRAINT identity_memberships_role_v2_check
 CHECK (role IN ('tenant_admin','noc_engineer','helpdesk','auditor','security_admin'))
 NOT VALID;
ALTER TABLE ipat_platform.identity_memberships
 VALIDATE CONSTRAINT identity_memberships_role_v2_check;
ALTER TABLE ipat_platform.identity_memberships
 DROP CONSTRAINT identity_memberships_role_check;
ALTER TABLE ipat_platform.identity_memberships
 RENAME CONSTRAINT identity_memberships_role_v2_check TO identity_memberships_role_check;
COMMIT;
