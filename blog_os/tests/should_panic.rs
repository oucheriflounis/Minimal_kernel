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

use blog_os::{exit_qemu, serial_println, QemuExitCode};
use blog_os::fat32::{Fat32, MemoryDisk, DirectoryEntry};

#[test_case]
fn invalid_cluster_panics() {
    serial_println!("should_panic::invalid_cluster...");
    let disk = MemoryDisk::new();
    let mut fs = Fat32::new(disk).expect("fs");
    // invalid file with cluster 99 to trigger panic
    let entry = DirectoryEntry {
        name: *b"BADFILE BIN", // 8.3 filename padded to 11 bytes
        attr: 0x20,
        first_cluster: 99,
        size: 1,
    };
    let _ = fs.open_file(&entry);
    serial_println!("[test did not panic]");
    exit_qemu(QemuExitCode::Failed);
}
