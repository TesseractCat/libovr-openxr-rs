# libovr-openxr-rs

A Rust implementation of the **LibOVRRT64_1** CAPI surface required by Echo VR,
translated to OpenXR. The target is Wine plus a native Linux OpenXR runtime.

> Status: Echo now initializes its LibOVR D3D12 swap chains using shim-created
> `ID3D12Resource` textures. The next blocker is its existing `pnsovr.dll`
> provider module; OpenXR frame submission is not implemented yet.

## Layout

- `src/capi.rs` — tiny, fail-closed Windows ABI boundary.
- `src/runtime.rs` — backend-independent lifecycle contract.
- `src/mock.rs` — deterministic no-HMD backend.
- `reference/echovr-ovr-symbols.txt` — 93 `ovr_*` names found in this Echo build.
- `reference/libovr-1.94-exports.txt` — 125 `ovr_*` exports from the examined
  CAPI 1.94 reference DLL. It is an ABI manifest, not a redistributable binary.

## Local development loop: no HMD required

Most work should first run on Linux without Wine, OpenXR, a GPU, or a headset:

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```

The core uses a `Runtime` trait. `MockRuntime` supplies reproducible poses,
velocities, mount/focus state, frame IDs, and injected session loss. Add unit
and property tests here for every CAPI-to-core conversion and lifecycle rule.

### Proposed test pyramid

1. **Pure deterministic tests (every edit/CI):** ABI conversions, coordinate
   transforms, button bitfields, timestamp/velocity interpolation, lifecycle,
   error mapping, and swapchain state transitions. Fixtures should be compact
   JSON or binary traces of `ovr_*` inputs and expected OpenXR-facing commands.
2. **Trace replay (every PR):** run a recorded Echo call stream through the
   shim with `MockRuntime`; compare the resulting command/event trace to a
   checked-in golden file. Capture traces from a logging proxy once, then replay
   them indefinitely without the game or headset.
3. **Headless OpenXR integration (nightly):** use an OpenXR runtime with a
   headless mode/extension where available (for example a configured Monado
   headless runtime). Exercise instance/session creation and frame sequencing;
   skip cleanly when the CI runner lacks it.
4. **Wine smoke test (nightly/manual):** build the Windows `cdylib`, load it
   into a minimal Windows C test host under Wine, and verify required exports,
   calling convention, C struct sizes, and failure paths. This host should not
   start Echo or require an HMD.
5. **Hardware acceptance (release candidates only):** one short scripted lobby
   pass validates compositor output, controller velocity/throwing, haptics,
   headset removal, and runtime recovery.

The essential artifact for steps 1–2 is a **record/replay command trace**:
record CAPI calls at the ABI boundary, normalize opaque pointers/GPU handles to
stable IDs, and replay them against the mock. GPU images are represented by
metadata and an acquire/commit/release state machine; no pixels are required
until graphics integration testing.

## First implementation milestones

1. Import and layout-test audited CAPI 1.94 types from official headers.
2. Add an export-complete probe DLL that logs which of the reference exports
   Echo resolves/calls.
3. Implement the D3D11 OpenXR frame path against a headless/mock backend.
4. Add real OpenXR tracking/actions/haptics after replay tests specify the
   expected Oculus semantics.

## NixOS development shell

This project uses a classic [`shell.nix`](shell.nix), not flakes. It provides
Rustup, MinGW-w64, and inspection/build utilities. Rustup is used only because
the project needs the pinned Rust Windows standard library; Nix's host Rust
package normally does not ship that target. Toolchains and Cargo caches stay in
`.rustup/` and `.cargo-home/` in this checkout.

```sh
nix-shell
cargo test
tools/build-windows.sh
# Starts Monado's simulated HMD/null compositor, then builds, deploys, and runs Echo.
tools/run-monado-game.sh
```

Host checks use Nix-built Rust. `tools/build-windows.sh` explicitly invokes the
Rustup toolchain pinned in `rust-toolchain.toml`; on its first run Rustup
downloads Rust 1.93.1 plus the Windows GNU target, Clippy, and rustfmt. The
Cargo target configuration uses Nix's `x86_64-w64-mingw32-gcc` linker.

The shell detects a Steam-managed Proton installation (preferring Proton
Experimental) and exposes it as `$LIBOVR_OPENXR_PROTON`. Its Experimental
prefix is isolated at `artifacts/proton-experimental-prefix`; this avoids
mixing Wine/OpenXR registry state with earlier Proton 11 tests. It does not
install or replace Proton.

For repeated headsetless boot attempts, use `tools/run-monado-game.sh` rather
than manually starting Monado, sourcing its environment, and launching Echo.
Pass `--no-build` when the Windows DLL is already current.

## Build targets

The host test loop uses `x86_64-unknown-linux-gnu`. A deployable DLL uses
`x86_64-pc-windows-gnu`. Use the GNU target for the initial Wine/Proton loop;
consider MSVC only if a dependency later requires it.
