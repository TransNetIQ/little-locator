//! Утилиты для приложения.

use ll_data::{Location, AnchorPos};
use salvo::{Depot, Request, Response, http::{ParseError, StatusCode}, Writer};
use salvo::async_trait;
use serde::{Deserialize, Serialize};
use std::{string::FromUtf8Error, sync::{Arc, OnceLock}};
use tokio::sync::{mpsc, Mutex};

/// Конфигурация приложения.
#[derive(Deserialize, Serialize)]
pub struct AppConfig {
  pub image_filepath: Option<String>,
  pub length: f32,
  pub width: f32,
  pub max_sticking_radius: Option<f32>,
  pub anchors: Vec<AnchorPos>,
  pub stnc_renaissance_username: Option<String>,
  pub stnc_renaissance_password: Option<String>,
  pub org_name: Option<String>,
  pub building_id: Option<i32>,
  pub floor_id: Option<i32>,
}

/// Ошибка сервера.
#[derive(Debug)]
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

impl<T> From<mpsc::error::SendError<T>> for ServerError {
  fn from(value: mpsc::error::SendError<T>) -> Self {
    value.to_string().into()
  }
}

impl From<tokio::sync::TryLockError> for ServerError {
  fn from(value: tokio::sync::TryLockError) -> Self {
    value.to_string().into()
  }
}

impl From<salvo::http::StatusError> for ServerError {
  fn from(value: salvo::http::StatusError) -> Self {
    value.to_string().into()
  }
}

impl From<std::io::Error> for ServerError {
  fn from(value: std::io::Error) -> Self {
    value.to_string().into()
  }
}

impl From<serde_json::Error> for ServerError {
  fn from(value: serde_json::Error) -> Self {
    value.to_string().into()
  }
}

impl From<reqwest::header::InvalidHeaderValue> for ServerError {
  fn from(value: reqwest::header::InvalidHeaderValue) -> Self {
    value.to_string().into()
  }
}

impl From<reqwest::Error> for ServerError {
  fn from(value: reqwest::Error) -> Self {
    value.to_string().into()
  }
}

impl From<FromUtf8Error> for ServerError {
  fn from(value: FromUtf8Error) -> Self {
    value.to_string().into()
  }
}

pub type MResult<T> = Result<T, ServerError>;

// Ячейка для обмена данными между бэкендом и фронтендом.
pub type DataTxQueue<T> = mpsc::Sender<T>;
pub type DataRxQueue<T> = Arc<Mutex<mpsc::Receiver<T>>>;
pub type WsTxQueue<T> = Arc<Mutex<mpsc::Sender<mpsc::Sender<T>>>>;
pub static DATA_TX_QUEUE: OnceLock<DataTxQueue<Location>> = OnceLock::new();
pub static DATA_RX_QUEUE: OnceLock<DataRxQueue<Location>> = OnceLock::new();
pub static WS_TX_QUEUE: OnceLock<WsTxQueue<Location>> = OnceLock::new();
