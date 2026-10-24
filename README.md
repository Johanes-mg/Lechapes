
# Lechapes

<div align="center">

<div align="center">
  <img src="images/logo.png" alt="Lechapes Logo" width="120">
</div>


**Éditeur de texte léger, moderne et multi-onglets développé en Rust avec egui.**

[![GitHub release](https://img.shields.io/github/v/release/Johanes-mg/Lechapes?style=flat-square)](https://github.com/Johanes-mg/Lechapes/releases)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg?style=flat-square)](https://www.rust-lang.org)

</div>

---

## À propos de Lechapes

**Lechapes** est un éditeur de texte minimaliste, rapide et riche en fonctionnalités, conçu pour offrir une expérience d'édition fluide et agréable. Développé entièrement en **Rust** en utilisant la bibliothèque d'interface graphique **egui/eframe**, il combine des performances de haut niveau avec une faible empreinte mémoire.

---

## Fonctionnalités principales

- **Multi-onglets** : Ouvrez et gérez plusieurs fichiers simultanément avec la prise en charge des raccourcis clavier et de la fermeture au clic molette.
- **Explorateur de fichiers intégré** : Naviguez facilement dans vos dossiers, ouvrez des fichiers et actualisez l'arborescence en un clic.
- **Gestion des encodages et fins de lignes** : Détection automatique des encodages (UTF-8, UTF-16, Windows-1252, etc.) et des fins de lignes (LF / CRLF).
- **Recherche et Remplacement** : Outils de recherche avancés avec prise en charge de la sensibilité à la casse, remplacement unitaire ou global (`Ctrl + F`, `Ctrl + H`).
- **Historique (Annuler / Rétablir)** : Système complet de gestion des modifications (`Ctrl + Z`, `Ctrl + Y`).
- **Persistance de session** : Restauration automatique de vos onglets ouverts lors de la relance de l'application.
- **Interface multilingue** : Disponible en **Français**, **Anglais** et **Espagnol**.
- **Mode Sombre / Clair** : Basculez facilement entre un thème sombre élégant et un thème clair lumineux.

---

## Technologies utilisées

- **[Rust](https://www.rust-lang.org/)** : Langage de programmation système garantissant sécurité et rapidité.
- **[egui / eframe](https://github.com/emilk/egui)** : Framework GUI immédiat et ultra-rapide pour Rust.
- **[rfd](https://github.com/PolyMeilex/rfd)** : Dialogue de fichiers natifs multi-plateforme.
- **[serde](https://serde.rs/)** : Sérialisation et désérialisation pour la gestion de session.

---

## Structure du projet

Voici l'arborescence et l'organisation des fichiers de la solution :

```
Lechapes/
│
├── images/                 # Ressources graphiques (logos, drapeaux, icônes, boutons)
│   ├── drapeau_en.png
│   ├── drapeau_es.png
│   ├── drapeau_fr.png
│   ├── logo.ico
│   ├── logo.png
│   ├── rafraichir.png
│   ├── redo.png
│   └── undo.png
│
├── Installeur/             # Fichiers de configuration et scripts d'installation (Inno Setup)
├── src/                    # Code source principal en Rust
│   ├── app.rs              # Logique globale de l'application UI
│   ├── buffer.rs           # Gestion des tampons de texte
│   ├── explorer.rs         # Explorateur de fichiers latéral
│   ├── fs_ops.rs           # Opérations système de fichiers (lecture/écriture)
│   ├── i18n.rs             # Internationalisation (FR, EN, ES)
│   ├── main.rs             # Point d'entrée de l'application
│   ├── persistence.rs      # Sauvegarde et restauration de session
│   ├── search.rs           # Algorithmes de recherche et remplacement
│   ├── tabs.rs             # Gestion des onglets
│   └── theme.rs            # Thèmes (clair / sombre)
│
├── .gitignore              # Fichiers et dossiers ignorés par Git
├── Cargo.lock              # Versions exactes des dépendances Cargo
├── Cargo.toml              # Configuration et métadonnées du projet Rust
├── installer.iss           # Script de création de l'installateur Windows
├── README.md               # Documentation du projet
└── rust-toolchain.toml     # Configuration de la chaîne d'outils Rust


```
## Installation et Compilation
### Prérequis
Assurez-vous d'avoir installé **Rust** (version stable recommandée). Vous pouvez l'obtenir sur rustup.rs.
### Compilation depuis les sources
 1. Clonez le dépôt :
   ```bash
   git clone [https://github.com/Johanes-mg/Lechapes.git](https://github.com/Johanes-mg/Lechapes.git)
   cd Lechapes
   
   ```
 2. Compilez le projet en mode **release** (optimisé pour les performances) :
   ```bash
   cargo build --release
   
   ```
 3. L'exécutable généré se trouvera dans :
   ```bash
   target/release/lechapes.exe
   
   ```
## Raccourcis Clavier Utiles
| Raccourci | Action |
|---|---|
| Ctrl + N | Nouveau fichier |
| Ctrl + O | Ouvrir un fichier |
| Ctrl + S | Enregistrer |
| Ctrl + Shift + S | Enregistrer sous |
| Ctrl + W | Fermer l'onglet actif |
| Ctrl + F | Rechercher |
| Ctrl + H | Remplacer |
| Ctrl + Z | Annuler |
| Ctrl + Y / Ctrl + Shift + Z | Rétablir |
| Ctrl + Tab | Passer à l'onglet suivant |
## 👤 Auteur
 * **Johanes-mg** 
## 📄 Licence
Ce projet est sous licence ouverte. Voir le fichier LICENSE pour plus de détails.
```

