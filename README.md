# Minimal_kernel
 

Un mini-kernel bare-metal inspiré du tutoriel **“Writing an OS in Rust”**.  
Il tient dans moins de 2 000 lignes de code et démontre :

* un **slab-allocator global** (16 – 128 o)  
* un **lecteur FAT32** en mémoire (boot-sector + chaîne de clusters)  
* un **harness de tests** personnalisé qui boote réellement le noyau sous QEMU  
* l’affichage VGA texte couleur et la sortie série COM1  
* une compilation **Clippy-clean** (0 warning) et des tests verts

---

## Arborescence

```

.
├── .cargo/config.toml      # cible + linker
├── x86\_64-blog\_os.json     # triple custom
├── Cargo.toml  Cargo.lock
├── src/
│   ├── allocator.rs   fat32.rs
│   ├── lib.rs         main.rs
│   ├── serial.rs      vga\_buffer.rs
│   └── …
└── tests/
├── basic\_boot.rs  fat32.rs
├── oom.rs         should\_panic.rs

````
:contentReference[oaicite:3]{index=3}

---

## 🔧 Prérequis

| Outil | Version min. | Installation |
|-------|--------------|--------------|
| **Rust nightly** | 1.88.0-nightly | `rustup toolchain install nightly` |
| **bootimage** | récent | `cargo install bootimage` |
| **QEMU (x86_64)** | ≥ 4.x | paquet distro ou site officiel |

Le fichier `.cargo/config.toml` fixe la cible et le linker ; aucune autre
configuration n’est nécessaire.

---

## 🚀 Démarrer

```bash
# cloner
git clone https://github.com/oucheriflounis/test_gpt.git
cd test_gpt

# générer l’image
cargo +nightly bootimage --features alloc

# lancer QEMU
qemu-system-x86_64 \
  -drive format=raw,file=target/x86_64-blog_os/debug/bootimage-blog_os.bin \
  -serial stdio -display none
````

---

## 🧪 Tests & lint

| Commande                                                   | Effet                                |
| ---------------------------------------------------------- | ------------------------------------ |
| `cargo test --target x86_64-blog_os.json --no-run`         | compile tous les tests d’intégration |
| `cargo clippy --target x86_64-blog_os.json -- -D warnings` | aucun warning autorisé               |

`tests/basic_boot.rs` vérifie que le noyau boote et renvoie 0x10 à QEMU ;
`tests/oom.rs` déclenche volontairement un OOM ; `tests/fat32.rs` lit le
répertoire racine et un fichier `HELLO.TXT`.

---

## 📚 Détails par module

| Fichier                 | Fonctions clés                                                                                                    | Rôle                                                                                                            |
| ----------------------- | ----------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------- |
| **`src/allocator.rs`**  | `Slab::uninit`, `SimpleAllocator::{new,init}`, `alloc`, `dealloc`                                                 | Implémente `GlobalAlloc` : 4 slabs (16 / 32 / 64 / 128 o). Le bitmap libre est stocké en tête de slab.          |
| **`src/fat32.rs`**      | `BootSector::parse`, `cluster_to_lba`, `read_fat_entry`, `read_cluster_chain`, `read_root_directory`, `open_file` | Lecture FAT32 : convertit cluster⇄LBA, suit la chaîne jusqu’à `0x0FFF_FFF8`. Retourne un `Vec<DirectoryEntry>`. |
| **`src/vga_buffer.rs`** | `Writer::write_byte`, `println!`                                                                                  | Écriture couleur 80×25, scroll automatique.                                                                     |
| **`src/serial.rs`**     | `serial_print!`, `serial_println!`                                                                                | Macros de debug vers COM1 (I/O-port 0x3F8).                                                                     |
| **`src/lib.rs`**        | `test_runner`, `test_main`, `test_panic_handler`                                                                  | Exporte le runner utilisé par tous les tests `no_std`.                                                          |
| **`src/main.rs`**       | `_start`                                                                                                          | Point d’entrée du kernel : init allocateur, affiche “Hello World”, spin-loop.                                   |

Le rapport détaillé sur l’allocateur se trouve dans
`Global Allocator Project Report.markdown` .

---

## ⚙️ Features Cargo

* **`alloc`** — active l’allocateur global et les tests utilisant `Vec` / `Box`.
* **`oom_integration`** — compile `tests/oom.rs`, redéfinit
  `#[alloc_error_handler]`.

---
## 🙏 Sources

* Philipp Oppermann — *Writing an OS in Rust*
* *The Rustonomicon* — chapitre allocateurs
* *Embedded Rust Book* — panic/alloc handlers

---
