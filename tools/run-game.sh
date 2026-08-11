#!/usr/bin/env bash
# Build/deploy the development shim and launch Echo VR through Steam Proton.
# Run from `nix-shell`; shell.nix supplies steam-run and discovers Proton.
set -euo pipefail

project_root=$(cd "$(dirname "$0")/.." && pwd)
game_root=$(cd "$project_root/.." && pwd)
game_exe="$game_root/bin/win10/echovr.exe"
shim_source="$project_root/target/x86_64-pc-windows-gnu/debug/libovr_openxr.dll"
bypass_source="$project_root/target/x86_64-pc-windows-gnu/debug/ovr_loader_bypass.dll"
launcher_source="$project_root/target/x86_64-pc-windows-gnu/debug/ovr-loader-launcher.exe"
shim_target="$game_root/bin/win10/LibOVRRT64_1.dll"
platform_target="$game_root/bin/win10/LibOVRPlatform64_1.dll"
launcher_target="$game_root/bin/win10/ovr-loader-launcher.exe"
bypass_target="$game_root/bin/win10/ovr-loader-bypass.dll"
prefix_root="${LIBOVR_OPENXR_PREFIX:-$project_root/artifacts/proton-prefix}/pfx"
oculus_runtime_dir="$prefix_root/drive_c/Program Files/Oculus/Support/oculus-runtime"

if [[ "${1:-}" == "--no-build" ]]; then
  shift
else
  (cd "$project_root" && "$project_root/tools/build-windows.sh")
fi

[[ -f "$game_exe" ]] || { echo "missing game executable: $game_exe" >&2; exit 1; }
[[ -f "$shim_source" ]] || { echo "missing shim output: $shim_source" >&2; exit 1; }
[[ -f "$bypass_source" ]] || { echo "missing loader bypass output: $bypass_source" >&2; exit 1; }
[[ -f "$launcher_source" ]] || { echo "missing loader launcher output: $launcher_source" >&2; exit 1; }
[[ -n "${LIBOVR_OPENXR_PROTON:-}" ]] || { echo "LIBOVR_OPENXR_PROTON is unset; enter nix-shell first" >&2; exit 1; }
command -v steam-run >/dev/null || { echo "steam-run is unavailable; enter nix-shell first" >&2; exit 1; }

# Echo's CAPI loader first discovers the Oculus installation via this registry
# key, then loads Support\\oculus-runtime\\LibOVRRT64_1.dll from that base.
# Keep a side-by-side copy too, as it is useful for loaders using normal DLL
# search order.
cp "$shim_source" "$shim_target"
cp "$shim_source" "$platform_target"
cp "$launcher_source" "$launcher_target"
cp "$bypass_source" "$bypass_target"
mkdir -p "$oculus_runtime_dir"
cp "$shim_source" "$oculus_runtime_dir/LibOVRRT64_1.dll"
cp "$shim_source" "$oculus_runtime_dir/LibOVRPlatform64_1.dll"
# Use reg.exe rather than writing system.reg directly: Wine can otherwise
# overwrite direct edits when its registry server exits.
mkdir -p "$prefix_root"
export STEAM_COMPAT_CLIENT_INSTALL_PATH="${STEAM_COMPAT_CLIENT_INSTALL_PATH:-$HOME/.local/share/Steam}"
export STEAM_COMPAT_DATA_PATH="${STEAM_COMPAT_DATA_PATH:-${LIBOVR_OPENXR_PREFIX:-$project_root/artifacts/proton-prefix}}"
export STEAM_COMPAT_APP_ID="${STEAM_COMPAT_APP_ID:-0}"
export SteamAppId="${SteamAppId:-$STEAM_COMPAT_APP_ID}"
export SteamGameId="${SteamGameId:-$STEAM_COMPAT_APP_ID}"
# Disable fsync by default for this disposable prefix: repeated reg.exe and
# launcher invocations otherwise race Proton's stale shared-memory cleanup.
export WINEFSYNC="${WINEFSYNC:-0}"
# Proton's reg.exe intermittently returns 1 after committing changes while
# tearing down fsync shared memory; continue so all values are attempted.
proton_reg_add() {
  steam-run "$LIBOVR_OPENXR_PROTON" run reg.exe add "$@" >/dev/null || \
    printf 'warning: Proton reg.exe returned non-zero after add\n' >&2
}
proton_reg_add 'HKLM\Software\Oculus VR, LLC\Oculus' /v Base /t REG_SZ \
  /d 'C:\Program Files\Oculus' /f
# Start Echo suspended and inject this helper before its entry point. This is
# deterministic under Proton and avoids AppInit_DLLs' global, unreliable hook.
game_exe_windows="Z:${game_exe//\//\\}"

run_dir="$project_root/artifacts/runs/$(date +%Y%m%d-%H%M%S)"
mkdir -p "$run_dir" "$STEAM_COMPAT_DATA_PATH"
export WINEDEBUG="${WINEDEBUG:--all}"

printf 'run directory: %s\n' "$run_dir"
printf 'proton: %s\n' "$LIBOVR_OPENXR_PROTON"
printf 'shim: %s\n' "$shim_target"
printf 'platform SDK shim: %s\n' "$platform_target"
printf 'prefix: %s\n' "$STEAM_COMPAT_DATA_PATH"
printf 'Oculus runtime shim: %s\n' "$oculus_runtime_dir/LibOVRRT64_1.dll"
printf 'launcher: %s\n' "$launcher_target"
printf 'loader bypass: %s (adjacent to launcher)\n' "$bypass_target"
printf 'stdout/stderr: %s\n' "$run_dir/proton.{out,err}"

steam-run "$LIBOVR_OPENXR_PROTON" run "$launcher_target" "$game_exe_windows" \
  >"$run_dir/proton.out" 2>"$run_dir/proton.err"
