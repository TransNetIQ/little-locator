use crate::app::app_data::LittleLocatorApp;
use crate::app::utils::to_map;
use crate::utils::MResult;

use egui::{pos2, vec2, Painter, TextureHandle, Pos2, Vec2};
use ll_data::Location;

impl LittleLocatorApp {
  /// Отрисовывает анкера на карте.
  pub fn draw_anchors(
    &self,
    painter: &Painter,
    scale: Vec2,
    icon_size: Vec2,
    txr: &TextureHandle
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
  
  /// Отрисовывает примерные пути на карте.
  pub fn draw_path_traversals(
    &self,
    painter: &Painter,
    scale: Vec2,
    icon_size: Vec2,
    txr: &TextureHandle
  ) {
    let mut shapes = vec![];
    for path in &self.path_traversal_graph {
      let points: Vec<Pos2> = path
        .iter()
        .map(|p| to_map(painter.clip_rect(), scale, *p, vec2(0f32, 0f32)))
        .collect();
        for point in &points {
          painter.image(
            txr.id(),
            egui::Rect::from_min_max(*point - icon_size / 2f32, *point + icon_size / 2f32),
            egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            egui::Color32::WHITE,
          );
        }
      shapes.push(egui::Shape::line(points, egui::Stroke::new(2.0, egui::Color32::from_rgb(224, 49, 166))));
    }
    painter.extend(shapes);
  }
  
  /// Отрисовывает точку на карте.
  pub fn draw_tag_point(
    painter: &Painter,
    txr: &TextureHandle,
    icon_position_scaled: Pos2,
    icon_size: Vec2,
    tag_id: String,
  ) {
    painter.image(
      txr.id(),
      egui::Rect::from_min_max(icon_position_scaled, icon_position_scaled + icon_size),
      egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
      egui::Color32::WHITE,
    );
    
    let text_position = icon_position_scaled + icon_size / 2f32 + vec2(0f32, icon_size.y);
    
    painter.text(
      text_position,
      egui::Align2::CENTER_CENTER,
      tag_id,
      egui::FontId::default(),
      egui::Color32::BLACK
    );
  }
  
  /// Отрисовывает вычисляемые пути до анкеров.
  pub fn draw_calculated_tag_distances(
    &self,
    painter: &Painter,
    tag: &Location,
    icon_position_scaled: Pos2,
    icon_size: Vec2,
    scale: Vec2,
  ) -> MResult<()> {
    let mut shapes = vec![];
    
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
      let dist = pos2(tag.x, tag.y).distance(anchor_pos);
      
      painter.text(
        text_position,
        egui::Align2::CENTER_CENTER,
        format!("Расстояние: {}", dist),
        egui::FontId::default(),
        egui::Color32::from_rgb(25, 200, 100)
      );
    }
    
    painter.extend(shapes);
    
    Ok(())
  }
}
