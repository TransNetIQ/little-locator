use egui::{Pos2, pos2, vec2};
use log::debug;

use crate::app::app_data::LittleLocatorApp;
use crate::app::utils::{load_texture, scale, to_map};
use crate::utils::{construct_dt, MResult, HOURS, MINUTES, Ignore, PositionExtractable};

impl LittleLocatorApp {
  /// Отображает интерфейс, связанный с картой и взаимодействием с ней.
  pub fn show_map(&mut self, ui: &mut egui::Ui) -> MResult<()> {
    // 1. Показываем заголовок с опциями
    ui.horizontal(|ui| {
      if ui.checkbox(&mut self.limit_tag_path, "Ограничить путь метки по времени").clicked() {
        self.limit_online = false;
      };
      ui.add(egui_extras::DatePickerButton::new(&mut self.current_limit.0));
      ui.label("С");
      egui::ComboBox::from_label("часов").show_index(ui, &mut self.current_limit.1, 24usize, |i| HOURS[i]);
      egui::ComboBox::from_label("минут").show_index(ui, &mut self.current_limit.2, 60usize, |i| MINUTES[i]);
    });
    egui::Frame::canvas(ui.style()).show(ui, |ui2| { let _ = self.paint_map(ui2); });
    Ok(())
  }
  
  /// Отображает карту здания и текущие местоположения объектов.
  pub fn paint_map(&mut self, ui: &mut egui::Ui) -> MResult<egui::Response> {
    let (response, painter) = ui.allocate_painter(ui.available_size_before_wrap(), egui::Sense::focusable_noninteractive());

    // 1. Рисуем здание
    egui::Image::from_bytes("bytes://location_map", self.location_image.get_cloned()?)
      .tint(egui::Color32::WHITE)
      .fit_to_original_size(1f32)
      .paint_at(ui, painter.clip_rect());

    // 2. Подготавливаем отрисовку местоположений объектов: загружаем текстуры
    let tag_txr = load_texture(ui, "tag", &self.tag_image_bytes)?;
    let anchor_txr = load_texture(ui, "anchor", &self.anchor_image_bytes)?;

    // 3. Рассчитываем масштаб изображения на экране
    let location_size = self.location_size.get_cloned()?;
    let icon_size = vec2(20.0, 20.0);
    let path_icon_size = vec2(10.0, 10.0);
    let scale = scale(painter.clip_rect(), location_size);

    // 4. Отрисовываем анкера и примерные пути
    self.draw_anchors(&painter, scale, icon_size, &anchor_txr).ignore();
    self.draw_path_traversals(&painter, scale, path_icon_size, &tag_txr);
    
    // 5. Обновляем значения лимитов времени
    if self.previous_limit != self.current_limit {
      self.previous_limit = self.current_limit;
      self.limit_online = false; // Переменная, которая отвечает за перерисовку путей
      debug!("Needed to redraw with time = {}", construct_dt(&self.current_limit));
    }
    let limit_time = construct_dt(&self.current_limit).timestamp_millis();
    
    // 6. Отрисовываем сами метки и всё, что к ним относится
    let keys = self.tracked_tags_locations.keys().cloned().collect::<Vec<String>>();
    let mut shapes = Vec::new();
    for key in keys {
      let tag = self.tracked_tags_locations.get(&key).unwrap();

      if tag.visible { // 6.1. Если сказано отображать метку
        let last_tag_position = tag.locations.back().unwrap();
        let icon_position_scaled = to_map(painter.clip_rect(), scale, last_tag_position.extract(), icon_size);
        LittleLocatorApp::draw_tag_point(&painter, &tag_txr, icon_position_scaled, icon_size, last_tag_position.id.clone());
        
        // 6.2. А если ещё и стоит отметка "Показывать расстояние от метки до анкеров"
        if tag.show_anchor_calculated_distance {
          self.draw_calculated_tag_distances(&painter, &last_tag_position, icon_position_scaled, icon_size, scale)?;
        }
      }

      if tag.show_path { // 6.3. Если сказано отображать путь метки
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
