use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, OnceLock};
use std::sync::mpsc;

/// Данные о местоположении.
#[derive(Deserialize, Serialize, Clone)]
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
pub type DataQueue<T> = Arc<Mutex<mpsc::Sender<T>>>;
pub static DATA_QUEUE: OnceLock<DataQueue<Location>> = OnceLock::new();
