#!/usr/bin/env bash
# Compare a reference LibOVRRT DLL's named ovr_* exports to Echo's observed
# string references. This is discovery tooling, not a proof that every symbol
# is invoked at runtime.
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 /path/to/LibOVRRT64_1.dll" >&2
  exit 64
fi

root=$(cd "$(dirname "$0")/.." && pwd)
tmp=$(mktemp)
trap 'rm -f "$tmp"' EXIT
objdump -p "$1" | grep -E '[[:space:]]ovr_[A-Za-z0-9_]+$' \
  | sed -E 's/.* (ovr_[A-Za-z0-9_]+)$/\1/' | sort -u > "$tmp"

echo "Reference ovr exports: $(wc -l < "$tmp")"
echo "Echo names absent from reference:"
comm -23 <(sort -u "$root/reference/echovr-ovr-symbols.txt") "$tmp" || true
