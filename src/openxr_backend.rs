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
    pub head_location_flags: openxr::SpaceLocationFlags,
    pub view_poses: [openxr::Posef; 2],
    pub view_fovs: [openxr::Fovf; 2],
    pub hand_poses: [openxr::Posef; 2],
    pub hand_location_flags: [openxr::SpaceLocationFlags; 2],
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
    pub last_logged_head_pose: openxr::Posef,
    pub last_logged_view_poses: [openxr::Posef; 2],
    pub active_profiles: [Option<openxr::Path>; 2],
    /// OpenXR's safe Rust wrapper does not expose XR_SPACE_VELOCITY, so keep
    /// a finite-difference estimate at the same action-sync cadence.
    pub last_velocity_hand_poses: [openxr::Posef; 2],
    pub last_velocity_sample: Option<std::time::Instant>,
    pub hand_linear_velocity: [openxr::Vector3f; 2],
    pub hand_angular_velocity: [openxr::Vector3f; 2],
    pub running: bool,
    pub frame_state: Option<openxr::FrameState>,
    /// Views located for the current frame. Reuse these at submission so the
    /// game and compositor receive poses for the same predicted display time.
    pub frame_views: Option<Vec<openxr::View>>,
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
    requested.mnd_headless = extensions.mnd_headless;
    requested.fb_display_refresh_rate = extensions.fb_display_refresh_rate;
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
    if let Ok(views) = instance
        .enumerate_view_configuration_views(system, openxr::ViewConfigurationType::PRIMARY_STEREO)
    {
        if let Some(view) = views.first() {
            crate::capi::set_openxr_eye_resolution(
                view.recommended_image_rect_width as i32,
                view.recommended_image_rect_height as i32,
            );
        }
    }
    // KHR_D3D12_enable requires this query before *any* session creation,
    // including an XR_MND_headless session on WineOpenXR.
    let requirements = instance
        .graphics_requirements::<openxr::D3D12>(system)
        .map_err(|error| error.to_string())?;
    if extensions.mnd_headless {
        match probe_headless_eye_fovs(&instance, system) {
            Ok(views) => {
                crate::capi::set_openxr_eye_fovs(views.fovs);
                crate::capi::set_openxr_eye_offsets(views.eye_offsets);
                crate::capi::log_call("OpenXR headless probe discovered eye FOV and offsets");
            }
            Err(error) => {
                crate::capi::log_call(&format!("OpenXR headless FOV probe unavailable: {error}"))
            }
        }
    }
    // Windows LUID is an 8-byte C structure; LibOVR uses the same representation.
    Ok(unsafe { core::mem::transmute(requirements.adapter_luid) })
}

/// Obtain runtime FOV before Echo supplies its D3D12 queue. Core OpenXR only
/// returns FOV from `xrLocateViews`; XR_MND_headless requires no graphics device.
#[cfg(windows)]
struct HeadlessViewData {
    fovs: [crate::abi::OvrFovPort; 2],
    eye_offsets: [crate::abi::OvrVector3f; 2],
}

