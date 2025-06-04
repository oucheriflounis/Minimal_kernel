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
#![reexport_test_harness_main = "test_main"]

// On active alloc_error_handler **seulement** si on n'est PAS en oom_integration.
#![cfg_attr(all(feature = "alloc", not(feature = "oom_integration")), feature(alloc_error_handler))]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "alloc")]
    #[allow(unused_imports)]
    use alloc::{boxed::Box, vec::Vec};

    // ... (vos trois tests unitaires, inchangés) ...
}

use core::panic::PanicInfo;

pub mod serial;
pub mod vga_buffer;
#[cfg(feature = "alloc")]
pub mod allocator;
pub mod fat32;

/// Trait étendant les tests pour permettre l'affichage de leur nom.
pub trait Testable {
    fn run(&self);
}

impl<T> Testable for T where T: Fn() {
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}


/// Exécute tous les tests fournis et renvoie le code de sortie à QEMU.
///
/// # Panics
/// Panique si un test échoue.
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
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


