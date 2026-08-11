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
    OVR_AUDIO_MAX_DEVICE_STR_SIZE, OVR_EYE_LEFT, OVR_SUCCESS, OvrErrorInfo, OvrEyeRenderDesc,
    OvrEyeType, OvrFovPort, OvrGraphicsLuid, OvrHmdDesc, OvrInitParams, OvrInputState, OvrResult,
    OvrSession, OvrSessionStatus, OvrSizei, OvrTextureSwapChain, OvrTextureSwapChainDesc,
    OvrTrackerPose, OvrTrackingState, OvrVector2f, OvrVector3f, OvrVersionString,
};

static INITIALIZED: AtomicBool = AtomicBool::new(false);
static SESSION_TOKEN: u8 = 1;
static SWAP_CHAIN_TOKEN: u8 = 2;
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

static SWAP_TEXTURES: std::sync::Mutex<[usize; 3]> = std::sync::Mutex::new([0; 3]);

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
    if std::env::var_os("LIBOVR_OPENXR_PROBE").is_some() {
        log_call(&format!(
            "openxr env XR_RUNTIME_JSON={:?} XDG_RUNTIME_DIR={:?}",
            std::env::var_os("XR_RUNTIME_JSON"),
            std::env::var_os("XDG_RUNTIME_DIR")
        ));
        match crate::openxr_backend::probe() {
            Ok(capabilities) => log_call(&format!(
                "openxr probe d3d11={} mnd_headless={}",
                capabilities.d3d11, capabilities.monado_headless
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
            *luid = OvrGraphicsLuid::default();
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
        // ovrStatus_OrientationTracked | ovrStatus_PositionTracked.
        status_flags: 0x3,
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
    unsafe {
        *input_state = OvrInputState {
            controller_type,
            ..Default::default()
        };
    }
    OVR_SUCCESS
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_GetTrackerPose(
    _session: OvrSession,
    _tracker_pose_index: u32,
) -> OvrTrackerPose {
    log_call("ovr_GetTrackerPose");
    OvrTrackerPose {
        pose: crate::abi::OvrPosef {
            orientation: crate::abi::OvrQuatf {
                w: 1.0,
                ..Default::default()
            },
            ..Default::default()
        },
        leveled_pose: crate::abi::OvrPosef {
            orientation: crate::abi::OvrQuatf {
                w: 1.0,
                ..Default::default()
            },
            ..Default::default()
        },
        status_flags: 0x3,
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
    unsafe {
        *index = 0;
    }
    OVR_SUCCESS
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_CommitTextureSwapChain(
    _session: OvrSession,
    _chain: OvrTextureSwapChain,
) -> OvrResult {
    log_call("ovr_CommitTextureSwapChain");
    OVR_SUCCESS
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_WaitToBeginFrame(_session: OvrSession, _frame_index: i64) -> OvrResult {
    log_call("ovr_WaitToBeginFrame");
    OVR_SUCCESS
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_BeginFrame(_session: OvrSession, _frame_index: i64) -> OvrResult {
    log_call("ovr_BeginFrame");
    OVR_SUCCESS
}

#[unsafe(no_mangle)]
pub extern "system" fn ovr_EndFrame(
    _session: OvrSession,
    _frame_index: i64,
    _view_scale_desc: *const core::ffi::c_void,
    _layer_ptr_list: *const *const core::ffi::c_void,
    _layer_count: u32,
) -> OvrResult {
    log_call("ovr_EndFrame");
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
        let release: Release = unsafe { core::mem::transmute(*queue_vtable.add(2)) };
        unsafe { release(queue) };
        if device_result < 0 {
            log_call(&format!(
                "ovr_CreateTextureSwapChainDX D3D12 GetDevice failed hr={device_result:#x}"
            ));
            return device_result;
        }
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
        let mut textures = match SWAP_TEXTURES.lock() {
            Ok(textures) => textures,
            Err(_) => return -1000,
        };
        for texture in textures.iter_mut() {
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
            "ovr_CreateTextureSwapChainDX D3D12 {}x{} format={} bind={:#x} flags={:#x}",
            resource_desc.width,
            resource_desc.height,
            resource_desc.format,
            desc.bind_flags,
            resource_desc.flags
        ));
        unsafe { *out_chain = (&SWAP_CHAIN_TOKEN as *const u8).cast_mut().cast() };
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
    let mut textures = match SWAP_TEXTURES.lock() {
        Ok(textures) => textures,
        Err(_) => return -1000,
    };
    for texture in textures.iter_mut() {
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
    unsafe { *out_chain = (&SWAP_CHAIN_TOKEN as *const u8).cast_mut().cast() };
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
    if out_buffer.is_null() || !(0..3).contains(&index) {
        return -1005;
    }
    let textures = match SWAP_TEXTURES.lock() {
        Ok(textures) => textures,
        Err(_) => return -1000,
    };
    let texture = textures[index as usize] as *mut core::ffi::c_void;
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
    ovr_GetConnectedControllerTypes,
    ovr_GetControllerVibrationState,
    ovr_GetDeviceExtensionsVk,
    ovr_GetDevicePoses,
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
    ovr_GetPerfStats,
    ovr_GetSessionPhysicalDeviceVk,
    ovr_GetString,
    ovr_GetTextureSwapChainBufferGL,
    ovr_GetTextureSwapChainBufferVk,
    ovr_GetTextureSwapChainDesc,
    ovr_GetTouchHapticsDesc,
    ovr_GetTrackerCount,
    ovr_GetTrackerDesc,
    ovr_GetTrackingOriginType,
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
