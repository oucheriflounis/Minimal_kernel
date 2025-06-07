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


#[test_case]
fn invalid_cluster_panics() {
    blog_os::serial_println!("should_panic::invalid_cluster...");
    let disk = blog_os::fat32::MemoryDisk::new();
    let mut fs = blog_os::fat32::Fat32::new(disk).expect("fs");
    // invalid file with cluster 99 to trigger panic
    let entry = blog_os::fat32::DirectoryEntry {
        name: *b"BADFILE BIN", // 8.3 filename padded to 11 bytes
        attr: 0x20,
        first_cluster: 99,
        size: 1,
    };
    let _ = fs.open_file(&entry);
    blog_os::serial_println!("[test did not panic]");
    blog_os::exit_qemu(blog_os::QemuExitCode::Failed);
}
