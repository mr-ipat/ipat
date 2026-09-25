#!/usr/bin/env bash
# Run in authorized Mac Terminal. Never paste the Ubuntu sudo password into chat.
# --backup-root streams Ubuntu root-read configuration directly into encrypted
# Restic via SSH: NO unencrypted tar file on the VPS or Mac during backup.
set -Eeuo pipefail
umask 077

mode="${1:-}"
[[ "$#" -eq 1 ]] || { echo "Usage: $0 --smoke|--backup-root|--verify-root|--status" >&2; exit 2; }
case "$mode" in
  --smoke|--backup-root|--verify-root|--status) ;;
  *) echo "Invalid mode" >&2; exit 2 ;;
esac
[[ "$(uname -s)" == Darwin ]] || { echo "Mac operator only" >&2; exit 3; }

repo="$HOME/IPAT-secure-backups/restic-lab-v1"
source="$HOME/Projects/ipat-current"
producer="$source/deploy/scripts/lab/root-config-stream.py"
export RESTIC_PASSWORD_COMMAND='/usr/bin/security find-generic-password -a ipat-lab-backup -s id.ipat.lab.restic.backup.v1 -w'
RESTORE_TMP=""
cleanup() {
  if [[ -n "${RESTORE_TMP:-}" && -d "$RESTORE_TMP" ]]; then
    rm -rf -- "$RESTORE_TMP"
  fi
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
test -s "$repo/config" && test -f "$producer" || { echo "Restic repo or producer missing" >&2; exit 4; }
[[ "$(stat -f '%Lp' "$repo")" == 700 ]] || { echo "Restic repo requires 0700 mode" >&2; exit 4; }
/usr/bin/fdesetup status | grep -Fxq "FileVault is On." ||
  { echo "FileVault is not confirmed ON; refusing sensitive backup" >&2; exit 5; }

find_latest() {
  local tag="$1"
  restic -r "$repo" snapshots --json |
    python3 -c '
import json,re,sys
tag=sys.argv[1]
pattern=r"^/ipat-ubuntu-root-config-[0-9]{8}T[0-9]{6}Z\.tar\.gz$" if tag=="root-config" else r"^/ipat-unprivileged-stream-[0-9]{8}T[0-9]{6}Z\.tar\.gz$"
snapshots=[
  (x,p) for x in json.load(sys.stdin)
  if tag in (x.get("tags") or []) for p in (x.get("paths") or [])
  if re.fullmatch(pattern,p)
]
if not snapshots:
    sys.exit("No matching encrypted snapshot found")
snapshot,path=max(snapshots,key=lambda item:item[0]["time"])
print(snapshot["id"],path.lstrip("/"),sep="\t")
' "$tag"
}
restore_check() {
  local tag="$1" expected_root="$2" snapshot name tmp archive
  IFS=$'\t' read -r snapshot name < <(find_latest "$tag")
  [[ -n "$snapshot" && -n "$name" ]] ||
    { echo "Missing matching encrypted snapshot" >&2; exit 6; }
  tmp="$(mktemp -d "$HOME/IPAT-secure-backups/restore-root-test.XXXXXXXX")"
  RESTORE_TMP="$tmp"
  restic -r "$repo" check --read-data
  restic -r "$repo" restore "$snapshot" --target "$tmp"
  archive="$tmp/$name"
  [[ -s "$archive" ]] || { echo "Restored stream is missing" >&2; exit 7; }
  # A successful gzip CRC + TAR archive listing validates the actual decrypted
  # restored stream; do not print sensitive file contents.
  tar -tzf "$archive" > "$tmp/archive-member-list.txt"
  grep -Fxq 'etc/os-release' "$tmp/archive-member-list.txt"
  grep -Fxq 'etc/ssh/sshd_config' "$tmp/archive-member-list.txt"
  if [[ "$expected_root" == yes ]]; then
    grep -Fxq 'etc/sudoers' "$tmp/archive-member-list.txt"
    grep -Fxq 'home/openai/.ssh/authorized_keys' "$tmp/archive-member-list.txt"
    grep -Fxq 'etc/ssh/sshd_config.d/00-ipat-lab-hardening.conf' "$tmp/archive-member-list.txt"
    tar -xOzf "$archive" etc/ssh/sshd_config.d/00-ipat-lab-hardening.conf |
      grep -Fxq 'PermitRootLogin no'
    tar -xOzf "$archive" etc/ssh/sshd_config.d/00-ipat-lab-hardening.conf |
      grep -Fxq 'PasswordAuthentication no'
    echo "ENCRYPTED_ROOT_READABLE_CONFIG_ISOLATED_RESTORE=PASS"
    echo "ROOT_ONLY_SUDOERS_PRESENT_IN_RESTORED_TAR=PASS"
    echo "HOST_SSH_PRIVATE_KEYS_AND_ALL_APPLICATION_STATE=NOT_INCLUDED"
  else
    echo "UNPRIVILEGED_SSH_STREAM_ISOLATED_RESTORE=PASS"
  fi
  echo "RESTIC_FULL_PACK_READ=PASS"
  rm -rf -- "$tmp"
  RESTORE_TMP=""
  echo "PRIVATE_PLAINTEXT_RESTORE_REMOVED=PASS"
}

case "$mode" in
  --status)
    restic -r "$repo" snapshots --compact
    echo "VPS_ROOT_CONFIG_RESTORE=RUN --verify-root TO CHECK"
    ;;
  --smoke)
    name="ipat-unprivileged-stream-$(date -u +%Y%m%dT%H%M%SZ).tar.gz"
    restic -r "$repo" backup --tag ipat-lab,root-stream-smoke-unprivileged \
      --stdin-filename "$name" --stdin-from-command -- \
      python3 "$producer" --smoke
    restore_check "root-stream-smoke-unprivileged" no
    echo "TEST_ONLY=ROOT_PRIVILEGES_NOT_USED"
    ;;
  --backup-root)
    [[ -t 0 ]] || { echo "Interactive owner Mac Terminal required" >&2; exit 8; }
    [[ "$(git -C "$source" branch --show-current)" == main ]] ||
      { echo "Switch to reviewed main first" >&2; exit 8; }
    [[ -z "$(git -C "$source" status --porcelain)" ]] ||
      { echo "Git source has uncommitted changes" >&2; exit 8; }
    local_sha="$(git -C "$source" rev-parse HEAD)"
    github_sha="$(gh api repos/mr-ipat/ipat/commits/main --jq .sha)"
    [[ "$local_sha" == "$github_sha" ]] ||
      { echo "Mac main differs from private GitHub main: no privileged backup" >&2; exit 8; }
    ssh -T -o BatchMode=yes -o StrictHostKeyChecking=yes -o ControlMaster=no \
      -o ControlPath=none ipat-lab \
      'test "$(id -un)" = openai && systemctl is-active --quiet ssh &&
       test -s "$HOME/.ssh/authorized_keys" &&
       test -r /etc/ssh/sshd_config.d/00-ipat-lab-hardening.conf' ||
      { echo "Fresh public-key-only preflight failed" >&2; exit 9; }
    printf '%s\n' \
      'This operation READS SENSITIVE ROOT CONFIG over encrypted SSH and' \
      'immediately writes the archive into encrypted Mac Restic; no plaintext' \
      'tar file is created during backup.' \
      'Ubuntu root SSH PRIVATE HOST KEYS and app/database data are NOT included.' \
      'The Keychain-held repository password was escrowed outside this Mac' \
      'according to the project owner; independent access was NOT verified.' \
      'This operation DOES NOT modify firewall, SSH, packages or K3s.'
    read -r -p 'Type ESCROW_CONFIRMED_AND_BACKUP_ROOT to continue: ' confirmation
    [[ "$confirmation" == ESCROW_CONFIRMED_AND_BACKUP_ROOT ]] ||
      { echo "No privileged backup performed" >&2; exit 9; }
    name="ipat-ubuntu-root-config-$(date -u +%Y%m%dT%H%M%SZ).tar.gz"
    restic -r "$repo" backup --tag ipat-lab,root-config \
      --stdin-filename "$name" --stdin-from-command -- \
      python3 "$producer" --root
    echo "ENCRYPTED_ROOT_CONFIG_STREAM_SNAPSHOT_CREATED"
    restore_check root-config yes
    echo "ROOT_CONFIG_PARTIAL_DISASTER_RECOVERY_BACKUP_VERIFIED"
    ;;
  --verify-root)
    restore_check root-config yes
    ;;
esac
