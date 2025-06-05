#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use blog_os::println;
use blog_os::fat32::{Fat32, MemoryDisk, BlockDevice};
use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    let disk = MemoryDisk::new();
    match Fat32::new(disk) {
        Ok(mut fs) => {
            println!("FAT32 root cluster {}", fs.boot_sector().root_cluster);
            demo_fat32(&mut fs);
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

fn demo_fat32<D: BlockDevice>(fs: &mut Fat32<D>) {
    let size = fs.cluster_size();
    println!("Cluster size: {} bytes", size);
    let first = fs.first_data_sector();
    println!("First data sector: {}", first);
    let lba = fs.cluster_to_lba(2);
    println!("Cluster 2 LBA: {}", lba);
    let entry = fs.read_fat_entry(2);
    println!("FAT entry[2] = {:#X}", entry);
    let mut buf = [0u8; 512];
    fs.read_cluster(2, &mut buf);
    println!("Cluster 2 first byte = {}", buf[0]);
}
