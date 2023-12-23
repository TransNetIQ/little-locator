use serde::{Deserialize, Serialize};

/// Данные о размерах карты.
#[derive(Deserialize, Serialize)]
pub struct MapSizes {
  pub l: f32,
  pub w: f32,
}

/// Данные о местоположении.
#[derive(Deserialize, Serialize, Clone)]
pub struct Location {
  pub id: String,
  pub x: f32,
  pub y: f32,
  pub z: f32,
  #[serde(default = "curr_ts")]
  pub ts: i64,
}

pub fn curr_ts() -> i64 {
  chrono::Local::now().naive_local().timestamp_millis()
}

impl std::fmt::Display for Location {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&format!("Местоположение объекта #{}: x - {:.3}, y - {:.3}, z - {:.3}; информация получена {}.", self.id, self.x, self.y, self.z, chrono::NaiveDateTime::from_timestamp_millis(self.ts).unwrap()))
  }
}

pub const MAX_QUEUE_LEN: usize = 1024;
