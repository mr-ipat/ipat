-- Second-phase independent maintenance migration. Never auto-map old Sites.
-- Production apply requires operator preflight, separate recovery and change
-- window; this code must not install itself into a live tenant database.
BEGIN;
DO $check$
BEGIN
 IF EXISTS(
  SELECT 1 FROM ipat_ops.tenant_sites s
  LEFT JOIN ipat_ops.tenant_pops p
  ON p.tenant_id=s.tenant_id AND p.code=s.parent_pop_code
  WHERE s.parent_pop_code IS NOT NULL AND p.code IS NULL
 ) THEN
  RAISE EXCEPTION 'Unreviewed or foreign tenant Site-POP association';
 END IF;
END $check$;
ALTER TABLE ipat_ops.tenant_sites
 VALIDATE CONSTRAINT tenant_sites_parent_tenant_pop;
COMMIT;
