use crate::app::utils::{ImageBytesOptionalRef, OptionalRef};
use crate::utils::LimitDateTime;

use egui::Pos2;
use ll_data::{Location, MapSizes, AnchorPos};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, atomic::AtomicBool};

/// Опции отображения метки.
#[allow(dead_code)]
pub enum ShowTagOps {
  RealCoords,
  GraphSticked,
  Both,
}

/// Опции отображения расстояния меток до анкеров.
#[allow(dead_code)]
pub enum ShowAnchorsDistOps {
  CoordsDists,
  RealDists,
}

/// Опции метки.
pub struct TagSettings {
  pub locations: VecDeque<Location>,
  pub redrawal_index: usize,
  pub visible: bool,
  pub visible_type: usize, // ShowTagOps
  pub show_path: bool,
  pub show_anchor_distance: bool,
  pub anchor_distance_type: usize, // ShowAnchorsDistOps
}

/// Структура приложения.
pub struct LittleLocatorApp {
  // Домен веб-приложения
  pub _server_origin: String,
  // Поля страницы выбора карты
  pub l_input: String,
  pub w_input: String,
  pub done: Arc<AtomicBool>,
  // Данные картинок
  pub tag_image_bytes: ImageBytesOptionalRef,
  pub green_tag_image_bytes: ImageBytesOptionalRef,
  pub anchor_image_bytes: ImageBytesOptionalRef,
  // Данные о местоположениях и карте
  pub location_image: ImageBytesOptionalRef,
  pub location_size: OptionalRef<MapSizes>,
  pub _data_sender: ewebsock::WsSender,
  pub data_receiver: ewebsock::WsReceiver,
  pub tracked_tags_locations: HashMap<String, TagSettings>,
  pub tracked_tags_paths: HashMap<String, VecDeque<Pos2>>,
  pub anchors: OptionalRef<HashMap<String, AnchorPos>>,
  // Ограничения по отрисовке трека
  pub limit_tag_path: bool,
  pub limit_online: bool,
  pub current_limit: LimitDateTime,
  pub previous_limit: LimitDateTime,
  // Выбор страницы для показа
  pub menu: usize,
  // Граф возможных путей
  pub path_traversal_graph: Vec<Vec<Pos2>>,
  pub path_to_add: (bool, Option<Vec<Pos2>>, bool),
  pub last_pos: Option<Pos2>,
  pub max_sticking_radius: OptionalRef<f32>,
  // Дополнительные опции
  pub show_path_traversal_graph: bool,
}
