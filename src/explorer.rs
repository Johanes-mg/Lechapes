use crate::fs_ops;
use crate::i18n::I18n;
use crate::theme;
use egui::{CollapsingHeader, Color32, ScrollArea, Ui};
use std::path::{Path, PathBuf};

pub struct Explorer {
    pub root: Option<PathBuf>,
    pub selected: Option<PathBuf>,
    expanded: std::collections::HashSet<PathBuf>,
    presse_papier: Option<PathBuf>,
    dialogue_renommer: Option<DialogueRenommer>,
    dialogue_supprimer: Option<DialogueSupprimer>,
    dialogue_deplacer: Option<DialogueDeplacer>,
    dialogue_nouveau: Option<DialogueNouveau>,
    message_erreur: Option<String>,
}

struct DialogueRenommer {
    cible: PathBuf,
    nom: String,
}

struct DialogueSupprimer {
    cible: PathBuf,
    definitif: bool,
}

struct DialogueDeplacer {
    cible: PathBuf,
    destination: Option<PathBuf>,
}

struct DialogueNouveau {
    parent: PathBuf,
    nom: String,
    est_dossier: bool,
}

pub struct ExplorerAction {
    pub open: Option<PathBuf>,
    pub undo: bool,
    pub redo: bool,
    pub modified: bool,
    pub refreshed: bool,
}

impl Explorer {
    pub fn new() -> Self {
        Self {
            root: None,
            selected: None,
            expanded: std::collections::HashSet::new(),
            presse_papier: None,
            dialogue_renommer: None,
            dialogue_supprimer: None,
            dialogue_deplacer: None,
            dialogue_nouveau: None,
            message_erreur: None,
        }
    }

    pub fn set_root(&mut self, path: PathBuf) {
        self.root = Some(path);
        self.expanded.clear();
        self.selected = None;
    }

