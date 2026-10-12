#!/usr/bin/env bash
# R10.18: verified-root, READ-ONLY real-VPS foundation readiness handoff.
# Do NOT install packages, mutate firewall, run deploy, edit sudoers or devices.
set -Eeuo pipefail
umask 077
refuse(){ printf 'R1018_ROOT_PREFLIGHT_REFUSED: %s\n' "$*" >&2; exit 4; }
[[ ${EUID:-$(id -u)} == 0 ]] || refuse "interactive owner sudo/root authentication required"
[[ $(uname -s) == Linux ]] || refuse "Linux only"
self=$(readlink -f -- "$0")
[[ $self == /root/ipat-r1018-root-readiness.sh ]] || refuse "install a root-owned reviewed copy under /root first"
[[ ! -L $self && $(stat -c '%u:%a:%h' "$self") == '0:500:1' ]] ||
  refuse "root-owned 0500 single-link reviewed script required"
. /etc/os-release
[[ ${ID:-} == ubuntu && ${VERSION_ID:-} == 26.04 ]] || refuse "Ubuntu Server 26.04 required"

commit=845eeac1f039168fd08af7f860ab50413f91d85a
digest=1753d706b6d076fb67ba9509782e1e5c82a5aa673f901e9e809935bd98a66f20
src=/home/openai/ipat-release-src-r1018
binary="/home/openai/.local/share/ipat/release-candidates/$commit/control-api"
attestation=/root/ipat-evidence/r996-recovery-attestation.json
missing=0
check(){
  local label=$1
  shift
  if "$@"; then
    printf '%s=PASS\n' "$label"
  else
    printf '%s=BLOCKED\n' "$label"
    missing=1
  fi
}
source_ok(){
  [[ -d $src && ! -L $src ]] || return 1
  [[ $(git -c safe.directory="$src" -C "$src" rev-parse HEAD 2>/dev/null || true) == "$commit" ]] || return 1
  [[ -z $(git -c safe.directory="$src" -C "$src" status --porcelain --untracked-files=no 2>/dev/null) ]]
}
binary_ok(){
  [[ -f $binary && ! -L $binary ]] || return 1
  [[ $(sha256sum "$binary" | awk '{print $1}') == "$digest" ]]
}
attestation_file_ok(){
  [[ -f $attestation && ! -L $attestation ]] || return 1
  [[ $(stat -c '%u:%a:%h' "$attestation") == '0:600:1' ]]
}
no_existing_cluster(){
  if command -v pg_lsclusters >/dev/null; then
    [[ -z $(pg_lsclusters --no-header 2>/dev/null || true) ]]
  else
    true
  fi
}
no_existing_runtime(){
  local p
  for p in /var/lib/ipat/r996-foundation.json /opt/ipat/bin/control-api \
    /etc/systemd/system/ipat-platform-api.service \
    /etc/systemd/system/ipat-platform-oidc.service; do
    [[ ! -e $p && ! -L $p ]] || return 1
  done
}
no_public_listeners(){
  [[ -z $(ss -H -ltn '( sport = :80 or sport = :443 or sport = :5432 or sport = :3003 or sport = :3005 or sport = :3006 )' 2>/dev/null) ]]
}

echo IPAT_R1018_REVIEWED_READ_ONLY_ROOT_PREFLIGHT
echo ROOT_AUTHENTICATION=PASS
echo HOST_RELEASE_COMMIT="$commit"
check SOURCE_EXACT_MAIN source_ok
check CONTROL_API_HASH binary_ok
check FIRST_INSTALL_RUNTIME_UNOCCUPIED no_existing_runtime
check POSTGRES_CLUSTER_UNOCCUPIED no_existing_cluster
check PRODUCTION_PORTS_UNOCCUPIED no_public_listeners
check ROOT_OWNED_RECOVERY_ATTESTATION_METADATA attestation_file_ok
echo PROVIDER_RESCUE_AND_INDEPENDENT_HOST_RESTORE=EXTERNAL_EVIDENCE_REQUIRED
echo ROOT_ATTESTATION_CONTENT_AND_OFFSITE_RESTORE=NOT_VALIDATED_BY_THIS_PREFLIGHT
echo REAL_POSTGRES_PITR_HA_FENCING=NOT_YET_TESTED_ON_LIVE_CLUSTER
echo REAL_MFA_TLS_IP_SAN_AND_BROWSER_ISOLATION=NOT_YET_TESTED
echo MUTATIONS_ATTEMPTED=false
echo PUBLIC_GO=false
if [[ $missing == 1 ]]; then
  echo R1018_ROOT_PREFLIGHT_BLOCKED_USE_INDEPENDENT_EVIDENCE_AND_REVIEW
  exit 3
fi
echo R1018_TECHNICAL_PREFLIGHT_PASS_NOT_DEPLOYMENT_APPROVAL
echo 'NEXT=review independent rescue+restore evidence, then validate with r996_validate_recovery_attestation.py; no automatic installation'