#[cfg(windows)]
fn probe_headless_eye_fovs(
    instance: &openxr::Instance,
    system: openxr::SystemId,
) -> Result<HeadlessViewData, String> {
    let info = openxr::headless::SessionCreateInfo {};
    let (session, mut waiter, _stream) =
        unsafe { instance.create_session::<openxr::Headless>(system, &info) }
            .map_err(|error| format!("xrCreateSession (headless): {error}"))?;
    if let Ok(rate) = session.get_display_refresh_rate() {
        crate::capi::set_openxr_refresh_rate(rate);
        crate::capi::log_call(&format!("OpenXR headless probe refresh rate={rate:.1}"));
    }
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
    let view_space = session
        .create_reference_space(openxr::ReferenceSpaceType::VIEW, identity)
        .map_err(|error| format!("xrCreateReferenceSpace (VIEW): {error}"))?;

    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(750);
    let mut event_data = openxr::EventDataBuffer::new();
    loop {
        while let Some(event) = instance
            .poll_event(&mut event_data)
            .map_err(|error| format!("xrPollEvent: {error}"))?
        {
            if let openxr::Event::SessionStateChanged(changed) = event {
                match changed.state() {
                    openxr::SessionState::READY => {
                        session
                            .begin(openxr::ViewConfigurationType::PRIMARY_STEREO)
                            .map_err(|error| format!("xrBeginSession: {error}"))?;
                        let frame = waiter
                            .wait()
                            .map_err(|error| format!("xrWaitFrame: {error}"))?;
                        // WineOpenXR rejects xrBeginFrame for an MND headless
                        // session. xrLocateViews only needs a running session and
                        // the prediction from xrWaitFrame, so do not begin a frame
                        // that has no layers to submit.
                        let located = session
                            .locate_views(
                                openxr::ViewConfigurationType::PRIMARY_STEREO,
                                frame.predicted_display_time,
                                &view_space,
                            )
                            .map_err(|error| format!("xrLocateViews: {error}"));
                        // `xrEndSession` is only valid after a STOPPING event;
                        // this short-lived probe has not requested exit. Dropping
                        // the session after collecting the views is sufficient.
                        let (_, views) = located?;
                        let [left, right, ..] = views.as_slice() else {
                            return Err("headless session did not provide stereo views".into());
                        };
                        let convert = |fov: openxr::Fovf| crate::abi::OvrFovPort {
                            up_tan: fov.angle_up.tan(),
                            down_tan: -fov.angle_down.tan(),
                            left_tan: -fov.angle_left.tan(),
                            right_tan: fov.angle_right.tan(),
                        };
                        let offset = |view: &openxr::View| crate::abi::OvrVector3f {
                            x: view.pose.position.x,
                            y: view.pose.position.y,
                            z: view.pose.position.z,
                        };
                        let data = HeadlessViewData {
                            fovs: [convert(left.fov), convert(right.fov)],
                            eye_offsets: [offset(left), offset(right)],
                        };
                        if let Err(error) = close_headless_session(instance, &session) {
                            // Do not discard valid descriptor data because a runtime
                            // declines to complete the optional probe teardown.
                            crate::capi::log_call(&format!(
                                "OpenXR headless probe cleanup: {error}"
                            ));
                        }
                        return Ok(data);
                    }
                    openxr::SessionState::EXITING | openxr::SessionState::LOSS_PENDING => {
                        return Err(format!("headless session entered {:?}", changed.state()));
                    }
                    _ => {}
                }
            }
        }
        if std::time::Instant::now() >= deadline {
            return Err("timed out waiting for headless session READY".into());
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

/// Request normal session shutdown after the one-shot headless query. Unlike
/// `xrDestroySession`, `xrEndSession` is only legal after STOPPING.
#[cfg(windows)]
fn close_headless_session(
    instance: &openxr::Instance,
    session: &openxr::Session<openxr::Headless>,
) -> Result<(), String> {
    session
        .request_exit()
        .map_err(|error| format!("xrRequestExitSession: {error}"))?;
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(250);
    let mut event_data = openxr::EventDataBuffer::new();
    loop {
        while let Some(event) = instance
            .poll_event(&mut event_data)
            .map_err(|error| format!("xrPollEvent during shutdown: {error}"))?
        {
            if let openxr::Event::SessionStateChanged(changed) = event {
                match changed.state() {
                    openxr::SessionState::STOPPING => {
                        return session
                            .end()
                            .map(|_| ())
                            .map_err(|error| format!("xrEndSession: {error}"));
                    }
                    openxr::SessionState::EXITING | openxr::SessionState::LOSS_PENDING => {
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }
        if std::time::Instant::now() >= deadline {
            return Err("timed out waiting for STOPPING after xrRequestExitSession".into());
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
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
    requested.fb_display_refresh_rate = extensions.fb_display_refresh_rate;
    // Required for translating LibOVR's current-time API to OpenXR's clock.
    // Current Wine/Proton OpenXR implementations expose this extension.
    requested.khr_win32_convert_performance_counter_time = true;
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
    if extensions.fb_display_refresh_rate {
        if let Ok(rate) = session.get_display_refresh_rate() {
            crate::capi::set_openxr_refresh_rate(rate);
        }
    }
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
        head_location_flags: openxr::SpaceLocationFlags::EMPTY,
        view_poses: [identity; 2],
        view_fovs: [openxr::Fovf {
            angle_left: -std::f32::consts::FRAC_PI_4,
            angle_right: std::f32::consts::FRAC_PI_4,
            angle_up: std::f32::consts::FRAC_PI_4,
            angle_down: -std::f32::consts::FRAC_PI_4,
        }; 2],
        hand_poses: [identity; 2],
        hand_location_flags: [openxr::SpaceLocationFlags::EMPTY; 2],
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
        last_logged_head_pose: identity,
        last_logged_view_poses: [identity; 2],
        active_profiles: [None; 2],
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
        frame_views: None,
        frame_begun: false,
        color_swapchain: None,
        color_image: None,
        color_extent: None,
    })
}

#[cfg(windows)]
fn valid_orientation(pose: openxr::Posef) -> bool {
    let q = pose.orientation;
    let norm = q.x * q.x + q.y * q.y + q.z * q.z + q.w * q.w;
    norm.is_finite() && norm > 0.5 && norm < 1.5
}

#[cfg(windows)]
pub fn hmd_to_eye_offset(head: openxr::Posef, eye: openxr::Posef) -> openxr::Vector3f {
    // Transform the LOCAL-space eye displacement into HMD/view space. LibOVR
    // requires this relative offset, not the absolute LOCAL eye position.
    let dx = eye.position.x - head.position.x;
    let dy = eye.position.y - head.position.y;
    let dz = eye.position.z - head.position.z;
    let q = head.orientation;
    // conjugate(q) * displacement * q
    let ix = q.w * dx - q.y * dz + q.z * dy;
    let iy = q.w * dy - q.z * dx + q.x * dz;
    let iz = q.w * dz - q.x * dy + q.y * dx;
    let iw = q.x * dx + q.y * dy + q.z * dz;
    openxr::Vector3f {
        x: ix * q.w + iw * q.x + iy * q.z - iz * q.y,
        y: iy * q.w + iw * q.y + iz * q.x - ix * q.z,
        z: iz * q.w + iw * q.z + ix * q.y - iy * q.x,
    }
}

#[cfg(windows)]
fn stable_pose(previous: openxr::Posef, location: openxr::SpaceLocation) -> openxr::Posef {
    let mut pose = previous;
    if location
        .location_flags
        .contains(openxr::SpaceLocationFlags::ORIENTATION_VALID)
        && valid_orientation(location.pose)
    {
        pose.orientation = location.pose.orientation;
    }
    if location
        .location_flags
        .contains(openxr::SpaceLocationFlags::POSITION_VALID)
        && location.pose.position.x.is_finite()
        && location.pose.position.y.is_finite()
        && location.pose.position.z.is_finite()
    {
        pose.position = location.pose.position;
    }
    // `previous` begins as identity, but retain this final guard if a future
    // runtime ever returns NaN or an invalid cached quaternion.
    if !valid_orientation(pose) {
        pose.orientation = openxr::Quaternionf {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        };
    }
    pose
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
    /// Return the current time in the same OpenXR clock domain used by frame
    /// predictions and action sampling.
    pub fn openxr_time_seconds(&self) -> f64 {
        self.instance
            .now()
            .expect("XR_KHR_win32_convert_performance_counter_time was requested")
            .as_nanos() as f64
            / 1_000_000_000.0
    }

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
                        crate::capi::log_call(&format!(
                            "OpenXR session state {:?}; exiting game",
                            changed.state()
                        ));
                        // LibOVR has no equivalent asynchronous quit callback
                        // for this path. Returning success while merely
                        // stopping frame submission leaves the game alive in
                        // a broken state, so terminate the process explicitly.
                        std::process::exit(0);
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
                self.head_pose = stable_pose(self.head_pose, location);
                self.head_location_flags = location.location_flags;
                let previous = self.last_logged_head_pose;
                let pose = self.head_pose;
                if (pose.position.x - previous.position.x).abs() > 0.002
                    || (pose.position.y - previous.position.y).abs() > 0.002
                    || (pose.position.z - previous.position.z).abs() > 0.002
                    || (pose.orientation.x - previous.orientation.x).abs() > 0.002
                    || (pose.orientation.y - previous.orientation.y).abs() > 0.002
                    || (pose.orientation.z - previous.orientation.z).abs() > 0.002
                    || (pose.orientation.w - previous.orientation.w).abs() > 0.002
                {
                    crate::capi::log_call(&format!(
                        "OpenXR HMD flags={:?} pos=({:.3},{:.3},{:.3}) quat=({:.3},{:.3},{:.3},{:.3})",
                        location.location_flags,
                        pose.position.x,
                        pose.position.y,
                        pose.position.z,
                        pose.orientation.x,
                        pose.orientation.y,
                        pose.orientation.z,
                        pose.orientation.w,
                    ));
                    self.last_logged_head_pose = pose;
                }
            }
            if let Ok((_, views)) = self.session.locate_views(
                openxr::ViewConfigurationType::PRIMARY_STEREO,
                display_time,
                &self.space,
            ) {
                self.frame_views = Some(views);
                if let Some(views) = self.frame_views.as_ref() {
                    for (index, view) in views.iter().take(2).enumerate() {
                        if valid_orientation(view.pose) {
                            self.view_poses[index] = view.pose;
                            self.view_fovs[index] = view.fov;
                            let previous = self.last_logged_view_poses[index];
                            if (view.pose.position.x - previous.position.x).abs() > 0.002
                                || (view.pose.position.y - previous.position.y).abs() > 0.002
                                || (view.pose.position.z - previous.position.z).abs() > 0.002
                            {
                                let offset = hmd_to_eye_offset(self.head_pose, view.pose);
                                crate::capi::log_call(&format!(
                                    "OpenXR {} view pos=({:.3},{:.3},{:.3}) hmd_to_eye=({:.4},{:.4},{:.4})",
                                    if index == 0 { "left" } else { "right" },
                                    view.pose.position.x,
                                    view.pose.position.y,
                                    view.pose.position.z,
                                    offset.x,
                                    offset.y,
                                    offset.z,
                                ));
                                self.last_logged_view_poses[index] = view.pose;
                            }
                        }
                    }
                }
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
                    self.hand_poses[index] = stable_pose(self.hand_poses[index], location);
                    self.hand_location_flags[index] = location.location_flags;
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
                    if let Ok(profile) = self.session.current_interaction_profile(hand_path) {
                        // XR_NULL_PATH means no profile is active yet. It is
                        // not legal to pass to xrPathToString; WiVRn reports
                        // this transiently while controller activation settles.
                        if profile != openxr::Path::NULL
                            && self.active_profiles[index] != Some(profile)
                        {
                            let profile_name = self
                                .instance
                                .path_to_string(profile)
                                .unwrap_or_else(|_| "<unknown>".to_owned());
                            crate::capi::log_call(&format!(
                                "OpenXR {} interaction profile={profile_name}",
                                if index == 0 { "left" } else { "right" }
                            ));
                            self.active_profiles[index] = Some(profile);
                        }
                    }
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
        }
        Ok(())
    }

    pub fn begin_frame(&mut self) -> Result<(), String> {
        if self.frame_state.is_some() {
            self.stream.begin().map_err(|error| error.to_string())?;
            if let Some(swapchain) = self.color_swapchain.as_mut() {
                let image = swapchain
                    .acquire_image()
                    .map_err(|error| error.to_string())?;
                swapchain
                    .wait_image(openxr::Duration::INFINITE)
                    .map_err(|error| error.to_string())?;
                self.color_image = Some(image);
            }
            self.frame_begun = true;
        }
        Ok(())
    }

    /// Finish the frame using Echo's submitted viewport and projection. Echo
    /// rendered these pixels with its layer FOV, so the compositor must use
    /// the same projection; only the poses remain runtime-located.
    pub fn end_frame(
        &mut self,
        submitted: Option<([openxr::Rect2Di; 2], [openxr::Fovf; 2])>,
    ) -> Result<(), String> {
        if self.frame_begun {
            let state = self
                .frame_state
                .take()
                .expect("frame state set before begin");
            let display_time = state.predicted_display_time;
            let runtime_views = if let Some(views) = self.frame_views.take() {
                views
            } else {
                // Defensive fallback for callers that bypass the normal
                // WaitToBeginFrame -> BeginFrame sequence.
                self.session
                    .locate_views(
                        openxr::ViewConfigurationType::PRIMARY_STEREO,
                        display_time,
                        &self.space,
                    )
                    .map_err(|error| error.to_string())?
                    .1
            };
            if runtime_views.len() >= 2 {
                crate::capi::log_call(&format!(
                    "OpenXR EndFrame runtime views left=p({:.3},{:.3},{:.3}) q({:.3},{:.3},{:.3},{:.3}) fov=({:.3},{:.3},{:.3},{:.3}) right=p({:.3},{:.3},{:.3}) q({:.3},{:.3},{:.3},{:.3}) fov=({:.3},{:.3},{:.3},{:.3})",
                    runtime_views[0].pose.position.x,
                    runtime_views[0].pose.position.y,
                    runtime_views[0].pose.position.z,
                    runtime_views[0].pose.orientation.x,
                    runtime_views[0].pose.orientation.y,
                    runtime_views[0].pose.orientation.z,
                    runtime_views[0].pose.orientation.w,
                    runtime_views[0].fov.angle_left.tan(),
                    runtime_views[0].fov.angle_right.tan(),
                    runtime_views[0].fov.angle_up.tan(),
                    runtime_views[0].fov.angle_down.tan(),
                    runtime_views[1].pose.position.x,
                    runtime_views[1].pose.position.y,
                    runtime_views[1].pose.position.z,
                    runtime_views[1].pose.orientation.x,
                    runtime_views[1].pose.orientation.y,
                    runtime_views[1].pose.orientation.z,
                    runtime_views[1].pose.orientation.w,
                    runtime_views[1].fov.angle_left.tan(),
                    runtime_views[1].fov.angle_right.tan(),
                    runtime_views[1].fov.angle_up.tan(),
                    runtime_views[1].fov.angle_down.tan(),
                ));
            }
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
                        let rect = submitted
                            .as_ref()
                            .map(|(rects, _)| rects[eye])
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
                        let fov = submitted
                            .as_ref()
                            .map(|(_, fovs)| fovs[eye])
                            .unwrap_or(view.fov);
                        // Some runtimes return a zero pose while their view
                        // state is not valid. xrEndFrame rejects that outright;
                        // retain the last valid view pose, or derive a valid
                        // eye pose from the current head pose until views are
                        // available.
                        let pose = if valid_orientation(view.pose) {
                            view.pose
                        } else if valid_orientation(self.view_poses[eye]) {
                            self.view_poses[eye]
                        } else {
                            let mut pose = self.head_pose;
                            pose.position.x += if eye == 0 { -0.032 } else { 0.032 };
                            pose
                        };
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