    pub fn ui(&mut self, ui: &mut Ui, i18n: &I18n) -> ExplorerAction {
        let mut action = ExplorerAction {
            open: None,
            undo: false,
            redo: false,
            modified: false,
            refreshed: false,
        };
        let taille_bouton = 28.0;
        ui.horizontal(|ui| {
            ui.set_min_height(taille_bouton);
            ui.label(
                egui::RichText::new(i18n.t("explorer"))
                    .size(16.0)
                    .strong(),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if theme::bouton_image_seule(ui, "rafraichir.png", taille_bouton, 16.0) {
                    let racine_invalide = self.root.as_ref().is_some_and(|root| !root.is_dir());
                    if racine_invalide {
                        self.root = None;
                        self.selected = None;
                    } else if self.selected.as_ref().is_some_and(|path| !path.exists()) {
                        self.selected = None;
                    }
                    self.expanded.clear();
                    action.modified = true;
                    action.refreshed = true;
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

        if let Some(root) = self.root.clone() {
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .id_source("explorer_scroll")
                .show(ui, |ui| {
                    let fond = ui.interact(
                        ui.available_rect_before_wrap(),
                        ui.id().with("fond_explorer"),
                        egui::Sense::click(),
                    );
                    fond.context_menu(|ui| {
                        self.menu_fond(ui, &root);
                    });

                    let (rect, resp) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), 22.0),
                        egui::Sense::click(),
                    );
                    let painter = ui.painter();
                    let mut x = rect.left() + 4.0;
                    if let Some(tex) = theme::image(ui.ctx(), "explorer.png") {
                        let taille = 16.0;
                        let img_rect = egui::Rect::from_min_size(
                            egui::pos2(x, rect.center().y - taille / 2.0),
                            egui::vec2(taille, taille),
                        );
                        painter.image(
                            tex.id(),
                            img_rect,
                            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                            Color32::WHITE,
                        );
                        x += taille + 6.0;
                    }
                    painter.text(
                        egui::pos2(x, rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        root.to_string_lossy(),
                        egui::FontId::proportional(13.0),
                        ui.visuals().text_color(),
                    );
                    resp.context_menu(|ui| {
                        self.menu_fond(ui, &root);
                    });

                    self.draw_dir(ui, &root, &mut action.open);
                });
        } else {
            ui.label(i18n.t("no_file"));
        }

        if self.dialogues(ui, i18n) {
            action.modified = true;
        }
        action
    }

    fn menu_fond(&mut self, ui: &mut Ui, parent: &Path) {
        if ui.button("Nouveau fichier").clicked() {
            self.dialogue_nouveau = Some(DialogueNouveau {
                parent: parent.to_path_buf(),
                nom: String::from("nouveau_fichier.txt"),
                est_dossier: false,
            });
            ui.close_menu();
        }
        if ui.button("Nouveau dossier").clicked() {
            self.dialogue_nouveau = Some(DialogueNouveau {
                parent: parent.to_path_buf(),
                nom: String::from("NouveauDossier"),
                est_dossier: true,
            });
            ui.close_menu();
        }
        ui.separator();
        if let Some(pp) = self.presse_papier.clone() {
            if ui
                .button(format!(
                    "Coller ({})",
                    pp.file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default()
                ))
                .clicked()
            {
                match fs_ops::copier(&pp, parent) {
                    Ok(_) => {
                        self.message_erreur = None;
                        self.expanded.clear();
                    }
                    Err(e) => self.message_erreur = Some(e),
                }
                ui.close_menu();
            }
        }
        if ui.button("Rafraichir").clicked() {
            self.expanded.retain(|path| path.is_dir());
            if self.selected.as_ref().is_some_and(|path| !path.exists()) {
                self.selected = None;
            }
            ui.close_menu();
        }
    }

    fn menu_fichier(&mut self, ui: &mut Ui, chemin: &Path) {
        if ui.button("Ouvrir").clicked() {
            self.selected = Some(chemin.to_path_buf());
            ui.close_menu();
        }
        ui.separator();
        if ui.button("Copier").clicked() {
            self.presse_papier = Some(chemin.to_path_buf());
            ui.close_menu();
        }
        if ui
            .button(egui::RichText::new("Deplacer").color(Color32::from_rgb(90, 160, 240)))
            .clicked()
        {
            self.dialogue_deplacer = Some(DialogueDeplacer {
                cible: chemin.to_path_buf(),
                destination: None,
            });
            ui.close_menu();
        }
        if ui.button("Renommer").clicked() {
            let nom = chemin
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            self.dialogue_renommer = Some(DialogueRenommer {
                cible: chemin.to_path_buf(),
                nom,
            });
            ui.close_menu();
        }
        ui.separator();
        if ui
            .button(egui::RichText::new("Supprimer").color(Color32::from_rgb(220, 90, 90)))
            .clicked()
        {
            self.dialogue_supprimer = Some(DialogueSupprimer {
                cible: chemin.to_path_buf(),
                definitif: false,
            });
            ui.close_menu();
        }
    }

    fn menu_dossier(&mut self, ui: &mut Ui, chemin: &Path) {
        if ui.button("Nouveau fichier ici").clicked() {
            self.dialogue_nouveau = Some(DialogueNouveau {
                parent: chemin.to_path_buf(),
                nom: String::from("nouveau_fichier.txt"),
                est_dossier: false,
            });
            ui.close_menu();
        }
        if ui.button("Nouveau dossier ici").clicked() {
            self.dialogue_nouveau = Some(DialogueNouveau {
                parent: chemin.to_path_buf(),
                nom: String::from("NouveauDossier"),
                est_dossier: true,
            });
            ui.close_menu();
        }
        ui.separator();
        if let Some(pp) = self.presse_papier.clone() {
            if ui
                .button(format!(
                    "Coller ici ({})",
                    pp.file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default()
                ))
                .clicked()
            {
                match fs_ops::copier(&pp, chemin) {
                    Ok(_) => {
                        self.message_erreur = None;
                        self.expanded.clear();
                    }
                    Err(e) => self.message_erreur = Some(e),
                }
                ui.close_menu();
            }
        }
        if ui
            .button(egui::RichText::new("Deplacer").color(Color32::from_rgb(90, 160, 240)))
            .clicked()
        {
            self.dialogue_deplacer = Some(DialogueDeplacer {
                cible: chemin.to_path_buf(),
                destination: None,
            });
            ui.close_menu();
        }
        if ui.button("Renommer").clicked() {
            let nom = chemin
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            self.dialogue_renommer = Some(DialogueRenommer {
                cible: chemin.to_path_buf(),
                nom,
            });
            ui.close_menu();
        }
        ui.separator();
        if ui
            .button(egui::RichText::new("Supprimer").color(Color32::from_rgb(220, 90, 90)))
            .clicked()
        {
            self.dialogue_supprimer = Some(DialogueSupprimer {
                cible: chemin.to_path_buf(),
                definitif: false,
            });
            ui.close_menu();
        }
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
                header.header_response.context_menu(|ui| {
                    self.selected = Some(entry.clone());
                    self.menu_dossier(ui, &entry);
                });
            } else {
                let selected = self.selected.as_ref() == Some(&entry);
                let resp = ui.selectable_label(selected, &name);
                if resp.clicked() {
                    self.selected = Some(entry.clone());
                    *clicked = Some(entry.clone());
                }
                resp.context_menu(|ui| {
                    self.selected = Some(entry.clone());
                    self.menu_fichier(ui, &entry);
                });
            }
        }
    }

