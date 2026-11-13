use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;

const URL_PRETTIER: &str =
    "https://github.com/Johanes-mg/Lechapes-plugins/releases/download/1.0.0/lechapes-prettier.zip";
const URL_BLACK: &str =
    "https://github.com/Johanes-mg/Lechapes-plugins/releases/download/1.0.0/lechapes-black.zip";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Formateur {
    Prettier,
    Black,
}

impl Formateur {
    pub fn nom(self) -> &'static str {
        match self {
            Formateur::Prettier => "Prettier",
            Formateur::Black => "Black",
        }
    }

    fn url(self) -> &'static str {
        match self {
            Formateur::Prettier => URL_PRETTIER,
            Formateur::Black => URL_BLACK,
        }
    }
}

pub enum Evenement {
    Progression { recu: u64, total: Option<u64> },
    Termine(Formateur),
    Echec(Formateur, String),
}

pub struct Plugins {
    pub open: bool,
    pub word_wrap: bool,
    pub status: String,
    pub installation_en_cours: Option<Formateur>,
    rx: Option<Receiver<Evenement>>,
}

impl Default for Plugins {
    fn default() -> Self {
        Self {
            open: false,
            word_wrap: true,
            status: String::new(),
            installation_en_cours: None,
            rx: None,
        }
    }
}

impl Plugins {
    pub fn root() -> PathBuf {
        if let Some(base) = std::env::var_os("APPDATA").map(PathBuf::from) {
            return base.join("Lechapes").join("plugins");
        }
        if let Ok(cwd) = std::env::current_dir() {
            return cwd.join("plugins");
        }
        std::env::temp_dir().join("Lechapes").join("plugins")
    }

    fn node_exe() -> PathBuf {
        Self::root().join("node").join("node.exe")
    }

    fn prettier_cjs() -> PathBuf {
        Self::root()
            .join("prettier")
            .join("node_modules")
            .join("prettier")
            .join("bin")
            .join("prettier.cjs")
    }

    fn black_exe() -> PathBuf {
        Self::root().join("black_windows.exe")
    }

    pub fn prettier_ready() -> bool {
        Self::node_exe().is_file() && Self::prettier_cjs().is_file()
    }

    pub fn black_ready() -> bool {
        Self::black_exe().is_file()
    }

    pub fn ready(f: Formateur) -> bool {
        match f {
            Formateur::Prettier => Self::prettier_ready(),
            Formateur::Black => Self::black_ready(),
        }
    }

