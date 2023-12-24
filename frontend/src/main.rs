#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;
mod utils;

use crate::app::app_data::LittleLocatorApp;

// Для запуска нативно.
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
  let native_options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default()
      .with_inner_size([400.0, 300.0])
      .with_min_inner_size([300.0, 220.0]),
    ..Default::default()
  };
  eframe::run_native(
    "eframe template",
    native_options,
    Box::new(|cc| Box::new(LittleLocatorApp::new(cc))),
  )
}

// Для сборки в WASM.
#[cfg(target_arch = "wasm32")]
fn main() {
  // Redirect `log` message to `console.log` and friends:
  #[cfg(debug_assertions)]
  wasm_logger::init(wasm_logger::Config::default());
  #[cfg(not(debug_assertions))]
  wasm_logger::init(wasm_logger::Config::new(log::Level::Warn));

  let web_options = eframe::WebOptions::default();

  wasm_bindgen_futures::spawn_local(async {
    eframe::WebRunner::new()
      .start(
        "main_canvas",
        web_options,
        Box::new(|cc| Box::new(LittleLocatorApp::new(cc))),
      )
      .await
      .expect("failed to start eframe");
  });
}
