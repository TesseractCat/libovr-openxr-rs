//! Bootstrap LibOVR exports and call tracing.
//!
//! This is intentionally a virtual, non-rendering session. Its purpose is to
//! identify Echo's required call sequence before OpenXR/D3D code is introduced.

use core::ffi::c_char;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::abi::{
    OVR_SUCCESS, OvrErrorInfo, OvrFovPort, OvrGraphicsLuid, OvrHmdDesc, OvrInitParams, OvrResult,
    OvrSession, OvrSessionStatus, OvrVersionString,
};

static INITIALIZED: AtomicBool = AtomicBool::new(false);
static SESSION_TOKEN: u8 = 1;
static VERSION: &[u8] = b"LibOVR OpenXR shim (bootstrap)\0";

fn log_call(name: &str) {
    let temp = std::env::var_os("TEMP").unwrap_or_else(|| "C:\\windows\\temp".into());
    let path = std::path::PathBuf::from(temp).join("libovr-openxr.log");
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{name}");
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Initialize(_params: *const OvrInitParams) -> OvrResult {
    log_call("ovr_Initialize");
    INITIALIZED.store(true, Ordering::Release);
    OVR_SUCCESS
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Shutdown() {
    log_call("ovr_Shutdown");
    INITIALIZED.store(false, Ordering::Release);
}

/// # Safety
/// `session` must reference writable CAPI storage; `luid`, if non-null, must
/// reference writable `ovrGraphicsLuid` storage.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn ovr_Create(
    session: *mut OvrSession,
    luid: *mut OvrGraphicsLuid,
) -> OvrResult {
    log_call("ovr_Create");
    if !INITIALIZED.load(Ordering::Acquire) || session.is_null() {
        return -1004; // ovrError_NotInitialized
    }
    unsafe {
        *session = (&SESSION_TOKEN as *const u8).cast_mut().cast();
        if !luid.is_null() {
            *luid = OvrGraphicsLuid::default();
        }
    }
    OVR_SUCCESS
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Destroy(_session: OvrSession) {
    log_call("ovr_Destroy");
}

/// # Safety
/// `status` must reference writable CAPI `ovrSessionStatus` storage.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn ovr_GetSessionStatus(
    _session: OvrSession,
    status: *mut OvrSessionStatus,
) -> OvrResult {
    log_call("ovr_GetSessionStatus");
    if status.is_null() {
        return -1005; // ovrError_InvalidParameter
    }
    unsafe {
        *status = OvrSessionStatus {
            is_visible: 1,
            hmd_present: 1,
            hmd_mounted: 1,
            has_input_focus: 1,
            ..OvrSessionStatus::default()
        };
    }
    OVR_SUCCESS
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_GetHmdDesc(_session: OvrSession) -> OvrHmdDesc {
    log_call("ovr_GetHmdDesc");
    let mut product = [0; 64];
    for (slot, byte) in product.iter_mut().zip(b"OpenXR HMD") {
        *slot = *byte as c_char;
    }
    let fov = OvrFovPort {
        up_tan: 1.0,
        down_tan: 1.0,
        left_tan: 1.0,
        right_tan: 1.0,
    };
    OvrHmdDesc {
        hmd_type: 9, // ovrHmd_Other
        product_name: product,
        resolution: crate::abi::OvrSizei { w: 1832, h: 1920 },
        default_eye_fov: [fov; 2],
        max_eye_fov: [fov; 2],
        display_refresh_rate: 90,
        ..unsafe { core::mem::zeroed() }
    }
}

/// # Safety
/// `error_info`, if non-null, must reference writable CAPI error-info storage.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn ovr_GetLastErrorInfo(error_info: *mut OvrErrorInfo) {
    log_call("ovr_GetLastErrorInfo");
    if !error_info.is_null() {
        unsafe {
            *error_info = OvrErrorInfo {
                result: OVR_SUCCESS,
                error_string: [0; 512],
            }
        };
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_GetVersionString() -> OvrVersionString {
    log_call("ovr_GetVersionString");
    VERSION.as_ptr().cast::<c_char>()
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_GetTimeInSeconds() -> f64 {
    log_call("ovr_GetTimeInSeconds");
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0.0, |duration| duration.as_secs_f64())
}

macro_rules! unresolved_exports {
    ($($name:ident),* $(,)?) => {$(
        #[unsafe(no_mangle)]
        pub extern "system" fn $name() {
            log_call(stringify!($name));
        }
    )*};
}

// Resolver-complete exports for this Echo executable. They are intentionally
// inert until each signature/behavior is implemented and tested.
unresolved_exports!(
    ovr_BeginFrame,
    ovr_ClearShouldRecenterFlag,
    ovr_CommitTextureSwapChain,
    ovr_CreateMirrorTextureDX,
    ovr_CreateMirrorTextureGL,
    ovr_CreateMirrorTextureWithOptionsDX,
    ovr_CreateMirrorTextureWithOptionsGL,
    ovr_CreateMirrorTextureWithOptionsVk,
    ovr_CreateTextureSwapChainDX,
    ovr_CreateTextureSwapChainGL,
    ovr_CreateTextureSwapChainVk,
    ovr_DestroyMirrorTexture,
    ovr_DestroyTextureSwapChain,
    ovr_EnableExtension,
    ovr_EndFrame,
    ovr_GetAudioDeviceInGuid,
    ovr_GetAudioDeviceInGuidStr,
    ovr_GetAudioDeviceInWaveId,
    ovr_GetAudioDeviceOutGuid,
    ovr_GetAudioDeviceOutGuidStr,
    ovr_GetAudioDeviceOutWaveId,
    ovr_GetBool,
    ovr_GetBoundaryDimensions,
    ovr_GetBoundaryGeometry,
    ovr_GetBoundaryVisible,
    ovr_GetConnectedControllerTypes,
    ovr_GetControllerVibrationState,
    ovr_GetDeviceExtensionsVk,
    ovr_GetDevicePoses,
    ovr_GetExternalCameras,
    ovr_GetFloat,
    ovr_GetFloatArray,
    ovr_GetFovStencil,
    ovr_GetFovTextureSize,
    ovr_GetHmdColorDesc,
    ovr_GetInputState,
    ovr_GetInstanceExtensionsVk,
    ovr_GetInt,
    ovr_GetMirrorTextureBufferDX,
    ovr_GetMirrorTextureBufferGL,
    ovr_GetMirrorTextureBufferVk,
    ovr_GetPerfStats,
    ovr_GetPredictedDisplayTime,
    ovr_GetRenderDesc2,
    ovr_GetSessionPhysicalDeviceVk,
    ovr_GetString,
    ovr_GetTextureSwapChainBufferDX,
    ovr_GetTextureSwapChainBufferGL,
    ovr_GetTextureSwapChainBufferVk,
    ovr_GetTextureSwapChainCurrentIndex,
    ovr_GetTextureSwapChainDesc,
    ovr_GetTextureSwapChainLength,
    ovr_GetTouchHapticsDesc,
    ovr_GetTrackerCount,
    ovr_GetTrackerDesc,
    ovr_GetTrackerPose,
    ovr_GetTrackingOriginType,
    ovr_GetTrackingState,
    ovr_IdentifyClient,
    ovr_IsExtensionSupported,
    ovr_Lookup,
    ovr_RecenterTrackingOrigin,
    ovr_ReportClientInfo,
    ovr_RequestBoundaryVisible,
    ovr_ResetBoundaryLookAndFeel,
    ovr_ResetPerfStats,
    ovr_SetBool,
    ovr_SetBoundaryLookAndFeel,
    ovr_SetClientColorDesc,
    ovr_SetControllerVibration,
    ovr_SetExternalCameraProperties,
    ovr_SetFloat,
    ovr_SetFloatArray,
    ovr_SetInt,
    ovr_SetString,
    ovr_SetSynchronizationQueueVk,
    ovr_SetTrackingOriginType,
    ovr_SpecifyTrackingOrigin,
    ovr_SubmitControllerVibration,
    ovr_SubmitFrame2,
    ovr_TestBoundary,
    ovr_TestBoundaryPoint,
    ovr_TraceMessage,
    ovr_WaitToBeginFrame,
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bootstrap_create_returns_a_session() {
        assert_eq!(ovr_Initialize(core::ptr::null()), OVR_SUCCESS);
        let mut session = core::ptr::null_mut();
        assert_eq!(
            unsafe { ovr_Create(&mut session, core::ptr::null_mut()) },
            OVR_SUCCESS
        );
        assert!(!session.is_null());
        ovr_Shutdown();
    }

    #[test]
    fn version_is_c_string() {
        let text = std::ffi::CStr::from_bytes_with_nul(VERSION).expect("NUL terminated version");
        assert!(text.to_str().expect("utf-8 version").contains("OpenXR"));
    }
}
