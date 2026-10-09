#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod engine;
mod studio;
mod audio;
mod pitched;
mod ghl;
mod star_power;
mod timing;
mod lyrics;
fn main() -> eframe::Result<()> {
    if std::env::args().any(|a| a == "--smoke-test") {
        engine::smoke_test().expect("end-to-end conversion failed");
        return Ok(());
    }
    eframe::run_native("Charting Toolkit · Studio", eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size([1240.0, 980.0]).with_min_inner_size([960.0, 640.0]),
        ..Default::default()
    }, Box::new(|cc| Ok(Box::new(studio::Studio::new(cc)))))
}
