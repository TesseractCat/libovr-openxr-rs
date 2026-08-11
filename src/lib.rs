//! A LibOVR CAPI 1.94 compatibility layer implemented over OpenXR.
//!
//! The project is deliberately split into a small, ABI-facing CAPI layer and
//! a testable runtime core. The core must be usable with [`mock::MockRuntime`]
//! so most development does not require an HMD, Wine, or a GPU.

pub mod abi;
pub mod capi;
pub mod config;
pub mod mock;
pub mod openxr_backend;
pub mod platform_exports;
pub mod runtime;

pub use mock::MockRuntime;
pub use runtime::{FrameId, HeadsetState, Pose, Runtime, RuntimeError, ShimCore, Vec3};
