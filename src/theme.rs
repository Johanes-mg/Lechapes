use egui::{Color32, Rounding, Stroke, TextureFilter, TextureHandle, TextureOptions, TextureWrapMode, Visuals};
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

fn filtrer_pour(taille: [usize; 2]) -> TextureOptions {
    let filtre = if taille[0] <= 64 && taille[1] <= 64 {
        TextureFilter::Nearest
    } else {
        TextureFilter::Linear
    };
    TextureOptions {
        magnification: filtre,
        minification: filtre,
        wrap_mode: TextureWrapMode::ClampToEdge,
    }
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
    let texture = ctx.load_texture(cle_cache.clone(), color_image, filtrer_pour(taille));
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
    painter.rect_filled(rect, 8.0, fond);

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
    let (rect, resp) = ui
        .push_id(fichier, |ui| {
            ui.allocate_exact_size(egui::vec2(taille, taille), egui::Sense::click())
        })
        .inner;
    let fond = if resp.hovered() {
        ui.visuals().widgets.hovered.bg_fill
    } else {
        Color32::TRANSPARENT
    };
    let painter = ui.painter();
    painter.rect_filled(rect, 8.0, fond);
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
    let mut visuals = if sombre { Visuals::dark() } else { Visuals::light() };

    let bg_panel = if sombre { Color32::from_rgb(24, 26, 30) } else { Color32::from_rgb(248, 249, 251) };
    let bg_window = if sombre { Color32::from_rgb(34, 37, 43) } else { Color32::WHITE };
    let bg_extreme = if sombre { Color32::from_rgb(18, 20, 24) } else { Color32::WHITE };
    let bg_faint = if sombre { Color32::from_rgb(30, 33, 38) } else { Color32::from_rgb(240, 242, 245) };
    let bg_widget = if sombre { Color32::from_rgb(42, 45, 52) } else { Color32::WHITE };
    let bg_widget_weak = if sombre { Color32::from_rgb(38, 41, 47) } else { Color32::from_rgb(244, 246, 249) };
    let border = if sombre { Color32::from_rgb(55, 58, 65) } else { Color32::from_rgb(222, 226, 232) };
    let text = if sombre { Color32::from_rgb(224, 228, 234) } else { Color32::from_rgb(30, 35, 45) };
    let selection = if sombre { Color32::from_rgb(60, 90, 140) } else { Color32::from_rgb(210, 225, 245) };

    visuals.panel_fill = bg_panel;
    visuals.window_fill = bg_window;
    visuals.extreme_bg_color = bg_extreme;
    visuals.faint_bg_color = bg_faint;
    visuals.window_rounding = Rounding::same(12.0);
    visuals.menu_rounding = Rounding::same(10.0);
    visuals.window_shadow = egui::epaint::Shadow {
        offset: egui::vec2(0.0, 6.0),
        blur: 16.0,
        spread: 0.0,
        color: Color32::from_black_alpha(60),
    };

    visuals.widgets.noninteractive.bg_fill = bg_faint;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, border);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, text);
    visuals.widgets.noninteractive.rounding = Rounding::same(8.0);

    visuals.widgets.inactive.bg_fill = bg_widget;
    visuals.widgets.inactive.weak_bg_fill = bg_widget_weak;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, border);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, text);
    visuals.widgets.inactive.rounding = Rounding::same(8.0);

    visuals.widgets.hovered.bg_fill = if sombre { Color32::from_rgb(52, 56, 64) } else { Color32::from_rgb(232, 234, 238) };
    visuals.widgets.hovered.weak_bg_fill = visuals.widgets.hovered.bg_fill;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, if sombre { Color32::from_rgb(90, 150, 220) } else { Color32::from_rgb(180, 200, 230) });
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, text);
    visuals.widgets.hovered.rounding = Rounding::same(8.0);

    visuals.widgets.active.bg_fill = if sombre { Color32::from_rgb(58, 64, 74) } else { Color32::from_rgb(220, 224, 230) };
    visuals.widgets.active.weak_bg_fill = visuals.widgets.active.bg_fill;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, if sombre { Color32::from_rgb(120, 170, 230) } else { Color32::from_rgb(120, 170, 220) });
    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, text);
    visuals.widgets.active.rounding = Rounding::same(8.0);

    visuals.widgets.open.bg_fill = bg_widget;
    visuals.widgets.open.weak_bg_fill = bg_widget_weak;
    visuals.widgets.open.bg_stroke = Stroke::new(1.0_f32, border);
    visuals.widgets.open.fg_stroke = Stroke::new(1.0_f32, text);
    visuals.widgets.open.rounding = Rounding::same(8.0);

    visuals.selection.bg_fill = selection;
    visuals.selection.stroke = Stroke::new(1.0_f32, if sombre { Color32::from_rgb(160, 200, 250) } else { Color32::from_rgb(60, 110, 180) });

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(10.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 8.0);
    style.spacing.menu_margin = egui::Margin::same(6.0);
    style.spacing.interact_size.y = 26.0;
    ctx.set_style(style);
}

pub fn configurer_style_editeur(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals.widgets.inactive.bg_stroke = Stroke::NONE;
    style.visuals.widgets.hovered.bg_stroke = Stroke::NONE;
    style.visuals.widgets.active.bg_stroke = Stroke::NONE;
    ctx.set_style(style);
}

pub fn line_number_width(total_lines: usize) -> f32 {
    let digits = total_lines.to_string().len().max(2) as f32;
    digits * 8.5 + 16.0
}
