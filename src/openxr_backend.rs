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

/// The OpenXR objects that must outlive every LibOVR frame and swapchain. Keeping
/// the waiter and stream together also enforces the OpenXR frame ordering.
#[cfg(windows)]
pub struct D3d12Session {
    pub instance: openxr::Instance,
    pub session: openxr::Session<openxr::D3D12>,
    pub waiter: openxr::FrameWaiter,
    pub stream: openxr::FrameStream<openxr::D3D12>,
    pub space: openxr::Space,
    pub view_space: openxr::Space,
    pub head_pose: openxr::Posef,
    pub hand_poses: [openxr::Posef; 2],
    pub input_action_set: openxr::ActionSet,
    pub select_action: openxr::Action<bool>,
    pub primary_action: openxr::Action<bool>,
    pub secondary_action: openxr::Action<bool>,
    pub menu_action: openxr::Action<bool>,
    pub thumbstick_click_action: openxr::Action<bool>,
    pub primary_touch_action: openxr::Action<bool>,
    pub secondary_touch_action: openxr::Action<bool>,
    pub thumbstick_touch_action: openxr::Action<bool>,
    pub trigger_touch_action: openxr::Action<bool>,
    pub haptic_action: openxr::Action<openxr::Haptic>,
    pub trigger_action: openxr::Action<f32>,
    pub squeeze_action: openxr::Action<f32>,
    pub thumbstick_action: openxr::Action<openxr::Vector2f>,
    pub hand_spaces: [openxr::Space; 2],
    pub hand_select: [bool; 2],
    pub hand_primary: [bool; 2],
    pub hand_secondary: [bool; 2],
    pub hand_menu: [bool; 2],
    pub hand_thumbstick_click: [bool; 2],
    pub hand_primary_touch: [bool; 2],
    pub hand_secondary_touch: [bool; 2],
    pub hand_thumbstick_touch: [bool; 2],
    pub hand_trigger_touch: [bool; 2],
    pub hand_trigger: [f32; 2],
    pub hand_squeeze: [f32; 2],
    pub hand_thumbstick: [openxr::Vector2f; 2],
    pub last_logged_hand_poses: [openxr::Posef; 2],
    /// OpenXR's safe Rust wrapper does not expose XR_SPACE_VELOCITY, so keep
    /// a finite-difference estimate at the same action-sync cadence.
    pub last_velocity_hand_poses: [openxr::Posef; 2],
    pub last_velocity_sample: Option<std::time::Instant>,
    pub hand_linear_velocity: [openxr::Vector3f; 2],
    pub hand_angular_velocity: [openxr::Vector3f; 2],
    pub running: bool,
    pub frame_state: Option<openxr::FrameState>,
    pub frame_begun: bool,
    pub color_swapchain: Option<openxr::Swapchain<openxr::D3D12>>,
    pub color_image: Option<u32>,
    pub color_extent: Option<(u32, u32)>,
}

#[cfg(windows)]
impl core::fmt::Debug for D3d12Session {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("D3d12Session")
    }
}

/// Discover the adapter selected by the active OpenXR runtime before Echo
/// creates its D3D12 queue. LibOVR exposes these bytes from `ovr_Create`.
#[cfg(windows)]
pub fn d3d12_adapter_luid() -> Result<[u8; 8], String> {
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
    let requirements = instance
        .graphics_requirements::<openxr::D3D12>(system)
        .map_err(|error| error.to_string())?;
    // Windows LUID is an 8-byte C structure; LibOVR uses the same representation.
    Ok(unsafe { core::mem::transmute(requirements.adapter_luid) })
}

