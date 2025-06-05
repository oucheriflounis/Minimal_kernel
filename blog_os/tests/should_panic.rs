#![no_std]
#![no_main]

use core::panic::PanicInfo;
use blog_os::{exit_qemu, serial_println, QemuExitCode};
use blog_os::fat32::{Fat32, MemoryDisk, DirectoryEntry};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_println!("should_panic::invalid_cluster...");
    let disk = MemoryDisk::new();
    let mut fs = Fat32::new(disk).expect("fs");
    let entry = DirectoryEntry { name: *b"BADFILEBIN  ", attr: 0x20, first_cluster: 99, size: 1 };
    let _ = fs.open_file(&entry);
    serial_println!("[test did not panic]");
    exit_qemu(QemuExitCode::Failed);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
   
}
