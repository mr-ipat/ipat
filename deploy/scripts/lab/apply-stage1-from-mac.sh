#!/usr/bin/env bash
# Run ONLY from the project owner's interactive Mac Terminal.
# It requests sudo on Ubuntu; never paste Linux passwords into ChatGPT.
set -Eeuo pipefail
umask 077

repo="$HOME/Projects/ipat-current"
server_repo="/home/openai/workspaces/ipat"
root_script="deploy/scripts/lab/stage1-root-preflight-and-toolchain.sh"

if [[ ! -t 0 ]] || [[ ! -d "$repo/.git" ]]; then
  echo "Run interactively on the authorized Mac with the IPAT repository." >&2
  exit 2
fi
backup="$(find "$HOME/IPAT-secure-backups" -name READABLE-CONFIG-PARTIAL.tar.gz -type f 2>/dev/null | sort | tail -1)"
if [[ -z "$backup" || ! -f "$backup" ]]; then
  echo "ABORT: verified partial off-host preflight backup missing." >&2
  exit 3
fi
tar -tzf "$backup" >/dev/null
echo "MAC_PARTIAL_BACKUP_VERIFIED=yes"
echo "WARNING: this is NOT a full VPS snapshot or disaster recovery backup."

if [[ -n "$(git -C "$repo" status --porcelain)" ]]; then
  echo "ABORT: dirty Mac IPAT worktree; preserve developer changes." >&2
  exit 4
fi
git -C "$repo" fetch --quiet origin main
git -C "$repo" switch main
git -C "$repo" merge --ff-only origin/main
test -s "$repo/$root_script"

ssh -T -o BatchMode=yes -o StrictHostKeyChecking=yes ipat-lab \
  'test -s /home/openai/.ssh/authorized_keys && test -d /home/openai/workspaces/ipat'
echo "SSH_PUBLIC_KEY_PREFLIGHT=PASS"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
git -C "$repo" bundle create "$tmp/ipat-main.bundle" main
git -C "$repo" bundle verify "$tmp/ipat-main.bundle" >/dev/null
expected="$(git -C "$repo" rev-parse main)"
scp -q -o BatchMode=yes -o StrictHostKeyChecking=yes \
  "$tmp/ipat-main.bundle" ipat-lab:.cache/ipat/ipat-main.bundle
ssh -T -o BatchMode=yes -o StrictHostKeyChecking=yes ipat-lab \
  'set -eu; repo="$HOME/workspaces/ipat"; test -z "$(git -C "$repo" status --porcelain)"; git -C "$repo" fetch "$HOME/.cache/ipat/ipat-main.bundle" main; git -C "$repo" merge --ff-only FETCH_HEAD; rm -f "$HOME/.cache/ipat/ipat-main.bundle"'
actual="$(ssh -T -o BatchMode=yes ipat-lab \
  'git -C /home/openai/workspaces/ipat rev-parse HEAD')"
if [[ "$actual" != "$expected" ]]; then
  echo "ABORT: Mac and VPS source hashes do not match" >&2
  exit 5
fi
echo "SOURCE_SHA_VERIFIED=$actual"
ssh -T -o BatchMode=yes ipat-lab \
  "/bin/bash '$server_repo/$root_script' --check"
echo
echo "STAGE 1 installs ONLY Ubuntu build-essential after creating a"
echo "root-only local config backup. No firewall, SSH policy, K3s, DB,"
echo "automatic reboot, or system service configuration will be changed."
echo "Provider console MUST remain accessible throughout the operation."
read -r -p 'Type APPLY to authorize this stage (anything else cancels): ' approval
if [[ "$approval" != "APPLY" ]]; then
  echo "CANCELLED: no privileged server changes."
  exit 0
fi

sha="$(shasum -a 256 "$repo/$root_script" | awk '{print $1}')"
echo "Ubuntu will request the openai Linux sudo password IN THIS TERMINAL."
ssh -tt -o BatchMode=yes -o StrictHostKeyChecking=yes ipat-lab \
  "sudo -k -v && \
   sudo install -d -o root -g root -m 0700 /root/ipat-bootstrap && \
   sudo install -o root -g root -m 0700 '$server_repo/$root_script' /root/ipat-bootstrap/stage1.sh && \
   printf '%s  %s\\n' '$sha' '/root/ipat-bootstrap/stage1.sh' | sudo sha256sum --check - && \
   sudo env IPAT_STAGE1_APPROVED=yes IPAT_EXTERNAL_PARTIAL_BACKUP_VERIFIED=yes /bin/bash /root/ipat-bootstrap/stage1.sh --apply"

echo "VERIFY: unprivileged Rust workspace on the actual Ubuntu VPS"
ssh -T -o BatchMode=yes -o StrictHostKeyChecking=yes ipat-lab \
  'set -eu; . "$HOME/.cargo/env"; cd "$HOME/workspaces/ipat"; cargo fmt --all -- --check; cargo test --workspace --locked; echo STAGE1_LOCAL_TESTS_PASS'
echo "STAGE1_DONE: do not start network services before next security gate."
