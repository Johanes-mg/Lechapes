use crate::buffer::LineEnding;
use crate::explorer::Explorer;
use crate::fs_ops;
use crate::i18n::{I18n, Lang};
use crate::persistence::{self, Session};
use crate::search::{self, SearchMode, SearchState};
use crate::tabs::TabsState;
use crate::theme;
use eframe::egui;
use std::path::PathBuf;

pub struct LechapesApp {
    i18n: I18n,
    explorer: Explorer,
    tabs: TabsState,
    search: SearchState,
    status: String,
    pending_close: bool,
    pending_tab_close: Option<usize>,
    last_session_save: f64,
    focus_editor: bool,
    sombre: bool,
}

impl LechapesApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self {
            i18n: I18n::new(),
            explorer: Explorer::new(),
            tabs: TabsState::new(),
            search: SearchState::default(),
            status: String::new(),
            pending_close: false,
            pending_tab_close: None,
            last_session_save: 0.0,
            focus_editor: true,
            sombre: true,
        };

        if let Some(session) = persistence::load(&cc.egui_ctx) {
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
        }
        app
    }

    fn open_path(&mut self, path: PathBuf) {
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
    }

    fn save_tab(&mut self, index: usize) {
        let Some(tab) = self.tabs.tabs.get_mut(index) else {
            return;
        };
        let Some(path) = tab.path.clone() else {
            self.save_tab_as(index);
            return;
        };
        match fs_ops::write_bytes(&path, &tab.buffer.to_bytes()) {
            Ok(_) => {
                tab.dirty = false;
                self.status.clear();
            }
            Err(e) => self.status = format!("{}: {}", self.i18n.t("error"), e),
        }
    }

    fn save_tab_as(&mut self, index: usize) {
        let Some(tab) = self.tabs.tabs.get_mut(index) else {
            return;
        };
        if let Some(path) = rfd::FileDialog::new().save_file() {
            match fs_ops::write_bytes(&path, &tab.buffer.to_bytes()) {
                Ok(_) => {
                    tab.path = Some(path);
                    tab.dirty = false;
                    self.status.clear();
                }
                Err(e) => self.status = format!("{}: {}", self.i18n.t("error"), e),
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
    }

    fn undo(&mut self) {
        if let Some(tab) = self.tabs.active_tab_mut() {
            if tab.buffer.undo() {
                tab.dirty = true;
            }
        }
        self.focus_editor = true;
    }

    fn redo(&mut self) {
        if let Some(tab) = self.tabs.active_tab_mut() {
            if tab.buffer.redo() {
                tab.dirty = true;
            }
        }
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
                                self.open_path(p);
                            }
                        }
                        egui::Key::W => {
                            let idx = self.tabs.active;
                            self.request_close_tab(idx);
                        }
                        egui::Key::F => {
                            self.search.open = true;
                            self.search.mode = SearchMode::Find;
                        }
                        egui::Key::H => {
                            self.search.open = true;
                            self.search.mode = SearchMode::Replace;
                        }
                        egui::Key::Z => self.undo(),
                        egui::Key::Y => self.redo(),
                        egui::Key::Tab => {
                            if self.tabs.tabs.len() > 1 {
                                self.tabs.active = (self.tabs.active + 1) % self.tabs.tabs.len();
                                self.focus_editor = true;
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
        let Some(tab) = self.tabs.active_tab() else {
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
        self.search.match_index = (self.search.match_index + 1) % matches.len();
        let start = matches[self.search.match_index].start;
        if let Some(tab) = self.tabs.active_tab_mut() {
            tab.cursor = start;
            tab.scroll_to = Some(start);
        }
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
        let idx = self.search.match_index.min(matches.len() - 1);
        let m = &matches[idx];
        let start = m.start;
        let end = m.end;
        let before = tab.buffer.text.clone();
        tab.buffer
            .text
            .replace_range(start..end, &self.search.replacement);
        tab.buffer.push_snapshot_after_change(before);
        tab.dirty = true;
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
                                self.open_path(p);
                            }
                            ui.close_menu();
                        }
                        if ui.button(self.i18n.t("open_folder")).clicked() {
                            if let Some(p) = rfd::FileDialog::new().pick_folder() {
                                self.explorer.set_root(p);
                            }
                            ui.close_menu();
                        }
                    });
                    ui.menu_button(self.i18n.t("edit"), |ui| {
                        if ui.button(self.i18n.t("undo")).clicked() {
                            self.undo();
                            ui.close_menu();
                        }
                        if ui.button(self.i18n.t("redo")).clicked() {
                            self.redo();
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button(self.i18n.t("find")).clicked() {
                            self.search.open = true;
                            self.search.mode = SearchMode::Find;
                            ui.close_menu();
                        }
                        if ui.button(self.i18n.t("replace")).clicked() {
                            self.search.open = true;
                            self.search.mode = SearchMode::Replace;
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
                        }
                        ui.separator();
                        if ui.button(self.i18n.t("language")).clicked() {
                            self.i18n.lang = match self.i18n.lang {
                                Lang::Fr => Lang::En,
                                Lang::En => Lang::Fr,
                            };
                            ui.close_menu();
                        }
                    });
                    ui.menu_button(self.i18n.t("help"), |ui| {
                        if ui.button(self.i18n.t("about")).clicked() {
                            self.status = format!(
                                "Lechapes {} - {}",
                                env!("CARGO_PKG_VERSION"),
                                self.i18n.t("editor")
                            );
                            ui.close_menu();
                        }
                    });
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if theme::bouton_image_seule(ui, "redo.png", 28.0, 16.0) {
                        self.redo();
                    }
                    ui.add_space(4.0);
                    if theme::bouton_image_seule(ui, "undo.png", 28.0, 16.0) {
                        self.undo();
                    }
                });
            });
        });
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
                        }
                        if let Some(i) = to_activate {
                            self.tabs.active = i;
                            self.focus_editor = true;
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
                ui.add(
                    egui::TextEdit::singleline(&mut self.search.query)
                        .hint_text(self.i18n.t("search_hint"))
                        .desired_width(220.0),
                );
                if self.search.mode == SearchMode::Replace {
                    ui.label("->");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.search.replacement)
                            .hint_text(self.i18n.t("replace_hint"))
                            .desired_width(220.0),
                    );
                }
                ui.checkbox(&mut self.search.case_sensitive, self.i18n.t("case_sensitive"));
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

                            let resp = ui.add_sized(
                                egui::vec2(largeur_editeur, 0.0),
                                egui::TextEdit::multiline(&mut tab.buffer.text)
                                    .font(egui::TextStyle::Monospace)
                                    .code_editor()
                                    .desired_width(largeur_editeur)
                                    .frame(false)
                                    .lock_focus(true)
                                    .id(editor_id)
                                    .text_color(couleur_texte_editeur),
                            );

                            ui.visuals_mut().extreme_bg_color = ancienne_couleur;

                            if resp.changed() {
                                tab.dirty = true;
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
        egui::Window::new(self.i18n.t("confirm_close"))
            .collapsible(false)
            .resizable(false)
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
                        confirmed = true;
                    }
                    if ui.button(self.i18n.t("quit_no_save")).clicked() {
                        confirmed = true;
                    }
                    if ui.button(self.i18n.t("cancel")).clicked() {
                        cancelled = true;
                    }
                });
            });
        if cancelled {
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
        let name = self
            .tabs
            .tabs
            .get(idx)
            .map(|t| t.title(self.i18n.t("untitled")))
            .unwrap_or_default();
        egui::Window::new(self.i18n.t("confirm_close"))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.label(format!(
                    "{} : {}",
                    name,
                    self.i18n.t("confirm_close_msg")
                ));
                ui.horizontal(|ui| {
                    if ui.button(self.i18n.t("save")).clicked() {
                        self.save_tab(idx);
                        confirmed = true;
                    }
                    if ui.button(self.i18n.t("quit_no_save")).clicked() {
                        confirmed = true;
                    }
                    if ui.button(self.i18n.t("cancel")).clicked() {
                        cancelled = true;
                    }
                });
            });
        if cancelled {
            self.pending_tab_close = None;
        }
        if confirmed {
            self.tabs.close_tab(idx);
            self.pending_tab_close = None;
            self.focus_editor = true;
        }
    }

    fn save_session(&mut self, ctx: &egui::Context) {
        let now = ctx.input(|i| i.time);
        if now - self.last_session_save < 5.0 {
            return;
        }
        self.last_session_save = now;
        let session = Session {
            open_paths: self
                .tabs
                .tabs
                .iter()
                .filter_map(|t| t.path.as_ref().map(|p| p.to_string_lossy().to_string()))
                .collect(),
            active: self.tabs.active,
            window_size: None,
        };
        persistence::save(ctx, &session);
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
                if let Some(p) = self.explorer.ui(ui, &self.i18n) {
                    self.open_path(p);
                }
            });

        self.editor(ctx);
        self.handle_pending_close(ctx);
        self.handle_pending_tab_close(ctx);
        self.save_session(ctx);
    }
}
