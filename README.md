# libovr-openxr-rs

> **Disclaimer:** This repository, and the following README (aside from this intro),
> is pretty much entirely LLM written. Use at your own risk.

A compatibility layer between the LibOVR API and OpenXR.

Similar to Revive, but since this directly replaces the LibOVR runtime, you
don't need to have the Oculus software installed.

It is primarily tested with Echo VR, on Linux through Wine/Proton.
However it should also work on Windows for users who want to use OpenXR directly.
It may also work with other LibOVR titles, but no guarantees.

## Usage (Echo VR)

1. Download the latest Windows DLLs and the `echo_patcher` release for your
   platform from the project's GitHub Releases.
2. Copy `echo_patcher.exe` (Windows) or `echo_patcher-x86_64.AppImage` (Linux),
   `LibOVRRT64_1.dll` and `LibOVRPlatform64_1.dll` into the game's
   `bin/win10` directory.
3. Run `echo_patcher.exe` on Windows or `echo_patcher-x86_64.AppImage` on
   Linux. Enter your desired Oculus ID, then click **Patch and generate configuration**.

   The patcher verifies the supported game build, preserves `.original`
   backups, applies both patches, and writes `bin/win10/libovr-openxr.toml`.

   You can re-run the patcher to pick a new Oculus ID.

4. On Linux, start the game through Proton with a native Linux OpenXR runtime selected.
   On Windows, just run echovr.exe. Make sure you have an OpenXR runtime configured.

> **EchoVRCE warning:** The patcher does not currently support the EchoVRCE
> `pnsovr.dll`. Run it against the original game's `pnsovr.dll`; EchoVRCE
> itself should still work with the `libovr-openxr-rs` compatibility patch.

### Manual patching alternative

If the GUI patcher is unavailable, use the scripts from this repository. Run
both commands against the EchoVR root directory (not `bin/win10` itself):

```sh
python3 tools/patch-local-binaries.py /path/to/echo-vr
python3 tools/generate-config.py /path/to/echo-vr
```

The configuration script writes `bin/win10/libovr-openxr.toml`. Use `--force`
to replace an existing configuration, or provide `--id`, `--org-id`, and
`--oculus-id` explicitly.

The patcher modifies two game-local files under `bin/win10`:

- `echovr.exe`: changes the existing DLL-loading branch so the game's DLL
  signature check does not reject the compatibility DLLs.
- `pnsovr.dll`: changes its preloaded-platform failure branch to continue into
  the existing platform API resolution path.

The configuration generator writes beside `echovr.exe`, where the shim looks
for `libovr-openxr.toml`. It does not modify the Windows registry or contact
Oculus services.

Audio device selection uses WASAPI inside the Windows/Proton environment. By
default, the shim asks WASAPI for the system default render and capture
endpoints, so the WiVRn device must be the default device exposed to Proton.
The optional `[audio]` `output_guid` and `input_guid` settings override those
endpoint IDs when needed.

Both patches are guarded by expected-byte checks and the tool refuses an
unsupported game build rather than patching it blindly. If either
`.original` file does not exist, the tool copies the current `echovr.exe` or
`pnsovr.dll` into the corresponding `.original` path before patching. Always
preserve those original files and use the original game `pnsovr.dll`, not an
already-patched copy. The release contains
the compatibility DLLs and patching tool, not the game itself.

## Current status

This project is in **alpha**. Most implemented functionality has been tested
and is working on the maintainer's computer, but it has not yet been validated
across a broad range of hardware, runtimes, or applications.

Known limitations and active work include:

- The patches are currently only designed for Echo VR. You will need to
  manually design a patch for any other game you wish to use this with.
- Echo's existing `pnsovr.dll` platform provider remains a compatibility focus;
  the shim supplies platform-export behavior while that call path is refined.
- Hardware and network behavior have not been validated as a release product.

Expect crashes, missing functionality, and compatibility changes as testing
expands and supported applications are added.