/// Create the retained OpenXR session used by the LibOVR frame bridge.
#[cfg(windows)]
pub unsafe fn create_d3d12_session(
    device: *mut core::ffi::c_void,
    queue: *mut core::ffi::c_void,
) -> Result<D3d12Session, String> {
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
    let (session, waiter, stream) =
        unsafe { instance.create_session::<openxr::D3D12>(system, &info) }
            .map_err(|error| error.to_string())?;
    let identity = openxr::Posef {
        orientation: openxr::Quaternionf {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        },
        position: openxr::Vector3f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
    };
    let space = session
        .create_reference_space(openxr::ReferenceSpaceType::LOCAL, identity)
        .map_err(|error| error.to_string())?;
    let view_space = session
        .create_reference_space(openxr::ReferenceSpaceType::VIEW, identity)
        .map_err(|error| error.to_string())?;
    let input_action_set = instance
        .create_action_set("echovr_input", "Echo VR controller poses", 0)
        .map_err(|error| error.to_string())?;
    let left_path = instance
        .string_to_path("/user/hand/left")
        .map_err(|error| error.to_string())?;
    let right_path = instance
        .string_to_path("/user/hand/right")
        .map_err(|error| error.to_string())?;
    // Explicit subaction paths prevent the runtime from resolving both action
    // spaces to its default (left) controller source.
    let hand_action = input_action_set
        .create_action::<openxr::Posef>("hand_pose", "Hand pose", &[left_path, right_path])
        .map_err(|error| error.to_string())?;
    let select_action = input_action_set
        .create_action::<bool>("select", "Select", &[left_path, right_path])
        .map_err(|error| error.to_string())?;
    let primary_action = input_action_set
        .create_action::<bool>("primary", "Primary", &[left_path, right_path])
        .map_err(|error| error.to_string())?;
    let secondary_action = input_action_set
        .create_action::<bool>("secondary", "Secondary", &[left_path, right_path])
        .map_err(|error| error.to_string())?;
    let menu_action = input_action_set
        .create_action::<bool>("menu", "Menu", &[left_path, right_path])
        .map_err(|error| error.to_string())?;
    let thumbstick_click_action = input_action_set
        .create_action::<bool>(
            "thumbstick_click",
            "Thumbstick click",
            &[left_path, right_path],
        )
        .map_err(|error| error.to_string())?;
    let primary_touch_action = input_action_set
        .create_action::<bool>("primary_touch", "Primary touch", &[left_path, right_path])
        .map_err(|error| error.to_string())?;
    let secondary_touch_action = input_action_set
        .create_action::<bool>(
            "secondary_touch",
            "Secondary touch",
            &[left_path, right_path],
        )
        .map_err(|error| error.to_string())?;
    let thumbstick_touch_action = input_action_set
        .create_action::<bool>(
            "thumbstick_touch",
            "Thumbstick touch",
            &[left_path, right_path],
        )
        .map_err(|error| error.to_string())?;
    let trigger_touch_action = input_action_set
        .create_action::<bool>("trigger_touch", "Trigger touch", &[left_path, right_path])
        .map_err(|error| error.to_string())?;
    let haptic_action = input_action_set
        .create_action::<openxr::Haptic>("haptic", "Haptic output", &[left_path, right_path])
        .map_err(|error| error.to_string())?;
    let trigger_action = input_action_set
        .create_action::<f32>("trigger", "Trigger", &[left_path, right_path])
        .map_err(|error| error.to_string())?;
    let squeeze_action = input_action_set
        .create_action::<f32>("squeeze", "Squeeze", &[left_path, right_path])
        .map_err(|error| error.to_string())?;
    let thumbstick_action = input_action_set
        .create_action::<openxr::Vector2f>("thumbstick", "Thumbstick", &[left_path, right_path])
        .map_err(|error| error.to_string())?;
    instance
        .suggest_interaction_profile_bindings(
            instance
                .string_to_path("/interaction_profiles/khr/simple_controller")
                .map_err(|error| error.to_string())?,
            &[
                openxr::Binding::new(
                    &hand_action,
                    instance
                        .string_to_path("/user/hand/left/input/grip/pose")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &hand_action,
                    instance
                        .string_to_path("/user/hand/right/input/grip/pose")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &select_action,
                    instance
                        .string_to_path("/user/hand/left/input/select/click")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &select_action,
                    instance
                        .string_to_path("/user/hand/right/input/select/click")
                        .map_err(|error| error.to_string())?,
                ),
            ],
        )
        .map_err(|error| error.to_string())?;
    instance
        .suggest_interaction_profile_bindings(
            instance
                .string_to_path("/interaction_profiles/oculus/touch_controller")
                .map_err(|error| error.to_string())?,
            &[
                openxr::Binding::new(
                    &hand_action,
                    instance
                        .string_to_path("/user/hand/left/input/grip/pose")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &hand_action,
                    instance
                        .string_to_path("/user/hand/right/input/grip/pose")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &primary_action,
                    instance
                        .string_to_path("/user/hand/left/input/x/click")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &primary_action,
                    instance
                        .string_to_path("/user/hand/right/input/a/click")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &secondary_action,
                    instance
                        .string_to_path("/user/hand/left/input/y/click")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &secondary_action,
                    instance
                        .string_to_path("/user/hand/right/input/b/click")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &menu_action,
                    instance
                        .string_to_path("/user/hand/left/input/menu/click")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &thumbstick_click_action,
                    instance
                        .string_to_path("/user/hand/left/input/thumbstick/click")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &thumbstick_click_action,
                    instance
                        .string_to_path("/user/hand/right/input/thumbstick/click")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &primary_touch_action,
                    instance
                        .string_to_path("/user/hand/left/input/x/touch")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &primary_touch_action,
                    instance
                        .string_to_path("/user/hand/right/input/a/touch")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &secondary_touch_action,
                    instance
                        .string_to_path("/user/hand/left/input/y/touch")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &secondary_touch_action,
                    instance
                        .string_to_path("/user/hand/right/input/b/touch")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &thumbstick_touch_action,
                    instance
                        .string_to_path("/user/hand/left/input/thumbstick/touch")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &thumbstick_touch_action,
                    instance
                        .string_to_path("/user/hand/right/input/thumbstick/touch")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &trigger_touch_action,
                    instance
                        .string_to_path("/user/hand/left/input/trigger/touch")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &trigger_touch_action,
                    instance
                        .string_to_path("/user/hand/right/input/trigger/touch")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &trigger_action,
                    instance
                        .string_to_path("/user/hand/left/input/trigger/value")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &trigger_action,
                    instance
                        .string_to_path("/user/hand/right/input/trigger/value")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &squeeze_action,
                    instance
                        .string_to_path("/user/hand/left/input/squeeze/value")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &squeeze_action,
                    instance
                        .string_to_path("/user/hand/right/input/squeeze/value")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &thumbstick_action,
                    instance
                        .string_to_path("/user/hand/left/input/thumbstick")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &thumbstick_action,
                    instance
                        .string_to_path("/user/hand/right/input/thumbstick")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &haptic_action,
                    instance
                        .string_to_path("/user/hand/left/output/haptic")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &haptic_action,
                    instance
                        .string_to_path("/user/hand/right/output/haptic")
                        .map_err(|error| error.to_string())?,
                ),
            ],
        )
        .map_err(|error| error.to_string())?;
    // Non-Oculus OpenXR controllers use the same abstract LibOVR Touch
    // semantics.  These cover the controller profiles commonly exposed by
    // SteamVR, WMR, and native OpenXR runtimes; unsupported suggestions are
    // harmless because profile selection happens at runtime.
    for (profile, bindings) in [
        (
            "/interaction_profiles/valve/index_controller",
            vec![
                openxr::Binding::new(
                    &hand_action,
                    instance
                        .string_to_path("/user/hand/left/input/grip/pose")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &hand_action,
                    instance
                        .string_to_path("/user/hand/right/input/grip/pose")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &primary_action,
                    instance
                        .string_to_path("/user/hand/left/input/a/click")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &primary_action,
                    instance
                        .string_to_path("/user/hand/right/input/a/click")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &secondary_action,
                    instance
                        .string_to_path("/user/hand/left/input/b/click")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &secondary_action,
                    instance
                        .string_to_path("/user/hand/right/input/b/click")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &menu_action,
                    instance
                        .string_to_path("/user/hand/left/input/system/click")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &trigger_action,
                    instance
                        .string_to_path("/user/hand/left/input/trigger/value")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &trigger_action,
                    instance
                        .string_to_path("/user/hand/right/input/trigger/value")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &squeeze_action,
                    instance
                        .string_to_path("/user/hand/left/input/squeeze/value")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &squeeze_action,
                    instance
                        .string_to_path("/user/hand/right/input/squeeze/value")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &thumbstick_action,
                    instance
                        .string_to_path("/user/hand/left/input/thumbstick")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &thumbstick_action,
                    instance
                        .string_to_path("/user/hand/right/input/thumbstick")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &thumbstick_click_action,
                    instance
                        .string_to_path("/user/hand/left/input/thumbstick/click")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &thumbstick_click_action,
                    instance
                        .string_to_path("/user/hand/right/input/thumbstick/click")
                        .map_err(|error| error.to_string())?,
                ),
            ],
        ),
        (
            "/interaction_profiles/microsoft/motion_controller",
            vec![
                openxr::Binding::new(
                    &hand_action,
                    instance
                        .string_to_path("/user/hand/left/input/grip/pose")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &hand_action,
                    instance
                        .string_to_path("/user/hand/right/input/grip/pose")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &primary_action,
                    instance
                        .string_to_path("/user/hand/left/input/select/click")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &primary_action,
                    instance
                        .string_to_path("/user/hand/right/input/select/click")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &menu_action,
                    instance
                        .string_to_path("/user/hand/left/input/menu/click")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &trigger_action,
                    instance
                        .string_to_path("/user/hand/left/input/trigger/value")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &trigger_action,
                    instance
                        .string_to_path("/user/hand/right/input/trigger/value")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &squeeze_action,
                    instance
                        .string_to_path("/user/hand/left/input/squeeze/value")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &squeeze_action,
                    instance
                        .string_to_path("/user/hand/right/input/squeeze/value")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &thumbstick_action,
                    instance
                        .string_to_path("/user/hand/left/input/thumbstick")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &thumbstick_action,
                    instance
                        .string_to_path("/user/hand/right/input/thumbstick")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &thumbstick_click_action,
                    instance
                        .string_to_path("/user/hand/left/input/thumbstick/click")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &thumbstick_click_action,
                    instance
                        .string_to_path("/user/hand/right/input/thumbstick/click")
                        .map_err(|error| error.to_string())?,
                ),
            ],
        ),
        (
            "/interaction_profiles/htc/vive_controller",
            vec![
                openxr::Binding::new(
                    &hand_action,
                    instance
                        .string_to_path("/user/hand/left/input/grip/pose")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &hand_action,
                    instance
                        .string_to_path("/user/hand/right/input/grip/pose")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &primary_action,
                    instance
                        .string_to_path("/user/hand/left/input/select/click")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &primary_action,
                    instance
                        .string_to_path("/user/hand/right/input/select/click")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &menu_action,
                    instance
                        .string_to_path("/user/hand/left/input/menu/click")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &trigger_action,
                    instance
                        .string_to_path("/user/hand/left/input/trigger/value")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &trigger_action,
                    instance
                        .string_to_path("/user/hand/right/input/trigger/value")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &thumbstick_action,
                    instance
                        .string_to_path("/user/hand/left/input/trackpad")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &thumbstick_action,
                    instance
                        .string_to_path("/user/hand/right/input/trackpad")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &thumbstick_click_action,
                    instance
                        .string_to_path("/user/hand/left/input/trackpad/click")
                        .map_err(|error| error.to_string())?,
                ),
                openxr::Binding::new(
                    &thumbstick_click_action,
                    instance
                        .string_to_path("/user/hand/right/input/trackpad/click")
                        .map_err(|error| error.to_string())?,
                ),
            ],
        ),
    ] {
        if let Err(error) = instance.suggest_interaction_profile_bindings(
            instance
                .string_to_path(profile)
                .map_err(|error| error.to_string())?,
            &bindings,
        ) {
            // Optional profile mismatches must not prevent the base Touch or
            // simple-controller session from starting.
            crate::capi::log_call(&format!(
                "OpenXR optional interaction profile {profile} unavailable: {error}"
            ));
        }
    }
    session
        .attach_action_sets(&[&input_action_set])
        .map_err(|error| error.to_string())?;
    let hand_spaces = [
        hand_action
            .create_space(&session, left_path, identity)
            .map_err(|error| error.to_string())?,
        hand_action
            .create_space(&session, right_path, identity)
            .map_err(|error| error.to_string())?,
    ];
    Ok(D3d12Session {
        instance,
        session,
        waiter,
        stream,
        space,
        view_space,
        head_pose: identity,
        hand_poses: [identity; 2],
        input_action_set,
        select_action,
        primary_action,
        secondary_action,
        menu_action,
        thumbstick_click_action,
        primary_touch_action,
        secondary_touch_action,
        thumbstick_touch_action,
        trigger_touch_action,
        haptic_action,
        trigger_action,
        squeeze_action,
        thumbstick_action,
        hand_spaces,
        hand_select: [false; 2],
        hand_primary: [false; 2],
        hand_secondary: [false; 2],
        hand_menu: [false; 2],
        hand_thumbstick_click: [false; 2],
        hand_primary_touch: [false; 2],
        hand_secondary_touch: [false; 2],
        hand_thumbstick_touch: [false; 2],
        hand_trigger_touch: [false; 2],
        hand_trigger: [0.0; 2],
        hand_squeeze: [0.0; 2],
        hand_thumbstick: [openxr::Vector2f { x: 0.0, y: 0.0 }; 2],
        last_logged_hand_poses: [identity; 2],
        last_velocity_hand_poses: [identity; 2],
        last_velocity_sample: None,
        hand_linear_velocity: [openxr::Vector3f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }; 2],
        hand_angular_velocity: [openxr::Vector3f {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }; 2],
        running: false,
        frame_state: None,
        frame_begun: false,
        color_swapchain: None,
        color_image: None,
        color_extent: None,
    })
}

