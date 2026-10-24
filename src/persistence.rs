use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Session {
    pub open_paths: Vec<String>,
    pub active: usize,
    pub root: Option<String>,
    pub dark: bool,
    pub lang: String,
    pub window_size: Option<[f32; 2]>,
}

fn chemin_fichier() -> PathBuf {
    let base = std::env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir());
    let dir = base.join("Lechapes");
    let _ = std::fs::create_dir_all(&dir);
    dir.join("session.json")
}

pub fn save(session: &Session) {
    let path = chemin_fichier();
    if let Ok(json) = serde_json::to_string_pretty(session) {
        let _ = std::fs::write(path, json);
    }
}

pub fn load() -> Option<Session> {
    let path = chemin_fichier();
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

pub fn paths_from_session(session: &Session) -> Vec<PathBuf> {
    session
        .open_paths
        .iter()
        .map(PathBuf::from)
        .filter(|p| p.exists())
        .collect()
}