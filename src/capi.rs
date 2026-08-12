//! Bootstrap LibOVR exports and call tracing.
//!
//! This is intentionally a virtual, non-rendering session. Its purpose is to
//! identify Echo's required call sequence before OpenXR/D3D code is introduced.

use core::ffi::c_char;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::abi::{
    OVR_AUDIO_MAX_DEVICE_STR_SIZE, OVR_EYE_LEFT, OVR_SUCCESS, OvrErrorInfo, OvrEyeRenderDesc,
    OvrEyeType, OvrFovPort, OvrGraphicsLuid, OvrHmdDesc, OvrInitParams, OvrInputState,
    OvrLayerEyeFov, OvrLayerHeader, OvrResult, OvrSession, OvrSessionStatus, OvrSizei,
    OvrTextureSwapChain, OvrTextureSwapChainDesc, OvrTrackerDesc, OvrTrackerPose, OvrTrackingState,
    OvrVector2f, OvrVector3f, OvrVersionString,
};

static INITIALIZED: AtomicBool = AtomicBool::new(false);
static TRACKING_ORIGIN: AtomicI32 = AtomicI32::new(1);
#[cfg(windows)]
static XR_ADAPTER_LUID: std::sync::Mutex<Option<[u8; 8]>> = std::sync::Mutex::new(None);
static SESSION_TOKEN: u8 = 1;
static VERSION: &[u8] = b"LibOVR OpenXR shim (bootstrap)\0";

