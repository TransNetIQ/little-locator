use serde::{Deserialize, Serialize};

/// Данные о размерах карты.
#[derive(Deserialize, Serialize, Clone)]
pub struct MapSizes {
  pub l: f32,
  pub w: f32,
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Clone)]
pub enum LocationType {
  Tag,
  Anchor,
}

/// Данные о местоположении.
#[derive(Deserialize, Serialize, Clone)]
pub struct Location {
  pub id: String,
  #[serde(default = "default_loc_type")]
  pub loc_type: LocationType,
  pub x: f32,
  pub y: f32,
  pub z: f32,
  #[serde(default = "curr_ts")]
  pub ts: i64,
  pub dist: Option<Vec<DistToAnchor>>,
}

/// Дистанция до анкера.
#[derive(Deserialize, Serialize, Clone)]
pub struct DistToAnchor {
  pub aid: String,
  pub dist: f32,
}

/// Позиция в трёхмерном пространстве.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AnchorPos {
  pub id: String,
  pub x: f32,
  pub y: f32,
  pub z: f32,
}

/// Максимальный радиус прилипания (для передачи по HTTP).
#[derive(Deserialize, Serialize, Clone)]
pub struct MaxStickingRadius {
  pub max_sticking_radius: f32,
}

pub fn curr_ts() -> i64 { chrono::Local::now().naive_local().and_utc().timestamp_millis() }
pub fn default_loc_type() -> LocationType { LocationType::Tag }

impl std::fmt::Display for Location {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&format!("Местоположение объекта #{}: x - {:.3}, y - {:.3}, z - {:.3}; информация получена {}.", self.id, self.x, self.y, self.z, chrono::DateTime::from_timestamp_millis(self.ts).unwrap()))
  }
}

pub const MAX_QUEUE_LEN: usize = 1024;
