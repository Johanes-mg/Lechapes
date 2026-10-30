use eframe::egui;

use crate::buffer::Buffer;
use crate::search::SearchState;

#[derive(Default)]
pub struct Editor;

impl Editor {
    pub fn ui(&mut self, ui: &mut egui::Ui, buffer: &mut Buffer, search: &SearchState) {
        let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
            let job = if search.visible && !search.matches.is_empty() {
                let mut job = egui::text::LayoutJob::default();
                let mut pos = 0usize;
                for (i, (start, end)) in search.matches.iter().enumerate() {
                    let s = (*start).min(string.len());
                    let e = (*end).min(string.len());
                    if s > pos {
                        job.append(
                            &string[pos..s],
                            0.0,
                            egui::TextFormat::simple(
                                egui::FontId::monospace(14.0),
                                ui.visuals().text_color(),
                            ),
                        );
                    }
                    let couleur = if Some(i) == search.current {
                        egui::Color32::from_rgb(255, 140, 0)
                    } else {
                        egui::Color32::from_rgb(120, 90, 0)
                    };
                    let mut fmt = egui::TextFormat::simple(
                        egui::FontId::monospace(14.0),
                        egui::Color32::BLACK,
                    );
                    fmt.background = couleur;
                    job.append(&string[s..e], 0.0, fmt);
                    pos = e;
                }
                if pos < string.len() {
                    job.append(
                        &string[pos..],
                        0.0,
                        egui::TextFormat::simple(
                            egui::FontId::monospace(14.0),
                            ui.visuals().text_color(),
                        ),
                    );
                }
                job.wrap.max_width = wrap_width;
                job
            } else {
                let mut job = egui::text::LayoutJob::single_section(
                    string.to_owned(),
                    egui::TextFormat::simple(
                        egui::FontId::monospace(14.0),
                        ui.visuals().text_color(),
                    ),
                );
                job.wrap.max_width = wrap_width;
                job
            };
            ui.fonts(|f| f.layout_job(job))
        };

        egui::ScrollArea::both().show(ui, |ui| {
            let reponse = ui.add(
                egui::TextEdit::multiline(&mut buffer.texte)
                    .desired_width(f32::INFINITY)
                    .desired_rows(30)
                    .font(egui::TextStyle::Monospace)
                    .layouter(&mut layouter),
            );

            if reponse.changed() {
                buffer.enregistrer_undo();
            }
        });
    }
}
