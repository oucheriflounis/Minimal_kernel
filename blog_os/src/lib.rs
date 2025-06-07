//! # blog_os
//!
//! Ce crate fournit les primitives de base pour un petit noyau en Rust `no_std`,
//! incluant :
//! - Gestion des tests via `cargo xtest`.
//! - Gestion de la sortie série et écran VGA.
//! - Optionnel : allocateur global (slab allocator).

#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]

// On active alloc_error_handler **seulement** si on n'est PAS en oom_integration.
#![cfg_attr(not(feature = "oom_integration"), feature(alloc_error_handler))]

extern crate alloc;

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use alloc::{boxed::Box, vec::Vec};

    // ... (vos trois tests unitaires, inchangés) ...
}

use core::panic::PanicInfo;

pub mod serial;
pub mod vga_buffer;
pub mod allocator;
pub mod fat32;

use crate::allocator::SimpleAllocator;

#[global_allocator]
static ALLOCATOR: SimpleAllocator = SimpleAllocator::new();

/// Test runner global (doit être visible des tests d’intégration)
pub fn test_runner(tests: &[&dyn Fn()]) {
    for test in tests {
        test();
    }
}

#[no_mangle]
pub extern "C" fn test_main() {
    let tests: &[&dyn Fn()] = &[];
    test_runner(tests);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
}

/// Envoie un code de sortie à QEMU puis boucle indéfiniment.
///
/// # Example
/// ```rust
/// exit_qemu(QemuExitCode::Failed);
/// ```
pub fn exit_qemu(exit_code: QemuExitCode) -> ! {
    use x86_64::instructions::port::Port;
    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
    loop {}
}

/// Code de sortie utilisable pour QEMU.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed  = 0x11,
}

/// Handler global d’OOM, **désactivé** quand feature = "oom_integration".
#[cfg(all(feature = "alloc", not(feature = "oom_integration")))]
//#[alloc_error_handler]
//fn on_oom(layout: alloc::alloc::Layout) -> ! {
//    panic!("Out of memory: {:?}", layout);
//}

/// Point d'entrée pour `cargo xtest`.
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}
/// Gestionnaire de panique en mode test, renvoie un code d'échec.
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

pub fn fat32_checks<D: fat32::BlockDevice>(mut fat: fat32::Fat32<D>) {
    let size = fat.cluster_size();
    let sector = fat.first_data_sector();
    let lba = fat.cluster_to_lba(2);
    let entry = fat.read_fat_entry(2);
    let mut buf = [0u8; 512];
    fat.read_cluster(2, &mut buf);

    println!(
        "Cluster size: {}, sector: {}, lba: {}, entry: {}",
        size, sector, lba, entry
    );
}


