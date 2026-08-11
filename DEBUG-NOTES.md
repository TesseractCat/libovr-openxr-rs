# Current debugging notes

## Renderer milestone

Echo VR is using **D3D12**, despite some external references describing it as D3D11:

```text
Creating D3D12 device on adapter 'Intel(R) Graphics (LNL)'
```

`ovr_CreateTextureSwapChainDX` receives an `ID3D12CommandQueue`. The shim now:

1. `QueryInterface`s it for `IID_ID3D12CommandQueue`.
2. Calls `ID3D12CommandQueue::GetDevice` (vtable slot 7) for `ID3D12Device`.
3. Calls `ID3D12Device::CreateCommittedResource` (vtable slot 27) to create 3
   default-heap `ID3D12Resource` texture images.
4. Returns them from `ovr_GetTextureSwapChainBufferDX`.

Critical IID values:

- `ID3D12CommandQueue`: `0ec870a6-5d7e-4c22-8cfc-5baae07616ed`
- `ID3D12Device`: `189819f1-1db6-4b57-be54-1821339b85f7`
- `ID3D12Resource`: `696442be-a72e-4059-bc79-5b5c98040fad`

The resource IID was initially mistyped, causing `0x80004002`; correcting it
allowed Echo to pass the prior blocker:

```text
Successfully initialized OVR D3D components.
Finished initializing engine
```

Echo creates two 3-image chains at `3664x1920`, formats `5` and `12`, and
requests `IID_ID3D12Resource`.

## Current blocker: pnsovr platform provider

After renderer initialization Echo loads its existing `bin/win10/pnsovr.dll`.
Do not replace `pnsovr.dll`.

It imports `LibOVRPlatform64_1.dll`, absent from the game/prefix. The same Rust
shim is now deployed under both names:

```text
bin/win10/LibOVRRT64_1.dll
bin/win10/LibOVRPlatform64_1.dll
```

and `tools/run-game.sh` also installs both in the prefix Oculus runtime
folder.

`src/platform_exports.rs` contains resolver stubs for all 155 normal PE imports
from pnsovr's Platform SDK dependency. It also provides guessed dynamic
platform initialization exports. All generic stubs now log calls through
`crate::capi::log_call`.

Current failure after pnsovr loads:

```text
Failed to initialize the Oculus VR Platform SDK
... Failed to initialize the Oculus VR Platform SDK (Success)
```

Observed calls from Platform shim immediately before failure:

```text
ovrPlatformInitializeResult_ToString
ovr_Microphone_Destroy
ovr_Voip_DestroyEncoder
```

`ovrPlatformInitializeResult_ToString` was corrected to return `"Success"`;
the suffix confirms it is called correctly. `pnsovr.dll` has no PE delay-import
directory. Wine loader tracing confirms it loads the game-directory platform
shim. There is currently no evidence that it contacted Oculus/auth services.

Important: generic `extern "system" fn foo()` stubs have invalid return ABI for
Platform SDK APIs returning booleans, result codes, handles, messages, etc.
The failure may be caused by one such return/state requirement. Do **not** jump
to bypassing `pnsovr`; first identify the provider initialization call path and
implement exact ABI/state for the functions actually used.

Additional reverse-engineering (current session): the delay-import directory is
empty. `ovrPlatformInitializeResult_ToString`'s IAT slot is RVA `0x1fa8b8`,
with thunk `0x1801e8770`. It has one call site at `0x1800975ff`; disassembly
shows it is solely an error-reporting path after the preceding internal call at
`0x180098bc0` returns a non-zero status. Thus ToString is not initialization.
Wine `+relay` tracing has not shown pnsovr dynamically calling
`GetProcAddress` for a Platform initializer, and no initializer stub logs.

## OpenXR / launch environment

- Native OpenXR probes successfully create a Monado instance, including via
  Proton's Linux `libopenxr_loader.so.1` inside `steam-run`.
- Nix `steam-run` does not bind host `/tmp`; Monado runtime directory was moved
  to `/run/user/$UID/libovr-monado`, which is visible to it.
- `tools/run-monado-null.sh` starts Monado simulated HMD/null compositor and
  creates `XDG_CONFIG_HOME/openxr/1/active_runtime.json`.
- `tools/run-monado-game.sh` encapsulates start/source/run steps.
- WineOpenXR diagnostic probe still returns initialization error `-6`; it does
  not stop Echo's boot. Keep it diagnostic while advancing LibOVR behavior.
- `shell.nix` prefers Proton Experimental and uses the isolated
  `artifacts/proton-experimental-prefix`, avoiding earlier Proton 11 prefix
  state.

## Current useful logs

- Shim: `artifacts/proton-experimental-prefix/pfx/drive_c/users/steamuser/AppData/Local/Temp/libovr-openxr.log`
- Latest Echo crash: `_temp/crashes/RAD_CRASHDUMP_steamuser_echovr_11_17_27_30.log`
- Provider loader trace: `artifacts/pns-load.log`
- Relay trace: `artifacts/pns-relay.log`