#[cfg(windows)]
fn finite_difference_velocity(
    previous: openxr::Posef,
    current: openxr::Posef,
    dt: f32,
) -> (openxr::Vector3f, openxr::Vector3f) {
    let linear = openxr::Vector3f {
        x: (current.position.x - previous.position.x) / dt,
        y: (current.position.y - previous.position.y) / dt,
        z: (current.position.z - previous.position.z) / dt,
    };
    // q_delta = current * conjugate(previous); use the shortest arc so a
    // quaternion sign flip cannot generate an artificial throw-speed spike.
    let mut x = current.orientation.w * -previous.orientation.x
        + current.orientation.x * previous.orientation.w
        + current.orientation.y * -previous.orientation.z
        - current.orientation.z * -previous.orientation.y;
    let mut y = current.orientation.w * -previous.orientation.y
        - current.orientation.x * -previous.orientation.z
        + current.orientation.y * previous.orientation.w
        + current.orientation.z * -previous.orientation.x;
    let mut z = current.orientation.w * -previous.orientation.z
        + current.orientation.x * -previous.orientation.y
        - current.orientation.y * -previous.orientation.x
        + current.orientation.z * previous.orientation.w;
    let mut w = current.orientation.w * previous.orientation.w
        + current.orientation.x * previous.orientation.x
        + current.orientation.y * previous.orientation.y
        + current.orientation.z * previous.orientation.z;
    if w < 0.0 {
        x = -x;
        y = -y;
        z = -z;
        w = -w;
    }
    let length = (x * x + y * y + z * z).sqrt();
    let scale = if length > 0.000_01 {
        2.0 * length.atan2(w.clamp(-1.0, 1.0)) / (length * dt)
    } else {
        0.0
    };
    (
        linear,
        openxr::Vector3f {
            x: x * scale,
            y: y * scale,
            z: z * scale,
        },
    )
}

