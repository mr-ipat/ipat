-- READ-ONLY operator review before applying 0019. Run with privileged schema
-- migration identity, never as a tenant/API role. Code+tenant is NOT proof
-- of real site ownership; review each unresolved row against authoritative
-- customer inventory and approve each Site independently first.
SELECT d.tenant_id::text AS tenant_id,d.pop_id AS missing_site_code,
       count(*)::bigint AS referenced_device_count
 FROM ipat_ops.managed_devices d
 LEFT JOIN ipat_ops.tenant_sites s ON s.tenant_id=d.tenant_id AND s.code=d.pop_id
 WHERE s.code IS NULL GROUP BY d.tenant_id,d.pop_id ORDER BY 1,2;