    fn dialogues(&mut self, ui: &mut Ui, i18n: &I18n) -> bool {
        let ctx = ui.ctx().clone();
        let mut modifie = false;

        if let Some(err) = self.message_erreur.clone() {
            let mut open = true;
            egui::Window::new("Erreur")
                .collapsible(false)
                .resizable(false)
                .open(&mut open)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(&ctx, |ui| {
                    ui.colored_label(Color32::from_rgb(220, 90, 90), &err);
                    if ui.button("OK").clicked() {
                        self.message_erreur = None;
                    }
                });
            if !open {
                self.message_erreur = None;
            }
        }

        if let Some(d) = &mut self.dialogue_nouveau {
            let mut valider = false;
            let mut annuler = false;
            let mut window_open = true;
            let titre = if d.est_dossier {
                "Nouveau dossier"
            } else {
                "Nouveau fichier"
            };
            egui::Window::new(titre)
                .collapsible(false)
                .resizable(false)
                .open(&mut window_open)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(&ctx, |ui| {
                    ui.set_min_width(320.0);
                    ui.horizontal(|ui| {
                        ui.label("Nom :");
                        ui.text_edit_singleline(&mut d.nom);
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Creer").clicked() {
                            valider = true;
                        }
                        if ui.button("Annuler").clicked() {
                            annuler = true;
                        }
                    });
                });
            if !window_open {
                annuler = true;
            }
            if valider {
                let cible = d.parent.join(&d.nom);
                let res = if d.est_dossier {
                    fs_ops::create_folder(&cible).map_err(|e| format!("{}", e))
                } else {
                    fs_ops::creer_fichier_vide(&cible)
                };
                match res {
                    Ok(_) => {
                        self.expanded.clear();
                        self.dialogue_nouveau = None;
                        modifie = true;
                    }
                    Err(e) => self.message_erreur = Some(e),
                }
            }
            if annuler {
                self.dialogue_nouveau = None;
            }
        }

