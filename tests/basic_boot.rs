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

#[cfg(feature = "alloc")]
extern crate alloc;

#[test_case]
fn test_println() {
    blog_os::println!("test_println output");
}

#[cfg(feature = "alloc")]
#[test_case]
fn basic_allocator_test() {
    let value = alloc::boxed::Box::new(123_u32);
    assert_eq!(*value, 123);
}
