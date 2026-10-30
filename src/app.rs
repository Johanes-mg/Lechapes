use crate::buffer::LineEnding;
use crate::explorer::Explorer;
use crate::fs_ops;
use crate::i18n::{I18n, Lang};
use crate::persistence::{self, Session};
use crate::plugins::Plugins;
use crate::search::{self, SearchMode, SearchState};
use crate::tabs::TabsState;
use crate::theme;
use crate::viewer::{est_image, ImageViewer};
use eframe::egui;
use std::path::PathBuf;

pub struct LechapesApp {
    i18n: I18n,
    explorer: Explorer,
    tabs: TabsState,
    search: SearchState,
    viewer: ImageViewer,
    status: String,
    pending_close: bool,
    pending_tab_close: Option<usize>,
    last_session_save: f64,
    focus_editor: bool,
    sombre: bool,
    about_open: bool,
    language_open: bool,
    session_dirty: bool,
    plugins: Plugins,
    focus_search: bool,
}

impl LechapesApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self {
            i18n: I18n::new(),
            explorer: Explorer::new(),
            tabs: TabsState::new(),
            search: SearchState::default(),
            viewer: ImageViewer::new(),
            status: String::new(),
            pending_close: false,
            pending_tab_close: None,
            last_session_save: 0.0,
            focus_editor: true,
            sombre: true,
            about_open: false,
            language_open: false,
            session_dirty: false,
            plugins: Plugins::default(),
            focus_search: false,
        };

        if let Some(session) = persistence::load() {
            app.sombre = session.dark;
            app.i18n.lang = match session.lang.as_str() {
                "en" => Lang::En,
                "es" => Lang::Es,
                _ => Lang::Fr,
            };
            if let Some(root) = session.root.clone() {
                let p = PathBuf::from(root);
                if p.exists() && p.is_dir() {
                    app.explorer.set_root(p);
                }
            }
            let paths = persistence::paths_from_session(&session);
            if !paths.is_empty() {
                app.tabs.tabs.clear();
                for p in paths {
                    app.open_path(p);
                }
                if !app.tabs.tabs.is_empty() {
                    app.tabs.active = session.active.min(app.tabs.tabs.len() - 1);
                }
            }
            theme::configurer_style(&cc.egui_ctx, app.sombre);
            theme::configurer_style_editeur(&cc.egui_ctx);
        }
        app
    }

    fn open_path(&mut self, path: PathBuf) {
        if est_image(&path) {
            return;
        }
        for (i, t) in self.tabs.tabs.iter().enumerate() {
            if t.path.as_ref() == Some(&path) {
                self.tabs.active = i;
                self.focus_editor = true;
                return;
            }
        }
        let bytes = match fs_ops::read_bytes(&path) {
            Ok(b) => b,
            Err(e) => {
                self.status = format!("{}: {}", self.i18n.t("error"), e);
                return;
            }
        };
        let buffer = crate::buffer::FileBuffer::from_bytes(&bytes);
        let reuse_empty = self.tabs.tabs.len() == 1
            && self.tabs.tabs[0].path.is_none()
            && !self.tabs.tabs[0].dirty
            && self.tabs.tabs[0].buffer.text.is_empty();

        if reuse_empty {
            let tab = &mut self.tabs.tabs[0];
            tab.buffer = buffer;
            tab.path = Some(path);
            tab.dirty = false;
            tab.cursor = 0;
        } else {
            let id = self.tabs.alloc_id();
            let mut tab = crate::tabs::Tab::new(id);
            tab.buffer = buffer;
            tab.path = Some(path);
            self.tabs.tabs.push(tab);
            self.tabs.active = self.tabs.tabs.len() - 1;
        }
        self.focus_editor = true;
        self.status.clear();
        self.session_dirty = true;
    }

    fn open_path_avec_ctx(&mut self, ctx: &egui::Context, path: PathBuf) {
        if est_image(&path) {
            self.viewer.ouvrir(ctx, path.to_string_lossy().to_string());
            self.session_dirty = true;
            return;
        }
        self.open_path(path);
    }

    fn run_formatter(path: &PathBuf) -> Result<bool, String> {
        Plugins::format_file(path)
    }

    fn format_active_file(&mut self) {
        let Some(tab) = self.tabs.active_tab() else {
            self.status = "Aucun fichier a formater".to_string();
            return;
        };
        let Some(path) = tab.path.clone() else {
            self.status = "Enregistre d'abord le fichier pour le formater".to_string();
            return;
        };
        let bytes = tab.buffer.to_bytes();
        if let Err(error) = fs_ops::write_bytes(&path, &bytes) {
            self.status = format!("Ecriture impossible : {}", error);
            return;
        }
        match Self::run_formatter(&path) {
            Ok(true) => match fs_ops::read_bytes(&path) {
                Ok(formatted) => {
                    if let Some(tab) = self.tabs.active_tab_mut() {
                        let before = tab.buffer.text.clone();
                        tab.buffer = crate::buffer::FileBuffer::from_bytes(&formatted);
                        tab.buffer.push_snapshot_after_change(before);
                        tab.dirty = false;
                        tab.cursor = tab.cursor.min(tab.buffer.text.len());
                    }
                    self.status = "Fichier formate".to_string();
                }
                Err(error) => self.status = format!("Formatage effectue, relecture impossible : {}", error),
            },
            Ok(false) => {
                self.status = format!(
                    "Aucun formateur configure pour l'extension .{}",
                    path.extension().and_then(|e| e.to_str()).unwrap_or("?")
                );
            }
            Err(error) => self.status = error,
        }
    }

    fn save_tab(&mut self, index: usize) {
        let Some(tab) = self.tabs.tabs.get(index) else {
            return;
        };
        let Some(path) = tab.path.clone() else {
            self.save_tab_as(index);
            return;
        };
        let bytes = tab.buffer.to_bytes();
        match fs_ops::write_bytes(&path, &bytes) {
            Ok(_) => {
                let format_result = Self::run_formatter(&path);
                match fs_ops::read_bytes(&path) {
                    Ok(saved_bytes) => {
                        if let Some(tab) = self.tabs.tabs.get_mut(index) {
                            tab.buffer = crate::buffer::FileBuffer::from_bytes(&saved_bytes);
                            tab.dirty = false;
                        }
                        self.status = match format_result {
                            Ok(true) => "Fichier enregistre et formate".to_string(),
                            Ok(false) => "Fichier enregistre".to_string(),
                            Err(error) => format!("Fichier enregistre, formatage impossible : {}", error),
                        };
                    }
                    Err(error) => {
                        self.status = format!("Fichier enregistre, relecture impossible : {}", error);
                    }
                }
            }
            Err(error) => self.status = format!("{}: {}", self.i18n.t("error"), error),
        }
    }

    fn save_tab_as(&mut self, index: usize) {
        let Some(tab) = self.tabs.tabs.get(index) else {
            return;
        };
        let bytes = tab.buffer.to_bytes();
        if let Some(path) = rfd::FileDialog::new().save_file() {
            match fs_ops::write_bytes(&path, &bytes) {
                Ok(_) => {
                    let format_result = Self::run_formatter(&path);
                    match fs_ops::read_bytes(&path) {
                        Ok(saved_bytes) => {
                            if let Some(tab) = self.tabs.tabs.get_mut(index) {
                                tab.buffer = crate::buffer::FileBuffer::from_bytes(&saved_bytes);
                                tab.path = Some(path);
                                tab.dirty = false;
                            }
                            self.status = match format_result {
                                Ok(true) => "Fichier enregistre et formate".to_string(),
                                Ok(false) => "Fichier enregistre".to_string(),
                                Err(error) => format!("Fichier enregistre, formatage impossible : {}", error),
                            };
                        }
                        Err(error) => {
                            self.status = format!("Fichier enregistre, relecture impossible : {}", error);
                        }
                    }
                    self.session_dirty = true;
                }
                Err(error) => self.status = format!("{}: {}", self.i18n.t("error"), error),
            }
        }
    }

    fn new_file(&mut self) {
        self.tabs.new_tab();
        self.focus_editor = true;
    }

    fn new_folder(&mut self) {
        let base = self
            .explorer
            .root
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
        let path = base.join(self.i18n.t("new_folder_default"));
        match fs_ops::create_folder(&path) {
            Ok(_) => self.status.clear(),
            Err(e) => self.status = format!("{}: {}", self.i18n.t("error"), e),
        }
        self.session_dirty = true;
    }

    fn request_close_tab(&mut self, index: usize) {
        if let Some(tab) = self.tabs.tabs.get(index) {
            if tab.dirty {
                self.pending_tab_close = Some(index);
                return;
            }
        }
        self.tabs.close_tab(index);
        self.focus_editor = true;
        self.session_dirty = true;
    }

    fn undo(&mut self) {
        let mut modifie = false;
        if let Some(tab) = self.tabs.active_tab_mut() {
            modifie = tab.buffer.undo();
            if modifie {
                tab.dirty = true;
            }
        }
        self.status = if modifie {
            String::new()
        } else {
            "Rien a annuler".to_string()
        };
        self.focus_editor = true;
    }

    fn redo(&mut self) {
        let mut modifie = false;
        if let Some(tab) = self.tabs.active_tab_mut() {
            modifie = tab.buffer.redo();
            if modifie {
                tab.dirty = true;
            }
        }
        self.status = if modifie {
            String::new()
        } else {
            "Rien a retablir".to_string()
        };
        self.focus_editor = true;
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let events = ctx.input(|i| i.events.clone());
        for ev in events {
            if let egui::Event::Key {
                key,
                pressed: true,
                modifiers,
                ..
            } = ev
            {
                let ctrl = modifiers.ctrl || modifiers.command;
                if ctrl && !modifiers.shift {
                    match key {
                        egui::Key::S => {
                            let idx = self.tabs.active;
                            self.save_tab(idx);
                        }
                        egui::Key::N => self.new_file(),
                        egui::Key::O => {
                            if let Some(p) = rfd::FileDialog::new().pick_file() {
                                self.open_path_avec_ctx(ctx, p);
                            }
                        }
                        egui::Key::W => {
                            let idx = self.tabs.active;
                            self.request_close_tab(idx);
                        }
                        egui::Key::F => {
                            self.search.open = true;
                            self.search.mode = SearchMode::Find;
                            self.focus_search = true;
                        }
                        egui::Key::H => {
                            self.search.open = true;
                            self.search.mode = SearchMode::Replace;
                            self.focus_search = true;
                        }
                        egui::Key::Z => self.undo(),
                        egui::Key::Y => self.redo(),
                        egui::Key::Tab => {
                            if self.tabs.tabs.len() > 1 {
                                self.tabs.active = (self.tabs.active + 1) % self.tabs.tabs.len();
                                self.focus_editor = true;
                                self.session_dirty = true;
                            }
                        }
                        _ => {}
                    }
                } else if ctrl && modifiers.shift {
                    match key {
                        egui::Key::S => {
                            let idx = self.tabs.active;
                            self.save_tab_as(idx);
                        }
                        egui::Key::Z => self.redo(),
                        egui::Key::Tab => {
                            if self.tabs.tabs.len() > 1 {
                                self.tabs.active = if self.tabs.active == 0 {
                                    self.tabs.tabs.len() - 1
                                } else {
                                    self.tabs.active - 1
                                };
                                self.focus_editor = true;
                                self.session_dirty = true;
                            }
                        }
                        _ => {}
                    }
                } else if key == egui::Key::Escape && self.search.open {
                    self.search.open = false;
                } else if key == egui::Key::F3 {
                    self.search_next();
                }
            }
        }
    }

    fn search_next(&mut self) {
        let Some(tab) = self.tabs.active_tab_mut() else {
            return;
        };
        let matches = search::find_all(
            &tab.buffer.text,
            &self.search.query,
            self.search.case_sensitive,
        );
        if matches.is_empty() {
            self.status = "Aucune correspondance trouvee".to_string();
            return;
        }
        self.search.match_index = if self.search.match_index >= matches.len() {
            0
        } else {
            (self.search.match_index + 1) % matches.len()
        };
        let m = &matches[self.search.match_index];
        tab.cursor = m.start;
        tab.scroll_to = Some(m.start);
        self.focus_editor = true;
        let prefix = &tab.buffer.text[..m.start];
        let ligne = prefix.bytes().filter(|b| *b == b'\n').count() + 1;
        let colonne = prefix.rsplit('\n').next().unwrap_or("").chars().count() + 1;
        self.status = format!(
            "Occurrence {} sur {} — ligne {}, colonne {}",
            self.search.match_index + 1,
            matches.len(),
            ligne,
            colonne
        );
    }

    fn replace_current(&mut self) {
        let Some(tab) = self.tabs.active_tab_mut() else {
            return;
        };
        let matches = search::find_all(
            &tab.buffer.text,
            &self.search.query,
            self.search.case_sensitive,
        );
        if matches.is_empty() {
            return;
        }
        let idx = if self.search.match_index == usize::MAX {
            0
        } else {
            self.search.match_index.min(matches.len() - 1)
        };
        let m = matches[idx].clone();
        let start = m.start;
        let end = m.end;
        let before = tab.buffer.text.clone();
        tab.buffer
            .text
            .replace_range(start..end, &self.search.replacement);
        tab.buffer.push_snapshot_after_change(before);
        tab.dirty = true;
        self.search.match_index = usize::MAX;
        self.search_next();
    }

    fn replace_all(&mut self) {
        let replacement = self.search.replacement.clone();
        let Some(tab) = self.tabs.active_tab_mut() else {
            return;
        };
        let matches = search::find_all(
            &tab.buffer.text,
            &self.search.query,
            self.search.case_sensitive,
        );
        if matches.is_empty() {
            return;
        }
        let before = tab.buffer.text.clone();
        let mut result = String::with_capacity(tab.buffer.text.len());
        let mut cursor = 0;
        for m in &matches {
            result.push_str(&tab.buffer.text[cursor..m.start]);
            result.push_str(&replacement);
            cursor = m.end;
        }
        result.push_str(&tab.buffer.text[cursor..]);
        tab.buffer.text = result;
        tab.buffer.push_snapshot_after_change(before);
        tab.dirty = true;
        self.search.match_index = usize::MAX;
    }

    fn top_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("topbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button(self.i18n.t("file"), |ui| {
                        if ui.button(self.i18n.t("new_file")).clicked() {
                            self.new_file();
                            ui.close_menu();
                        }
                        if ui.button(self.i18n.t("new_folder")).clicked() {
                            self.new_folder();
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button(self.i18n.t("save")).clicked() {
                            let idx = self.tabs.active;
                            self.save_tab(idx);
                            ui.close_menu();
                        }
                        if ui.button(self.i18n.t("save_as")).clicked() {
                            let idx = self.tabs.active;
                            self.save_tab_as(idx);
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button(self.i18n.t("close_tab")).clicked() {
                            let idx = self.tabs.active;
                            self.request_close_tab(idx);
                            ui.close_menu();
                        }
                        if ui.button(self.i18n.t("quit")).clicked() {
                            if self.tabs.any_dirty() {
                                self.pending_close = true;
                            } else {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        }
                    });
                    ui.menu_button(self.i18n.t("open"), |ui| {
                        if ui.button(self.i18n.t("open_file")).clicked() {
                            if let Some(p) = rfd::FileDialog::new().pick_file() {
                                self.open_path_avec_ctx(ctx, p);
                            }
                            ui.close_menu();
                        }
                        if ui.button(self.i18n.t("open_folder")).clicked() {
                            if let Some(p) = rfd::FileDialog::new().pick_folder() {
                                self.explorer.set_root(p);
                                self.session_dirty = true;
                            }
                            ui.close_menu();
                        }
                    });
                    ui.menu_button(self.i18n.t("edit"), |ui| {
                        if ui.button(self.i18n.t("find")).clicked() {
                            self.search.open = true;
                            self.search.mode = SearchMode::Find;
                            self.focus_search = true;
                            ui.close_menu();
                        }
                        if ui.button(self.i18n.t("replace")).clicked() {
                            self.search.open = true;
                            self.search.mode = SearchMode::Replace;
                            self.focus_search = true;
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("Formatter").clicked() {
                            self.format_active_file();
                            ui.close_menu();
                        }
                    });
                    ui.menu_button("Plugins", |ui| {
                        if ui.button("Gérer les plugins…").clicked() {
                            self.plugins.open = true;
                            ui.close_menu();
                        }
                    });
                    ui.menu_button(self.i18n.t("view"), |ui| {
                        let mut change = false;
                        ui.horizontal(|ui| {
                            ui.label(self.i18n.t("dark_mode"));
                            if ui.checkbox(&mut self.sombre, "").changed() {
                                change = true;
                            }
                        });
                        if change {
                            theme::configurer_style(ctx, self.sombre);
                            theme::configurer_style_editeur(ctx);
                            self.session_dirty = true;
                        }
                        ui.separator();
                        if ui.button(self.i18n.t("change_language")).clicked() {
                            self.language_open = true;
                            ui.close_menu();
                        }
                    });
                    ui.menu_button(self.i18n.t("help"), |ui| {
                        if ui.button(self.i18n.t("about")).clicked() {
                            self.about_open = true;
                            ui.close_menu();
                        }
                    });
                });
                ui.separator();
                if ui.button(self.i18n.t("format")).on_hover_text("Formater le fichier ouvert (Prettier ou rustfmt)").clicked() {
                    self.format_active_file();
                }
            });
        });
    }

    fn plugins_window(&mut self, ctx: &egui::Context) {
        self.plugins.poll();
        if !self.plugins.open {
            return;
        }
        let mut open = true;
        egui::Window::new("Plugins")
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_width(420.0)
            .show(ctx, |ui| {
                ui.heading("Outils et extensions");
                ui.separator();
                ui.label("Formateurs");
                ui.horizontal(|ui| {
                    ui.label("Prettier — JavaScript, TypeScript, JSON, HTML, CSS, Markdown…");
                    if Plugins::prettier_ready() {
                        ui.label("✓ Installé");
                    } else if self.plugins.installing {
                        ui.spinner();
                        ui.label("Installation en arrière-plan…");
                    } else if ui.button("Télécharger / Installer").clicked() {
                        self.plugins.install_prettier();
                    }
                });
                ui.label("Le plugin installe une version portable de Node.js et Prettier dans le dossier utilisateur de Lechapes. Aucun Node.js global ni npx n’est requis.");
                ui.separator();
                ui.label("Affichage");
                ui.checkbox(&mut self.plugins.word_wrap, "Word wrap — retour automatique à la ligne");
                if !self.plugins.status.is_empty() {
                    ui.separator();
                    ui.label(&self.plugins.status);
                }
                if ui.button("Fermer").clicked() {
                    self.plugins.open = false;
                }
            });
        if !open {
            self.plugins.open = false;
        }
        if self.plugins.installing {
            ctx.request_repaint_after(std::time::Duration::from_millis(200));
        }
    }

    fn language_window(&mut self, ctx: &egui::Context) {
        if !self.language_open {
            return;
        }
        let mut close = false;
        let mut window_open = true;
        egui::Window::new(self.i18n.t("change_language"))
            .collapsible(false)
            .resizable(false)
            .open(&mut window_open)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.set_min_width(260.0);
                egui::ScrollArea::vertical()
                    .max_height(220.0)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let langs = [
                            (Lang::Fr, "fr", "drapeau_fr.png"),
                            (Lang::En, "en", "drapeau_en.png"),
                            (Lang::Es, "es", "drapeau_es.png"),
                        ];
                        for (lang, key, img) in langs {
                            let selected = self.i18n.lang == lang;
                            ui.horizontal(|ui| {
                                if theme::bouton_image_seule(ui, img, 32.0, 20.0) {
                                    self.i18n.lang = lang;
                                    self.session_dirty = true;
                                    close = true;
                                }
                                if ui.selectable_label(selected, self.i18n.t(key)).clicked() {
                                    self.i18n.lang = lang;
                                    self.session_dirty = true;
                                    close = true;
                                }
                            });
                            ui.separator();
                        }
                    });
                ui.add_space(8.0);
                if ui.button(self.i18n.t("close")).clicked() {
                    close = true;
                }
            });
        if close || !window_open {
            self.language_open = false;
        }
    }

    fn about_window(&mut self, ctx: &egui::Context) {
        if !self.about_open {
            return;
        }
        let mut close = false;
        let mut window_open = true;
        egui::Window::new(self.i18n.t("about"))
            .collapsible(false)
            .resizable(false)
            .open(&mut window_open)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.set_min_width(320.0);
                ui.vertical_centered(|ui| {
                    if let Some(tex) = theme::image(ctx, "logo.png") {
                        let hauteur = 64.0;
                        let ratio = tex.size_vec2().x / tex.size_vec2().y;
                        ui.add(
                            egui::Image::new(&tex)
                                .fit_to_exact_size(egui::vec2(hauteur * ratio, hauteur)),
                        );
                    }
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("Lechapes").size(20.0).strong());
                    ui.add_space(4.0);
                    ui.label(self.i18n.t("about_desc"));
                    ui.add_space(10.0);
                    ui.label(format!("{} : Johanes-mg", self.i18n.t("about_author")));
                    ui.add_space(4.0);
                    ui.hyperlink_to(
                        self.i18n.t("about_github"),
                        "https://github.com/Johanes-mg/Lechapes",
                    );
                    ui.add_space(12.0);
                    if ui.button(self.i18n.t("close_window")).clicked() {
                        close = true;
                    }
                });
            });
        if close || !window_open {
            self.about_open = false;
        }
    }

    fn tabs_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("tabsbar").show(ctx, |ui| {
            egui::ScrollArea::horizontal()
                .id_source("tabs_scroll")
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let mut to_activate: Option<usize> = None;
                        let mut to_close: Option<usize> = None;
                        let untitled = self.i18n.t("untitled").to_string();
                        let mut labels: Vec<(usize, String, bool)> = Vec::new();
                        for (i, tab) in self.tabs.tabs.iter().enumerate() {
                            let label = if tab.dirty {
                                format!("{} *", tab.title(&untitled))
                            } else {
                                tab.title(&untitled)
                            };
                            labels.push((i, label, i == self.tabs.active));
                        }
                        for (i, label, active) in labels {
                            let mut text = egui::RichText::new(&label);
                            if active {
                                text = text.strong();
                            }
                            let resp = ui.selectable_label(active, text);
                            if resp.clicked() {
                                to_activate = Some(i);
                            }
                            if resp.middle_clicked() {
                                to_close = Some(i);
                            }
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new("\u{00D7}")
                                            .size(14.0)
                                            .color(ui.visuals().weak_text_color()),
                                    )
                                    .frame(false)
                                    .min_size(egui::vec2(18.0, 18.0)),
                                )
                                .on_hover_text("Fermer")
                                .clicked()
                            {
                                to_close = Some(i);
                            }
                            ui.separator();
                        }
                        if ui.button("+").clicked() {
                            self.tabs.new_tab();
                            self.focus_editor = true;
                            self.session_dirty = true;
                        }
                        if let Some(i) = to_activate {
                            self.tabs.active = i;
                            self.focus_editor = true;
                            self.session_dirty = true;
                        }
                        if let Some(i) = to_close {
                            self.request_close_tab(i);
                        }
                    });
                });
        });
    }

    fn search_panel(&mut self, ctx: &egui::Context) {
        if !self.search.open {
            return;
        }
        egui::TopBottomPanel::bottom("search_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let mode_label = if self.search.mode == SearchMode::Replace {
                    self.i18n.t("replace")
                } else {
                    self.i18n.t("find")
                };
                ui.label(mode_label);
                let search_response = ui.add(
                    egui::TextEdit::singleline(&mut self.search.query)
                        .hint_text(self.i18n.t("search_hint"))
                        .desired_width(220.0)
                        .id(egui::Id::new("lechapes_search_input")),
                );
                if self.focus_search {
                    search_response.request_focus();
                    self.focus_search = false;
                }
                let query_changed = search_response.changed();
                if query_changed {
                    self.search.match_index = usize::MAX;
                }
                if self.search.mode == SearchMode::Replace {
                    ui.label("->");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.search.replacement)
                            .hint_text(self.i18n.t("replace_hint"))
                            .desired_width(220.0),
                    );
                }
                if ui
                    .checkbox(&mut self.search.case_sensitive, self.i18n.t("case_sensitive"))
                    .changed()
                {
                    self.search.match_index = usize::MAX;
                }
                let nombre = self.tabs.active_tab().map(|tab| {
                    search::find_all(
                        &tab.buffer.text,
                        &self.search.query,
                        self.search.case_sensitive,
                    )
                    .len()
                }).unwrap_or(0);
                ui.label(format!("{} resultat(s)", nombre));
                if ui.button(self.i18n.t("find_next")).clicked() {
                    self.search_next();
                }
                if self.search.mode == SearchMode::Replace {
                    if ui.button(self.i18n.t("replace_current")).clicked() {
                        self.replace_current();
                    }
                    if ui.button(self.i18n.t("replace_all")).clicked() {
                        self.replace_all();
                    }
                }
                if ui.button(self.i18n.t("close")).clicked() {
                    self.search.open = false;
                }
            });

            let resultats: Vec<(usize, usize, usize, usize, String)> =
                if let Some(tab) = self.tabs.active_tab() {
                    if self.search.query.is_empty() {
                        Vec::new()
                    } else {
                        search::find_all(
                            &tab.buffer.text,
                            &self.search.query,
                            self.search.case_sensitive,
                        )
                        .into_iter()
                        .enumerate()
                        .map(|(index, range)| {
                            let prefix = &tab.buffer.text[..range.start];
                            let ligne = prefix.bytes().filter(|b| *b == b'\n').count() + 1;
                            let colonne = prefix.rsplit('\n').next().unwrap_or("").chars().count() + 1;
                            let debut_ligne = prefix.rfind('\n').map(|p| p + 1).unwrap_or(0);
                            let fin_ligne = tab.buffer.text[range.end..]
                                .find('\n')
                                .map(|p| range.end + p)
                                .unwrap_or(tab.buffer.text.len());
                            let extrait = tab.buffer.text[debut_ligne..fin_ligne].trim().to_string();
                            (index, range.start, ligne, colonne, extrait)
                        })
                        .collect()
                    }
                } else {
                    Vec::new()
                };

            if !resultats.is_empty() {
                ui.separator();
                let fichier_actif = self.tabs.active_tab().map(|tab| {
                    tab.path
                        .as_ref()
                        .map(|path| path.to_string_lossy().to_string())
                        .unwrap_or_else(|| self.i18n.t("untitled").to_string())
                }).unwrap_or_default();
                ui.label(format!("{} — {} emplacement(s) :", fichier_actif, resultats.len()));
                egui::ScrollArea::vertical()
                    .id_source("search_locations")
                    .max_height(150.0)
                    .show(ui, |ui| {
                        for (index, byte_pos, ligne, colonne, extrait) in &resultats {
                            let selectionne = *index == self.search.match_index;
                            let label = format!(
                                "{}  |  ligne {}, colonne {}  |  {}",
                                index + 1,
                                ligne,
                                colonne,
                                extrait
                            );
                            if ui.selectable_label(selectionne, label).clicked() {
                                self.search.match_index = *index;
                                if let Some(tab) = self.tabs.active_tab_mut() {
                                    tab.cursor = *byte_pos;
                                    tab.scroll_to = Some(*byte_pos);
                                }
                                self.focus_editor = true;
                            }
                        }
                    });
            }
        });
    }

    fn status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("statusbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if let Some(tab) = self.tabs.active_tab() {
                    if let Some(p) = &tab.path {
                        ui.label(p.to_string_lossy());
                    } else {
                        ui.label(self.i18n.t("untitled"));
                    }
                    ui.separator();
                    ui.label(tab.buffer.encoding.name());
                    ui.separator();
                    let le = match tab.buffer.line_ending {
                        LineEnding::Lf => "LF",
                        LineEnding::Crlf => "CRLF",
                    };
                    ui.label(le);
                    ui.separator();
                    let lines = tab.buffer.text.lines().count().max(1);
                    ui.label(format!("{} L", lines));
                }
                if !self.status.is_empty() {
                    ui.separator();
                    ui.label(&self.status);
                }
            });
        });
    }

    fn editor(&mut self, ctx: &egui::Context) {
        let couleur_fond_editeur = if self.sombre {
            egui::Color32::from_rgb(28, 30, 34)
        } else {
            egui::Color32::WHITE
        };
        let couleur_texte_editeur = if self.sombre {
            egui::Color32::from_rgb(220, 224, 230)
        } else {
            egui::Color32::from_rgb(30, 35, 45)
        };

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(couleur_fond_editeur))
            .show(ctx, |ui| {
                let Some(tab) = self.tabs.active_tab_mut() else {
                    return;
                };
                let total_lines = tab.buffer.text.lines().count().max(1);
                let gutter_w = theme::line_number_width(total_lines);
                let tab_id = tab.id;

                let mut numbers = String::with_capacity(total_lines * 5);
                let digits = (gutter_w / 8.5) as usize;
                for i in 1..=total_lines {
                    numbers.push_str(&format!("{:>width$}\n", i, width = digits));
                }

                let editor_id = egui::Id::new(("editor_text", tab_id));
                let texte_avant = tab.buffer.text.clone();
                if let Some(byte_cursor) = tab.scroll_to.take() {
                    let mut byte_cursor = byte_cursor.min(tab.buffer.text.len());
                    while !tab.buffer.text.is_char_boundary(byte_cursor) {
                        byte_cursor -= 1;
                    }
                    let char_cursor = tab.buffer.text[..byte_cursor].chars().count();
                    let mut state = egui::text_edit::TextEditState::load(ctx, editor_id)
                        .unwrap_or_default();
                    state.cursor.set_char_range(Some(
                        egui::text::CCursorRange::one(egui::text::CCursor::new(char_cursor)),
                    ));
                    state.store(ctx, editor_id);
                }
                let hauteur = ui.available_height();
                let largeur_totale = ui.available_width();

                egui::ScrollArea::vertical()
                    .id_source(("editor_scroll", tab_id))
                    .max_height(hauteur)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.horizontal_top(|ui| {
                            ui.allocate_ui_with_layout(
                                egui::vec2(gutter_w, 0.0),
                                egui::Layout::top_down(egui::Align::RIGHT),
                                |ui| {
                                    ui.add(
                                        egui::Label::new(
                                            egui::RichText::new(&numbers)
                                                .monospace()
                                                .color(ui.visuals().weak_text_color()),
                                        )
                                        .selectable(false),
                                    );
                                },
                            );

                            ui.separator();

                            let largeur_editeur = (largeur_totale - gutter_w - 8.0).max(100.0);

                            let ancienne_couleur = ui.visuals().extreme_bg_color;
                            ui.visuals_mut().extreme_bg_color = couleur_fond_editeur;

                            let query = self.search.query.clone();
                            let case_sensitive = self.search.case_sensitive;
                            let current_match = self.search.match_index;
                            let highlights = search::find_all(&tab.buffer.text, &query, case_sensitive);
                            let word_wrap = self.plugins.word_wrap;
                            let mut layouter = move |ui: &egui::Ui, string: &str, wrap_width: f32| {
                                let mut job = egui::text::LayoutJob::default();
                                let mut pos = 0usize;
                                for (i, range) in highlights.iter().enumerate() {
                                    let start = range.start.min(string.len());
                                    let end = range.end.min(string.len());
                                    if start < pos || end <= start || !string.is_char_boundary(start) || !string.is_char_boundary(end) {
                                        continue;
                                    }
                                    if start > pos {
                                        job.append(&string[pos..start], 0.0, egui::TextFormat::simple(
                                            egui::FontId::monospace(14.0), couleur_texte_editeur));
                                    }
                                    let bg = if i == current_match {
                                        egui::Color32::from_rgb(255, 145, 35)
                                    } else {
                                        egui::Color32::from_rgb(115, 100, 35)
                                    };
                                    let mut fmt = egui::TextFormat::simple(
                                        egui::FontId::monospace(14.0), egui::Color32::BLACK);
                                    fmt.background = bg;
                                    job.append(&string[start..end], 0.0, fmt);
                                    pos = end;
                                }
                                if pos < string.len() {
                                    job.append(&string[pos..], 0.0, egui::TextFormat::simple(
                                        egui::FontId::monospace(14.0), couleur_texte_editeur));
                                }
                                job.wrap.max_width = if word_wrap { wrap_width } else { f32::INFINITY };
                                ui.fonts(|fonts| fonts.layout_job(job))
                            };
                            let resp = ui.add_sized(
                                egui::vec2(largeur_editeur, 0.0),
                                egui::TextEdit::multiline(&mut tab.buffer.text)
                                    .font(egui::TextStyle::Monospace)
                                    .code_editor()
                                    .desired_width(if word_wrap { largeur_editeur } else { 10000.0 })
                                    .frame(false)
                                    .lock_focus(true)
                                    .id(editor_id)
                                    .text_color(couleur_texte_editeur)
                                    .layouter(&mut layouter),
                            );

                            ui.visuals_mut().extreme_bg_color = ancienne_couleur;

                            if resp.changed() {
                                tab.buffer.push_snapshot_after_change(texte_avant);
                                tab.dirty = true;
                                self.search.match_index = usize::MAX;
                            }
                            if self.focus_editor {
                                resp.request_focus();
                            }
                        });
                    });

                if self.focus_editor {
                    self.focus_editor = false;
                }
            });
    }

    fn handle_pending_close(&mut self, ctx: &egui::Context) {
        if !self.pending_close {
            return;
        }
        let mut confirmed = false;
        let mut cancelled = false;
        let mut window_open = true;
        egui::Window::new(self.i18n.t("confirm_close"))
            .collapsible(false)
            .resizable(false)
            .open(&mut window_open)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.label(self.i18n.t("confirm_close_msg"));
                ui.horizontal(|ui| {
                    if ui.button(self.i18n.t("save")).clicked() {
                        for i in 0..self.tabs.tabs.len() {
                            if self.tabs.tabs[i].dirty {
                                self.save_tab(i);
                            }
                        }
                        confirmed = !self.tabs.any_dirty();
                        if !confirmed {
                            self.status = "Fermeture annulee : certains fichiers restent non enregistres".to_string();
                        }
                    }
                    if ui.button(self.i18n.t("quit_no_save")).clicked() {
                        confirmed = true;
                    }
                    if ui.button(self.i18n.t("cancel")).clicked() {
                        cancelled = true;
                    }
                });
            });
        if cancelled || !window_open {
            self.pending_close = false;
        }
        if confirmed {
            self.pending_close = false;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    fn handle_pending_tab_close(&mut self, ctx: &egui::Context) {
        let Some(idx) = self.pending_tab_close else {
            return;
        };
        let mut confirmed = false;
        let mut cancelled = false;
        let mut window_open = true;
        let name = self
            .tabs
            .tabs
            .get(idx)
            .map(|t| t.title(self.i18n.t("untitled")))
            .unwrap_or_default();
        egui::Window::new(self.i18n.t("confirm_close"))
            .collapsible(false)
            .resizable(false)
            .open(&mut window_open)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.label(format!("{} : {}", name, self.i18n.t("confirm_close_msg")));
                ui.horizontal(|ui| {
                    if ui.button(self.i18n.t("save")).clicked() {
                        self.save_tab(idx);
                        confirmed = self.tabs.tabs.get(idx).is_some_and(|tab| !tab.dirty);
                        if !confirmed {
                            self.status = "Onglet conserve : enregistrement non termine".to_string();
                        }
                    }
                    if ui.button(self.i18n.t("quit_no_save")).clicked() {
                        confirmed = true;
                    }
                    if ui.button(self.i18n.t("cancel")).clicked() {
                        cancelled = true;
                    }
                });
            });
        if cancelled || !window_open {
            self.pending_tab_close = None;
        }
        if confirmed {
            self.tabs.close_tab(idx);
            self.pending_tab_close = None;
            self.focus_editor = true;
            self.session_dirty = true;
        }
    }

    fn build_session(&self) -> Session {
        Session {
            open_paths: self
                .tabs
                .tabs
                .iter()
                .filter_map(|t| t.path.as_ref().map(|p| p.to_string_lossy().to_string()))
                .collect(),
            active: self.tabs.active,
            root: self
                .explorer
                .root
                .as_ref()
                .map(|p| p.to_string_lossy().to_string()),
            dark: self.sombre,
            lang: match self.i18n.lang {
                Lang::Fr => "fr".to_string(),
                Lang::En => "en".to_string(),
                Lang::Es => "es".to_string(),
            },
            window_size: None,
        }
    }

    fn save_session(&mut self, ctx: &egui::Context) {
        let now = ctx.input(|i| i.time);
        let fermeture = ctx.input(|i| i.viewport().close_requested());
        if !fermeture && !self.session_dirty && now - self.last_session_save < 5.0 {
            return;
        }
        self.last_session_save = now;
        self.session_dirty = false;
        persistence::save(&self.build_session());
    }
}

