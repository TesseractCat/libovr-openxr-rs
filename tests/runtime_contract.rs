use libovr_openxr::{HeadsetState, MockRuntime, RuntimeError, ShimCore};

#[test]
fn lifecycle_is_deterministic_without_an_hmd() {
    let mut shim = ShimCore::new(MockRuntime::with_state(HeadsetState {
        mounted: true,
        ..Default::default()
    }));

    assert_eq!(shim.wait_begin_frame(), Err(RuntimeError::NotInitialized));
    shim.initialize().expect("mock initialization");
    assert_eq!(shim.wait_begin_frame().expect("first frame").0, 0);
    assert_eq!(shim.wait_begin_frame().expect("second frame").0, 1);
    assert!(shim.headset_state(42.0).expect("state").mounted);

    shim.shutdown();
    assert_eq!(shim.headset_state(42.0), Err(RuntimeError::NotInitialized));
}

#[test]
fn session_loss_is_observable_and_recoverable() {
    let mut runtime = MockRuntime::default();
    runtime.fail_next_frame = true;
    let mut shim = ShimCore::new(runtime);
    shim.initialize().expect("mock initialization");

    assert_eq!(shim.wait_begin_frame(), Err(RuntimeError::SessionLost));
    assert_eq!(shim.wait_begin_frame().expect("next frame").0, 0);
}
