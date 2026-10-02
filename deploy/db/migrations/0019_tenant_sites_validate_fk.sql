-- Explicit second-phase, one-way commercial Site release gate.
-- The operator first reconciles ALL historical managed_devices POP references
-- to independently authorized tenant-owned Sites after 0018. Never invent
-- POP-A/DEFAULT to make this migration pass. If unresolved, BEGIN rolls back.
BEGIN;
DO $body$
BEGIN
 IF EXISTS(SELECT 1 FROM ipat_ops.managed_devices d LEFT JOIN ipat_ops.tenant_sites s
     ON s.tenant_id=d.tenant_id AND s.code=d.pop_id WHERE s.code IS NULL)
 THEN RAISE EXCEPTION 'EXPLICIT_TENANT_SITE_BACKFILL_REQUIRED_BEFORE_FK_VALIDATION'; END IF;
END $body$;
ALTER TABLE ipat_ops.managed_devices VALIDATE CONSTRAINT managed_devices_registered_site_fk;
DO $body$
BEGIN
 IF NOT EXISTS(SELECT 1 FROM pg_constraint
    WHERE conrelid='ipat_ops.managed_devices'::regclass
      AND conname='managed_devices_registered_site_fk' AND convalidated)
 THEN RAISE EXCEPTION 'TENANT_SITE_FK_VALIDATION_FAILED'; END IF;
END $body$;
COMMIT;
