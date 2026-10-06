#!/usr/bin/env bash
# R9.99 final per-customer release evidence marker after REAL human browser acceptance.
# Does not alter DNS, ingress, database lifecycle, firewall or device state.
set -Eeuo pipefail
umask 077

die(){ echo "R999_RELEASE_REFUSED: $*" >&2; exit 4; }

[[ ${EUID:-$(id -u)} -eq 0 ]] || die "root required"
[[ ${IPAT_R999_FINALIZE:-} == FINALIZE_REVIEWED_CUSTOMER_RELEASE ]] || die "explicit final release opt-in missing"

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)
domain=${IPAT_R999_DOMAIN_ID:-}
host=${IPAT_R999_HOST:-}
acceptance=${IPAT_R999_BROWSER_ACCEPTANCE:-}
activation=${IPAT_R999_ACTIVATION_MARKER:-/var/lib/ipat/customer-activations/$domain.json}
db=${IPAT_R999_DB_NAME:-}
sock=${IPAT_R999_DB_SOCKET:-}

[[ $domain =~ ^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$ ]] || die "invalid exact domain UUID"
[[ -n $host && -n $acceptance ]] || die "exact host and acceptance required"
[[ $db =~ ^[A-Za-z0-9_]{1,63}$ && $sock == /* && -d $sock && ! -L $sock ]] || die "unsafe PostgreSQL target"

for c in python3 stat runuser psql install mktemp rm; do
  command -v "$c" >/dev/null || die "missing tool: $c"
done

for f in "$activation" "$acceptance"; do
  [[ -f $f && ! -L $f ]] || die "unsafe evidence file: $f"
  [[ $(stat -c '%u:%a:%h' "$f") == 0:600:1 ]] || die "evidence file must be root-owned 0600 single-link: $f"
done

validated=$(python3 "$root/deploy/scripts/production/r999_validate_customer_browser_acceptance.py"   --activation "$activation" --acceptance "$acceptance"   --domain-id "$domain" --hostname "$host")   || die "real customer browser/MFA acceptance rejected"

psql_cmd=(psql -X -w -AtF $'\t' -v ON_ERROR_STOP=1 -h "$sock" -d "$db" -U ipat_domain_ingress_verifier_login)
row=$(runuser -u ipatdverify -- "${psql_cmd[@]}" -c  "SELECT id::text,hostname,activation_state,COALESCE(activated_at::text,'')
    FROM ipat_platform.get_tenant_domain_activation_result('$domain'::uuid)")  || die "cannot read exact active customer result"

IFS=$'\t' read -r got_id got_host state activated_at <<<"$row"
[[ $got_id == "$domain" && $got_host == "$host" && $state == active && -n $activated_at ]]   || die "database exact customer is not active"

install -d -o root -g root -m 0700 /var/lib/ipat/customer-releases
marker="/var/lib/ipat/customer-releases/$domain.json"
[[ ! -e $marker && ! -L $marker ]] || die "existing release marker requires explicit separate review"

tmp=$(mktemp /var/lib/ipat/customer-releases/.r999.pending.XXXXXX)
python3 - "$tmp" "$validated" "$activated_at" <<'PY'
import json,sys
from datetime import datetime,timezone
p,validated,activated_at=sys.argv[1:]
v=json.loads(validated)
with open(p,"w") as f:
    json.dump({
      "schema":"ipat.customer-release.v1",
      "domain_id":v["domain_id"],
      "hostname":v["hostname"],
      "tenant_id":v["tenant_id"],
      "control_tenant_id":v["control_tenant_id"],
      "control_hostname":v["control_hostname"],
      "browser_evidence_id":v["evidence_id"],
      "browser_artifact_sha256":v["artifact_sha256"],
      "domain_activated_at":activated_at,
      "released_at_utc":datetime.now(timezone.utc).isoformat(),
      "customer_release_go":True
    },f,sort_keys=True)
PY
install -o root -g root -m 0600 "$tmp" "$marker"
rm -f "$tmp"

echo "R999_EXACT_CUSTOMER_RELEASE_GO domain_id=$domain host=$host"
