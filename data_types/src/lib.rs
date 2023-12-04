use chrono::{serde::ts_microseconds, DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Данные о местоположении.
#[derive(Deserialize, Serialize, Clone)]
pub struct Location {
  pub id: String,
  pub x: f32,
  pub y: f32,
  pub z: f32,
  #[serde(default = "curr_ts")]
  #[serde(with = "ts_microseconds")]
  pub ts: DateTime<Utc>,
}

fn curr_ts() -> DateTime<Utc> {
  chrono::Utc::now()
}

impl std::fmt::Display for Location {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&format!("Местоположение объекта #{}: x - {}, y - {}, z - {}; информация получена {}.", self.id, self.x, self.y, self.z, self.ts))
  }
}
