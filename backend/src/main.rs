#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod server;
mod stnc;
mod threaded_location_sender;
mod utils;

use crate::server::{
  get_tag_img,
  get_green_tag_img,
  get_anchor_img,
  post_new_location,
  post_new_anchor,
  ws_location_sender,
  get_config,
  get_anchors,
  get_location_img,
  get_max_sticking_radius,
  update_data,
};
use crate::threaded_location_sender::start_threaded_location_sender;
use crate::utils::{DATA_TX_QUEUE, DATA_RX_QUEUE};

use ll_data::MAX_QUEUE_LEN;
use log::debug;
use salvo::{Listener, Router, Server, conn::TcpListener};
use simple_logger::SimpleLogger;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

/// Запускает программу.
#[tokio::main]
async fn main() {
  #[cfg(debug_assertions)]
  SimpleLogger::new().with_level(log::LevelFilter::Debug).init().unwrap();
  #[cfg(not(debug_assertions))]
  SimpleLogger::new().with_level(log::LevelFilter::Info).init().unwrap();

  let (tx, rx) = mpsc::channel(MAX_QUEUE_LEN);
  DATA_TX_QUEUE.set(tx).unwrap();
  DATA_RX_QUEUE.set(Arc::new(Mutex::new(rx))).unwrap();
  debug!("Created DATA_QUEUE");

  start_threaded_location_sender().await.unwrap();
  
  stnc::update_data().await.unwrap();

  let router = Router::new()
    .post(post_new_location)
    .push(Router::with_path("config").get(get_config))
    .push(Router::with_path("anchors").get(get_anchors).post(post_new_anchor))
    .push(Router::with_path("location_img").get(get_location_img))
    .push(Router::with_path("tag_img").get(get_tag_img))
    .push(Router::with_path("green_tag_img").get(get_green_tag_img))
    .push(Router::with_path("anchor_img").get(get_anchor_img))
    .push(Router::with_path("msr").get(get_max_sticking_radius))
    .push(Router::with_path("ws_updater").goal(ws_location_sender))
    .push(Router::with_path("update_data").goal(update_data))
    .push(Router::with_path("<**path>").get(salvo::serve_static::StaticDir::new(["./dist"]).defaults("index.html")));
  let acceptor = TcpListener::new("0.0.0.0:5800").bind().await;
  Server::new(acceptor).serve(router).await;
}