### Initialization-order limitations

Echo requests its initial display descriptor before it supplies the D3D12
command queue required to create an OpenXR graphics session. Core OpenXR only
provides calibrated per-eye FOV and eye offsets through `xrLocateViews`, which
requires that session. A temporary headless or graphics session is not a safe
solution: WiVRn can return synthetic headless views and can crash when a second
graphics session is created.

The shim therefore uses a game-local calibration cache:

1. On a first launch without a cache, startup descriptor queries use compatible
   fallback values.
2. Once Echo has created its retained D3D12 OpenXR session and valid live views
   are located, the shim stores FOV, eye offsets/IPD, recommended eye size, and
   refresh rate in `bin/win10/libovr-openxr-hmd-cache.toml` beside
   `echovr.exe`.
3. On the next launch, the cache is loaded during `ovr_Initialize`, before
   Echo's startup `ovr_GetHmdDesc` and render-descriptor queries. This lets
   Echo build its initial projection from the calibrated values.

After changing headset, OpenXR runtime, or relevant runtime display settings,
delete `libovr-openxr-hmd-cache.toml` and launch once to regenerate it, then
restart the game. The cache is only written from the retained real graphics
session; no temporary OpenXR session is created.

## Requirements

- NixOS, or a Linux environment that can provide the dependencies in
  `shell.nix` through `nix-shell`.
- A 64-bit Proton installation managed by Steam.
- A native Linux OpenXR runtime configured outside this repository.
- The Echo VR files, including the original Windows binaries.

The initial Windows target is `x86_64-pc-windows-gnu`. MSVC is not currently
supported by the development workflow.

## Try the current development build

From this directory, enter the development shell:

```sh
nix-shell
```

Build the optimized Windows DLL and run the game through Proton:

```sh
tools/run-game.sh
```

The launcher builds with Cargo's `--release` profile by default. To build the
DLL without launching the game, run `tools/build-windows.sh` directly.

To reuse an existing DLL:

```sh
tools/run-game.sh --no-build
```

The launcher copies the shim beside the game as both `LibOVRRT64_1.dll` and
`LibOVRPlatform64_1.dll`, sets the Proton environment, and writes captured
stdout/stderr under `artifacts/runs/`. It does not modify Windows registry
entries or install a system-wide Oculus runtime.

The game-local binary patches are required because the game performs DLL
signature checks. The patcher verifies expected bytes and refuses an unexpected
game build:

```sh
python3 tools/patch-local-binaries.py
```

Keep backups of the original game files. The patcher retains `.original` files
and should only be used with a copy of the installation that you are prepared
to repair or restore.

## Development checks

Run the host-side checks inside `nix-shell`:

```sh
cargo fmt --check
cargo test
```

The test suite uses a deterministic mock runtime for lifecycle behavior. The
OpenXR probes are optional and are ignored when no runtime is available.

The Windows DLL can be built with:

```sh
tools/build-windows.sh
```

## Repository layout

- `src/capi.rs` — LibOVR ABI exports and the current D3D12 swap-chain bridge.
- `src/platform_exports.rs` — platform-provider export shims and diagnostics.
- `src/runtime.rs` — backend-independent lifecycle contract.
- `src/mock.rs` — deterministic no-HMD runtime used by tests.
- `src/openxr_backend.rs` — OpenXR integration under active development.
- `src/abi.rs` — C-compatible LibOVR types and constants.
- `reference/` — symbol manifests used during ABI investigation; these are not
  redistributable Oculus binaries.
- `DEBUG-NOTES.md` — detailed reverse-engineering notes and known blockers.

## Safety and distribution

Do not distribute Oculus, Echo VR, Proton, or other third-party binaries with
this project. The reference files describe observed exports and are not SDK
redistributions. This project is an unofficial compatibility effort and is not
affiliated with Meta, Oculus, Ready at Dawn, or Echo VR's original developers.
