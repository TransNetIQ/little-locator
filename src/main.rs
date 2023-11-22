#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod server;
mod utils;

use eframe::{NativeOptions as GuiNativeOptions, run_native as run_app};
use salvo::{Listener, Router, Server, conn::TcpListener};
use tokio::task::spawn as async_spawn;

use crate::app::LittleLocatorApp;
use crate::server::post_new_location;

/// Запускает программу.
#[tokio::main]
async fn main() {
  tracing_subscriber::fmt().init();
  async_spawn(async {
    let router = Router::new().post(post_new_location);
    let acceptor = TcpListener::new("127.0.0.1:5800").bind().await;
    Server::new(acceptor).serve(router).await;
  });
  run_app("Little Locator", GuiNativeOptions::default(), Box::new(|cc| Box::new(LittleLocatorApp::new(cc)))).unwrap();
}
