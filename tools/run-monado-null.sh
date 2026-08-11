#!/usr/bin/env bash
# Start a disposable headset-free Monado runtime for OpenXR probing.
set -euo pipefail

project_root=$(cd "$(dirname "$0")/.." && pwd)
state_root="$project_root/artifacts/monado-null"
mkdir -p "$state_root"

command -v monado-service >/dev/null || {
  echo "monado-service is unavailable; enter nix-shell" >&2
  exit 1
}

# Monado's simulated HMD provides a system without physical VR hardware. The
# null compositor avoids presenting to a display. The service inherits these
# variables and clients need the same runtime JSON selection.
export XDG_CONFIG_HOME="$state_root/config"
export XDG_RUNTIME_DIR="$state_root/runtime"
export XRT_SIMULATE_HMD=1
export XRT_COMPOSITOR_NULL=1
export XR_RUNTIME_JSON="$(dirname "$(command -v monado-service)")/../share/openxr/1/openxr_monado.json"
mkdir -p "$XDG_CONFIG_HOME" "$XDG_RUNTIME_DIR"

env_file="$state_root/env.sh"
cat >"$env_file" <<EOF
export XDG_CONFIG_HOME='$XDG_CONFIG_HOME'
export XDG_RUNTIME_DIR='$XDG_RUNTIME_DIR'
export XRT_SIMULATE_HMD=1
export XRT_COMPOSITOR_NULL=1
export XR_RUNTIME_JSON='$XR_RUNTIME_JSON'
export LIBOVR_OPENXR_PROBE=1
EOF

log="$state_root/monado-service.log"
monado-service >"$log" 2>&1 &
pid=$!
printf '%s\n' "$pid" >"$state_root/monado-service.pid"

cat <<EOF
Monado null backend started (pid $pid)
XR_RUNTIME_JSON=$XR_RUNTIME_JSON
XDG_CONFIG_HOME=$XDG_CONFIG_HOME
XDG_RUNTIME_DIR=$XDG_RUNTIME_DIR
log=$log

Source this before run-game.sh: source $env_file
Stop it with: kill $pid
EOF
