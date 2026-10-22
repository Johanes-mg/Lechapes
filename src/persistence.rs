use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default)]
pub struct Session {
    pub open_paths: Vec<String>,
    pub active: usize,
    pub window_size: Option<[f32; 2]>,
}

const KEY: &str = "lechapes_session_v1";

pub fn save(ctx: &egui::Context, session: &Session) {
    if let Ok(json) = serde_json::to_string(session) {
        let mut storage = ctx.data_mut(|d| d.clone());
        storage.insert_temp(egui::Id::new(KEY), json);
        ctx.data_mut(|d| *d = storage);
    }
}

pub fn load(ctx: &egui::Context) -> Option<Session> {
    ctx.data(|d| d.get_temp::<String>(egui::Id::new(KEY)))
        .and_then(|s| serde_json::from_str(&s).ok())
}

pub fn paths_from_session(session: &Session) -> Vec<PathBuf> {
    session
        .open_paths
        .iter()
        .map(PathBuf::from)
        .filter(|p| p.exists())
        .collect()
}
