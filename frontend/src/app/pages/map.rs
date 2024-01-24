use egui::{Pos2, pos2, Vec2, vec2, Painter, TextureHandle};
use log::debug;

use crate::app::app_data::LittleLocatorApp;
use crate::app::utils::load_texture;
use crate::utils::{construct_dt, MResult, HOURS, MINUTES};

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
    let (response, painter) = ui.allocate_painter(ui.available_size_before_wrap(), egui::Sense::drag());

    // 1. Рисуем здание
    egui::Image::from_bytes("bytes://location_map", self.location_image.get_cloned()?)
      .tint(egui::Color32::WHITE)
      .fit_to_original_size(1f32)
      .paint_at(ui, painter.clip_rect());

    // 2. Подготавливаем отрисовку местоположений объектов: загружаем текстуры
    debug!("Загрузка текстур");
    let tag_txr = load_texture(ui, "tag", &self.tag_image_bytes)?;
    let anchor_txr = load_texture(ui, "anchor", &self.anchor_image_bytes)?;
    debug!("Текстуры загружены");

    // 3. Рассчитываем масштаб изображения на экране
    let location_size = self.location_size.get_cloned()?;
    let icon_size = vec2(20.0, 20.0);
    let scale = vec2(painter.clip_rect().width() / location_size.l, painter.clip_rect().height() / location_size.w);

    // 4. Отрисовываем анкера
    let _ = self.draw_anchors(&painter, scale, icon_size, anchor_txr);
    
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
      let tag = self.tracked_tags_locations.get_mut(&key).unwrap();

      if tag.1 { // 6.1. Если сказано отображать метку
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
        
        // 6.2. А если ещё и стоит отметка "Показывать расстояние от метки до анкеров"
        if tag.4 {
          let anchors_pos_list = self.anchors.ref_cx(|anchors| {
            let mut anchors_pos_list = vec![];
            for anchor in anchors { anchors_pos_list.push(pos2(anchor.x, anchor.y)) }
            anchors_pos_list
          })?;
          
          let tag_center_pos = pos2(
            icon_position_scaled.x + icon_size.x / 2f32,
            icon_position_scaled.y + icon_size.y / 2f32
          );
          
          for anchor_pos in anchors_pos_list {
            let anchor_center_pos = pos2(
              painter.clip_rect().left() + anchor_pos.x * scale.x,
              painter.clip_rect().top() + anchor_pos.y * scale.y
            );
            
            shapes.extend(egui::Shape::dashed_line(
              &vec![tag_center_pos, anchor_center_pos],
              egui::Stroke::new(1.0, egui::Color32::from_rgb(25, 200, 100)),
              6.0,
              2.0
            ));
            
            let text_position = (tag_center_pos + anchor_center_pos.to_vec2()) / 2f32;
            let dist = pos2(last_tag_position.x, last_tag_position.y).distance(anchor_pos);
            
            painter.text(
              text_position,
              egui::Align2::CENTER_CENTER,
              format!("Расстояние: {}", dist),
              egui::FontId::default(),
              egui::Color32::from_rgb(25, 200, 100)
            );
          }
        }
      }

      if tag.2 { // 6.3. Если сказано отображать путь метки
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
  
  /// Отрисовывает анкера на карте
  pub fn draw_anchors(
    &mut self,
    painter: &Painter,
    scale: Vec2,
    icon_size: Vec2,
    txr: TextureHandle
  ) -> MResult<()> {
    self.anchors.ref_cx(|anchors| { for anchor in anchors {
      // 1. Рассчитываем позицию для показа иконки
      let icon_position_scaled = pos2(
        painter.clip_rect().left() + anchor.x * scale.x - icon_size.x / 2f32,
        painter.clip_rect().top() + anchor.y * scale.y - icon_size.y / 2f32
      );
      
      // 2. Рисуем иконку на карте
      painter.image(
        txr.id(),
        egui::Rect::from_min_max(icon_position_scaled, icon_position_scaled + icon_size),
        egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
        egui::Color32::WHITE,
      );
      
      // 3. Рассчитываем позицию текста с ID анкера
      let text_position = icon_position_scaled + icon_size / 2f32 + vec2(0f32, icon_size.y);
      
      // 4. Отображаем ID анкера
      painter.text(
        text_position,
        egui::Align2::CENTER_CENTER,
        anchor.id.clone(),
        egui::FontId::monospace(12.0),
        egui::Color32::BLACK
      );
    }})
  }
}
