-- R10.06: deterministic epoch serialization for diagnostic/browser boundaries.
-- PostgreSQL numeric->bigint casts round fractional seconds. Browser/runtime
-- freshness uses integer Unix seconds that MUST NOT round into the future.
BEGIN;

CREATE OR REPLACE FUNCTION ipat_platform.list_subscriber360(
 p_issuer text,p_subject text,p_tenant uuid)
 RETURNS TABLE(
  subscriber_id text,display_name text,pppoe_username text,pop_code text,
  site_code text,distribution_device_id uuid,access_device_id uuid,
  ont_reference text,revision bigint,topology_state text,
  topology_source_id text,topology_verified_epoch bigint)
 LANGUAGE sql STABLE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform,ipat_ops AS $body$
 SELECT s.customer_ref,s.display_name,s.pppoe_username,s.pop_id,s.site_code,
   s.distribution_device_id,s.access_device_id,s.ont_reference,s.revision,
   s.topology_state,s.topology_source_id,
   CASE WHEN s.topology_verified_at IS NULL THEN NULL
        ELSE floor(extract(epoch from s.topology_verified_at))::bigint END
 FROM ipat_ops.subscribers s
 WHERE s.tenant_id=p_tenant AND s.subscriber360_profile
 AND ipat_platform.can_read_subscriber_scope(
   p_issuer,p_subject,p_tenant,s.pop_id,s.site_code)
 ORDER BY s.customer_ref LIMIT 500
$body$;
ALTER FUNCTION ipat_platform.list_subscriber360(text,text,uuid)
 OWNER TO ipat_subscriber_diag_owner;
REVOKE ALL ON FUNCTION ipat_platform.list_subscriber360(text,text,uuid) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.list_subscriber360(text,text,uuid)
 TO ipat_tenant_api_exec;

CREATE OR REPLACE FUNCTION ipat_platform.list_diagnostic_observations(
 p_issuer text,p_subject text,p_tenant uuid,p_distribution uuid)
 RETURNS TABLE(
  subscriber_id text,signal text,source_id text,
  observed_epoch bigint,evidence_sha256 text)
 LANGUAGE sql STABLE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform,ipat_ops AS $body$
 SELECT o.subscriber_id,o.signal,o.source_id,
   floor(extract(epoch from o.observed_at))::bigint,o.evidence_sha256
 FROM ipat_ops.diagnostic_observations o
 WHERE o.tenant_id=p_tenant AND o.distribution_device_id=p_distribution
 AND o.observed_at>=clock_timestamp()-interval '1 hour'
 AND (
   (o.subscriber_id IS NULL AND EXISTS(
     SELECT 1 FROM ipat_ops.subscribers s
     WHERE s.tenant_id=p_tenant AND s.subscriber360_profile
       AND s.distribution_device_id=p_distribution
       AND ipat_platform.can_read_subscriber_scope(
         p_issuer,p_subject,p_tenant,s.pop_id,s.site_code)))
   OR
   (o.subscriber_id IS NOT NULL AND EXISTS(
     SELECT 1 FROM ipat_ops.subscribers s
     WHERE s.tenant_id=p_tenant AND s.subscriber360_profile
       AND s.customer_ref=o.subscriber_id
       AND s.distribution_device_id=p_distribution
       AND ipat_platform.can_read_subscriber_scope(
         p_issuer,p_subject,p_tenant,s.pop_id,s.site_code)))
 )
 ORDER BY o.observed_at DESC,o.id DESC LIMIT 1024
$body$;
ALTER FUNCTION ipat_platform.list_diagnostic_observations(text,text,uuid,uuid)
 OWNER TO ipat_subscriber_diag_owner;
REVOKE ALL ON FUNCTION ipat_platform.list_diagnostic_observations(
 text,text,uuid,uuid) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.list_diagnostic_observations(
 text,text,uuid,uuid) TO ipat_tenant_api_exec;

COMMIT;
