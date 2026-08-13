# libovr-openxr-rs

> **Disclaimer:** This repository, and the following README (aside from this intro),
> is pretty much entirely LLM written. Use at your own risk.

A compatibility layer between the LibOVR API and OpenXR.

Similar to Revive, but since this directly replaces the LibOVR runtime, you don't need to have
the Oculus software installed.

It is primarily tested with Echo VR, on Linux through Wine/Proton.
However it should also work on Windows for users who want to use OpenXR directly.
It may also work with other LibOVR titles, but no guarantees.

## Usage

1. Download the latest Windows DLLs from the project's GitHub Releases.
2. Copy `LibOVRRT64_1.dll` and `LibOVRPlatform64_1.dll` into the game's
   `bin/win10` directory.
3. From this repository, run the required game patch, providing the game's
   root directory:

   ```sh
   python3 tools/patch-local-binaries.py /path/to/echo-vr
   ```

   The path should be the EchoVR root directory; do not pass the `bin/win10` directory
   itself.

4. Generate a local identity configuration if you need one:

   ```sh
   python3 tools/generate-config.py /path/to/echo-vr
   ```

   This writes `bin/win10/libovr-openxr.toml`. The script generates a stable
   local ID and uses it as the organization ID. Keep this file after linking;
   changing the IDs can make the game treat you as a different user. Use
   `--force` to replace an existing configuration, or provide `--id`,
   `--org-id`, and `--oculus-id` explicitly.

5. Start the game through Proton with a native Linux OpenXR runtime selected.

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

> **EchoVRCE warning:** The patcher does not currently support the EchoVRCE
> `pnsovr.dll`. Run it against the original game's `pnsovr.dll`; EchoVRCE
> itself should still work with the `libovr-openxr-rs` compatibility patch.

## Current status

This project is in **alpha**. Most implemented functionality has been tested
and is working on the maintainer's computer, but it has not yet been validated
across a broad range of hardware, runtimes, or applications.

Known limitations and active work include:

- Echo's current test target is D3D12, not D3D11.
- OpenXR frame submission, tracking, input, haptics, and recovery continue to
  receive testing and development.
- Echo's existing `pnsovr.dll` platform provider remains a compatibility focus;
  the shim supplies platform-export behavior while that call path is refined.
- Hardware and network behavior have not been validated as a release product.

Expect crashes, missing functionality, and compatibility changes as testing
expands and supported applications are added.

### Initialization-order limitations

Some LibOVR queries happen before Echo has supplied its D3D12 device, so the
shim does not yet have a valid OpenXR graphics session when they run. Those
first responses use compatibility fallbacks and may be replaced with runtime
values on later calls:

- `ovr_GetHmdDesc` reports the fallback refresh rate and default display FOV.
  The refresh rate becomes runtime-derived after the OpenXR session exists.
- `ovr_GetFovTextureSize` uses the fallback FOV and eye pixel density until
  runtime view information is available.
- `ovr_GetRenderDesc2` uses fallback eye poses/IPD until OpenXR supplies valid
  view poses and FOVs.

This ordering is expected for the current LibOVR startup sequence. It is a
known area for future improvement, especially if an application caches the
first descriptor instead of querying it again after session creation. In
practice, preliminary Echo VR testing indicates that these values are
re-queried over time, allowing the later runtime-derived values to take effect.

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
