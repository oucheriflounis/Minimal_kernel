# Rapport sur le projet de Global Allocator en Rust no_std

## Introduction

Ce rapport décrit un projet Rust développé pour un environnement `no_std`, implémentant un allocateur global basé sur un *slab allocator* multi-tailles, intégré dans un système minimal bare-metal pour une architecture x86_64. Les fichiers fournis (`allocator.rs`, `lib.rs`, `main.rs`, `serial.rs`, `vga_buffer.rs`, `oom.rs`, `should_panic.rs`, `basic_boot.rs`, et `config.toml`) constituent un système capable de démarrer, d'afficher des messages via VGA et série, d'exécuter des tests, et de gérer la mémoire dynamiquement. Le focus est mis sur l'allocateur global défini dans `allocator.rs`, avec une analyse des choix techniques, des méthodes d'implémentation, et des raisons derrière ces décisions.

## Description du projet

Le projet vise à créer un environnement bare-metal fonctionnel avec un allocateur global pour supporter des structures dynamiques comme `Box` et `Vec` dans un contexte `no_std`. Il inclut :

- **Sortie VGA** : Affichage texte via le tampon VGA à l'adresse 0xb8000.
- **Sortie série** : Communication via UART 16550 pour les tests dans QEMU.
- **Framework de test** : Exécution de tests unitaires et d'intégration avec des résultats signalés via QEMU.
- **Allocateur global** : Gestion de la mémoire dynamique avec un *slab allocator* multi-tailles.
- **Gestion des erreurs** : Handlers pour les panics et les erreurs d'allocation (OOM).

Le projet est configuré pour une cible x86_64 (`x86_64-blog_os.json`) et utilise `bootimage runner` pour construire et exécuter dans QEMU, comme défini dans `config.toml`.

## Implémentation de l'allocateur global (`allocator.rs`)

### Choix de conception

1. **Slab Allocator multi-tailles** :

   - **Choix** : L'allocateur utilise un modèle de *slab allocator* avec quatre tailles de blocs fixes (16, 32, 64, 128 octets).
   - **Raison** : Ce modèle est simple à implémenter dans un environnement `no_std` où les ressources sont limitées. Les tailles fixes réduisent la fragmentation externe et simplifient la gestion de la mémoire par rapport à un allocateur généraliste comme un *buddy allocator*. Les tailles choisies (puissances de 2) couvrent les besoins typiques des petites structures dynamiques dans un système minimal.

2. **Heap statique** :

   - **Choix** : Un tableau statique de 16 Ko (`HEAP_SIZE = 16 * 1024`) est défini comme espace mémoire.
   - **Raison** : Dans un environnement bare-metal sans système d'exploitation, un heap dynamique nécessiterait un mécanisme de gestion de la mémoire virtuelle, ce qui est hors de portée pour ce projet. Un heap statique est prévisible, facile à initialiser, et adapté à la taille limitée des applications ciblées.

3. **Division équitable du heap** :

   - **Choix** : Le heap est divisé en quatre portions égales, une pour chaque taille de slab.
   - **Raison** : Cette approche garantit que chaque taille de bloc a un espace dédié, évitant la concurrence entre slabs pour l'espace mémoire. La division équitable simplifie l'initialisation et assure une répartition prévisible des ressources.

4. **Synchronisation avec** `Mutex` :

   - **Choix** : Chaque slab est protégé par un `Mutex` de la crate `spin`.
   - **Raison** : Bien que le projet soit principalement mono-tâche dans QEMU, l'utilisation de `Mutex` rend l'allocateur sûr dans un contexte multi-tâches potentiel, comme un futur noyau avec interruptions. La crate `spin` est choisie pour sa compatibilité avec `no_std` et sa simplicité.

5. **Initialisation paresseuse** :

   - **Choix** : Les slabs sont initialisés lors de la première allocation, en vérifiant si `slab.count == 0`.
   - **Raison** : Cela évite d'initialiser les slabs inutilisés, économisant des cycles CPU au démarrage. L'initialisation paresseuse est particulièrement adaptée à un système minimal où seules certaines tailles de blocs peuvent être utilisées.

### Méthodes d'implémentation

1. **Structure** `Slab` :

   - Chaque slab contient une liste libre (`free_list`) chaînée, où chaque bloc libre stocke l'adresse du bloc suivant.
   - **Méthode** : Lors de l'initialisation, le slab découpe sa portion du heap en blocs, chaînant chaque bloc en écrivant le pointeur suivant au début du bloc courant. L'allocation retire un bloc de la liste libre, et la désallocation le repousse en tête.
   - **Raison** : Cette approche minimise l'overhead mémoire, car seule l'adresse du bloc suivant est stockée. Le chaînage en liste simple est suffisant pour un allocateur bare-metal où la performance est moins critique que la simplicité.

