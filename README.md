# Lechapes

Editeur de texte leger multi-onglets pour Windows, ecrit en Rust avec egui / eframe.

Lechapes vise la simplicite : ouvrir, ecrire, enregistrer, gerer un dossier de travail. Pas de plugins, pas de comptes, pas de cloud. Un binaire autonome, un dossier d'images, et c'est tout.

---

## Sommaire

- [Apercu](#apercu)
- [Fonctionnalites](#fonctionnalites)
- [Raccourcis clavier](#raccourcis-clavier)
- [Captures](#captures)
- [Installation](#installation)
- [Compilation depuis les sources](#compilation-depuis-les-sources)
- [Structure du projet](#structure-du-projet)
- [Donnees utilisateur](#donnees-utilisateur)
- [Creation de l'installeur](#creation-de-linstalleur)
- [Version portable](#version-portable)
- [Depannage](#depannage)
- [Feuille de route](#feuille-de-route)
- [Licence](#licence)

---

## Apercu

| Aspect | Valeur |
|---|---|
| Langage | Rust (edition 2021) |
| Interface | egui / eframe 0.28 (backend glow) |
| Cible de compilation | `x86_64-pc-windows-gnullvm` |
| Version | 0.4.0 |
| Auteur | Johanes-mg |
| Licence | MIT |

---

## Fonctionnalites

### Edition

- Onglets multiples avec indicateur de modification (`*` apres le nom).
- Fermeture d'un onglet par `×`, clic du milieu, ou `Ctrl+W`.
- Confirmation d'enregistrement a la fermeture d'un onglet ou de l'application si des modifications non sauvegardees existent.
- Editeur direct, sans cadre, fond et texte qui suivent le theme clair ou sombre.
- Numeros de ligne alignes a droite qui defilent avec le texte.
- Detection d'encodage : UTF-8, UTF-8 avec BOM, UTF-16LE, UTF-16BE, repli Windows-1252.
- Detection des fins de ligne LF et CRLF, conservees a l'enregistrement.
- Undo / redo par onglet (profondeur 256 entrees).

### Recherche

- Rechercher et remplacer, avec option sensible a la casse.
- Remplacer l'occurrence courante ou toutes les occurrences.
- Cycle entre les occurrences avec `F3`.

### Explorateur

- Ouverture d'un dossier racine.
- Arborescence depliable.
- Clic sur un fichier l'ouvre dans un onglet.
- Rafraichissement de l'arborescence via l'icone dediee.

### Interface

- Mode sombre par defaut, bascule en mode clair via **Affichage → Mode sombre**.
- Francais et anglais via **Affichage → Langue**.
- Icone de fenetre et icone d'installeur personnalisees.
- Restauration de session : les fichiers ouverts sont rouvert au prochain demarrage.

### Creation de fichiers et dossiers

- **Fichier → Nouveau fichier** cree un nouvel onglet vide (remplacable par un enregistrement sous).
- **Fichier → Nouveau dossier** cree un dossier dans le repertoire racine de l'explorateur (ou le repertoire courant si aucun dossier n'est ouvert).

---

## Raccourcis clavier

| Raccourci | Action |
|---|---|
| `Ctrl+N` | Nouvel onglet |
| `Ctrl+O` | Ouvrir un fichier |
| `Ctrl+S` | Enregistrer |
| `Ctrl+Shift+S` | Enregistrer sous |
| `Ctrl+W` | Fermer l'onglet |
| `Ctrl+F` | Rechercher |
| `Ctrl+H` | Remplacer |
| `F3` | Occurrence suivante |
| `Echap` | Fermer la barre de recherche |
| `Ctrl+Z` | Annuler |
| `Ctrl+Y` | Retablir |
| `Ctrl+Shift+Z` | Retablir |
| `Ctrl+Tab` | Onglet suivant |
| `Ctrl+Shift+Tab` | Onglet precedent |

---

## Captures

![Fenetre principale](images/logo.png)

---

## Installation

### Installeur

1. Telecharger `LechapesSetup.exe`.
2. Executer, accepter les droits administrateur.
3. L'application s'installe dans `C:\Program Files\Lechapes\`.
4. Raccourcis crees dans le menu Demarrer et optionnellement sur le bureau.

### Portable

1. Telecharger `Lechapes-0.4.0-portable.zip`.
2. Decompresser n'importe ou.
3. Lancer `lechapes.exe`.

Le dossier `images/` doit rester a cote de `lechapes.exe` pour que les icones s'affichent correctement.

---

## Compilation depuis les sources

### Prerequis

- Windows 10 ou 11 (x86_64).
- Rust via [rustup](https://rustup.rs).
- Toolchain `gnullvm` (indispensable : la toolchain `gnu` provoque un crash au demarrage sur certaines configurations).

```powershell
rustup toolchain install stable-x86_64-pc-windows-gnullvm


Lechapes/
├── images/
│   ├── logo.ico
│   ├── logo.png
│   ├── rafraichir.png
│   ├── redo.png
│   └── undo.png
├── src/
│   ├── app.rs           logique principale, barre de menu, editeur, raccourcis
│   ├── buffer.rs        FileBuffer, encodage, undo/redo
│   ├── explorer.rs      arborescence de fichiers
│   ├── fs_ops.rs        operations fichier et dossier
│   ├── i18n.rs          traductions FR / EN
│   ├── main.rs          point d'entree, options de fenetre
│   ├── persistence.rs   sauvegarde et restauration de session
│   ├── search.rs        recherche et remplacement
│   ├── tabs.rs          gestion des onglets
│   └── theme.rs         theme clair/sombre, boutons image
├── .gitignore
├── Cargo.toml
├── README.md
└── rust-toolchain.toml