    pub fn install(&mut self, f: Formateur) {
        if self.installation_en_cours.is_some() || Self::ready(f) {
            return;
        }
        self.installation_en_cours = Some(f);
        self.status = format!("Preparation du telechargement de {}...", f.nom());
        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);
        thread::spawn(move || {
            let resultat = Self::telecharger_et_extraire(f, &tx);
            match resultat {
                Ok(()) => {
                    let _ = tx.send(Evenement::Termine(f));
                }
                Err(e) => {
                    let _ = tx.send(Evenement::Echec(f, e));
                }
            }
        });
    }

    pub fn remove(&mut self, f: Formateur) {
        let root = Self::root();
        let resultat = match f {
            Formateur::Prettier => {
                let _ = std::fs::remove_dir_all(root.join("node"));
                let _ = std::fs::remove_dir_all(root.join("prettier"));
                Ok(())
            }
            Formateur::Black => std::fs::remove_file(root.join("black_windows.exe"))
                .map_err(|e| format!("Impossible de supprimer Black : {e}")),
        };
        self.status = match resultat {
            Ok(()) => format!("{} supprime.", f.nom()),
            Err(e) => e,
        };
    }

    pub fn poll(&mut self) {
        let Some(rx) = self.rx.as_ref() else { return; };
        loop {
            match rx.try_recv() {
                Ok(Evenement::Progression { recu, total }) => {
                    let msg = match total {
                        Some(t) if t > 0 => format!(
                            "Telechargement : {:.1} / {:.1} Mo",
                            recu as f64 / 1_048_576.0,
                            t as f64 / 1_048_576.0
                        ),
                        _ => format!("Telechargement : {:.1} Mo", recu as f64 / 1_048_576.0),
                    };
                    self.status = msg;
                }
                Ok(Evenement::Termine(f)) => {
                    self.status = format!("{} installe.", f.nom());
                    self.installation_en_cours = None;
                    self.rx = None;
                    return;
                }
                Ok(Evenement::Echec(f, e)) => {
                    self.status = format!("Echec de l'installation de {} : {}", f.nom(), e);
                    self.installation_en_cours = None;
                    self.rx = None;
                    return;
                }
                Err(TryRecvError::Empty) => return,
                Err(TryRecvError::Disconnected) => {
                    self.installation_en_cours = None;
                    self.rx = None;
                    return;
                }
            }
        }
    }

    fn telecharger_et_extraire(
        f: Formateur,
        tx: &mpsc::Sender<Evenement>,
    ) -> Result<(), String> {
        let racine = Self::root();
        std::fs::create_dir_all(&racine)
            .map_err(|e| format!("Creation du dossier plugins impossible ({:?}) : {e}", racine))?;

        let url = f.url();
        let reponse = ureq::get(url)
            .call()
            .map_err(|e| format!("Telechargement impossible ({url}) : {e}"))?;
        let total = reponse
            .header("Content-Length")
            .and_then(|s| s.parse::<u64>().ok());

        let nom_zip = match f {
            Formateur::Prettier => "lechapes-prettier.zip",
            Formateur::Black => "lechapes-black.zip",
        };
        let chemin_zip = racine.join(format!("{}.partiel", nom_zip));
        let mut fichier = File::create(&chemin_zip).map_err(|e| {
            format!(
                "Creation du fichier temporaire impossible ({:?}) : {e}",
                chemin_zip
            )
        })?;

        let mut lecteur = reponse.into_reader();
        let mut tampon = [0u8; 65536];
        let mut recu: u64 = 0;
        loop {
            let n = lecteur
                .read(&mut tampon)
                .map_err(|e| format!("Lecture du flux impossible : {e}"))?;
            if n == 0 {
                break;
            }
            fichier
                .write_all(&tampon[..n])
                .map_err(|e| format!("Ecriture impossible : {e}"))?;
            recu += n as u64;
            let _ = tx.send(Evenement::Progression { recu, total });
        }
        drop(fichier);

        let archive_fichier = File::open(&chemin_zip)
            .map_err(|e| format!("Ouverture du zip impossible : {e}"))?;
        let mut archive = zip::ZipArchive::new(archive_fichier)
            .map_err(|e| format!("Lecture du zip impossible : {e}"))?;

        for i in 0..archive.len() {
            let mut entree = archive
                .by_index(i)
                .map_err(|e| format!("Entree {} illisible : {e}", i))?;
            let nom = match entree.enclosed_name() {
                Some(p) => p.to_path_buf(),
                None => continue,
            };
            let destination = racine.join(&nom);
            if entree.is_dir() {
                std::fs::create_dir_all(&destination)
                    .map_err(|e| format!("Creation du dossier impossible : {e}"))?;
            } else {
                if let Some(parent) = destination.parent() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| format!("Creation du dossier parent impossible : {e}"))?;
                }
                let mut sortie = File::create(&destination)
                    .map_err(|e| format!("Creation du fichier impossible ({:?}) : {e}", destination))?;
                std::io::copy(&mut entree, &mut sortie)
                    .map_err(|e| format!("Extraction impossible : {e}"))?;
            }
        }

        let _ = std::fs::remove_file(&chemin_zip);
        Ok(())
    }

    pub fn format_file(path: &PathBuf) -> Result<bool, String> {
        let ext = path
            .extension()
            .and_then(|x| x.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        if ext == "rs" {
            let output = Command::new("rustfmt")
                .args(["--edition", "2021"])
                .arg(path)
                .output()
                .map_err(|e| format!("Impossible de lancer rustfmt : {e}"))?;
            if !output.status.success() {
                let detail = String::from_utf8_lossy(&output.stderr).trim().to_string();
                return Err(if detail.is_empty() {
                    "rustfmt a echoue".to_string()
                } else {
                    detail
                });
            }
            return Ok(true);
        }

        if ext == "py" {
            if !Self::black_ready() {
                return Err(
                    "Black n'est pas installe. Ouvre le panneau Plugins et clique sur Telecharger."
                        .to_string(),
                );
            }
            let output = Command::new(Self::black_exe())
                .arg("--quiet")
                .arg(path)
                .output()
                .map_err(|e| format!("Impossible de lancer Black : {e}"))?;
            if !output.status.success() {
                let detail = String::from_utf8_lossy(&output.stderr).trim().to_string();
                return Err(if detail.is_empty() {
                    "Black a echoue".to_string()
                } else {
                    detail
                });
            }
            return Ok(true);
        }

        let supported = matches!(
            ext.as_str(),
            "js" | "jsx" | "cjs" | "mjs" | "ts" | "tsx" | "cts" | "mts"
                | "json" | "jsonc" | "html" | "htm" | "css" | "scss" | "less"
                | "md" | "mdx" | "yaml" | "yml" | "graphql" | "gql"
                | "vue" | "svelte" | "astro"
        );
        if !supported {
            return Ok(false);
        }

        if !Self::prettier_ready() {
            return Err(
                "Prettier n'est pas installe. Ouvre le panneau Plugins et clique sur Telecharger."
                    .to_string(),
            );
        }

        let output = Command::new(Self::node_exe())
            .arg(Self::prettier_cjs())
            .arg("--write")
            .arg(path)
            .output()
            .map_err(|e| format!("Impossible de lancer Prettier : {e}"))?;
        if !output.status.success() {
            let detail = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(if detail.is_empty() {
                "Prettier a echoue".to_string()
            } else {
                detail
            });
        }
        Ok(true)
    }
}
