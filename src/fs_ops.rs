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
