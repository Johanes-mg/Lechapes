use egui::{Color32, TextureFilter, TextureHandle, TextureOptions, TextureWrapMode};
use std::collections::HashMap;
use std::sync::Mutex;

static IMAGES: Mutex<Option<HashMap<String, TextureHandle>>> = Mutex::new(None);

fn chemins_possibles(nom: &str) -> Vec<String> {
    let mut chemins = vec![format!("images/{}", nom)];
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            chemins.push(dir.join("images").join(nom).to_string_lossy().to_string());
        }
    }
    chemins.push(format!("{}/images/{}", env!("CARGO_MANIFEST_DIR"), nom));
    chemins
}

fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

pub fn image(ctx: &egui::Context, nom: &str) -> Option<TextureHandle> {
    let mut contenu: Option<Vec<u8>> = None;
    for chemin in chemins_possibles(nom) {
        if let Ok(bytes) = std::fs::read(&chemin) {
            contenu = Some(bytes);
            break;
        }
    }
    let bytes = contenu?;
    let hash = hash_bytes(&bytes);
    let cle_cache = format!("{}#{}", nom, hash);

    {
        let cache = IMAGES.lock().unwrap();
        if let Some(map) = cache.as_ref() {
            if let Some(tex) = map.get(&cle_cache) {
                return Some(tex.clone());
            }
        }
    }

    let img = image::load_from_memory(&bytes).ok()?;
    let rgba = img.to_rgba8();
    let taille = [rgba.width() as usize, rgba.height() as usize];
    let pixels = rgba.into_raw();
    let color_image = egui::ColorImage::from_rgba_unmultiplied(taille, &pixels);
    let options = TextureOptions {
        magnification: TextureFilter::Linear,
        minification: TextureFilter::Linear,
        wrap_mode: TextureWrapMode::ClampToEdge,
    };
    let texture = ctx.load_texture(cle_cache.clone(), color_image, options);
    let mut cache = IMAGES.lock().unwrap();
    let map = cache.get_or_insert_with(HashMap::new);
    map.insert(cle_cache, texture.clone());
    Some(texture)
}

#[allow(dead_code)]
pub fn bouton_image_texte(
    ui: &mut egui::Ui,
    fichier_image: &str,
    texte: &str,
    largeur: f32,
    hauteur: f32,
) -> bool {
    let ctx = ui.ctx().clone();
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(largeur, hauteur), egui::Sense::click());
    let fond = if resp.hovered() {
        ui.visuals().widgets.hovered.bg_fill
    } else {
        Color32::TRANSPARENT
    };
    let painter = ui.painter();
    painter.rect_filled(rect, 6.0, fond);

    let taille_icone = hauteur - 10.0;
    let marge = 8.0;
    let largeur_icone = if image(&ctx, fichier_image).is_some() {
        taille_icone
    } else {
        0.0
    };
    let largeur_texte = ui.fonts(|f| {
        f.layout_no_wrap(
            texte.to_string(),
            egui::FontId::proportional(13.0),
            ui.visuals().text_color(),
        )
        .size()
        .x
    });
    let contenu_total = largeur_icone + if largeur_icone > 0.0 { marge } else { 0.0 } + largeur_texte;
    let mut x = rect.center().x - contenu_total / 2.0;

    if largeur_icone > 0.0 {
        if let Some(tex) = image(&ctx, fichier_image) {
            let img_rect = egui::Rect::from_center_size(
                egui::pos2(x + taille_icone / 2.0, rect.center().y),
                egui::vec2(taille_icone, taille_icone),
            );
            painter.image(
                tex.id(),
                img_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }
        x += largeur_icone + marge;
    }

    painter.text(
        egui::pos2(x, rect.center().y),
        egui::Align2::LEFT_CENTER,
        texte,
        egui::FontId::proportional(13.0),
        ui.visuals().text_color(),
    );

    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.clicked()
}

