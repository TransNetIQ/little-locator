//! Приложение для отрисовки местоположений.

pub mod app_data;
pub mod constructor;

mod image_selector;
mod path_drawer;
mod utils;

use crate::app::app_data::LittleLocatorApp;
use crate::app::image_selector::select_image_dialog;
use crate::app::utils::load_image_from_memory;
use crate::utils::{HOURS, MINUTES, construct_dt, MResult};

use egui::{Pos2, pos2, vec2};
use ewebsock::{WsEvent, WsMessage};
use ll_data::{Location, MAX_QUEUE_LEN, MapSizes};
use log::debug;
use std::collections::VecDeque;
use std::sync::{Arc, atomic::Ordering as AtomicOrdering};

impl eframe::App for LittleLocatorApp {
  /// Отрисовывает приложение.
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    ctx.request_repaint_after(std::time::Duration::from_millis(200));
    egui::CentralPanel::default().show(ctx, |ui| {
      if !self.done.load(AtomicOrdering::Relaxed) { let _ = self.show_map_selection_page(ui); }
      else { let _ = self.show_map_page(ui); }
    });
  }
}

impl LittleLocatorApp {
  /// Показывает страницу выбора карты.
  pub fn show_map_selection_page(&mut self, ui: &mut egui::Ui) -> MResult<()> {
    ui.horizontal(|ui| {
      ui.label("Длина здания:");
      ui.text_edit_singleline(&mut self.l_input);
    });
    ui.horizontal(|ui| {
      ui.label("Ширина здания:");
      ui.text_edit_singleline(&mut self.w_input);
    });

    let map_selection_text = {
      let map_image = self.location_image.clone();
      if map_image.is_some() { "Карта выбрана!" }
      else { "Выбор карты..." }
    };
    if ui.button(map_selection_text).clicked() { select_image_dialog(self.location_image.clone())?; }

    if ui.button("Готово").clicked() {
      let l = self.l_input.parse::<f32>();
      let w = self.w_input.parse::<f32>();
      if l.is_err() || w.is_err() { return Err("Не удалось распарсить значения длины и ширины.".into()) }
      self.location_size.set(MapSizes { l: l.unwrap(), w: w.unwrap() })?;
      self.done.store(true, AtomicOrdering::Relaxed);
    }
    Ok(())
  }
  
  /// Показывает страницу с картой.
  pub fn show_map_page(&mut self, ui: &mut egui::Ui) -> MResult<()> {
    ui.horizontal(|ui| {
      if ui.checkbox(&mut self.limit_tag_path, "Ограничить путь метки по времени").clicked() {
        self.limit_online = false;
      };
      ui.add(egui_extras::DatePickerButton::new(&mut self.current_limit.0));
      ui.label("С");
      egui::ComboBox::from_label("часов").show_index(ui, &mut self.current_limit.1, 24usize, |i| HOURS[i]);
      egui::ComboBox::from_label("минут").show_index(ui, &mut self.current_limit.2, 60usize, |i| MINUTES[i]);
    });

    // Обработаем входящие местоположения
    while let Some(event) = self.data_receiver.try_recv() {
      let message = match event {
        WsEvent::Message(message) => message,
        _ => continue,
      };
      let location_json = match message {
        WsMessage::Text(location_json) => location_json,
        _ => continue,
      };
      if let Ok(new_location) = serde_json::from_str::<Location>(&location_json) {
        if !self.tracked_tags_locations.contains_key(&new_location.id) {
          let mut new_vecdeque = VecDeque::new();
          let new_location_id = new_location.id.clone();
          new_vecdeque.push_back(new_location);
          self.tracked_tags_locations.insert(new_location_id, (new_vecdeque, true, false, 1usize));
        } else {
          let locations = self.tracked_tags_locations.get_mut(&new_location.id).unwrap();
          if locations.0.len() > MAX_QUEUE_LEN { locations.0.pop_front(); }
          locations.0.push_back(new_location);
          locations.3 += 1usize;
        };
      }
    }

    // Покажем основной интерфейс приложения
    ui.horizontal(|ui| {
      ui.checkbox(&mut self.show_only_tags_list, "Показать метки");
      ui.checkbox(&mut self.show_distance_between_tags_and_anchors, "Показывать расстояние от меток до анкеров");
    });
    if self.show_only_tags_list {
      let mut keys = { self.tracked_tags_locations.keys().cloned().collect::<Vec<String>>() };
      keys.sort();
      for key in keys {
        ui.horizontal(|ui| {
          let tag = self.tracked_tags_locations.get_mut(&key).unwrap();

          ui.label(format!("{}", tag.0.back().unwrap()));
          ui.checkbox(&mut tag.1, "Отобразить метку");
          ui.checkbox(&mut tag.2, "Показать путь");
        });
      }
    } else {
      egui::Frame::canvas(ui.style()).show(ui, |ui2| { let _ = self.paint_location(ui2); });
    }
    
    Ok(())
  }
  
