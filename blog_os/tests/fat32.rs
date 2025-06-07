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


#[test_case]
fn read_root_dir_test() {
    let disk = blog_os::fat32::MemoryDisk::new();
    let mut fs = blog_os::fat32::Fat32::new(disk).unwrap();
    let entries = fs.read_root_directory().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].filename(), "HELLO.TXT");
}

#[test_case]
fn read_file_test() {
    let disk = blog_os::fat32::MemoryDisk::new();
    let mut fs = blog_os::fat32::Fat32::new(disk).unwrap();
    let entries = fs.read_root_directory().unwrap();
    let data = fs.open_file(&entries[0]).unwrap();
    assert_eq!(core::str::from_utf8(&data).unwrap(), "Hello");
}
