#!/usr/bin/env bash
# Root-controlled, non-installation staging. No DB, service, firewall or device changes.
set -Eeuo pipefail
umask 077
[[ $(id -u) == 0 ]] || exit 4
commit=faf0b25e6f2456820c6a088719b26d0807fc0154
digest=7e041f843668e77d46c89c2fb7cc61972b794af12f3b674beb284cba3ecf91cd
upstream=/home/openai/ipat-r1034-merged
input=/home/openai/.local/share/ipat/release-candidates/$commit/control-api
source=/root/ipat-r1036-reviewed-src-v2
out=/root/ipat-r1036-reviewed-bin
[[ ! -e "$source" && ! -L "$source" && ! -e "$out" && ! -L "$out" ]] || { echo R1036_REFUSED_DESTINATION_EXISTS; exit 4; }
[[ -d "$upstream" && ! -L "$upstream" && -f "$input" && ! -L "$input" ]] || exit 4
[[ $(git -C "$upstream" rev-parse HEAD) == "$commit" ]] || exit 4
[[ -z $(git -C "$upstream" status --porcelain --untracked-files=all --ignored=matching) ]] || exit 4
[[ $(sha256sum "$input" | awk '{print $1}') == "$digest" ]] || exit 4
git clone --quiet --no-local --no-checkout "$upstream" "$source"
git -C "$source" checkout --quiet --detach "$commit"
git -C "$source" fsck --full --no-reflogs >/dev/null
[[ $(git -C "$source" rev-parse HEAD) == "$commit" ]] || exit 4
[[ -z $(git -C "$source" status --porcelain --untracked-files=all --ignored=matching) ]] || exit 4
unsafe=$(find "$source" \( -type l -o ! -uid 0 -o -perm /022 -o ! -type f -a ! -type d \) -print -quit)
[[ -z $unsafe ]] || { echo R1036_REFUSED_UNTRUSTED_SOURCE_INODE; exit 4; }
install -d -o root -g root -m 0700 "$out"
install -o root -g root -m 0500 "$input" "$out/control-api"
[[ $(stat -c '%u:%a:%h' "$out/control-api") == '0:500:1' ]] || exit 4
[[ $(sha256sum "$out/control-api" | awk '{print $1}') == "$digest" ]] || exit 4
echo R1036_ROOT_OWNED_RELEASE_HANDOFF=PASS
echo R1036_ROOT_SOURCE_COMMIT="$commit"
echo R1036_ROOT_BINARY_SHA256="$digest"
echo R1036_POSTGRES_INSTALL=NOT_ATTEMPTED
echo R1036_PUBLIC_GO=FALSE
