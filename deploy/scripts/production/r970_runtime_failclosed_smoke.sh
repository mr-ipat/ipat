#!/usr/bin/env bash
# Opt-in nonroot negative-only smoke on an authorized isolated owner VPS.
# Runs the actual binary ONLY with synthetic missing/conflicting flags.
# No real IDP requests, DB credentials, provider changes or new listeners.
set -Eeuo pipefail
umask 077
[[ ${IPAT_R970_APPROVE_NEGATIVE_SMOKE:-} == YES ]] || { echo 'Explicit synthetic negative-smoke opt-in required' >&2; exit 2; }
[[ $(id -u) != 0 ]] || { echo 'Refuse root execution' >&2; exit 2; }
[[ $# == 1 && -f $1 && -x $1 && ! -L $1 ]] || { echo 'Exact existing nonroot binary path required' >&2; exit 2; }
binary=$1
[[ -z $(ss -H -lnt | grep -E '127\.0\.0\.1:(3003|3004) ' || :) ]] || { echo 'Cannot safely smoke-test while another commercial listener is present' >&2; exit 3; }
private_service_status=unknown
if command -v systemctl >/dev/null;then
  private_service_status=$(systemctl --user is-active ipat-r911-preview.service ipat-r945-connector.service | tr '\n' ' ')
  [[ "$private_service_status" == 'active active ' ]] || { echo 'Existing owner-private service is not healthy' >&2; exit 3; }
fi
tmp=$(mktemp -d "${TMPDIR:-/tmp}/ipat-r970-negative-XXXXXXXX")
trap 'rm -rf -- "$tmp"' EXIT
probe(){
  local name=$1;shift
  if env -i PATH=/usr/bin:/bin HOME="$HOME" "$@" "$binary" >"$tmp/$name.out" 2>&1;then
    echo "FAIL: unexpectedly started $name" >&2; exit 4
  fi
  [[ -z $(ss -H -lnt | grep -E '127\.0\.0\.1:(3003|3004) ' || :) ]] || {
    echo "FAIL: unexpected listener appeared after $name" >&2; exit 4;
  }
  echo "R970_NEGATIVE_$name=PASS"
}
probe ISSUER_MISSING_CONFIG IPAT_R970_OIDC_ISSUER_SERVICE=YES IPAT_R970_SINGLE_INSTANCE_OIDC=YES
probe ISSUER_DNS_VERIFIER_CONFLICT IPAT_R970_OIDC_ISSUER_SERVICE=YES IPAT_TENANT_DOMAIN_VERIFIER=YES
probe ISSUER_BUSINESS_CONFLICT IPAT_R970_OIDC_ISSUER_SERVICE=YES IPAT_R969_COMMERCIAL_SERVICE=YES
probe BUSINESS_ISSUER_ENV_PRESENCE IPAT_R969_COMMERCIAL_SERVICE=YES IPAT_R970_OIDC_ISSUER_SERVICE=NO
if command -v systemctl >/dev/null;then
  [[ $(systemctl --user is-active ipat-r911-preview.service ipat-r945-connector.service | tr '\n' ' ') == "$private_service_status" ]] || { echo 'Existing private services changed' >&2; exit 4; }
fi
echo R970_NEGATIVE_ACTUAL_BINARY_NO_PUBLIC_OR_PRIVATE_SIDE_EFFECT=PASS
