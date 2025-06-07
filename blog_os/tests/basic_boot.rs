#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]

extern crate blog_os;
use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    blog_os::test_main();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}

use blog_os::println;
extern crate alloc;
use alloc::boxed::Box;

#[test_case]
fn test_println() {
    println!("test_println output");
}

#[cfg(feature = "alloc")]
#[test_case]
fn basic_allocator_test() {
    let value = Box::new(123_u32);
    assert_eq!(*value, 123);
}
