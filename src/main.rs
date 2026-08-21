#![windows_subsystem = "windows"]
use eframe::egui;
use std::env;
use std::path::PathBuf;

mod app;
mod image_utils;
mod state;
mod ui;
mod win7;

use app::ImageViewerApp;

fn main() -> eframe::Result<()>
{
    // Catch panics and write them to crash.log before dying
    std::panic::set_hook(Box::new(|info| 
        {
        let log = format!(
            "CRASH REASON:\n{info}\n\nBACKTRACE:\n{}",
            std::backtrace::Backtrace::force_capture()
        );

        let log_path = std::env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(|dir| dir.join("river_crash.log")))
            .unwrap_or_else(|| std::path::PathBuf::from("river_crash.log"));

        let _ = std::fs::write(log_path, log);
    }));

    let initial_file = env::args_os().nth(1).map(PathBuf::from);

    let options = eframe::NativeOptions
    {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 400.0])
            .with_title("Rust Image ViewER"),
        ..Default::default()
    };

    eframe::run_native(
        "Rust Image ViewER",
        options,
        Box::new(|cc| 
        {
            Ok(Box::new(ImageViewerApp::new(cc, initial_file)))
        }),
    )
}