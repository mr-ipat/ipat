-- R9.71 confidential OIDC HA-safe, one-use pending-state store.
-- No plaintext state, browser verifier, OIDC code, access token or password.
-- Requires 0021-0023, restricted issuer already independently provisioned.
BEGIN;
CREATE TABLE ipat_platform.oidc_pending_states (
 state_sha256 text PRIMARY KEY CHECK(state_sha256 ~ '^[0-9a-f]{64}$'),
 verifier_s256 text NOT NULL CHECK(verifier_s256 ~ '^[A-Za-z0-9_-]{43}$'),
 nonce text NOT NULL CHECK(nonce ~ '^[A-Za-z0-9_-]{43}$'),
 hostname text NOT NULL CHECK(length(hostname) BETWEEN 4 AND 253 AND hostname=lower(hostname)),
 issued_at timestamptz NOT NULL DEFAULT statement_timestamp(),
 expires_at timestamptz NOT NULL,
 CHECK(expires_at>issued_at AND expires_at<=issued_at+interval '3 minutes')
);
ALTER TABLE ipat_platform.oidc_pending_states OWNER TO ipat_schema_owner;
ALTER TABLE ipat_platform.oidc_pending_states ENABLE ROW LEVEL SECURITY;
ALTER TABLE ipat_platform.oidc_pending_states FORCE ROW LEVEL SECURITY;
REVOKE ALL ON ipat_platform.oidc_pending_states FROM PUBLIC,ipat_app_runtime,ipat_oidc_session_issuer_login;
CREATE INDEX oidc_pending_expiry ON ipat_platform.oidc_pending_states(expires_at);
CREATE ROLE ipat_oidc_pending_owner NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT;
GRANT USAGE ON SCHEMA ipat_platform TO ipat_oidc_pending_owner;
GRANT SELECT,INSERT,DELETE ON ipat_platform.oidc_pending_states TO ipat_oidc_pending_owner;
CREATE POLICY oidc_pending_owner_all ON ipat_platform.oidc_pending_states
 FOR ALL TO ipat_oidc_pending_owner USING(true) WITH CHECK(true);

-- Insert only if bounded capacity (per exact host); serialization avoids
-- simultaneous multi-replica overfill. Reusing state is never an upsert.
CREATE FUNCTION ipat_platform.begin_oidc_pending(
 p_state_hash text,p_verifier_s256 text,p_nonce text,p_host text)
 RETURNS boolean LANGUAGE plpgsql VOLATILE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform AS $body$
BEGIN
 IF p_state_hash IS NULL OR p_state_hash !~ '^[0-9a-f]{64}$'
 OR p_verifier_s256 IS NULL OR p_verifier_s256 !~ '^[A-Za-z0-9_-]{43}$'
 OR p_nonce IS NULL OR p_nonce !~ '^[A-Za-z0-9_-]{43}$'
 OR p_host IS NULL OR length(p_host) NOT BETWEEN 4 AND 253 OR p_host<>lower(p_host)
 OR p_host !~ '^[a-z0-9.-]+$' THEN RETURN false; END IF;
 PERFORM pg_advisory_xact_lock(hashtextextended('ipat/oidc/pending/'||p_host,0));
 DELETE FROM ipat_platform.oidc_pending_states WHERE expires_at<=clock_timestamp();
 IF (SELECT count(*) FROM ipat_platform.oidc_pending_states WHERE hostname=p_host)>=64
 THEN RETURN false; END IF;
 INSERT INTO ipat_platform.oidc_pending_states
  (state_sha256,verifier_s256,nonce,hostname,expires_at)
  VALUES(p_state_hash,p_verifier_s256,p_nonce,p_host,statement_timestamp()+interval '3 minutes');
 RETURN true;
EXCEPTION WHEN unique_violation THEN RETURN false;
END $body$;
ALTER FUNCTION ipat_platform.begin_oidc_pending(text,text,text,text) OWNER TO ipat_oidc_pending_owner;
REVOKE ALL ON FUNCTION ipat_platform.begin_oidc_pending(text,text,text,text) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.begin_oidc_pending(text,text,text,text)
 TO ipat_oidc_session_issuer_login;

-- Exactly one concurrent callback wins; digest equality+host checked on the
-- DB row before a token exchange may occur. Invalid verifier does not consume.
CREATE FUNCTION ipat_platform.consume_oidc_pending(
 p_state_hash text,p_verifier_s256 text,p_host text)
 RETURNS TABLE(nonce text) LANGUAGE sql VOLATILE SECURITY DEFINER
 SET search_path=pg_catalog,ipat_platform AS $body$
 DELETE FROM ipat_platform.oidc_pending_states
 WHERE state_sha256=p_state_hash AND verifier_s256=p_verifier_s256
   AND hostname=p_host AND expires_at>clock_timestamp()
 RETURNING nonce
$body$;
ALTER FUNCTION ipat_platform.consume_oidc_pending(text,text,text) OWNER TO ipat_oidc_pending_owner;
REVOKE ALL ON FUNCTION ipat_platform.consume_oidc_pending(text,text,text) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION ipat_platform.consume_oidc_pending(text,text,text)
 TO ipat_oidc_session_issuer_login;
COMMIT;
