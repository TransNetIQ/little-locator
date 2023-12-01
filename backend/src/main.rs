#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod server;
mod utils;

use salvo::{Listener, Router, Server, conn::TcpListener};
use salvo::cors::Cors;
use salvo::http::Method;

use crate::server::{get_position_img, post_new_location};

/// Запускает программу.
#[tokio::main]
async fn main() {
  tracing_subscriber::fmt().init();

  let cors_handler = Cors::new()
    .allow_origin("http://127.0.0.1:5800")
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
      Router::with_path("<**path>")
        .get(
          salvo::serve_static::StaticDir::new(["../frontend/dist"]).defaults("index.html")
        )
        .options(salvo::handler::empty())
    );
  let acceptor = TcpListener::new("0.0.0.0:5800").bind().await;
  Server::new(acceptor).serve(router).await;
}
