//! Dynamic OpenXR-loader probing used before the CAPI frame bridge is enabled.
//!
//! This deliberately creates only an OpenXR instance. Graphics sessions and
//! swapchains remain owned by the upcoming D3D bridge.

use openxr::{ApplicationInfo, Entry, ExtensionSet};

/// Capabilities required for the Windows D3D11 CAPI bridge and for automated
/// Monado headless testing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OpenXrCapabilities {
    pub d3d11: bool,
    pub d3d12: bool,
    pub monado_headless: bool,
}

/// Errors are strings because loader/runtime failures are operational details
/// intended for the shim trace, not CAPI result codes.
/// Create and immediately destroy a D3D12 OpenXR session. This is deliberately
/// diagnostic-only while the CAPI owns Echo's swapchains; it verifies that the
/// exact D3D12 device/queue passed by Echo are acceptable to the active runtime.
#[cfg(windows)]
pub unsafe fn probe_d3d12_session(
    device: *mut core::ffi::c_void,
    queue: *mut core::ffi::c_void,
) -> Result<(), String> {
    let entry = unsafe { Entry::load() }.map_err(|error| error.to_string())?;
    let extensions = entry
        .enumerate_extensions()
        .map_err(|error| error.to_string())?;
    if !extensions.khr_d3d12_enable {
        return Err("XR_KHR_d3d12_enable unavailable".into());
    }
    let mut requested = ExtensionSet::default();
    requested.khr_d3d12_enable = true;
    let instance = entry
        .create_instance(
            &ApplicationInfo {
                application_name: "libovr-openxr",
                application_version: 1,
                engine_name: "Echo VR",
                engine_version: 1,
                api_version: openxr::Version::new(1, 0, 0),
            },
            &requested,
            &[],
        )
        .map_err(|error| error.to_string())?;
    let system = instance
        .system(openxr::FormFactor::HEAD_MOUNTED_DISPLAY)
        .map_err(|error| error.to_string())?;
    let info = openxr::d3d::SessionCreateInfoD3D12 {
        device: device.cast(),
        queue: queue.cast(),
    };
    let _session = unsafe { instance.create_session::<openxr::D3D12>(system, &info) }
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn probe() -> Result<OpenXrCapabilities, String> {
    // `Entry::load` dynamically locates the platform OpenXR loader. This is
    // essential for Wine: no OpenXR import library is baked into our DLL.
    // Wine registers wineopenxr.dll as an OpenXR *runtime*. Applications must
    // still use the standard OpenXR loader, exactly as they do on Windows.
    let entry = unsafe { Entry::load() }.map_err(|error| error.to_string())?;
    let extensions = entry
        .enumerate_extensions()
        .map_err(|error| error.to_string())?;
    let mut requested = ExtensionSet::default();
    requested.mnd_headless = extensions.mnd_headless;
    #[cfg(windows)]
    {
        requested.khr_d3d11_enable = extensions.khr_d3d11_enable;
        requested.khr_d3d12_enable = extensions.khr_d3d12_enable;
    }
    #[cfg(windows)]
    let d3d11 = extensions.khr_d3d11_enable;
    #[cfg(windows)]
    let d3d12 = extensions.khr_d3d12_enable;
    #[cfg(not(windows))]
    let d3d11 = false;
    #[cfg(not(windows))]
    let d3d12 = false;
    let _instance = entry
        .create_instance(
            &ApplicationInfo {
                application_name: "libovr-openxr",
                application_version: 1,
                engine_name: "libovr-openxr",
                engine_version: 1,
                api_version: openxr::Version::new(1, 0, 0),
            },
            &requested,
            &[],
        )
        .map_err(|error| error.to_string())?;

    Ok(OpenXrCapabilities {
        d3d11,
        d3d12,
        monado_headless: extensions.mnd_headless,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "requires an installed OpenXR runtime"]
    fn probes_the_active_runtime() {
        super::probe().expect("OpenXR runtime probe");
    }

    #[test]
    #[ignore = "requires an installed OpenXR runtime"]
    fn probes_vulkan_extensions_used_by_wineopenxr() {
        let entry = unsafe { openxr::Entry::load() }.expect("OpenXR loader");
        let available = entry.enumerate_extensions().expect("extension enumeration");
        eprintln!("available OpenXR extensions: {available:?}");

        for (name, enable_vulkan_1) in [("vulkan1", true), ("vulkan2", false)] {
            let mut requested = openxr::ExtensionSet::default();
            requested.khr_vulkan_enable = enable_vulkan_1 && available.khr_vulkan_enable;
            requested.khr_vulkan_enable2 = !enable_vulkan_1 && available.khr_vulkan_enable2;
            requested.khr_convert_timespec_time = available.khr_convert_timespec_time;
            entry
                .create_instance(
                    &openxr::ApplicationInfo {
                        application_name: "libovr-openxr-wine-probe",
                        application_version: 1,
                        engine_name: "test",
                        engine_version: 1,
                        api_version: openxr::Version::new(1, 0, 0),
                    },
                    &requested,
                    &[],
                )
                .unwrap_or_else(|error| panic!("{name} instance creation: {error}"));
        }
    }
}
