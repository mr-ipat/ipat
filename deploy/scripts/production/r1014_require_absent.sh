#!/usr/bin/env bash
# R10.14 shared, strictly non-mutating preflight for rollback-owned artifacts.
# Caller supplies fixed trusted paths, never browser or customer text directly.
r1014_require_absent(){
  local path
  [[ "$#" -gt 0 ]] || return 1
  for path in "$@"; do
    [[ -n "$path" ]] || return 1
    if [[ -e "$path" || -L "$path" ]]; then
      printf 'R1014_PREEXISTING_ARTIFACT_REFUSED\n' >&2
      return 1
    fi
  done
  return 0
}