impl eframe::App for LechapesApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.viewport().close_requested())
            && self.tabs.any_dirty()
            && !self.pending_close
        {
            self.pending_close = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }

        self.handle_shortcuts(ctx);
        self.top_bar(ctx);
        self.tabs_bar(ctx);
        self.search_panel(ctx);
        self.status_bar(ctx);

        egui::SidePanel::left("explorer_panel")
            .resizable(true)
            .default_width(240.0)
            .show(ctx, |ui| {
                let action = self.explorer.ui(ui, &self.i18n);
                if action.undo {
                    self.undo();
                }
                if action.redo {
                    self.redo();
                }
                if action.modified {
                    self.session_dirty = true;
                }
                if action.refreshed {
                    self.status = "Explorateur actualise".to_string();
                }
                if let Some(p) = action.open {
                    if est_image(&p) {
                        self.viewer.ouvrir(ctx, p.to_string_lossy().to_string());
                        self.session_dirty = true;
                    } else {
                        self.open_path(p);
                    }
                }
            });

        self.editor(ctx);
        self.handle_pending_close(ctx);
        self.handle_pending_tab_close(ctx);
        self.language_window(ctx);
        self.about_window(ctx);
        self.plugins_window(ctx);
        self.viewer.ui(ctx);
        self.save_session(ctx);
    }
}
