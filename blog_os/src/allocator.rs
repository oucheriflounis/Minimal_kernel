//! Slab Allocator `no_std` multi‑taille pour noyau Rust.
//!
//! Fournit un allocateur global basé sur plusieurs "slabs" de tailles fixes,
//! optimisé pour l'allocation fréquente de petits objets. Chaque slab gère
//! des blocs de même taille, améliorant la rapidité et réduisant la fragmentation.

#![cfg(feature = "alloc")]

extern crate alloc;

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use spin::Mutex;

/// Taille totale du heap alloué pour le slab allocator (16 KiB).
const HEAP_SIZE: usize = 16 * 1024;
/// Zone de mémoire statique servant de heap (non initialisée). 
static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

/// Tailles des blocs gérés par chaque slab (doivent être en ordre croissant).
const SLAB_SIZES: [usize; 4] = [16, 32, 64, 128];
const N_SLABS: usize = SLAB_SIZES.len();

/// Calcule combien de blocs de taille `obj_size` peuvent tenir dans une portion de heap.
const fn slab_count(obj_size: usize) -> usize {
    HEAP_SIZE / (N_SLABS * obj_size)
}

/// Structure représentant un cache de blocs de taille fixée (slab).
///
/// Chaque slab maintient une liste libre simple (singly linked list) des blocs disponibles.
struct Slab {
    /// Pointeur vers le premier bloc libre.
    free_list: Option<*mut u8>,
    /// Taille de chaque bloc géré par ce slab.
    obj_size: usize,
    /// Nombre total de blocs initialisés dans ce slab.
    count: usize,
}

impl Slab {
    /// Crée un slab non initialisé pour la taille d'objet donnée.
    pub const fn uninit(obj_size: usize) -> Self {
        Slab { free_list: None, obj_size, count: 0 }
    }

    /// Initialise la liste libre en découpant la région de mémoire spécifiée.
    ///
    /// # Safety
    /// - Doit être appelé **exactement une fois** par slab avant toute allocation ou libération.
    /// - `heap_base` doit pointer vers un segment de mémoire valide d'au moins
    ///   `obj_size * slab_count(obj_size)` octets.
    /// - Aucune autre référence mutable ne doit exister sur cette région pendant l'initialisation.
    unsafe fn init(&mut self, heap_base: *mut u8) {
        let mut ptr = heap_base;
        let cnt = slab_count(self.obj_size);
        self.count = cnt;

        // Chaînage des blocs : chaque bloc contient au début l'adresse du bloc suivant.
        for i in 0..cnt {
            let next = if i + 1 < cnt { ptr.add(self.obj_size) } else { null_mut() };
            (ptr as *mut *mut u8).write(next);
            ptr = ptr.add(self.obj_size);
        }
        self.free_list = Some(heap_base);
    }

    /// Alloue un bloc de ce slab.
    ///
    /// Retourne un pointeur vers un bloc libre, ou `null_mut()` si épuisé.
    ///
    /// # Safety
    /// - Doit être appelé **après** un unique appel à `init`.
    /// - Appel protégé par un `Mutex` pour éviter les accès concurrents.
    /// - L'alignement retourné garantit au moins l'alignement de `usize`.
    unsafe fn alloc(&mut self) -> *mut u8 {
        match self.free_list {
            Some(block) => {
                let next = (block as *mut *mut u8).read();
                self.free_list = if next.is_null() { None } else { Some(next) };
                block
            }
            None => null_mut(),
        }
    }

    /// Libère un bloc préalablement alloué.
    ///
    /// # Safety
    /// - `ptr` doit provenir d'un appel antérieur à `alloc` pour ce slab.
    /// - Appel protégé par un `Mutex` pour éviter les accès concurrents.
    unsafe fn dealloc(&mut self, ptr: *mut u8) {
        let old_head = self.free_list.unwrap_or(null_mut());
        (ptr as *mut *mut u8).write(old_head);
        self.free_list = Some(ptr);
    }
}

/// Allocateur global basé sur plusieurs slabs de tailles décroissantes.
///
/// Implémente le trait `GlobalAlloc` pour prendre en charge les allocations
/// et libérations via la macro `#[global_allocator]`.
pub struct SimpleAllocator {
    slabs: [Mutex<Slab>; N_SLABS],
}

unsafe impl Sync for SimpleAllocator {}
unsafe impl Send for SimpleAllocator {}

impl SimpleAllocator {
    /// Construit un allocateur avec tous les slabs initialement non configurés.
    pub const fn new() -> Self {
        SimpleAllocator {
            slabs: [
                Mutex::new(Slab::uninit(SLAB_SIZES[0])),
                Mutex::new(Slab::uninit(SLAB_SIZES[1])),
                Mutex::new(Slab::uninit(SLAB_SIZES[2])),
                Mutex::new(Slab::uninit(SLAB_SIZES[3])),
            ],
        }
    }

    /// Initialise explicitement la zone de heap utilisée par l'allocateur.
    ///
    /// Cette implémentation conserve un heap statique interne, les paramètres
    /// sont donc simplement ignorés pour compatibilité.
    pub unsafe fn init(&self, _heap_start: usize, _heap_size: usize) {
        // Le heap est statique, aucune action n’est nécessaire ici.
    }
}

unsafe impl GlobalAlloc for SimpleAllocator {
    /// Alloue un bloc de mémoire correspondant à `layout`.
    ///
    /// # Safety
    /// - La région de heap globale doit être invalide autrement.
    /// - Cette fonction utilise des blocs `unsafe` internes protégés par `Mutex`.
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let base = core::ptr::addr_of_mut!(HEAP) as *mut u8;
        // Initialisation paresseuse : configure chaque slab si nécessaire.
        for (i, &size) in SLAB_SIZES.iter().enumerate() {
            let mut slab = self.slabs[i].lock();
            if slab.count == 0 {
                let offset = i * slab_count(size) * size;
                slab.init(base.add(offset));
            }
        }
        // Sélection de la slab adaptée
        for (i, &size) in SLAB_SIZES.iter().enumerate() {
            if layout.size() <= size {
                return self.slabs[i].lock().alloc();
            }
        }
        // Pas de slab adaptée → `null_mut()` (OOM)
        null_mut()
    }

    /// Libère le bloc pointé par `ptr` avec la taille `layout`.
    ///
    /// # Safety
    /// - `ptr` et `layout` doivent correspondre à un appel antérieur à `alloc`.
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        for (i, &size) in SLAB_SIZES.iter().enumerate() {
            if layout.size() <= size {
                return self.slabs[i].lock().dealloc(ptr);
            }
        }
        // Si la taille est trop grande, on ignore
    }
}

/// Gestionnaire global d'erreur d'allocation (OOM).
///
/// Panique en détaillant la `layout` requise.
/// Désactivé si la feature `oom_integration` est activée.
#[cfg(all(feature = "alloc", not(feature = "oom_integration")))]
#[alloc_error_handler]
fn on_oom(layout: Layout) -> ! {
    panic!("Out of memory: {:?}", layout);
}


