//! Приложение для отрисовки местоположений.

pub mod app_data;
pub mod constructor;
pub mod utils;

use crate::app::app_data::LittleLocatorApp;
use crate::app::utils::load_image_from_memory;
use crate::utils::{HOURS, MINUTES, construct_datetime_utc};

use chrono::{DateTime, Utc};
use egui::{Pos2, pos2, vec2};
use ewebsock::{WsEvent, WsMessage};
use ll_data::Location;
use std::sync::Arc;

#[cfg(target_arch = "wasm32")]
use rfd::AsyncFileDialog;
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;

impl eframe::App for LittleLocatorApp {
  /// Отрисовывает приложение.
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    ctx.request_repaint_after(std::time::Duration::from_millis(100));
    egui::CentralPanel::default().show(ctx, |ui| {
      if !self.done { self.show_map_selection_page(ui); }
      else { self.show_map_page(ui); }
    });
  }
}

impl LittleLocatorApp {
  /// Показывает страницу выбора карты.
  pub fn show_map_selection_page(&mut self, ui: &mut egui::Ui) {
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
      if map_image.lock().unwrap().is_some() {
        "Карта выбрана!"
      } else {
        "Выбор карты..."
      }
    };
    if ui.button(map_selection_text).clicked() {
      let map_image = self.location_image.clone();
      #[cfg(target_arch = "wasm32")] {
        wasm_bindgen_futures::spawn_local(async move {
          let file = AsyncFileDialog::new()
            .add_filter("image", &["png", "jpg"])
            .set_directory("/")
            .pick_file()
            .await;
          let data = file.unwrap().read().await;
          *map_image.lock().unwrap() = Some(data.clone());
        });
      }
      #[cfg(not(target_arch = "wasm32"))] {
        let file = FileDialog::new()
          .add_filter("image", &["png", "jpg"])
          .set_directory("/")
          .pick_file();
        let data = std::fs::read(file.unwrap()).unwrap();
        *map_image.lock().unwrap() = Some(data);
      }
    }

    if ui.button("Готово").clicked() {
      let l = self.l_input.parse::<f32>();
      let w = self.w_input.parse::<f32>();
      if l.is_err() || w.is_err() { return }
      self.location_size = Some([l.unwrap(), w.unwrap()]);
      self.done = true;
    }
  }
  
  /// Показывает страницу с картой.
  pub fn show_map_page(&mut self, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
      ui.checkbox(&mut self.limit_tag_path, "Ограничить путь метки по времени");
      ui.add(egui_extras::DatePickerButton::new(&mut self.current_limit.0));
      ui.label("С");
      egui::ComboBox::from_label("часов").show_index(ui, &mut self.current_limit.1, 24usize, |i| HOURS[i]);
      egui::ComboBox::from_label("минут").show_index(ui, &mut self.current_limit.2, 60usize, |i| MINUTES[i]);
    });
    
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
          self.tracked_tags_locations.insert(new_location.id.clone(), (vec![new_location], true, false, 0usize));
        } else {
          self.tracked_tags_locations.get_mut(&new_location.id).unwrap().0.push(new_location);
        };
      }
    }

    let keys = { self.tracked_tags_locations.keys().cloned().collect::<Vec<String>>() };
    for key in keys {
      ui.horizontal(|ui| {
        let tag = self.tracked_tags_locations.get_mut(&key).unwrap();

        ui.label(format!("{}", tag.0.last().unwrap()));
        ui.checkbox(&mut tag.1, "Отобразить метку");
        ui.checkbox(&mut tag.2, "Показать путь");
      });
    }
    egui::Frame::canvas(ui.style()).show(ui, |ui2| { self.paint_location(ui2); });
  }
  
  /// Отображает карту здания и текущее местоположение объектов.
  pub fn paint_location(&mut self, ui: &mut egui::Ui) -> egui::Response {
    let (response, painter) = ui.allocate_painter(ui.available_size_before_wrap(), egui::Sense::drag());

    // Рисуем здание
    let loc_img = self.location_image.lock().unwrap();
    egui::Image::from_bytes("bytes://location_map", loc_img.as_ref().unwrap().clone())
      .tint(egui::Color32::WHITE)
      .fit_to_original_size(1f32)
      .paint_at(ui, painter.clip_rect());

    // Рисуем местоположения объектов
    let pos_img = self.position_image_bytes.lock().unwrap();
    let tag_txr = ui.ctx().load_texture(
      "tag",
      egui::ImageData::Color(Arc::new(load_image_from_memory(pos_img.as_ref().unwrap()).unwrap())),
      Default::default(),
    );

    let location_size = self.location_size.as_ref().unwrap();
    let icon_size = vec2(20.0, 20.0);

    let scale = vec2(painter.clip_rect().width() / location_size[0], painter.clip_rect().height() / location_size[1]);

    let keys = { self.tracked_tags_locations.keys().cloned().collect::<Vec<String>>() };
    let mut shapes = Vec::new();
    for key in keys {
      let tag = self.tracked_tags_locations.get_mut(&key).unwrap();

      if tag.1 { // Если сказано отображать метку
        let last_tag_position = tag.0.last().unwrap();

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

        // Получаем вектор позиций, которые нам необходимо отрисовать
        let current_tag_line = match self.tracked_tags_paths.get_mut(&key) {
          Some(line) => line,
          None => {
            self.tracked_tags_paths.insert(key.clone(), Vec::new());
            self.tracked_tags_paths.get_mut(&key).unwrap()
          }
        };
        
        // Если сменились настройки времени, - перерисовываем путь меток
        if self.limit_tag_path && (self.previous_limit != self.current_limit || !self.limited) {
          self.limited = true;
          let limit_time: DateTime<Utc> = construct_datetime_utc(&self.current_limit);
          current_tag_line.clear();
          tag.3 = 0;
          for pos in &tag.0 {
            if pos.ts >= limit_time { current_tag_line.push(pos2(pos.x, pos.y)); }
            tag.3 += 1;
          }
        } else if !self.limit_tag_path {
          self.limited = false;
          current_tag_line.clear();
          tag.3 = 0;
        }
        
        // Дополняем путь меток, если в нём чего-то нет
        let index = &mut tag.3;
        while *index < tag.0.len() {
          current_tag_line.push(pos2(tag.0[*index].x, tag.0[*index].y));
          *index += 1;
        }

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
    if self.previous_limit != self.current_limit {
      self.previous_limit = self.current_limit;
    }
    
    painter.extend(shapes);

    response
  }
}
