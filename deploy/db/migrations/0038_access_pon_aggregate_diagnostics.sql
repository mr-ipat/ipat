-- R10.07: access/PON aggregate diagnostic evidence, never direct remediation.
BEGIN;

ALTER TABLE ipat_ops.diagnostic_observations
  ADD COLUMN access_device_id uuid,
  ADD CONSTRAINT diagnostic_observations_access_device_fk
    FOREIGN KEY(tenant_id,access_device_id)
    REFERENCES ipat_ops.managed_devices(tenant_id,id);

ALTER TABLE ipat_ops.diagnostic_observations
  DROP CONSTRAINT diagnostic_observations_signal_check,
  ADD CONSTRAINT diagnostic_observations_signal_check CHECK(signal IN(
    'distribution_uplink_down','distribution_uplink_healthy',
    'subscriber_unreachable','ont_optical_los','neighbor_ont_healthy',
    'ont_optical_normal','pppoe_authentication_rejected','cwmp_inform_missing',
    'access_pon_all_configured_offline')),
  ADD CONSTRAINT diagnostic_observations_access_signal_shape CHECK(
    (signal='access_pon_all_configured_offline'
      AND subscriber_id IS NULL AND access_device_id IS NOT NULL)
    OR
    (signal<>'access_pon_all_configured_offline'
      AND access_device_id IS NULL));

CREATE FUNCTION ipat_platform.record_access_pon_aggregate_observation(
 p_tenant uuid,p_distribution uuid,p_access uuid,p_source text,
 p_observed_at timestamptz,p_evidence_sha256 text,
 p_configured integer,p_online integer,p_offline integer)
 RETURNS bigint LANGUAGE plpgsql VOLATILE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform,ipat_ops AS $body$
DECLARE saved bigint;
BEGIN
 IF p_tenant IS NULL OR p_distribution IS NULL OR p_access IS NULL
 OR p_source IS NULL OR p_observed_at IS NULL OR p_evidence_sha256 IS NULL
 OR p_observed_at>clock_timestamp()+interval '30 seconds'
 OR p_observed_at<clock_timestamp()-interval '24 hours'
 OR p_source !~ '^[A-Za-z0-9_.:-]{1,128}$'
 OR p_evidence_sha256 !~ '^[0-9a-f]{64}$'
 OR p_configured IS NULL OR p_online IS NULL OR p_offline IS NULL
 OR p_configured<=0 OR p_online<>0 OR p_offline<>p_configured
 OR p_configured>65535
 OR NOT EXISTS(SELECT 1 FROM ipat_ops.managed_devices d
    WHERE d.tenant_id=p_tenant AND d.id=p_distribution
      AND d.device_kind='router' AND d.archived_at IS NULL)
 OR NOT EXISTS(SELECT 1 FROM ipat_ops.managed_devices a
    WHERE a.tenant_id=p_tenant AND a.id=p_access
      AND a.device_kind='olt' AND a.archived_at IS NULL)
 OR NOT EXISTS(
    SELECT 1 FROM ipat_ops.managed_devices d
    JOIN ipat_ops.managed_devices a
      ON a.tenant_id=d.tenant_id AND a.pop_id=d.pop_id
    WHERE d.tenant_id=p_tenant AND d.id=p_distribution AND a.id=p_access)
 THEN RETURN NULL; END IF;

 INSERT INTO ipat_ops.diagnostic_observations(
  tenant_id,distribution_device_id,subscriber_id,access_device_id,signal,
  source_id,observed_at,evidence_sha256)
 VALUES(p_tenant,p_distribution,NULL,p_access,'access_pon_all_configured_offline',
  p_source,p_observed_at,p_evidence_sha256)
 RETURNING id INTO saved;
 RETURN saved;
EXCEPTION WHEN foreign_key_violation OR check_violation THEN
 RETURN NULL;
END $body$;
ALTER FUNCTION ipat_platform.record_access_pon_aggregate_observation(
 uuid,uuid,uuid,text,timestamptz,text,integer,integer,integer)
 OWNER TO ipat_subscriber_diag_owner;
REVOKE ALL ON FUNCTION ipat_platform.record_access_pon_aggregate_observation(
 uuid,uuid,uuid,text,timestamptz,text,integer,integer,integer) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.record_access_pon_aggregate_observation(
 uuid,uuid,uuid,text,timestamptz,text,integer,integer,integer)
 TO ipat_diag_ingest_exec;

DROP FUNCTION ipat_platform.list_diagnostic_observations(text,text,uuid,uuid);

CREATE FUNCTION ipat_platform.list_diagnostic_observations(
 p_issuer text,p_subject text,p_tenant uuid,p_distribution uuid)
 RETURNS TABLE(
  subscriber_id text,access_device_id uuid,signal text,source_id text,
  observed_epoch bigint,evidence_sha256 text)
 LANGUAGE sql STABLE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform,ipat_ops AS $body$
 SELECT o.subscriber_id,o.access_device_id,o.signal,o.source_id,
   floor(extract(epoch from o.observed_at))::bigint,o.evidence_sha256
 FROM ipat_ops.diagnostic_observations o
 WHERE o.tenant_id=p_tenant AND o.distribution_device_id=p_distribution
 AND o.observed_at>=clock_timestamp()-interval '1 hour'
 AND (
   (o.subscriber_id IS NULL AND EXISTS(
     SELECT 1 FROM ipat_ops.subscribers s
     WHERE s.tenant_id=p_tenant AND s.subscriber360_profile
       AND s.distribution_device_id=p_distribution
       AND (o.access_device_id IS NULL OR s.access_device_id=o.access_device_id)
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

COMMIT;