  /// Отображает карту здания и текущие местоположения объектов.
  pub fn paint_location(&mut self, ui: &mut egui::Ui) -> MResult<egui::Response> {
    let (response, painter) = ui.allocate_painter(ui.available_size_before_wrap(), egui::Sense::drag());

    // Рисуем здание
    egui::Image::from_bytes("bytes://location_map", self.location_image.get_cloned()?)
      .tint(egui::Color32::WHITE)
      .fit_to_original_size(1f32)
      .paint_at(ui, painter.clip_rect());

    // Подготавливаем отрисовку местоположений объектов: загружаем текстуры
    let tag_txr = ui.ctx().load_texture(
      "tag",
      egui::ImageData::Color(Arc::new(self.tag_image_bytes.ref_cx(|val| load_image_from_memory(val))??)),
      Default::default(),
    );
    
    let anchor_txr = ui.ctx().load_texture(
      "anchor",
      egui::ImageData::Color(Arc::new(self.anchor_image_bytes.ref_cx(|val| load_image_from_memory(val))??)),
      Default::default(),
    );

    let location_size = self.location_size.get_cloned()?;
    let icon_size = vec2(20.0, 20.0);

    let scale = vec2(painter.clip_rect().width() / location_size.l, painter.clip_rect().height() / location_size.w);

    // Отрисовка анкеров
    self.anchors.ref_cx(|anchors| {
      for anchor in anchors {
        let icon_position_scaled = pos2(
          painter.clip_rect().left() + anchor.x * scale.x - icon_size.x / 2f32,
          painter.clip_rect().top() + anchor.y * scale.y - icon_size.y / 2f32
        );
        
        painter.image(
          anchor_txr.id(),
          egui::Rect::from_min_max(icon_position_scaled, icon_position_scaled + icon_size),
          egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
          egui::Color32::WHITE,
        );
        
        let text_position = icon_position_scaled + icon_size / 2f32 + vec2(0f32, icon_size.y);
        
        painter.text(
          text_position,
          egui::Align2::CENTER_CENTER,
          anchor.id.clone(),
          egui::FontId::default(),
          egui::Color32::BLACK
        );
      }
    })?;
    
    // Обновляем значения лимитов времени
    if self.previous_limit != self.current_limit {
      self.previous_limit = self.current_limit;
      self.limit_online = false; // Переменная, которая отвечает за перерисовку путей
      debug!("Needed to redraw with time = {}", construct_dt(&self.current_limit));
    }
    let limit_time = construct_dt(&self.current_limit).timestamp_millis();
    
    let keys = self.tracked_tags_locations.keys().cloned().collect::<Vec<String>>();
    let mut shapes = Vec::new();
    for key in keys {
      let tag = self.tracked_tags_locations.get_mut(&key).unwrap();

      if tag.1 { // Если сказано отображать метку
        let last_tag_position = tag.0.back().unwrap();

        let icon_position_scaled = pos2(
          painter.clip_rect().left() + last_tag_position.x * scale.x - icon_size.x / 2f32,
          painter.clip_rect().top() + last_tag_position.y * scale.y - icon_size.y / 2f32
        );

        painter.image(
          tag_txr.id(),
          egui::Rect::from_min_max(icon_position_scaled, icon_position_scaled + icon_size),
          egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
          egui::Color32::WHITE,
        );
        
        let text_position = icon_position_scaled + icon_size / 2f32 + vec2(0f32, icon_size.y);
        
        painter.text(
          text_position,
          egui::Align2::CENTER_CENTER,
          last_tag_position.id.clone(),
          egui::FontId::default(),
          egui::Color32::BLACK
        );
      }

      if tag.2 { // Если сказано отображать путь метки
        self.redraw_tag_path_with_time_limit(&key, limit_time);
        
        let current_tag_line = self.tracked_tags_paths.get(&key).unwrap();
        if current_tag_line.len() >= 2 {
          let points: Vec<Pos2> = current_tag_line
            .iter()
            .map(|p| pos2(
              painter.clip_rect().left() + p.x * scale.x,
              painter.clip_rect().top() + p.y * scale.y
            ))
            .collect();
          shapes.push(egui::Shape::line(points, egui::Stroke::new(2.0, egui::Color32::from_rgb(25, 200, 100))));
        }
      }
    }
    
    self.limit_online = true;
    
    painter.extend(shapes);

    Ok(response)
  }
}