        if let Some(d) = &mut self.dialogue_renommer {
            let mut valider = false;
            let mut annuler = false;
            let mut window_open = true;
            egui::Window::new("Renommer")
                .collapsible(false)
                .resizable(false)
                .open(&mut window_open)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(&ctx, |ui| {
                    ui.set_min_width(360.0);
                    ui.horizontal(|ui| {
                        ui.label("Nouveau nom :");
                        ui.text_edit_singleline(&mut d.nom);
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Renommer").clicked() {
                            valider = true;
                        }
                        if ui.button("Annuler").clicked() {
                            annuler = true;
                        }
                    });
                });
            if !window_open {
                annuler = true;
            }
            if valider {
                match fs_ops::renommer(&d.cible, &d.nom) {
                    Ok(_) => {
                        self.expanded.clear();
                        self.dialogue_renommer = None;
                        modifie = true;
                    }
                    Err(e) => self.message_erreur = Some(e),
                }
            }
            if annuler {
                self.dialogue_renommer = None;
            }
        }

        if let Some(d) = &mut self.dialogue_deplacer {
            let mut valider = false;
            let mut annuler = false;
            let mut choisir = false;
            let mut window_open = true;
            egui::Window::new("Deplacer")
                .collapsible(false)
                .resizable(false)
                .open(&mut window_open)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(&ctx, |ui| {
                    ui.set_min_width(420.0);
                    ui.label(format!(
                        "Deplacer : {}",
                        d.cible
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_default()
                    ));
                    ui.horizontal(|ui| {
                        ui.label("Destination :");
                        if let Some(dest) = &d.destination {
                            ui.label(dest.to_string_lossy());
                        } else {
                            ui.weak("(aucune)");
                        }
                    });
                    if ui.button("Choisir dossier...").clicked() {
                        choisir = true;
                    }
                    ui.horizontal(|ui| {
                        if ui.button("Deplacer").clicked() {
                            valider = true;
                        }
                        if ui.button("Annuler").clicked() {
                            annuler = true;
                        }
                    });
                });
            if !window_open {
                annuler = true;
            }
            if choisir {
                if let Some(dest) = rfd::FileDialog::new().pick_folder() {
                    d.destination = Some(dest);
                }
            }
            if valider {
                if let Some(dest) = d.destination.clone() {
                    match fs_ops::deplacer(&d.cible, &dest) {
                        Ok(_) => {
                            self.expanded.clear();
                            self.dialogue_deplacer = None;
                            modifie = true;
                        }
                        Err(e) => self.message_erreur = Some(e),
                    }
                } else {
                    self.message_erreur = Some("Choisir une destination".to_string());
                }
            }
            if annuler {
                self.dialogue_deplacer = None;
            }
        }

        if let Some(d) = &mut self.dialogue_supprimer {
            let mut valider = false;
            let mut annuler = false;
            let mut window_open = true;
            let nom = d
                .cible
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            egui::Window::new("Supprimer")
                .collapsible(false)
                .resizable(false)
                .open(&mut window_open)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(&ctx, |ui| {
                    ui.set_min_width(400.0);
                    ui.label(format!("Supprimer : {}", nom));
                    ui.label(if d.definitif {
                        "Suppression definitive, irreversible."
                    } else {
                        "Le fichier ira dans la corbeille (recuperable)."
                    });
                    ui.horizontal(|ui| {
                        if ui
                            .button(
                                egui::RichText::new("Supprimer")
                                    .color(Color32::from_rgb(220, 90, 90)),
                            )
                            .clicked()
                        {
                            valider = true;
                        }
                        if ui.button("Annuler").clicked() {
                            annuler = true;
                        }
                    });
                });
            if !window_open {
                annuler = true;
            }
            if valider {
                let res = if d.definitif {
                    fs_ops::supprimer_definitif(&d.cible)
                } else {
                    fs_ops::vers_corbeille(&d.cible)
                };
                match res {
                    Ok(_) => {
                        self.expanded.clear();
                        if self.selected.as_ref() == Some(&d.cible) {
                            self.selected = None;
                        }
                        self.dialogue_supprimer = None;
                        modifie = true;
                    }
                    Err(e) => self.message_erreur = Some(e),
                }
            }
            if annuler {
                self.dialogue_supprimer = None;
            }
        }

        let _ = i18n;
        modifie
    }
}