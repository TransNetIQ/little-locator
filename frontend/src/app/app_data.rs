use crate::app::utils::{ImageBytesOptionalRef, OptionalRef};
use crate::utils::LimitDateTime;

use egui::Pos2;
use ll_data::{Location, MapSizes, AnchorPos};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, atomic::AtomicBool};

/// Опции метки.
pub struct TagSettings {
  pub locations: VecDeque<Location>,
  pub redrawal_index: usize,
  pub visible: bool,
  pub show_path: bool,
  pub show_anchor_calculated_distance: bool,
  pub show_anchor_real_distance: bool,
  pub show_nearest_graph_distance: bool,
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
  pub anchor_image_bytes: ImageBytesOptionalRef,
  // Данные о местоположениях и карте
  pub location_image: ImageBytesOptionalRef,
  pub location_size: OptionalRef<MapSizes>,
  pub _data_sender: ewebsock::WsSender,
  pub data_receiver: ewebsock::WsReceiver,
  pub tracked_tags_locations: HashMap<String, TagSettings>,
  pub tracked_tags_paths: HashMap<String, VecDeque<Pos2>>,
  pub anchors: OptionalRef<Vec<AnchorPos>>,
  // Ограничения по отрисовке трека
  pub limit_tag_path: bool,
  pub limit_online: bool,
  pub current_limit: LimitDateTime,
  pub previous_limit: LimitDateTime,
  // Выбор страницы для показа
  pub menu: usize,
  // Тестовый граф путей
  pub path_traversal_graph: Vec<Vec<Pos2>>,
  pub path_to_add: (bool, Option<Vec<Pos2>>, bool),
  pub last_pos: Option<Pos2>,
}