2. **Allocation et désallocation** :

   - **Méthode** : L'allocateur sélectionne le premier slab dont la taille est suffisante pour le `Layout` demandé. La désallocation utilise la même logique pour identifier le slab.
   - **Raison** : Cette stratégie de *first-fit* est simple et rapide, adaptée à un système avec peu de tailles de blocs. Elle garantit que les allocations sont satisfaites dès que possible, réduisant la complexité par rapport à une recherche de *best-fit*.

3. **Gestion OOM** :

   - **Choix** : Un handler OOM par défaut déclenche un panic, sauf si la feature `oom_integration` est activée.
   - **Méthode** : Le handler OOM affiche les détails du \`physics et termine par un panic.
   - **Raison** : Dans un environnement bare-metal, un OOM est une erreur fatale. Afficher les détails du `Layout` aide au débogage. La feature `oom_integration` permet de personnaliser ce comportement pour des tests spécifiques, comme dans `oom.rs`.

### Intégration avec `GlobalAlloc`

- **Choix** : La structure `SlabAllocator` implémente le trait `GlobalAlloc` pour être utilisée par la crate `alloc`.
- **Méthode** : Une instance statique `ALLOCATOR` est déclarée comme allocateur global, et les traits `Sync` et `Send` sont implémentés pour garantir la sécurité.
- **Raison** : Cela permet l'utilisation transparente de structures comme `Box` et `Vec`, essentielles pour un système minimal souhaitant supporter des allocations dynamiques. Les implémentations `unsafe` de `Sync` et `Send` sont justifiées par l'utilisation de `Mutex` pour la synchronisation.

## Infrastructure du projet

### Modules de support

1. **Sortie série (**`serial.rs`**)** :

   - **Choix** : Utilisation d'un port série UART 16550 à l'adresse 0x3F8 avec des macros `serial_print!` et `serial_println!`.
   - **Méthode** : Une instance statique `SERIAL1` protégée par un `Mutex` gère l'accès au port série.
   - **Raison** : La sortie série est idéale pour les tests dans QEMU, car elle permet de journaliser les résultats sans dépendre de l'affichage VGA. Le `Mutex` garantit la sécurité dans un contexte potentiellement concurrent.

2. **Sortie VGA (**`vga_buffer.rs`**)** :

   - **Choix** : Écriture dans le tampon VGA à l'adresse 0xb8000 avec prise en charge des couleurs et du défilement.
   - **Méthode** : Une structure `Writer` gère l'écriture caractère par caractère, avec des macros `print!` et `println!`.
   - **Raison** : Le VGA est la méthode standard pour l'affichage texte dans un environnement bare-metal x86_64. La prise en charge des nouvelles lignes et du défilement simplifie l'affichage interactif.

3. **Framework de test (**`lib.rs`**)** :

   - **Choix** : Un framework de test personnalisé avec le trait `Testable` et la fonction `test_runner`.
   - **Méthode** : Les tests sont exécutés via `test_main`, affichent leurs résultats via `serial_println!`, et sortent via `exit_qemu`.
   - **Raison** : Un framework personnalisé est nécessaire dans `no_std`, car les outils de test standard dépendent de `std`. L'utilisation de la série pour les résultats et de QEMU pour la sortie simplifie l'intégration avec CI/CD.

4. **Gestion des panics et sortie QEMU (**`lib.rs`**)** :

   - **Choix** : Des handlers de panic distincts pour les modes test et non-test, avec sortie via le port 0xf4.
   - **Méthode** : En mode test, les panics affichent l'erreur et sortent avec `QemuExitCode::Failed`. En mode normal, ils affichent l'erreur et bouclent.
   - **Raison** : La distinction entre modes permet de supporter à la fois le débogage (mode normal) et les tests automatisés (mode test). Le port 0xf4 est une convention standard pour signaler la fin des tests dans QEMU.

### Points d'entrée

1. **Main (**`main.rs`**)** :

   - **Choix** : Affiche "Hello World!" et exécute les tests en mode test.
   - **Raison** : Fournit un point d'entrée simple pour valider le démarrage et l'affichage, avec un test trivial (`trivial_assertion`) pour vérifier le framework.

2. **Basic Boot (**`basic_boot.rs`**)** :

   - **Choix** : Point d'entrée minimal avec un test `println!`.
   - **Raison** : Permet de valider l'infrastructure de démarrage dans un contexte simplifié, isolant les dépendances.

3. **OOM Test (**`oom.rs`**)** :

   - **Choix** : Tente une allocation de 1024 octets pour déclencher OOM.
   - **Méthode** : Le handler OOM renvoie `QemuExitCode::Success`.
   - **Raison** : Teste spécifiquement la gestion OOM, un cas critique pour l'allocateur, en s'assurant que l'échec est géré correctement.

4. **Should Panic (**`should_panic.rs`**)** :

   - **Choix** : Force une assertion échouée pour déclencher un panic.
   - **Méthode** : Le handler de panic affiche "\[ok\]" et sort avec `QemuExitCode::Success`.
   - **Raison** : Vérifie que les panics dans les tests sont gérés comme des succès, une convention courante pour les tests de type *should_panic*.

### Configuration (`config.toml`)

- **Choix** : Inclusion de `core`, `alloc`, et `compiler_builtins`, avec une cible personnalisée `x86_64-blog_os.json` et `bootimage runner`.
- **Raison** : Les crates incluses sont nécessaires pour `no_std` et l'allocation dynamique. La cible personnalisée définit un environnement bare-metal x86_64. `bootimage runner` simplifie la construction et l'exécution dans QEMU, un choix standard pour les projets de noyaux.

## Tests implémentés

### Tests unitaires

1. **Dans** `vga_buffer.rs` :

   - `test_println_simple` : Vérifie que `println!` écrit une chaîne.
   - `test_println_many` : Teste l'affichage répété pour valider le défilement.
   - `test_println_output` : Vérifie que chaque caractère est correctement écrit dans le tampon VGA.
   - **Raison** : Ces tests valident l'affichage VGA, une fonctionnalité critique pour l'interaction utilisateur.

2. **Dans** `main.rs` :

   - `trivial_assertion` : Vérifie `assert_eq!(1, 1)`.
   - **Raison** : Un test simple pour confirmer que le framework fonctionne.

### Tests d'intégration

1. **Dans** `oom.rs` : Vérifie que l'allocation excessive déclenche OOM correctement.
2. **Dans** `should_panic.rs` : Confirme que les panics sont gérés comme des succès.
3. **Dans** `basic_boot.rs` : Valide `println!` dans un contexte minimal.
   - **Raison** : Ces tests couvrent des scénarios critiques (OOM, panics, démarrage) pour garantir la robustesse du système.

## Conclusion

Le projet implémente un allocateur global fonctionnel pour un environnement `no_std` Rust, basé sur un *slab allocator* multi-tailles avec un heap statique de 16 Ko et quatre tailles de blocs (16, 32, 64, 128 octets). Les choix de conception, comme l'utilisation de slabs fixes, d'un heap statique, et d'une initialisation paresseuse, privilégient la simplicité et la prévisibilité, adaptées à un système bare-metal minimal. L'intégration avec la crate `alloc`, la synchronisation via `Mutex`, et la gestion OOM personnalisable assurent une utilisation robuste. Le projet est complété par une infrastructure de test complète, des sorties VGA et série, et une configuration ciblée pour QEMU. Ces choix reflètent un équilibre entre fonctionnalité, simplicité, et compatibilité avec les contraintes d'un environnement sans système d'exploitation.


## Sources externes

1.  **Philipp Oppermann, “Writing an OS in Rust”**

    -   [https://os.phil-opp.com/](https://os.phil-opp.com/)

    -   Tutoriel de référence pour un kernel `no_std`, gestion VGA, tests QEMU, allocators.

2.  **The Rustonomicon**

    -   [https://doc.rust-lang.org/nomicon/allocators.html](https://doc.rust-lang.org/nomicon/allocators.html)

    -   Concepts avancés d’allocateurs, sécurité `unsafe`, gestion des erreurs d’allocation.

3.  **Blog “The Embedded Rust Book”**

    -   [https://docs.rust-embedded.org/book/](https://docs.rust-embedded.org/book/)

    -   Bonnes pratiques pour `no_std`, `panic_handler`, `alloc_error_handler`.

4.  **Crate Documentation**

    -   `spin`: [https://docs.rs/spin/latest/spin/](https://docs.rs/spin/latest/spin/)

    -   `volatile`: [https://docs.rs/volatile/latest/volatile/](https://docs.rs/volatile/latest/volatile/)

5.  **RFC Rust**

    -   `alloc_error_handler` (unstable feature) : discussions et implémentations de paniques sur OOM.
