#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]

extern crate blog_os;
use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    blog_os::test_main();
    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}

extern crate alloc;

#[cfg(feature = "oom_integration")]
#[alloc_error_handler]
fn on_oom(_layout: alloc::alloc::Layout) -> ! {
    blog_os::exit_qemu(blog_os::QemuExitCode::Success);
}

#[test_case]
fn test_oom() {
    let _buf: alloc::boxed::Box<[u8; 1024]> = alloc::boxed::Box::new([0; 1024]);
    blog_os::exit_qemu(blog_os::QemuExitCode::Failed);
}
