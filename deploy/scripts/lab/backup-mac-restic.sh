#!/usr/bin/env bash
# Mac-only IPAT LAB backup. Encryption password is held by macOS Keychain,
# never in a command-line argument, env value, Git repository or chat.
# This backs up reviewed SOURCE and the existing PARTIAL readable config.
# It is NOT a complete VPS/DB/K3s backup and does not enable cluster install.
set -Eeuo pipefail
umask 077

mode="${1:-}"
[[ "$#" == 1 ]] || { echo "Usage: $0 --status|--backup-partial|--backup-source|--verify" >&2; exit 2; }
case "$mode" in
  --status|--backup-partial|--backup-source|--verify) ;;
  *) echo "Unsupported mode (no action taken)" >&2; exit 2 ;;
esac
if [[ "$(uname -s)" != Darwin ]]; then
  echo "Mac operator only. Backups must not reside on the VPS itself." >&2
  exit 3
fi
repo="$HOME/IPAT-secure-backups/restic-lab-v1"
source="$HOME/Projects/ipat-current"
partial="$HOME/IPAT-secure-backups/readonly-config-20260925T073601Z/READABLE-CONFIG-PARTIAL.tar.gz"
expected_partial_sha=2d74782d0e418262701107be01e91c0ea87d3b7406803ca58acdb91a57338a77
export RESTIC_PASSWORD_COMMAND='/usr/bin/security find-generic-password -a ipat-lab-backup -s id.ipat.lab.restic.backup.v1 -w'

command -v restic >/dev/null || { echo "Restic not installed on Mac" >&2; exit 4; }
[[ -r "$repo/config" && -d "$repo/data" ]] ||
  { echo "Missing reviewed encrypted repository" >&2; exit 5; }
[[ "$(stat -f '%Lp' "$repo")" == 700 ]] ||
  { echo "Repository directory must be mode 700" >&2; exit 6; }

check_source() {
  [[ -d "$source/.git" ]] || { echo "Canonical source not found" >&2; exit 7; }
  [[ "$(git -C "$source" branch --show-current)" == main ]] ||
    { echo "Switch to reviewed main before backup" >&2; exit 7; }
  [[ -z "$(git -C "$source" status --porcelain)" ]] ||
    { echo "Uncommitted source changes: refusing ambiguous backup" >&2; exit 7; }
}
check_partial() {
  [[ -s "$partial" ]] || { echo "Original partial config archive absent" >&2; exit 8; }
  local observed
  observed="$(shasum -a 256 "$partial" | awk '{print $1}')"
  [[ "$observed" == "$expected_partial_sha" ]] ||
    { echo "Original partial archive digest differs from verified source" >&2; exit 8; }
  tar -tzf "$partial" >/dev/null
}
find_snapshot() {
  # Inputs: tag and exact path recorded by restic. Prints only a snapshot ID.
  local tag="$1" path="$2"
  restic -r "$repo" snapshots --json |
    python3 -c '
import json,sys
tag,path=sys.argv[1:3]
candidates=[
  x for x in json.load(sys.stdin)
  if tag in (x.get("tags") or []) and path in (x.get("paths") or [])
]
if not candidates:
    sys.exit("No verified snapshot exists for this requested source/path")
print(sorted(candidates,key=lambda x:x["time"])[-1]["id"])
' "$tag" "$path"
}

case "$mode" in
  --status)
    restic version
    restic -r "$repo" snapshots --compact
    /usr/bin/fdesetup status || true
    echo "WARNING: repository and Keychain secret are on the SAME Mac."
    echo "Make a separate secure recovery-key escrow and enable FileVault."
    ;;
  --backup-partial)
    check_partial
    restic -r "$repo" backup --tag ipat-lab,stage1-existing-partial "$partial"
    echo "ENCRYPTED_EXISTING_PARTIAL_CONFIG_BACKUP_COMPLETE"
    ;;
  --backup-source)
    check_source
    sha="$(git -C "$source" rev-parse HEAD)"
    restic -r "$repo" backup --tag ipat-lab,canonical-source-main \
      --stdin-filename "ipat-canonical-main-${sha}.tar" \
      --stdin-from-command -- git -C "$source" archive --format=tar main
    echo "ENCRYPTED_MAIN_SOURCE_BACKUP_COMPLETE_COMMIT=$sha"
    ;;
  --verify)
    check_source
    # Full pack read is important; metadata-only success is insufficient.
    restic -r "$repo" check --read-data
    sha="$(git -C "$source" rev-parse HEAD)"
    snap_source="$(find_snapshot canonical-source-main "/ipat-canonical-main-${sha}.tar")"
    snap_partial="$(find_snapshot stage1-existing-partial "$partial")"
    testdir="$(mktemp -d "$HOME/IPAT-secure-backups/restore-check.XXXXXXXX")"
    trap 'rm -rf "$testdir"' EXIT
    restic -r "$repo" restore "$snap_source" --target "$testdir/source"
    restored="$testdir/source/ipat-canonical-main-${sha}.tar"
    [[ -s "$restored" ]] || { echo "Source restore missing" >&2; exit 9; }
    actual_sha="$(shasum -a 256 "$restored" | awk '{print $1}')"
    expected_sha="$(git -C "$source" archive --format=tar main | shasum -a 256 | awk '{print $1}')"
    [[ "$actual_sha" == "$expected_sha" ]] ||
      { echo "Source tar content mismatch after independent restore" >&2; exit 10; }
    tar -tf "$restored" | grep -Fqx IPAT_PROJECT_BRIEF.md
    tar -tf "$restored" | grep -Fqx docs/DECISIONS.md
    tar -tf "$restored" | grep -Fqx docs/PROJECT_STATUS.md
    restic -r "$repo" restore "$snap_partial" --target "$testdir/partial"
    restored_partial="$testdir/partial$partial"
    [[ -s "$restored_partial" ]] ||
      { echo "Partial config restore missing" >&2; exit 11; }
    [[ "$(shasum -a 256 "$restored_partial" | awk '{print $1}')" == "$expected_partial_sha" ]] ||
      { echo "Partial config archive digest mismatch" >&2; exit 12; }
    tar -tzf "$restored_partial" >/dev/null
    echo "RESTIC_READ_ALL_PACKS=PASS"
    echo "ISOLATED_MAIN_SOURCE_RESTORE_SHA256=PASS"
    echo "ISOLATED_PARTIAL_CONFIG_RESTORE_SHA256=PASS"
    echo "TEMPORARY_PLAINTEXT_RESTORE_REMOVED_ON_EXIT"
    echo "VERIFIED_SCOPE: this mode validates Git source and historical readable-config ONLY."
    echo "Verify selected encrypted root configuration separately with mac-root-config-backup.sh --verify-root."
    echo "LIMITATION: whole-host, PostgreSQL and K3s datastore recovery remain unverified."
    ;;
esac
