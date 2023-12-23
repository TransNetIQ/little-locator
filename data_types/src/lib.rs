use chrono::{serde::ts_seconds, DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Данные о местоположении.
#[derive(Deserialize, Serialize, Clone)]
pub struct Location {
  pub id: String,
  pub x: f32,
  pub y: f32,
  pub z: f32,
  #[serde(default = "curr_ts")]
  #[serde(with = "ts_seconds")]
  pub ts: DateTime<Utc>,
}

pub fn curr_ts() -> DateTime<Utc> {
  chrono::Utc::now()
}

impl std::fmt::Display for Location {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&format!("Местоположение объекта #{}: x - {:.3}, y - {:.3}, z - {:.3}; информация получена {}.", self.id, self.x, self.y, self.z, self.ts))
  }
}

pub const MAX_QUEUE_LEN: usize = 1024;