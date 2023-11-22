//! Утилиты для приложения.

use once_cell::sync::OnceCell;
use salvo::{Depot, Request, Response, http::{ParseError, StatusCode}, Writer};
use salvo::async_trait;
use serde::Deserialize;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// Ошибка сервера.
pub struct ServerError {
  pub msg: String
}

#[async_trait]
impl Writer for ServerError {
  async fn write(mut self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
    res.render(self.msg);
  }
}

impl From<String> for ServerError {
  fn from(string: String) -> Self {
    Self { msg: string }
  }
}

impl From<&str> for ServerError {
  fn from(str: &str) -> Self {
    str.to_owned().into()
  }
}

impl From<ParseError> for ServerError {
  fn from(value: ParseError) -> Self {
    value.to_string().into()
  }
}

impl<T> From<std::sync::PoisonError<T>> for ServerError {
  fn from(value: std::sync::PoisonError<T>) -> Self {
    value.to_string().into()
  }
}

pub type MResult<T> = Result<T, ServerError>;

/// Данные о местоположении.
#[derive(Deserialize, Clone)]
pub struct Location {
  pub id: String,
  pub x: f32,
  pub y: f32,
  pub z: f32,
}

impl std::fmt::Display for Location {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&format!("Местоположение объекта #{}: x - {}, y - {}, z - {}", self.id, self.x, self.y, self.z))
  }
}

// Ячейка для обмена данными между бэкендом и фронтендом.
pub type DataQueue<T> = Arc<Mutex<VecDeque<T>>>;
pub static DATA_QUEUE: OnceCell<DataQueue<Location>> = OnceCell::new();
