#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod server;
mod threaded_location_sender;
mod utils;

use salvo::{Listener, Router, Server, conn::TcpListener};
use salvo::cors::Cors;
use salvo::http::Method;

use crate::server::{get_position_img, post_new_location, ws_location_sender};
use crate::threaded_location_sender::start_threaded_location_sender;
use crate::utils::{DATA_TX_QUEUE, DATA_RX_QUEUE};

/// Запускает программу.
#[tokio::main]
async fn main() {
  tracing_subscriber::fmt().init();

  let (tx, rx) = std::sync::mpsc::channel();
  DATA_TX_QUEUE.set(tx).unwrap();
  DATA_RX_QUEUE.set(std::sync::Arc::new(tokio::sync::Mutex::new(rx))).unwrap();

  start_threaded_location_sender().await.unwrap();
  
  let cors_handler = Cors::new()
    .allow_origin("*")
    .allow_methods(vec![Method::GET, Method::POST])
    .into_handler();

  let router = Router::with_hoop(cors_handler)
    .post(post_new_location)
    .options(salvo::handler::empty())
    .push(
      Router::with_path("position_img")
        .get(get_position_img)
        .options(salvo::handler::empty())
    )
    .push(
      Router::with_path("ws_updater")
        .goal(ws_location_sender)
        .options(salvo::handler::empty())
    )
    .push(
      Router::with_path("<**path>")
        .get(salvo::serve_static::StaticDir::new(["../frontend/dist"]).defaults("index.html"))
        .options(salvo::handler::empty())
    );
  let acceptor = TcpListener::new("0.0.0.0:5800").bind().await;
  Server::new(acceptor).serve(router).await;
}