#[cfg(windows)]
impl D3d12Session {
    /// Drive OpenXR's session state machine from the LibOVR frame thread.
    pub fn poll_events(&mut self) -> Result<(), String> {
        let mut storage = openxr::EventDataBuffer::new();
        while let Some(event) = self
            .instance
            .poll_event(&mut storage)
            .map_err(|error| error.to_string())?
        {
            if let openxr::Event::SessionStateChanged(changed) = event {
                match changed.state() {
                    openxr::SessionState::READY if !self.running => {
                        self.session
                            .begin(openxr::ViewConfigurationType::PRIMARY_STEREO)
                            .map_err(|error| error.to_string())?;
                        self.running = true;
                    }
                    openxr::SessionState::STOPPING if self.running => {
                        self.session.end().map_err(|error| error.to_string())?;
                        self.running = false;
                    }
                    openxr::SessionState::EXITING | openxr::SessionState::LOSS_PENDING => {
                        self.running = false;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    pub fn create_color_swapchain(
        &mut self,
        format: u32,
        width: u32,
        height: u32,
        array_size: u32,
        mip_count: u32,
        sample_count: u32,
    ) -> Result<Vec<usize>, String> {
        let swapchain = self
            .session
            .create_swapchain(&openxr::SwapchainCreateInfo {
                create_flags: openxr::SwapchainCreateFlags::EMPTY,
                usage_flags: openxr::SwapchainUsageFlags::COLOR_ATTACHMENT,
                format,
                sample_count,
                width,
                height,
                face_count: 1,
                array_size,
                mip_count,
            })
            .map_err(|error| error.to_string())?;
        let images = swapchain
            .enumerate_images()
            .map_err(|error| error.to_string())?;
        let raw_images = images.into_iter().map(|image| image as usize).collect();
        self.color_swapchain = Some(swapchain);
        self.color_extent = Some((width, height));
        Ok(raw_images)
    }

    pub fn submit_haptic(
        &self,
        controller_type: u32,
        amplitude: f32,
        duration: std::time::Duration,
    ) -> Result<(), String> {
        let duration = openxr::Duration::try_from(duration).map_err(|error| error.to_string())?;
        let event = openxr::HapticVibration::new()
            .amplitude(amplitude.clamp(0.0, 1.0))
            .frequency(openxr::FREQUENCY_UNSPECIFIED)
            .duration(duration);
        for (bit, path) in [
            (0x0001_u32, "/user/hand/left"),
            (0x0002_u32, "/user/hand/right"),
        ] {
            if controller_type & bit != 0 {
                let path = self
                    .instance
                    .string_to_path(path)
                    .map_err(|error| error.to_string())?;
                self.haptic_action
                    .apply_feedback(&self.session, path, &event)
                    .map_err(|error| error.to_string())?;
            }
        }
        Ok(())
    }

    pub fn wait_frame(&mut self) -> Result<(), String> {
        if self.running {
            self.frame_state = Some(self.waiter.wait().map_err(|error| error.to_string())?);
            let display_time = self
                .frame_state
                .expect("frame state set")
                .predicted_display_time;
            if let Ok(location) = self.view_space.locate(&self.space, display_time) {
                self.head_pose = location.pose;
            }
            self.session
                .sync_actions(&[openxr::ActiveActionSet::new(&self.input_action_set)])
                .map_err(|error| error.to_string())?;
            let now = std::time::Instant::now();
            let velocity_dt = self
                .last_velocity_sample
                .map(|previous| now.duration_since(previous).as_secs_f32());
            for (index, hand_space) in self.hand_spaces.iter().enumerate() {
                if let Ok(location) = hand_space.locate(&self.space, display_time) {
                    self.hand_poses[index] = location.pose;
                    if let Some(dt) = velocity_dt.filter(|dt| (0.001..0.100).contains(dt)) {
                        let (linear, angular) = finite_difference_velocity(
                            self.last_velocity_hand_poses[index],
                            location.pose,
                            dt,
                        );
                        self.hand_linear_velocity[index] = linear;
                        self.hand_angular_velocity[index] = angular;
                    } else {
                        self.hand_linear_velocity[index] = openxr::Vector3f {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                        };
                        self.hand_angular_velocity[index] = openxr::Vector3f {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                        };
                    }
                    self.last_velocity_hand_poses[index] = location.pose;
                    let previous = self.last_logged_hand_poses[index];
                    let pose = location.pose;
                    let changed = (pose.position.x - previous.position.x).abs() > 0.002
                        || (pose.position.y - previous.position.y).abs() > 0.002
                        || (pose.position.z - previous.position.z).abs() > 0.002
                        || (pose.orientation.x - previous.orientation.x).abs() > 0.002
                        || (pose.orientation.y - previous.orientation.y).abs() > 0.002
                        || (pose.orientation.z - previous.orientation.z).abs() > 0.002
                        || (pose.orientation.w - previous.orientation.w).abs() > 0.002;
                    if changed {
                        crate::capi::log_call(&format!(
                            "OpenXR {} grip flags={:?} pos=({:.3},{:.3},{:.3}) quat=({:.3},{:.3},{:.3},{:.3})",
                            if index == 0 { "left" } else { "right" },
                            location.location_flags,
                            pose.position.x,
                            pose.position.y,
                            pose.position.z,
                            pose.orientation.x,
                            pose.orientation.y,
                            pose.orientation.z,
                            pose.orientation.w,
                        ));
                        self.last_logged_hand_poses[index] = pose;
                    }
                }
                let hand_path = if index == 0 {
                    self.instance.string_to_path("/user/hand/left")
                } else {
                    self.instance.string_to_path("/user/hand/right")
                };
                if let Ok(hand_path) = hand_path {
                    if let Ok(state) = self.select_action.state(&self.session, hand_path) {
                        self.hand_select[index] = state.is_active && state.current_state;
                        if state.changed_since_last_sync {
                            crate::capi::log_call(&format!(
                                "OpenXR {} simple select active={} pressed={}",
                                if index == 0 { "left" } else { "right" },
                                state.is_active,
                                state.current_state,
                            ));
                        }
                    }
                    if let Ok(state) = self.primary_action.state(&self.session, hand_path) {
                        self.hand_primary[index] = state.is_active && state.current_state;
                    }
                    macro_rules! bool_state {
                        ($field:ident, $action:ident) => {
                            if let Ok(state) = self.$action.state(&self.session, hand_path) {
                                self.$field[index] = state.is_active && state.current_state;
                            }
                        };
                    }
                    bool_state!(hand_secondary, secondary_action);
                    bool_state!(hand_menu, menu_action);
                    bool_state!(hand_thumbstick_click, thumbstick_click_action);
                    bool_state!(hand_primary_touch, primary_touch_action);
                    bool_state!(hand_secondary_touch, secondary_touch_action);
                    bool_state!(hand_thumbstick_touch, thumbstick_touch_action);
                    bool_state!(hand_trigger_touch, trigger_touch_action);
                    if let Ok(state) = self.trigger_action.state(&self.session, hand_path) {
                        self.hand_trigger[index] = if state.is_active {
                            state.current_state
                        } else {
                            0.0
                        };
                    }
                    if let Ok(state) = self.squeeze_action.state(&self.session, hand_path) {
                        self.hand_squeeze[index] = if state.is_active {
                            state.current_state
                        } else {
                            0.0
                        };
                    }
                    if let Ok(state) = self.thumbstick_action.state(&self.session, hand_path) {
                        self.hand_thumbstick[index] = if state.is_active {
                            state.current_state
                        } else {
                            openxr::Vector2f { x: 0.0, y: 0.0 }
                        };
                    }
                }
            }
            self.last_velocity_sample = Some(now);
            if let Some(swapchain) = self.color_swapchain.as_mut() {
                let image = swapchain
                    .acquire_image()
                    .map_err(|error| error.to_string())?;
                swapchain
                    .wait_image(openxr::Duration::INFINITE)
                    .map_err(|error| error.to_string())?;
                self.color_image = Some(image);
            }
        }
        Ok(())
    }

    pub fn begin_frame(&mut self) -> Result<(), String> {
        if self.frame_state.is_some() {
            self.stream.begin().map_err(|error| error.to_string())?;
            self.frame_begun = true;
        }
        Ok(())
    }

    /// Finish the frame using Echo's submitted viewport. The viewport is the
    /// dynamic-resolution window. FOV and poses remain runtime-located: they
    /// must agree with the shim's own HMD/render-desc projection.
    pub fn end_frame(
        &mut self,
        submitted_rects: Option<[openxr::Rect2Di; 2]>,
    ) -> Result<(), String> {
        if self.frame_begun {
            let state = self
                .frame_state
                .take()
                .expect("frame state set before begin");
            let display_time = state.predicted_display_time;
            let runtime_views = self
                .session
                .locate_views(
                    openxr::ViewConfigurationType::PRIMARY_STEREO,
                    display_time,
                    &self.space,
                )
                .map_err(|error| error.to_string())?
                .1;
            if let Some(swapchain) = self.color_swapchain.as_mut() {
                swapchain
                    .release_image()
                    .map_err(|error| error.to_string())?;
            }
            self.color_image = None;
            if let (Some(swapchain), Some((width, height))) =
                (&self.color_swapchain, self.color_extent)
            {
                let projection_views: Vec<_> = (0..2)
                    .map(|eye| {
                        let view = &runtime_views[eye];
                        let rect = submitted_rects
                            .as_ref()
                            .map(|rects| rects[eye])
                            .unwrap_or_else(|| {
                                let half_width = (width / 2) as i32;
                                openxr::Rect2Di {
                                    offset: openxr::Offset2Di {
                                        x: eye as i32 * half_width,
                                        y: 0,
                                    },
                                    extent: openxr::Extent2Di {
                                        width: half_width,
                                        height: height as i32,
                                    },
                                }
                            });
                        let fov = view.fov;
                        let pose = view.pose;
                        openxr::CompositionLayerProjectionView::new()
                            .pose(pose)
                            .fov(fov)
                            .sub_image(
                                openxr::SwapchainSubImage::new()
                                    .swapchain(swapchain)
                                    .image_array_index(0)
                                    .image_rect(rect),
                            )
                    })
                    .collect();
                let layer = openxr::CompositionLayerProjection::new()
                    .space(&self.space)
                    .views(&projection_views);
                self.stream
                    .end(
                        display_time,
                        openxr::EnvironmentBlendMode::OPAQUE,
                        &[&layer],
                    )
                    .map_err(|error| error.to_string())?;
            } else {
                self.stream
                    .end(display_time, openxr::EnvironmentBlendMode::OPAQUE, &[])
                    .map_err(|error| error.to_string())?;
            }
            self.frame_begun = false;
        }
        Ok(())
    }
}

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