pub fn bouton_image_seule(ui: &mut egui::Ui, fichier: &str, taille: f32, taille_icone: f32) -> bool {
    let ctx = ui.ctx().clone();
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(taille, taille), egui::Sense::click());
    let fond = if resp.hovered() {
        ui.visuals().widgets.hovered.bg_fill
    } else {
        Color32::TRANSPARENT
    };
    let painter = ui.painter();
    painter.rect_filled(rect, 6.0, fond);
    if let Some(tex) = image(&ctx, fichier) {
        let img_rect = egui::Rect::from_center_size(
            rect.center(),
            egui::vec2(taille_icone, taille_icone),
        );
        painter.image(
            tex.id(),
            img_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            Color32::WHITE,
        );
    } else {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "?",
            egui::FontId::proportional(taille_icone * 0.6),
            ui.visuals().text_color(),
        );
    }
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.clicked()
}

pub fn charger_icone_fenetre(nom: &str) -> Option<egui::IconData> {
    for chemin in chemins_possibles(nom) {
        if let Ok(bytes) = std::fs::read(&chemin) {
            if let Ok(img) = image::load_from_memory(&bytes) {
                let rgba = img.to_rgba8();
                let (w, h) = rgba.dimensions();
                return Some(egui::IconData {
                    rgba: rgba.into_raw(),
                    width: w,
                    height: h,
                });
            }
        }
    }
    None
}

pub fn configurer_style(ctx: &egui::Context, sombre: bool) {
    let mut visuals = if sombre {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };

    visuals.panel_fill = if sombre {
        Color32::from_rgb(28, 30, 34)
    } else {
        Color32::from_rgb(248, 249, 251)
    };
    visuals.window_fill = if sombre {
        Color32::from_rgb(38, 41, 46)
    } else {
        Color32::WHITE
    };
    visuals.extreme_bg_color = if sombre {
        Color32::from_rgb(20, 22, 26)
    } else {
        Color32::WHITE
    };
    visuals.faint_bg_color = if sombre {
        Color32::from_rgb(34, 36, 41)
    } else {
        Color32::from_rgb(240, 242, 245)
    };
    visuals.widgets.noninteractive.bg_fill = if sombre {
        Color32::from_rgb(34, 36, 41)
    } else {
        Color32::from_rgb(240, 242, 245)
    };
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(
        1.0_f32,
        if sombre {
            Color32::from_rgb(60, 64, 70)
        } else {
            Color32::from_rgb(222, 226, 232)
        },
    );
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(
        1.0_f32,
        if sombre {
            Color32::from_rgb(220, 224, 230)
        } else {
            Color32::from_rgb(30, 35, 45)
        },
    );
    visuals.widgets.inactive.bg_fill = if sombre {
        Color32::from_rgb(45, 48, 54)
    } else {
        Color32::WHITE
    };
    visuals.widgets.inactive.weak_bg_fill = if sombre {
        Color32::from_rgb(40, 43, 48)
    } else {
        Color32::from_rgb(244, 246, 249)
    };
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(
        1.0_f32,
        if sombre {
            Color32::from_rgb(60, 64, 70)
        } else {
            Color32::from_rgb(222, 226, 232)
        },
    );
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(
        1.0_f32,
        if sombre {
            Color32::from_rgb(220, 224, 230)
        } else {
            Color32::from_rgb(30, 35, 45)
        },
    );
    visuals.widgets.inactive.rounding = egui::Rounding::same(6.0);
    visuals.widgets.hovered.rounding = egui::Rounding::same(6.0);
    visuals.widgets.active.rounding = egui::Rounding::same(6.0);
    visuals.selection.bg_fill = if sombre {
        Color32::from_rgb(60, 90, 140)
    } else {
        Color32::from_rgb(210, 225, 245)
    };

    ctx.set_visuals(visuals);
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(10.0, 6.0);
    ctx.set_style(style);
}

pub fn configurer_style_editeur(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
    style.visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
    style.visuals.widgets.active.bg_stroke = egui::Stroke::NONE;
    ctx.set_style(style);
}

pub fn line_number_width(total_lines: usize) -> f32 {
    let digits = total_lines.to_string().len().max(2) as f32;
    digits * 8.5 + 16.0
}