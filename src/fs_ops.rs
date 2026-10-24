use std::fs;
use std::path::{Path, PathBuf};

pub fn create_folder(path: &Path) -> std::io::Result<()> {
    fs::create_dir_all(path)
}

pub fn read_bytes(path: &Path) -> std::io::Result<Vec<u8>> {
    fs::read(path)
}

pub fn write_bytes(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, bytes)
}

pub fn list_dir(path: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut entries: Vec<PathBuf> = fs::read_dir(path)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    entries.sort_by(|a, b| {
        let ad = a.is_dir();
        let bd = b.is_dir();
        if ad != bd {
            bd.cmp(&ad)
        } else {
            a.file_name().cmp(&b.file_name())
        }
    });
    Ok(entries)
}

pub fn vers_corbeille(path: &Path) -> Result<(), String> {
    trash::delete(path).map_err(|e| format!("{}", e))
}

pub fn supprimer_definitif(path: &Path) -> Result<(), String> {
    if path.is_dir() {
        fs::remove_dir_all(path).map_err(|e| format!("{}", e))
    } else {
        fs::remove_file(path).map_err(|e| format!("{}", e))
    }
}

pub fn deplacer(source: &Path, destination_dir: &Path) -> Result<PathBuf, String> {
    let nom = source
        .file_name()
        .ok_or_else(|| "Nom de fichier introuvable".to_string())?;
    let cible = destination_dir.join(nom);
    if cible.exists() {
        return Err(format!("Existe deja : {}", cible.to_string_lossy()));
    }
    fs::rename(source, &cible).map_err(|e| format!("{}", e))?;
    Ok(cible)
}

pub fn copier(source: &Path, destination_dir: &Path) -> Result<PathBuf, String> {
    let nom = source
        .file_name()
        .ok_or_else(|| "Nom de fichier introuvable".to_string())?;
    let mut cible = destination_dir.join(nom);
    if cible.exists() {
        let stem = source
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "copie".to_string());
        let ext = source
            .extension()
            .map(|s| format!(".{}", s.to_string_lossy()))
            .unwrap_or_default();
        let mut n = 1;
        loop {
            let nom_candidat = format!("{}_copie{}{}", stem, n, ext);
            cible = destination_dir.join(nom_candidat);
            if !cible.exists() {
                break;
            }
            n += 1;
        }
    }
    if source.is_dir() {
        copier_dossier(source, &cible)?;
    } else {
        fs::copy(source, &cible).map_err(|e| format!("{}", e))?;
    }
    Ok(cible)
}

fn copier_dossier(source: &Path, cible: &Path) -> Result<(), String> {
    fs::create_dir_all(cible).map_err(|e| format!("{}", e))?;
    for entry in fs::read_dir(source).map_err(|e| format!("{}", e))? {
        let entry = entry.map_err(|e| format!("{}", e))?;
        let src = entry.path();
        let dst = cible.join(entry.file_name());
        if src.is_dir() {
            copier_dossier(&src, &dst)?;
        } else {
            fs::copy(&src, &dst).map_err(|e| format!("{}", e))?;
        }
    }
    Ok(())
}

pub fn renommer(source: &Path, nouveau_nom: &str) -> Result<PathBuf, String> {
    if nouveau_nom.is_empty() || nouveau_nom.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|']) {
        return Err("Nom invalide".to_string());
    }
    let parent = source
        .parent()
        .ok_or_else(|| "Dossier parent introuvable".to_string())?;
    let cible = parent.join(nouveau_nom);
    if cible.exists() {
        return Err(format!("Existe deja : {}", cible.to_string_lossy()));
    }
    fs::rename(source, &cible).map_err(|e| format!("{}", e))?;
    Ok(cible)
}

pub fn creer_fichier_vide(path: &Path) -> Result<(), String> {
    if path.exists() {
        return Err(format!("Existe deja : {}", path.to_string_lossy()));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("{}", e))?;
    }
    fs::write(path, b"").map_err(|e| format!("{}", e))
}