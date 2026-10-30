#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod buffer;
mod explorer;
mod fs_ops;
mod i18n;
mod persistence;
mod plugins;
mod search;
mod tabs;
mod theme;
mod viewer;

use app::LechapesApp;
use std::sync::Arc;

fn main() -> eframe::Result<()> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1200.0, 760.0])
        .with_min_inner_size([760.0, 480.0])
        .with_title("Lechapes");

    if let Some(icone) = theme::charger_icone_fenetre("logo.png") {
        viewport = viewport.with_icon(Arc::new(icone));
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Lechapes",
        options,
        Box::new(|cc| {
            theme::configurer_style(&cc.egui_ctx, true);
            theme::configurer_style_editeur(&cc.egui_ctx);
            Ok(Box::new(LechapesApp::new(cc)))
        }),
    )
}
