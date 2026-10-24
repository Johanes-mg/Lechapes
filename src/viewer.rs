use egui::{ColorImage, Context, TextureHandle, TextureOptions, Window};

pub struct ImageViewer {
    pub open: bool,
    pub path: Option<String>,
    texture: Option<TextureHandle>,
    error: Option<String>,
    taille: (u32, u32),
}

impl ImageViewer {
    pub fn new() -> Self {
        Self {
            open: false,
            path: None,
            texture: None,
            error: None,
            taille: (0, 0),
        }
    }

    pub fn ouvrir(&mut self, ctx: &Context, chemin: String) {
        self.open = true;
        self.path = Some(chemin.clone());
        self.texture = None;
        self.error = None;
        self.taille = (0, 0);
        match std::fs::read(&chemin) {
            Ok(bytes) => match image::load_from_memory(&bytes) {
                Ok(img) => {
                    let rgba = img.to_rgba8();
                    self.taille = rgba.dimensions();
                    let pixels = rgba.into_raw();
                    let color = ColorImage::from_rgba_unmultiplied(
                        [self.taille.0 as usize, self.taille.1 as usize],
                        &pixels,
                    );
                    self.texture = Some(ctx.load_texture(
                        format!("viewer#{}", chemin),
                        color,
                        TextureOptions::LINEAR,
                    ));
                }
                Err(e) => {
                    self.error = Some(format!("Format non supporte : {}", e));
                }
            },
            Err(e) => {
                self.error = Some(format!("Lecture impossible : {}", e));
            }
        }
    }

    pub fn ui(&mut self, ctx: &Context) {
        if !self.open {
            return;
        }
        let mut close = false;
        let nom = self
            .path
            .as_ref()
            .and_then(|p| std::path::Path::new(p).file_name())
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Image".to_string());

        Window::new(nom)
            .resizable(true)
            .default_size([720.0, 540.0])
            .open(&mut self.open)
            .show(ctx, |ui| {
                if let Some(err) = &self.error {
                    ui.colored_label(ui.visuals().error_fg_color, err);
                    return;
                }
                if let Some(tex) = &self.texture {
                    let dispo = ui.available_size();
                    let ratio = self.taille.0 as f32 / self.taille.1.max(1) as f32;
                    let (mut w, mut h) = (dispo.x, dispo.x / ratio);
                    if h > dispo.y {
                        h = dispo.y;
                        w = dispo.y * ratio;
                    }
                    egui::ScrollArea::both().show(ui, |ui| {
                        ui.add(
                            egui::Image::new(tex)
                                .fit_to_exact_size(egui::vec2(w, h))
                                .sense(egui::Sense::hover()),
                        );
                    });
                    ui.label(format!("{} x {}", self.taille.0, self.taille.1));
                }
                if ui.button("Fermer").clicked() {
                    close = true;
                }
            });
        if close {
            self.open = false;
        }
    }
}

pub fn est_image(chemin: &std::path::Path) -> bool {
    let Some(ext) = chemin.extension().and_then(|e| e.to_str()) else {
        return false;
    };
    matches!(
        ext.to_ascii_lowercase().as_str(),
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "ico" | "tif" | "tiff" | "webp"
    )
}