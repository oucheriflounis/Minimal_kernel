# Minimal_kernel


Un mini‑kernel bare‑metal en Rust **`no_std`**, basé sur le tutoriel “Writing an OS in Rust” de Philipp Oppermann, avec :

- Gestion VGA text mode (`src/vga_buffer.rs`)  
- Sortie série (`src/serial.rs`)  
- Slab allocator global `no_std` pour petits blocs (16 – 128 octets) (`src/allocator.rs`)  
- Harness de tests personnalisés + QEMU + `bootimage`  

---

## Arborescence


```

.
├── Cargo.lock
├── Cargo.toml
├── README.md
├── Global Allocator Project Report.markdown
├── src
│   ├── allocator.rs
│   ├── lib.rs
│   ├── main.rs
│   ├── serial.rs
│   └── vga_buffer.rs
├── tests
│   ├── basic_boot.rs
│   ├── oom.rs
│   └── should_panic.rs
└── x86_64-blog_os.json


```

---

## Prérequis

- Rust **nightly** (≥ 1.88.0‑nightly)  
- Cible custom : `rustup target add x86_64-blog_os.json`  
- `cargo‑bootimage` (installé via `cargo install bootimage`)  
- QEMU (≥ 4.2)

---

## Configuration `.cargo/config.toml`

```toml
[unstable]
build-std = ["core", "compiler_builtins"]
build-std-features = ["compiler-builtins-mem"]

[build]
target = "x86_64-blog_os.json"

[target.'cfg(target_os = "none")']
runner = "bootimage runner"

```

----------

## Compilation & exécution

1.  **Compiler l’image**
    
    ```bash
    cargo +nightly bootimage --features alloc --target x86_64-blog_os.json
    
    ```
    
    produit :  
    `target/x86_64-blog_os/debug/bootimage-blog_os.bin`
    
2.  **Lancer dans QEMU**
    
    ```bash
    qemu-system-x86_64 \
      -drive format=raw,file=target/x86_64-blog_os/debug/bootimage-blog_os.bin \
      -no-reboot -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
      -serial stdio -display none
    
    ```
    

----------

## Tests

-   **Unitaire & d’intégration** via `cargo xtest` (alias `cargo test`):
    
    ```bash
    cargo +nightly test \
      --features alloc,oom_integration \
      --target x86_64-blog_os.json
    
    ```
    
-   Le runner QEMU exit code 0x10 indique succès, 0x11 échec.
    

----------

## Features

-   `alloc`  
    Active le slab allocator global (`#[global_allocator]`) et les tests `Box`, `Vec`, etc.
    
-   `oom_integration`  
    Active les tests d’intégration OOM (`tests/oom.rs`) qui redéfinissent `#[alloc_error_handler]`.
    

----------

## Conception de l’allocateur

Voir `Global Allocator Project Report.markdown` pour :

-   découpage du heap en 4 slabs (16–128 o)
    
-   calcul de `slab_count`
    
-   init paresseuse, protections `Mutex`
    
-   contraintes & évolutions
    

----------

## Sources & tutoriels

-   **Writing an OS in Rust** – Philipp Oppermann  
    [https://os.phil-opp.com/](https://os.phil-opp.com/)
    
-   **The Rustonomicon** – allocators & `unsafe`  
    [https://doc.rust-lang.org/nomicon/allocators.html](https://doc.rust-lang.org/nomicon/allocators.html)
    
-   **Embedded Rust Book** – `no_std`, panic/alloc handlers  
    [https://docs.rust-embedded.org/book/](https://docs.rust-embedded.org/book/)
    
