#![cfg(not(test))]
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]

use blog_os::println;
use blog_os::fat32::{Fat32, MemoryDisk};
use blog_os::fat32_checks;
use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    let disk = MemoryDisk::new();
    match Fat32::new(disk) {
        Ok(fs) => {
            println!("FAT32 root cluster {}", fs.boot_sector().root_cluster);
            fat32_checks(fs);
        }
        Err(_) => println!("FAT32 init failed"),
    }

    #[cfg(test)]
    test_main();

    loop {}
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
