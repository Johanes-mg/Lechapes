use crate::fs_ops;
use crate::i18n::I18n;
use crate::theme;
use egui::{CollapsingHeader, ScrollArea, Ui};
use std::path::{Path, PathBuf};

pub struct Explorer {
    pub root: Option<PathBuf>,
    pub selected: Option<PathBuf>,
    expanded: std::collections::HashSet<PathBuf>,
}

pub struct ExplorerAction {
    pub open: Option<PathBuf>,
    pub undo: bool,
    pub redo: bool,
}

impl Explorer {
    pub fn new() -> Self {
        Self {
            root: None,
            selected: None,
            expanded: std::collections::HashSet::new(),
        }
    }

    pub fn set_root(&mut self, path: PathBuf) {
        self.root = Some(path);
        self.expanded.clear();
    }

    pub fn ui(&mut self, ui: &mut Ui, i18n: &I18n) -> ExplorerAction {
        let mut action = ExplorerAction {
            open: None,
            undo: false,
            redo: false,
        };
        let taille_bouton = 28.0;
        let hauteur_barre = taille_bouton + ui.spacing().item_spacing.y;
        ui.horizontal(|ui| {
            ui.set_min_height(hauteur_barre);
            ui.label(
                egui::RichText::new(i18n.t("explorer"))
                    .size(16.0)
                    .strong(),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if theme::bouton_image_seule(ui, "rafraichir.png", taille_bouton, 16.0) {
                    self.expanded.clear();
                }
                if theme::bouton_image_seule(ui, "redo.png", taille_bouton, 16.0) {
                    action.redo = true;
                }
                if theme::bouton_image_seule(ui, "undo.png", taille_bouton, 16.0) {
                    action.undo = true;
                }
            });
        });
        ui.separator();
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if let Some(root) = self.root.clone() {
                    self.draw_dir(ui, &root, &mut action.open);
                } else {
                    ui.label(i18n.t("no_file"));
                }
            });
        action
    }

    fn draw_dir(&mut self, ui: &mut Ui, path: &Path, clicked: &mut Option<PathBuf>) {
        let entries = match fs_ops::list_dir(path) {
            Ok(e) => e,
            Err(_) => return,
        };
        for entry in entries {
            let name = entry
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            if name.starts_with('.') {
                continue;
            }
            let is_dir = entry.is_dir();
            if is_dir {
                let id = entry.clone();
                let open = self.expanded.contains(&id);
                let header = CollapsingHeader::new(&name)
                    .id_source(&entry)
                    .open(Some(open))
                    .show(ui, |ui| {
                        self.draw_dir(ui, &entry, clicked);
                    });
                if header.header_response.clicked() {
                    if self.expanded.contains(&id) {
                        self.expanded.remove(&id);
                    } else {
                        self.expanded.insert(id);
                    }
                }
            } else {
                let selected = self.selected.as_ref() == Some(&entry);
                if ui.selectable_label(selected, &name).clicked() {
                    self.selected = Some(entry.clone());
                    *clicked = Some(entry);
                }
            }
        }
    }
}