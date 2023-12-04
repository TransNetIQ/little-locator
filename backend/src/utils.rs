//! Утилиты для приложения.

use ll_data::Location;
use salvo::{Depot, Request, Response, http::{ParseError, StatusCode}, Writer};
use salvo::async_trait;
use std::sync::mpsc;
use std::sync::OnceLock;

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

impl<T> From<mpsc::SendError<T>> for ServerError {
  fn from(value: mpsc::SendError<T>) -> Self {
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

pub type MResult<T> = Result<T, ServerError>;

// Ячейка для обмена данными между бэкендом и фронтендом.
pub type DataTxQueue<T> = mpsc::Sender<T>;
pub type DataRxQueue<T> = std::sync::Arc<tokio::sync::Mutex<mpsc::Receiver<T>>>;
pub static DATA_TX_QUEUE: OnceLock<DataTxQueue<Location>> = OnceLock::new();
pub static DATA_RX_QUEUE: OnceLock<DataRxQueue<Location>> = OnceLock::new();