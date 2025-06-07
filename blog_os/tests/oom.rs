#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(alloc_error_handler)]

extern crate blog_os;
extern crate alloc;

use core::panic::PanicInfo;
use alloc::boxed::Box;
use blog_os::{exit_qemu, QemuExitCode};

#[cfg(feature = "oom_integration")]
#[alloc_error_handler]
fn on_oom(_layout: alloc::alloc::Layout) -> ! {
    exit_qemu(QemuExitCode::Success);
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    exit_qemu(QemuExitCode::Failed);
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

#[test_case]
fn test_oom() {
    let _buf: Box<[u8; 1024]> = Box::new([0; 1024]);
    exit_qemu(QemuExitCode::Failed);
}

