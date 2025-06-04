# test_gpt

Ce dépôt contient un mini noyau en Rust situe dans le dossier `blog_os`.
Il s'appuie sur `no_std` et permet l'exécution de tests dans QEMU.

Fonctionnalités principales :

- affichage VGA et sortie série
- allocateur global de type *slab*
- ébauche d'implémentation du système de fichiers **FAT32**


Consultez `blog_os/README.md` pour les instructions de compilation et de test.
