use crate::utils::LimitDateTime;

use egui::Pos2;
use ll_data::{Location, MapSizes, AnchorPos};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, atomic::AtomicBool};

/// Структура приложения.
pub struct LittleLocatorApp {
  // Домен веб-приложения
  pub _server_origin: String,
  // Поля страницы выбора карты
  pub l_input: String,
  pub w_input: String,
  pub done: Arc<AtomicBool>,
  // Данные картинок
  pub tag_image_bytes: Arc<Mutex<Option<Vec<u8>>>>,
  pub anchor_image_bytes: Arc<Mutex<Option<Vec<u8>>>>,
  // Данные о местоположениях и карте
  pub location_image: Arc<Mutex<Option<Vec<u8>>>>,
  pub location_size: Arc<Mutex<Option<MapSizes>>>,
  pub _data_sender: ewebsock::WsSender,
  pub data_receiver: ewebsock::WsReceiver,
  pub tracked_tags_locations: HashMap<String, (VecDeque<Location>, bool, bool, usize)>,
  pub tracked_tags_paths: HashMap<String, VecDeque<Pos2>>,
  pub anchors: Arc<Mutex<Option<Vec<AnchorPos>>>>,
  // Ограничения по отрисовке трека
  pub limit_tag_path: bool,
  pub limit_online: bool,
  pub current_limit: LimitDateTime,
  pub previous_limit: LimitDateTime,
  // Показать список меток или карту
  pub show_only_tags_list: bool,
}
