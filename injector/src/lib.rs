#![no_std]

use core::ffi::c_void;

const DLL_PROCESS_ATTACH: u32 = 1;
const PAGE_EXECUTE_READWRITE: u32 = 0x40;
const ECHO_LOADER_SIGNATURE_CHECK_RVA: usize = 0x0136_5bd0;

extern "system" {
    fn GetModuleHandleW(module_name: *const u16) -> *mut c_void;
    fn VirtualProtect(
        address: *mut c_void,
        size: usize,
        new_protect: u32,
        old_protect: *mut u32,
    ) -> i32;
    fn FlushInstructionCache(process: *mut c_void, address: *const c_void, size: usize) -> i32;
    fn GetCurrentProcess() -> *mut c_void;
}

/// This is loaded through Wine's AppInit_DLLs support before Echo begins its
/// OVR setup. It changes only Echo's local Authenticode/publisher validation
/// helper; it does not alter WinVerifyTrust or any system-wide trust decision.
#[no_mangle]
pub unsafe extern "system" fn DllMain(
    _module: *mut c_void,
    reason: u32,
    _reserved: *mut c_void,
) -> i32 {
    if reason != DLL_PROCESS_ATTACH {
        return 1;
    }

    // Do nothing when AppInit loads us into Proton helper processes.
    let echo: [u16; 11] = [
        b'e' as u16, b'c' as u16, b'h' as u16, b'o' as u16, b'v' as u16, b'r' as u16, b'.' as u16,
        b'e' as u16, b'x' as u16, b'e' as u16, 0,
    ];
    let base = GetModuleHandleW(echo.as_ptr());
    if base.is_null() {
        return 1;
    }

    let target = (base as usize + ECHO_LOADER_SIGNATURE_CHECK_RVA) as *mut u8;
    let mut old_protect = 0;
    if VirtualProtect(
        target.cast(),
        6,
        PAGE_EXECUTE_READWRITE,
        &mut old_protect,
    ) == 0
    {
        return 1;
    }

    // mov eax, 1; ret -- force only the loader's certificate check to accept.
    target.copy_from_nonoverlapping([0xb8, 1, 0, 0, 0, 0xc3].as_ptr(), 6);
    let mut ignored = 0;
    let _ = VirtualProtect(target.cast(), 6, old_protect, &mut ignored);
    let _ = FlushInstructionCache(GetCurrentProcess(), target.cast(), 6);
    1
}

// `core` still emits an unwind-personality reference for some Windows target
// metadata even though this cdylib is built with panic=abort.
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop {}
}
