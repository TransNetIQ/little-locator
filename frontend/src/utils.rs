use chrono::{NaiveDate, NaiveDateTime};
use wasm_bindgen::JsValue;
use std::sync::PoisonError;

pub const HOURS: [&'static str; 24] = ["00", "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11", "12", "13", "14", "15", "16", "17", "18", "19", "20", "21", "22", "23"];

pub const MINUTES: [&'static str; 60] = ["00", "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11", "12", "13", "14", "15", "16", "17", "18", "19", "20", "21", "22", "23", "24", "25", "26", "27", "28", "29", "30", "31", "32", "33", "34", "35", "36", "37", "38", "39", "40", "41", "42", "43", "44", "45", "46", "47", "48", "49", "50", "51", "52", "53", "54", "55", "56", "57", "58", "59"];

pub type LimitDateTime = (NaiveDate, usize, usize);

pub fn construct_dt(limit_dt: &LimitDateTime) -> NaiveDateTime {
  limit_dt.0
    .and_hms_opt(
      limit_dt.1 as u32,
      limit_dt.2 as u32,
      0u32
    )
    .unwrap()
}

/// Структура данных для ошибок.
#[derive(Clone, Debug)]
pub struct AppError {
  pub message: String,
}

impl From<String> for AppError {
  fn from(value: String) -> Self { AppError { message: value } }
}

impl From<&str> for AppError {
  fn from(value: &str) -> Self { AppError { message: value.to_owned() } }
}

impl<T> From<PoisonError<T>> for AppError {
  fn from(value: PoisonError<T>) -> Self { value.to_string().into() }
}

impl From<std::io::Error> for AppError {
  fn from(value: std::io::Error) -> Self { value.to_string().into() }
}

impl From<image::ImageError> for AppError {
  fn from(value: image::ImageError) -> Self { value.to_string().into() }
}

impl From<JsValue> for AppError {
  fn from(value: JsValue) -> Self { value.as_string().unwrap_or("Не удалось получить текст ошибки JsValue.".into()).into() }
}

pub type MResult<T> = Result<T, AppError>;
