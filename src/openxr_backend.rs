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
    pub hand_spaces: [openxr::Space; 2],
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
            ],
        )
        .map_err(|error| error.to_string())?;
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
        hand_spaces,
        running: false,
        frame_state: None,
        frame_begun: false,
        color_swapchain: None,
        color_image: None,
        color_extent: None,
    })
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
            for (index, hand_space) in self.hand_spaces.iter().enumerate() {
                if let Ok(location) = hand_space.locate(&self.space, display_time) {
                    self.hand_poses[index] = location.pose;
                }
            }
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

    pub fn end_frame_without_layers(&mut self) -> Result<(), String> {
        if self.frame_begun {
            let state = self
                .frame_state
                .take()
                .expect("frame state set before begin");
            let display_time = state.predicted_display_time;
            let (_, views) = self
                .session
                .locate_views(
                    openxr::ViewConfigurationType::PRIMARY_STEREO,
                    display_time,
                    &self.space,
                )
                .map_err(|error| error.to_string())?;
            if let Some(swapchain) = self.color_swapchain.as_mut() {
                swapchain
                    .release_image()
                    .map_err(|error| error.to_string())?;
            }
            self.color_image = None;
            if let (Some(swapchain), Some((width, height))) =
                (&self.color_swapchain, self.color_extent)
            {
                let half_width = (width / 2) as i32;
                let projection_views: Vec<_> = views
                    .iter()
                    .take(2)
                    .enumerate()
                    .map(|(eye, view)| {
                        openxr::CompositionLayerProjectionView::new()
                            .pose(view.pose)
                            .fov(view.fov)
                            .sub_image(
                                openxr::SwapchainSubImage::new()
                                    .swapchain(swapchain)
                                    .image_array_index(if width >= 2 { 0 } else { eye as u32 })
                                    .image_rect(openxr::Rect2Di {
                                        offset: openxr::Offset2Di {
                                            x: eye as i32 * half_width,
                                            y: 0,
                                        },
                                        extent: openxr::Extent2Di {
                                            width: half_width,
                                            height: height as i32,
                                        },
                                    }),
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
