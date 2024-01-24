#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;
mod utils;

use crate::app::app_data::LittleLocatorApp;

// Для сборки в WASM.
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
        Box::new(|cc| Box::new(LittleLocatorApp::new(cc).unwrap())),
      )
      .await
      .expect("failed to start eframe");
  });
}