#[repr(C)]
struct D3d11Texture2dDesc {
    width: u32,
    height: u32,
    mip_levels: u32,
    array_size: u32,
    format: u32,
    sample_count: u32,
    sample_quality: u32,
    usage: u32,
    bind_flags: u32,
    cpu_access_flags: u32,
    misc_flags: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Guid {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}

type CreateTexture2d = unsafe extern "system" fn(
    *mut core::ffi::c_void,
    *const D3d11Texture2dDesc,
    *const core::ffi::c_void,
    *mut *mut core::ffi::c_void,
) -> i32;
type QueryInterface = unsafe extern "system" fn(
    *mut core::ffi::c_void,
    *const Guid,
    *mut *mut core::ffi::c_void,
) -> i32;
type Release = unsafe extern "system" fn(*mut core::ffi::c_void) -> u32;
type GetDevice = unsafe extern "system" fn(
    *mut core::ffi::c_void,
    *const Guid,
    *mut *mut core::ffi::c_void,
) -> i32;
type CreateCommittedResource = unsafe extern "system" fn(
    *mut core::ffi::c_void,
    *const D3d12HeapProperties,
    u32,
    *const D3d12ResourceDesc,
    u32,
    *const core::ffi::c_void,
    *const Guid,
    *mut *mut core::ffi::c_void,
) -> i32;

#[repr(C)]
struct D3d12HeapProperties {
    heap_type: u32,
    cpu_page_property: u32,
    memory_pool_preference: u32,
    creation_node_mask: u32,
    visible_node_mask: u32,
}

#[repr(C)]
struct D3d12ClearValue {
    format: u32,
    depth: f32,
    stencil: u8,
    _padding: [u8; 3],
}

#[repr(C)]
struct D3d12ResourceDesc {
    dimension: u32,
    alignment: u64,
    width: u64,
    height: u32,
    depth_or_array_size: u16,
    mip_levels: u16,
    format: u32,
    sample_count: u32,
    sample_quality: u32,
    layout: u32,
    flags: u32,
}

const IID_ID3D12_COMMAND_QUEUE: Guid = Guid {
    data1: 0x0ec8_70a6,
    data2: 0x5d7e,
    data3: 0x4c22,
    data4: [0x8c, 0xfc, 0x5b, 0xaa, 0xe0, 0x76, 0x16, 0xed],
};
const IID_ID3D12_DEVICE: Guid = Guid {
    data1: 0x1898_19f1,
    data2: 0x1db6,
    data3: 0x4b57,
    data4: [0xbe, 0x54, 0x18, 0x21, 0x33, 0x9b, 0x85, 0xf7],
};
const IID_ID3D12_RESOURCE: Guid = Guid {
    data1: 0x6964_42be,
    data2: 0xa72e,
    data3: 0x4059,
    data4: [0xbc, 0x79, 0x5b, 0x5c, 0x98, 0x04, 0x0f, 0xad],
};

/// LibOVR swapchain state. Echo creates separate color and depth chains; each
/// needs stable opaque identity and independently-owned D3D resources.
#[derive(Debug)]
struct SwapChainState {
    textures: [usize; 3],
    current_index: i32,
    /// OpenXR chooses this chain's image through xrAcquireSwapchainImage.
    openxr_color: bool,
}

static SWAP_CHAINS: std::sync::Mutex<Vec<Box<SwapChainState>>> = std::sync::Mutex::new(Vec::new());

// OpenXR session lifetime is process-wide, matching LibOVR's singleton HMD
// session. The frame bridge will use this retained session rather than the old
// create-and-destroy diagnostic probe.
#[cfg(windows)]
static XR_D3D12_SESSION: std::sync::Mutex<Option<crate::openxr_backend::D3d12Session>> =
    std::sync::Mutex::new(None);

#[cfg(windows)]
unsafe fn ensure_d3d12_session(
    device: *mut core::ffi::c_void,
    queue: *mut core::ffi::c_void,
) -> Result<(), String> {
    let mut slot = XR_D3D12_SESSION
        .lock()
        .map_err(|_| "OpenXR session lock poisoned".to_owned())?;
    if slot.is_none() {
        *slot = Some(unsafe { crate::openxr_backend::create_d3d12_session(device, queue) }?);
    }
    Ok(())
}

fn register_swap_chain(
    textures: [usize; 3],
    openxr_color: bool,
) -> Result<OvrTextureSwapChain, OvrResult> {
    let mut chains = SWAP_CHAINS.lock().map_err(|_| -1000)?;
    let chain = Box::new(SwapChainState {
        textures,
        current_index: 0,
        openxr_color,
    });
    let handle = (&*chain as *const SwapChainState).cast_mut().cast();
    chains.push(chain);
    Ok(handle)
}

fn swap_chain_state_mut(
    chains: &mut [Box<SwapChainState>],
    chain: OvrTextureSwapChain,
) -> Result<&mut SwapChainState, OvrResult> {
    chains
        .iter_mut()
        .find(|candidate| std::ptr::eq(&***candidate, chain.cast()))
        .map(|candidate| &mut **candidate)
        .ok_or(-1005)
}

fn swap_chain_texture(chain: OvrTextureSwapChain, index: i32) -> Result<usize, OvrResult> {
    if !(0..3).contains(&index) {
        return Err(-1005);
    }
    let chains = SWAP_CHAINS.lock().map_err(|_| -1000)?;
    chains
        .iter()
        .find(|candidate| std::ptr::eq(&***candidate, chain.cast()))
        .map(|candidate| candidate.textures[index as usize])
        .ok_or(-1005)
}

pub(crate) fn log_call(name: &str) {
    let temp = std::env::var_os("TEMP").unwrap_or_else(|| "C:\\windows\\temp".into());
    let path = std::path::PathBuf::from(temp).join("libovr-openxr.log");
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{name}");
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_Initialize(_params: *const OvrInitParams) -> OvrResult {
    log_call("ovr_Initialize");
    #[cfg(windows)]
    match crate::openxr_backend::d3d12_adapter_luid() {
        Ok(luid) => {
            if let Ok(mut cached) = XR_ADAPTER_LUID.lock() {
                *cached = Some(luid);
            }
            log_call("openxr D3D12 adapter LUID discovered");
        }
        Err(error) => log_call(&format!("openxr D3D12 adapter LUID unavailable: {error}")),
    }
    if std::env::var_os("LIBOVR_OPENXR_PROBE").is_some() {
        log_call(&format!(
            "openxr env XR_RUNTIME_JSON={:?} XDG_RUNTIME_DIR={:?}",
            std::env::var_os("XR_RUNTIME_JSON"),
            std::env::var_os("XDG_RUNTIME_DIR")
        ));
        match crate::openxr_backend::probe() {
            Ok(capabilities) => log_call(&format!(
                "openxr probe d3d11={} d3d12={} mnd_headless={}",
                capabilities.d3d11, capabilities.d3d12, capabilities.monado_headless
            )),
            Err(error) => log_call(&format!("openxr probe failed: {error}")),
        }
    }
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
            #[cfg(windows)]
            {
                *luid = XR_ADAPTER_LUID
                    .lock()
                    .ok()
                    .and_then(|cached| *cached)
                    .map(|reserved| OvrGraphicsLuid { reserved })
                    .unwrap_or_default();
            }
            #[cfg(not(windows))]
            {
                *luid = OvrGraphicsLuid::default();
            }
        }
    }
    OVR_SUCCESS
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_GetTrackingState(
    _session: OvrSession,
    absolute_time: f64,
    _latency_marker: u8,
) -> OvrTrackingState {
    log_call("ovr_GetTrackingState");
    #[cfg(windows)]
    if let Ok(slot) = XR_D3D12_SESSION.lock() {
        if let Some(session) = slot.as_ref() {
            let pose = session.head_pose;
            let head_pose = crate::abi::OvrPosef {
                orientation: crate::abi::OvrQuatf {
                    x: pose.orientation.x,
                    y: pose.orientation.y,
                    z: pose.orientation.z,
                    w: pose.orientation.w,
                },
                position: OvrVector3f {
                    x: pose.position.x,
                    y: pose.position.y,
                    z: pose.position.z,
                },
            };
            return OvrTrackingState {
                head_pose: crate::abi::OvrPoseStatef {
                    pose: head_pose,
                    time_in_seconds: absolute_time,
                    ..Default::default()
                },
                hand_poses: core::array::from_fn(|index| crate::abi::OvrPoseStatef {
                    pose: crate::abi::OvrPosef {
                        orientation: crate::abi::OvrQuatf {
                            x: session.hand_poses[index].orientation.x,
                            y: session.hand_poses[index].orientation.y,
                            z: session.hand_poses[index].orientation.z,
                            w: session.hand_poses[index].orientation.w,
                        },
                        position: OvrVector3f {
                            x: session.hand_poses[index].position.x,
                            y: session.hand_poses[index].position.y,
                            z: session.hand_poses[index].position.z,
                        },
                    },
                    angular_velocity: OvrVector3f {
                        x: session.hand_angular_velocity[index].x,
                        y: session.hand_angular_velocity[index].y,
                        z: session.hand_angular_velocity[index].z,
                    },
                    linear_velocity: OvrVector3f {
                        x: session.hand_linear_velocity[index].x,
                        y: session.hand_linear_velocity[index].y,
                        z: session.hand_linear_velocity[index].z,
                    },
                    time_in_seconds: absolute_time,
                    ..Default::default()
                }),
                hand_status_flags: [0x3; 2],
                calibrated_origin: crate::abi::OvrPosef {
                    orientation: crate::abi::OvrQuatf {
                        w: 1.0,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                status_flags: 0xe3,
                ..Default::default()
            };
        }
    }
    OvrTrackingState {
        head_pose: crate::abi::OvrPoseStatef {
            pose: crate::abi::OvrPosef {
                orientation: crate::abi::OvrQuatf {
                    w: 1.0,
                    ..Default::default()
                },
                ..Default::default()
            },
            time_in_seconds: absolute_time,
            ..Default::default()
        },
        calibrated_origin: crate::abi::OvrPosef {
            orientation: crate::abi::OvrQuatf {
                w: 1.0,
                ..Default::default()
            },
            ..Default::default()
        },
        // Report both tracking and connection: Echo treats a tracked-but-not-
        // connected HMD as a sensor/device failure.
        // ovrStatus_OrientationTracked | ovrStatus_PositionTracked |
        // ovrStatus_OrientationConnected | ovrStatus_PositionConnected |
        // ovrStatus_HmdConnected.
        status_flags: 0xe3,
        ..Default::default()
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_GetInputState(
    _session: OvrSession,
    controller_type: u32,
    input_state: *mut OvrInputState,
) -> OvrResult {
    log_call("ovr_GetInputState");
    if input_state.is_null() {
        return -1005;
    }
    #[allow(unused_mut)]
    let mut state = OvrInputState {
        controller_type,
        ..Default::default()
    };
    #[cfg(windows)]
    if let Ok(slot) = XR_D3D12_SESSION.lock() {
        if let Some(session) = slot.as_ref() {
            state.time_in_seconds = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_secs_f64())
                .unwrap_or_default();
            for hand in 0..2 {
                // ovrButton: A/B/RThumb/RShoulder and X/Y/LThumb/LShoulder.
                // The simple-controller select action remains a useful trigger
                // fallback for runtimes without a vendor controller profile.
                if session.hand_primary[hand] || session.hand_select[hand] {
                    state.buttons |= if hand == 0 { 0x0000_0100 } else { 0x0000_0001 };
                }
                if session.hand_secondary[hand] {
                    state.buttons |= if hand == 0 { 0x0000_0200 } else { 0x0000_0002 };
                }
                if session.hand_thumbstick_click[hand] {
                    state.buttons |= if hand == 0 { 0x0000_0400 } else { 0x0000_0004 };
                }
                if session.hand_menu[hand] {
                    state.buttons |= if hand == 0 { 0x0000_0800 } else { 0x0000_0008 };
                }
                // ovrTouch: A/B/RThumb/RIndex and X/Y/LThumb/LIndex.
                if session.hand_primary_touch[hand] {
                    state.touches |= if hand == 0 { 0x0000_0100 } else { 0x0000_0001 };
                }
                if session.hand_secondary_touch[hand] {
                    state.touches |= if hand == 0 { 0x0000_0200 } else { 0x0000_0002 };
                }
                if session.hand_thumbstick_touch[hand] {
                    state.touches |= if hand == 0 { 0x0000_0400 } else { 0x0000_0004 };
                }
                let trigger = session.hand_trigger[hand].max(if session.hand_select[hand] {
                    1.0
                } else {
                    0.0
                });
                state.index_trigger[hand] = trigger;
                state.index_trigger_no_deadzone[hand] = trigger;
                if session.hand_trigger_touch[hand] || trigger > 0.0 {
                    state.touches |= if hand == 0 { 0x0000_1000 } else { 0x0000_0010 };
                }
                state.hand_trigger[hand] = session.hand_squeeze[hand];
                state.hand_trigger_no_deadzone[hand] = session.hand_squeeze[hand];
                state.index_trigger_raw[hand] = trigger;
                state.hand_trigger_raw[hand] = session.hand_squeeze[hand];
                state.thumbstick[hand] = OvrVector2f {
                    x: session.hand_thumbstick[hand].x,
                    y: session.hand_thumbstick[hand].y,
                };
                state.thumbstick_no_deadzone[hand] = state.thumbstick[hand];
                state.thumbstick_raw[hand] = state.thumbstick[hand];
                if session.hand_select[hand]
                    || session.hand_primary[hand]
                    || trigger > 0.0
                    || session.hand_squeeze[hand] > 0.0
                {
                    log_call(&format!(
                        "ovr_GetInputState {} buttons={:#x} trigger={:.2} squeeze={:.2} stick=({:.2},{:.2})",
                        if hand == 0 { "left" } else { "right" },
                        state.buttons,
                        trigger,
                        session.hand_squeeze[hand],
                        state.thumbstick[hand].x,
                        state.thumbstick[hand].y,
                    ));
                }
            }
        }
    }
    unsafe { *input_state = state };
    OVR_SUCCESS
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_GetTrackerDesc(
    _session: OvrSession,
    _tracker_desc_index: u32,
) -> OvrTrackerDesc {
    log_call("ovr_GetTrackerDesc");
    // CV1 constellation-camera envelope; Echo uses this to classify the
    // synthetic trackers as room-scale cameras rather than unknown devices.
    OvrTrackerDesc {
        frustum_hfov_in_radians: 1.75,
        frustum_vfov_in_radians: 1.40,
        frustum_near_z_in_meters: 0.4,
        frustum_far_z_in_meters: 2.5,
    }
}

/// # Safety
/// All output arrays must hold `device_count` entries when non-null.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn ovr_GetDevicePoses(
    _session: OvrSession,
    device_types: *const i32,
    device_count: i32,
    absolute_time: f64,
    out_device_poses: *mut crate::abi::OvrPoseStatef,
    out_tracking_state: *mut OvrTrackingState,
) -> OvrResult {
    log_call("ovr_GetDevicePoses");
    if device_count < 0
        || (device_count > 0 && (device_types.is_null() || out_device_poses.is_null()))
    {
        return -1005;
    }
    let tracking = ovr_GetTrackingState(_session, absolute_time, 0);
    for index in 0..device_count as usize {
        let device_type = unsafe { *device_types.add(index) };
        // ovrTrackedDeviceType is a bitmask: HMD=1, LTouch=2, RTouch=4.
        unsafe {
            *out_device_poses.add(index) = match device_type {
                0x0001 => tracking.head_pose,
                0x0002 => tracking.hand_poses[0],
                0x0004 => tracking.hand_poses[1],
                _ => crate::abi::OvrPoseStatef {
                    pose: tracking.head_pose.pose,
                    time_in_seconds: absolute_time,
                    ..Default::default()
                },
            };
            let pose = (*out_device_poses.add(index)).pose;
            log_call(&format!(
                "ovr_GetDevicePoses type={device_type:#x} pos=({:.3},{:.3},{:.3}) quat=({:.3},{:.3},{:.3},{:.3})",
                pose.position.x,
                pose.position.y,
                pose.position.z,
                pose.orientation.x,
                pose.orientation.y,
                pose.orientation.z,
                pose.orientation.w,
            ));
        }
    }
    if !out_tracking_state.is_null() {
        unsafe { *out_tracking_state = tracking };
    }
    OVR_SUCCESS
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_GetTrackerPose(
    _session: OvrSession,
    tracker_pose_index: u32,
) -> OvrTrackerPose {
    log_call("ovr_GetTrackerPose");
    // Do not collapse the three reported CV1 cameras onto the origin: Echo's
    // hardware screen de-duplicates coincident trackers and showed one red
    // sensor as a result. These are room-scale camera locations in LibOVR's
    // local coordinate system.
    let (position, orientation) = match tracker_pose_index {
        // Facing -Z toward the player at the local origin.
        0 => (
            OvrVector3f {
                x: 0.0,
                y: 1.8,
                z: 1.5,
            },
            crate::abi::OvrQuatf {
                w: 1.0,
                ..Default::default()
            },
        ),
        // Side cameras are yawed inward, so their optical axes intersect the
        // HMD rather than pointing parallel to the front camera.
        1 => (
            OvrVector3f {
                x: -1.5,
                y: 1.8,
                z: 0.3,
            },
            crate::abi::OvrQuatf {
                y: -0.634,
                w: 0.773,
                ..Default::default()
            },
        ),
        _ => (
            OvrVector3f {
                x: 1.5,
                y: 1.8,
                z: 0.3,
            },
            crate::abi::OvrQuatf {
                y: 0.634,
                w: 0.773,
                ..Default::default()
            },
        ),
    };
    let pose = crate::abi::OvrPosef {
        orientation,
        position,
    };
    OvrTrackerPose {
        pose,
        leveled_pose: pose,
        // ovrTracker_Connected | ovrTracker_PoseTracked.
        status_flags: 0x24,
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_GetPredictedDisplayTime(_session: OvrSession, _frame_index: i64) -> f64 {
    log_call("ovr_GetPredictedDisplayTime");
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0.0, |d| d.as_secs_f64())
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_GetTextureSwapChainCurrentIndex(
    _session: OvrSession,
    _chain: OvrTextureSwapChain,
    index: *mut i32,
) -> OvrResult {
    log_call("ovr_GetTextureSwapChainCurrentIndex");
    if index.is_null() {
        return -1005;
    }
    let chains = match SWAP_CHAINS.lock() {
        Ok(chains) => chains,
        Err(_) => return -1000,
    };
    let current = match chains
        .iter()
        .find(|candidate| std::ptr::eq(&***candidate, _chain.cast()))
    {
        Some(chain) => chain.current_index,
        None => return -1005,
    };
    unsafe { *index = current };
    OVR_SUCCESS
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_CommitTextureSwapChain(
    _session: OvrSession,
    _chain: OvrTextureSwapChain,
) -> OvrResult {
    log_call("ovr_CommitTextureSwapChain");
    let mut chains = match SWAP_CHAINS.lock() {
        Ok(chains) => chains,
        Err(_) => return -1000,
    };
    let chain = match swap_chain_state_mut(&mut chains, _chain) {
        Ok(chain) => chain,
        Err(error) => return error,
    };
    // OpenXR selects the color image in ovr_WaitToBeginFrame. Advancing that
    // index here would make Echo render into a different image than the one
    // released to the compositor. The local depth chain still cycles here.
    if !chain.openxr_color {
        chain.current_index = (chain.current_index + 1) % chain.textures.len() as i32;
    }
    OVR_SUCCESS
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_WaitToBeginFrame(_session: OvrSession, _frame_index: i64) -> OvrResult {
    log_call("ovr_WaitToBeginFrame");
    #[cfg(windows)]
    {
        let mut acquired_index = None;
        if let Ok(mut slot) = XR_D3D12_SESSION.lock() {
            if let Some(session) = slot.as_mut() {
                if let Err(error) = session.poll_events().and_then(|()| session.wait_frame()) {
                    log_call(&format!("openxr wait frame failed: {error}"));
                }
                acquired_index = session.color_image.map(|index| index as i32);
            }
        }
        // Echo queries both its color and depth chains after this call. Its
        // chains have identical image counts, so keep their indices aligned.
        if let Some(index) = acquired_index {
            if let Ok(mut chains) = SWAP_CHAINS.lock() {
                for chain in chains.iter_mut() {
                    chain.current_index = index;
                }
            }
        }
    }
    OVR_SUCCESS
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_BeginFrame(_session: OvrSession, _frame_index: i64) -> OvrResult {
    log_call("ovr_BeginFrame");
    #[cfg(windows)]
    if let Ok(mut slot) = XR_D3D12_SESSION.lock() {
        if let Some(session) = slot.as_mut() {
            if let Err(error) = session.begin_frame() {
                log_call(&format!("openxr begin frame failed: {error}"));
            }
        }
    }
    OVR_SUCCESS
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn ovr_EndFrame(
    _session: OvrSession,
    _frame_index: i64,
    _view_scale_desc: *const core::ffi::c_void,
    layer_ptr_list: *const *const core::ffi::c_void,
    layer_count: u32,
) -> OvrResult {
    log_call("ovr_EndFrame");
    // Record the real stereo submission topology before replacing the virtual
    // swapchain with OpenXR-owned D3D12 images. Echo supplies LibOVR-owned,
    // valid pointers for the duration of this call.
    if !layer_ptr_list.is_null() && layer_count != 0 {
        let layer = unsafe { *layer_ptr_list };
        if !layer.is_null() {
            let header = unsafe { &*layer.cast::<OvrLayerHeader>() };
            log_call(&format!(
                "ovr_EndFrame layer_type={} flags={:#x} count={layer_count}",
                header.layer_type, header.flags
            ));
            // `ovrLayerEyeFov` (1) and `ovrLayerEyeFovDepth` (2) share this
            // color-layer prefix; Echo currently submits the latter.
            if matches!(header.layer_type, 1 | 2) {
                let eye_fov = unsafe { &*layer.cast::<OvrLayerEyeFov>() };
                log_call(&format!(
                    "ovr_EndFrame EyeFov{} chains=({:p},{:p}) left={}x{}+{},{} right={}x{}+{},{}",
                    if header.layer_type == 2 { "Depth" } else { "" },
                    eye_fov.color_texture[0],
                    eye_fov.color_texture[1],
                    eye_fov.viewport[0].size.w,
                    eye_fov.viewport[0].size.h,
                    eye_fov.viewport[0].pos.x,
                    eye_fov.viewport[0].pos.y,
                    eye_fov.viewport[1].size.w,
                    eye_fov.viewport[1].size.h,
                    eye_fov.viewport[1].pos.x,
                    eye_fov.viewport[1].pos.y,
                ));
            } else {
                log_call("ovr_EndFrame unsupported layer type");
            }
        }
    }
    #[cfg(windows)]
    if let Ok(mut slot) = XR_D3D12_SESSION.lock() {
        if let Some(session) = slot.as_mut() {
            if let Err(error) = session.end_frame_without_layers() {
                log_call(&format!("openxr end frame failed: {error}"));
            }
        }
    }
    OVR_SUCCESS
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_SubmitControllerVibration(
    _session: OvrSession,
    _controller_type: u32,
    _buffer: *const core::ffi::c_void,
) -> OvrResult {
    log_call("ovr_SubmitControllerVibration");
    OVR_SUCCESS
}

/// Echo uses this before creating its world renderer. The unresolved export
/// previously had a `void` ABI, so the game observed an arbitrary register
/// value and could reject the virtual sensor setup.
#[unsafe(no_mangle)]
pub extern "system" fn ovr_GetTrackerCount(_session: OvrSession) -> u32 {
    log_call("ovr_GetTrackerCount");
    // Rift S uses inside-out tracking. Reporting CV1 cameras alongside a Rift
    // S HMD makes Echo enter its external-sensor validation screen.
    0
}

/// Report the two Touch-style controllers exposed by the OpenXR runtime. Input
/// values remain neutral until action bindings are added, but connectivity must
/// be truthful and deterministic for Echo's device validation.
#[unsafe(no_mangle)]
pub extern "system" fn ovr_GetConnectedControllerTypes(_session: OvrSession) -> u32 {
    log_call("ovr_GetConnectedControllerTypes");
    0x3 // ovrControllerType_LTouch | ovrControllerType_RTouch
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_GetTrackingOriginType(_session: OvrSession) -> i32 {
    log_call("ovr_GetTrackingOriginType");
    TRACKING_ORIGIN.load(Ordering::Acquire)
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_SetTrackingOriginType(_session: OvrSession, origin: i32) -> OvrResult {
    log_call(&format!("ovr_SetTrackingOriginType origin={origin}"));
    if !(0..=1).contains(&origin) {
        return -1005;
    }
    TRACKING_ORIGIN.store(origin, Ordering::Release);
    OVR_SUCCESS
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_RecenterTrackingOrigin(_session: OvrSession) -> OvrResult {
    log_call("ovr_RecenterTrackingOrigin");
    // Qwerty's LOCAL space is already floor-relative. Capture/compose a
    // recenter transform once Echo requests one in the next tracking pass.
    OVR_SUCCESS
}

/// `ovr_GetPerfStats` returns an `ovrResult`; it was previously emitted as a
/// void resolver stub, leaving Echo to read an arbitrary failure code.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn ovr_GetPerfStats(
    _session: OvrSession,
    out_stats: *mut core::ffi::c_void,
) -> OvrResult {
    log_call("ovr_GetPerfStats");
    // The leading fields of CAPI perf stats are counters/timestamps. Zeroing a
    // conservative prefix reports no dropped frames without assuming a newer
    // SDK's complete private layout.
    if !out_stats.is_null() {
        unsafe { core::ptr::write_bytes(out_stats, 0, 64) };
    }
    OVR_SUCCESS
}

/// CAPI property setters return `ovrBool`, not void.
#[unsafe(no_mangle)]
pub extern "system" fn ovr_SetBool(
    _session: OvrSession,
    _property_name: *const c_char,
    _value: u8,
) -> u8 {
    log_call("ovr_SetBool");
    1
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
    for (slot, byte) in product.iter_mut().zip(b"Oculus Rift S") {
        *slot = *byte as c_char;
    }
    let mut manufacturer = [0; 64];
    for (slot, byte) in manufacturer.iter_mut().zip(b"Oculus") {
        *slot = *byte as c_char;
    }
    let mut serial = [0; 24];
    for (slot, byte) in serial.iter_mut().zip(b"OPENXR-RIFTS-0001") {
        *slot = *byte as c_char;
    }
    let fov = OvrFovPort {
        up_tan: 1.0,
        down_tan: 1.0,
        left_tan: 1.0,
        right_tan: 1.0,
    };
    OvrHmdDesc {
        hmd_type: 16, // ovrHmd_RiftS
        product_name: product,
        manufacturer,
        vendor_id: 0x2833,
        product_id: 0x021e,
        serial_number: serial,
        firmware_major: 1,
        firmware_minor: 0,
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

#[unsafe(no_mangle)]
pub extern "system" fn ovr_GetFovTextureSize(
    _session: OvrSession,
    _eye: OvrEyeType,
    fov: OvrFovPort,
    pixels_per_display_pixel: f32,
) -> OvrSizei {
    log_call("ovr_GetFovTextureSize");
    let scale = pixels_per_display_pixel.clamp(0.25, 4.0);
    let width = ((fov.left_tan + fov.right_tan).max(0.1) * 916.0 * scale).ceil() as i32;
    let height = ((fov.up_tan + fov.down_tan).max(0.1) * 960.0 * scale).ceil() as i32;
    OvrSizei {
        w: width,
        h: height,
    }
}

/// # Safety
/// `out_guid` must point to `OVR_AUDIO_MAX_DEVICE_STR_SIZE` UTF-16 code units.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn ovr_GetAudioDeviceOutGuidStr(out_guid: *mut u16) -> OvrResult {
    log_call("ovr_GetAudioDeviceOutGuidStr");
    if out_guid.is_null() {
        return -1005;
    }
    unsafe { core::ptr::write_bytes(out_guid, 0, OVR_AUDIO_MAX_DEVICE_STR_SIZE) };
    OVR_SUCCESS
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_GetRenderDesc2(
    _session: OvrSession,
    eye: OvrEyeType,
    fov: OvrFovPort,
) -> OvrEyeRenderDesc {
    log_call("ovr_GetRenderDesc2");
    OvrEyeRenderDesc {
        eye,
        fov,
        pixels_per_tan_angle_at_center: OvrVector2f { x: 916.0, y: 960.0 },
        hmd_to_eye_offset: OvrVector3f {
            x: if eye == OVR_EYE_LEFT { -0.032 } else { 0.032 },
            y: 0.0,
            z: 0.0,
        },
        ..OvrEyeRenderDesc::default()
    }
}

/// # Safety
/// `out_chain` must be writable. The D3D device and descriptor are accepted
/// for bootstrap only; actual D3D11 resources are implemented in the next
/// compositor stage.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn ovr_CreateTextureSwapChainDX(
    _session: OvrSession,
    _d3d_device: *mut core::ffi::c_void,
    _desc: *const OvrTextureSwapChainDesc,
    out_chain: *mut OvrTextureSwapChain,
) -> OvrResult {
    log_call("ovr_CreateTextureSwapChainDX");
    if out_chain.is_null() {
        return -1005;
    }
    if _d3d_device.is_null() || _desc.is_null() {
        return -1005;
    }
    let desc = unsafe { *_desc };
    let device_vtable = unsafe { *(_d3d_device as *const *const *const core::ffi::c_void) };
    let query_device: QueryInterface = unsafe { core::mem::transmute(*device_vtable.add(0)) };
    let mut queue = core::ptr::null_mut();
    let queue_result = unsafe { query_device(_d3d_device, &IID_ID3D12_COMMAND_QUEUE, &mut queue) };
    if queue_result >= 0 {
        let queue_vtable = unsafe { *(queue as *const *const *const core::ffi::c_void) };
        let get_device: GetDevice = unsafe { core::mem::transmute(*queue_vtable.add(7)) };
        let mut d3d12_device = core::ptr::null_mut();
        let device_result = unsafe { get_device(queue, &IID_ID3D12_DEVICE, &mut d3d12_device) };
        if device_result < 0 {
            log_call(&format!(
                "ovr_CreateTextureSwapChainDX D3D12 GetDevice failed hr={device_result:#x}"
            ));
            let release: Release = unsafe { core::mem::transmute(*queue_vtable.add(2)) };
            unsafe { release(queue) };
            return device_result;
        }
        #[cfg(windows)]
        match unsafe { ensure_d3d12_session(d3d12_device, queue) } {
            Ok(()) => log_call("openxr D3D12 session retained"),
            Err(error) => log_call(&format!("openxr D3D12 session creation failed: {error}")),
        }
        let release: Release = unsafe { core::mem::transmute(*queue_vtable.add(2)) };
        unsafe { release(queue) };
        // ovrTextureFormat values are an Oculus enum, not raw DXGI_FORMAT
        // values. Echo uses RGBA8 sRGB (5) for color and D24S8 (12) for its
        // depth chain. D3D12 requires a typeless resource for the latter.
        let (resource_format, clear_format) = match desc.format {
            4 => (28, 28),  // R8G8B8A8_UNORM
            5 => (29, 29),  // R8G8B8A8_UNORM_SRGB
            6 => (87, 87),  // B8G8R8A8_UNORM
            7 => (91, 91),  // B8G8R8A8_UNORM_SRGB
            10 => (10, 10), // R16G16B16A16_FLOAT
            11 => (55, 55), // D16_UNORM
            12 => (44, 45), // R24G8_TYPELESS resource, D24_UNORM_S8_UINT view
            13 => (40, 40), // D32_FLOAT
            _ => (desc.format as u32, desc.format as u32),
        };
        let resource_desc = D3d12ResourceDesc {
            dimension: 3, // D3D12_RESOURCE_DIMENSION_TEXTURE2D
            alignment: 0,
            width: desc.width.max(1) as u64,
            height: desc.height.max(1) as u32,
            depth_or_array_size: desc.array_size.max(1) as u16,
            mip_levels: desc.mip_levels.max(1) as u16,
            format: resource_format,
            sample_count: desc.sample_count.max(1) as u32,
            sample_quality: 0,
            layout: 0, // D3D12_TEXTURE_LAYOUT_UNKNOWN
            // Keep resources broadly usable until the LibOVR bind-flag to
            // DXGI-format mapping is fully decoded. In particular, the
            // incoming depth descriptor format is not yet a valid D3D12
            // optimized-clear format under Wine vkd3d.
            flags: 0,
        };
        let heap = D3d12HeapProperties {
            heap_type: 1,
            cpu_page_property: 0,
            memory_pool_preference: 0,
            creation_node_mask: 1,
            visible_node_mask: 1,
        };
        let device_vtable = unsafe { *(d3d12_device as *const *const *const core::ffi::c_void) };
        let create: CreateCommittedResource =
            unsafe { core::mem::transmute(*device_vtable.add(27)) };
        let depth_clear = D3d12ClearValue {
            format: clear_format,
            depth: 1.0,
            stencil: 0,
            _padding: [0; 3],
        };
        let optimized_clear = if resource_desc.flags & 0x2 != 0 {
            (&depth_clear as *const D3d12ClearValue).cast()
        } else {
            core::ptr::null()
        };
        #[cfg(windows)]
        if desc.bind_flags & 1 != 0 {
            if let Ok(mut slot) = XR_D3D12_SESSION.lock() {
                if let Some(session) = slot.as_mut() {
                    match session.create_color_swapchain(
                        resource_desc.format,
                        desc.width.max(1) as u32,
                        desc.height.max(1) as u32,
                        desc.array_size.max(1) as u32,
                        desc.mip_levels.max(1) as u32,
                        desc.sample_count.max(1) as u32,
                    ) {
                        Ok(images) if images.len() >= 3 => {
                            let textures = [images[0], images[1], images[2]];
                            let release: Release =
                                unsafe { core::mem::transmute(*device_vtable.add(2)) };
                            unsafe { release(d3d12_device) };
                            let chain = match register_swap_chain(textures, true) {
                                Ok(chain) => chain,
                                Err(error) => return error,
                            };
                            unsafe { *out_chain = chain };
                            log_call("ovr_CreateTextureSwapChainDX using OpenXR D3D12 images");
                            return OVR_SUCCESS;
                        }
                        Ok(images) => log_call(&format!(
                            "OpenXR color swapchain returned only {} images",
                            images.len()
                        )),
                        Err(error) => {
                            log_call(&format!("OpenXR color swapchain creation failed: {error}"))
                        }
                    }
                }
            }
        }
        let mut textures = [0; 3];
        for texture in &mut textures {
            let mut created = core::ptr::null_mut();
            let result = unsafe {
                create(
                    d3d12_device,
                    &heap,
                    0,
                    &resource_desc,
                    0,
                    optimized_clear,
                    &IID_ID3D12_RESOURCE,
                    &mut created,
                )
            };
            if result < 0 {
                log_call(&format!(
                    "ovr_CreateTextureSwapChainDX D3D12 CreateCommittedResource failed hr={result:#x}"
                ));
                return result;
            }
            *texture = created as usize;
        }
        let release: Release = unsafe { core::mem::transmute(*device_vtable.add(2)) };
        unsafe { release(d3d12_device) };
        log_call(&format!(
            "ovr_CreateTextureSwapChainDX D3D12 {}x{} array={} mip={} samples={} format={} bind={:#x} flags={:#x}",
            resource_desc.width,
            resource_desc.height,
            resource_desc.depth_or_array_size,
            resource_desc.mip_levels,
            resource_desc.sample_count,
            resource_desc.format,
            desc.bind_flags,
            resource_desc.flags
        ));
        let chain = match register_swap_chain(textures, false) {
            Ok(chain) => chain,
            Err(error) => return error,
        };
        unsafe { *out_chain = chain };
        return OVR_SUCCESS;
    }
    let d3d_desc = D3d11Texture2dDesc {
        width: desc.width.max(1) as u32,
        height: desc.height.max(1) as u32,
        mip_levels: desc.mip_levels.max(1) as u32,
        array_size: desc.array_size.max(1) as u32,
        format: desc.format as u32,
        sample_count: desc.sample_count.max(1) as u32,
        sample_quality: 0,
        usage: 0, // D3D11_USAGE_DEFAULT
        bind_flags: if desc.bind_flags == 0 {
            0x28
        } else {
            desc.bind_flags
        },
        cpu_access_flags: 0,
        misc_flags: desc.misc_flags,
    };
    let vtable = unsafe { *(_d3d_device as *const *const *const core::ffi::c_void) };
    let create: CreateTexture2d = unsafe { core::mem::transmute(*vtable.add(5)) };
    let mut textures = [0; 3];
    for texture in &mut textures {
        let mut created = core::ptr::null_mut();
        let result = unsafe { create(_d3d_device, &d3d_desc, core::ptr::null(), &mut created) };
        if result < 0 {
            log_call(&format!(
                "ovr_CreateTextureSwapChainDX failed hr={result:#x} format={} {}x{} bind={:#x}",
                d3d_desc.format, d3d_desc.width, d3d_desc.height, d3d_desc.bind_flags
            ));
            return result;
        }
        *texture = created as usize;
    }
    let chain = match register_swap_chain(textures, false) {
        Ok(chain) => chain,
        Err(error) => return error,
    };
    unsafe { *out_chain = chain };
    OVR_SUCCESS
}

/// # Safety
/// `out_length` must be writable.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn ovr_GetTextureSwapChainLength(
    _session: OvrSession,
    _chain: OvrTextureSwapChain,
    out_length: *mut i32,
) -> OvrResult {
    log_call("ovr_GetTextureSwapChainLength");
    if out_length.is_null() {
        return -1005;
    }
    unsafe { *out_length = 3 };
    OVR_SUCCESS
}

/// # Safety
/// `out_buffer` must be writable and `iid` must identify a D3D interface.
///
/// LibOVR declares `IID` by value (a 16-byte GUID), not as a GUID pointer.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn ovr_GetTextureSwapChainBufferDX(
    _session: OvrSession,
    _chain: OvrTextureSwapChain,
    index: i32,
    iid: Guid,
    out_buffer: *mut *mut core::ffi::c_void,
) -> OvrResult {
    log_call("ovr_GetTextureSwapChainBufferDX");
    if out_buffer.is_null() {
        return -1005;
    }
    let texture = match swap_chain_texture(_chain, index) {
        Ok(texture) => texture as *mut core::ffi::c_void,
        Err(error) => return error,
    };
    if texture.is_null() {
        return -1004;
    }
    let vtable = unsafe { *(texture as *const *const *const core::ffi::c_void) };
    let query: QueryInterface = unsafe { core::mem::transmute(*vtable.add(0)) };
    let result = unsafe { query(texture, &iid, out_buffer) };
    if result < 0 {
        log_call(&format!(
            "ovr_GetTextureSwapChainBufferDX failed hr={result:#x} iid={iid:?}"
        ));
        return result;
    }
    log_call(&format!(
        "ovr_GetTextureSwapChainBufferDX ok iid={iid:?} buffer={:p}",
        unsafe { *out_buffer }
    ));
    OVR_SUCCESS
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
    ovr_ClearShouldRecenterFlag,
    ovr_CreateMirrorTextureDX,
    ovr_CreateMirrorTextureGL,
    ovr_CreateMirrorTextureWithOptionsDX,
    ovr_CreateMirrorTextureWithOptionsGL,
    ovr_CreateMirrorTextureWithOptionsVk,
    ovr_CreateTextureSwapChainGL,
    ovr_CreateTextureSwapChainVk,
    ovr_DestroyMirrorTexture,
    ovr_DestroyTextureSwapChain,
    ovr_EnableExtension,
    ovr_GetAudioDeviceInGuid,
    ovr_GetAudioDeviceInGuidStr,
    ovr_GetAudioDeviceInWaveId,
    ovr_GetAudioDeviceOutGuid,
    ovr_GetAudioDeviceOutWaveId,
    ovr_GetBool,
    ovr_GetBoundaryDimensions,
    ovr_GetBoundaryGeometry,
    ovr_GetBoundaryVisible,
    ovr_GetControllerVibrationState,
    ovr_GetDeviceExtensionsVk,
    ovr_GetExternalCameras,
    ovr_GetFloat,
    ovr_GetFloatArray,
    ovr_GetFovStencil,
    ovr_GetHmdColorDesc,
    ovr_GetInstanceExtensionsVk,
    ovr_GetInt,
    ovr_GetMirrorTextureBufferDX,
    ovr_GetMirrorTextureBufferGL,
    ovr_GetMirrorTextureBufferVk,
    ovr_GetSessionPhysicalDeviceVk,
    ovr_GetString,
    ovr_GetTextureSwapChainBufferGL,
    ovr_GetTextureSwapChainBufferVk,
    ovr_GetTextureSwapChainDesc,
    ovr_GetTouchHapticsDesc,
    ovr_IdentifyClient,
    ovr_IsExtensionSupported,
    ovr_Lookup,
    ovr_ReportClientInfo,
    ovr_RequestBoundaryVisible,
    ovr_ResetBoundaryLookAndFeel,
    ovr_ResetPerfStats,
    ovr_SetBoundaryLookAndFeel,
    ovr_SetClientColorDesc,
    ovr_SetControllerVibration,
    ovr_SetExternalCameraProperties,
    ovr_SetFloat,
    ovr_SetFloatArray,
    ovr_SetInt,
    ovr_SetString,
    ovr_SetSynchronizationQueueVk,
    ovr_SpecifyTrackingOrigin,
    ovr_SubmitFrame2,
    ovr_TestBoundary,
    ovr_TestBoundaryPoint,
    ovr_TraceMessage,
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
