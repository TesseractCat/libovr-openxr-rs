//! Audited-in-stages C ABI definitions for the LibOVR CAPI 1.94 bootstrap.
//!
//! Only structures used by the bootstrap exports belong here. Every added type
//! needs a layout test against the official CAPI headers before it is relied on
//! for rendering or input.

use core::ffi::{c_char, c_void};

pub type OvrResult = i32;
pub type OvrBool = u8;
pub type OvrSession = *mut c_void;

pub const OVR_SUCCESS: OvrResult = 0;

/// Opaque until the complete CAPI 1.94 `ovrInitParams` layout is imported.
#[repr(C)]
#[derive(Debug)]
pub struct OvrInitParams {
    _private: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct OvrGraphicsLuid {
    pub reserved: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct OvrErrorInfo {
    pub result: OvrResult,
    pub error_string: [c_char; 512],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct OvrSessionStatus {
    pub is_visible: OvrBool,
    pub hmd_present: OvrBool,
    pub hmd_mounted: OvrBool,
    pub display_lost: OvrBool,
    pub should_quit: OvrBool,
    pub should_recenter: OvrBool,
    pub has_input_focus: OvrBool,
    pub overlay_present: OvrBool,
    pub depth_requested: OvrBool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct OvrSizei {
    pub w: i32,
    pub h: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct OvrFovPort {
    pub up_tan: f32,
    pub down_tan: f32,
    pub left_tan: f32,
    pub right_tan: f32,
}

pub type OvrEyeType = i32;
pub type OvrTextureSwapChain = *mut c_void;
pub const OVR_EYE_LEFT: OvrEyeType = 0;
pub const OVR_EYE_RIGHT: OvrEyeType = 1;
pub const OVR_AUDIO_MAX_DEVICE_STR_SIZE: usize = 128;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct OvrVector2i {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct OvrRecti {
    pub pos: OvrVector2i,
    pub size: OvrSizei,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct OvrVector2f {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct OvrVector3f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// CAPI 1.94 `ovrEyeRenderDesc`; returned by value by `ovr_GetRenderDesc2`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct OvrEyeRenderDesc {
    pub eye: OvrEyeType,
    pub fov: OvrFovPort,
    pub distorted_viewport: OvrRecti,
    pub pixels_per_tan_angle_at_center: OvrVector2f,
    pub hmd_to_eye_offset: OvrVector3f,
}

/// CAPI 1.94 `ovrTextureSwapChainDesc`. DX texture creation only needs this
/// layout at this stage; D3D texture allocation is deliberately deferred.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct OvrTextureSwapChainDesc {
    pub texture_type: i32,
    pub format: i32,
    pub array_size: i32,
    pub width: i32,
    pub height: i32,
    pub mip_levels: i32,
    pub sample_count: i32,
    pub static_image: OvrBool,
    pub misc_flags: u32,
    pub bind_flags: u32,
}

/// CAPI 1.94 `ovrHmdDesc`.  Its layout is required by `ovr_GetHmdDesc`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct OvrHmdDesc {
    pub hmd_type: i32,
    pub product_name: [c_char; 64],
    pub manufacturer: [c_char; 64],
    pub vendor_id: i16,
    pub product_id: i16,
    pub serial_number: [c_char; 24],
    pub firmware_major: i16,
    pub firmware_minor: i16,
    pub resolution: OvrSizei,
    pub default_eye_fov: [OvrFovPort; 2],
    pub max_eye_fov: [OvrFovPort; 2],
    pub default_eye: i32,
    pub display_refresh_rate: i32,
}

pub type OvrVersionString = *const c_char;
