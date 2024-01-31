use crate::app::app_data::LittleLocatorApp;
use crate::app::utils::to_map;
use crate::utils::{MResult, PositionExtractable};

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
        painter.clip_rect().left() + anchor.1.x * scale.x - icon_size.x / 2f32,
        painter.clip_rect().top() + anchor.1.y * scale.y - icon_size.y / 2f32
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
        anchor.1.id.clone(),
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
  
  
  /// Отрисовывает ближайшую к метке точку графа на карте.
  pub fn draw_nearest_graph_point(
    &self,
    painter: &Painter,
    txr: &TextureHandle,
    scale: Vec2,
    icon_size: Vec2,
    tag: &Location,
  ) {
    // Ищем ближайшую точку на графе.
    let mut nearest_dist = f32::INFINITY;
    let mut nearest_pt = pos2(0.0, 0.0);
    let (tx, ty) = (tag.x, tag.y);
    
    for subgraph in &self.path_traversal_graph {
      let mut nearest_pt_dist = f32::INFINITY;
      let mut pt_id = 0usize;
      
      // Ищем ближайшую вершину подграфа.
      for (index, point) in subgraph.iter().enumerate() {
        let dist = tag.extract().distance(*point);
        if dist < nearest_pt_dist {
          nearest_pt_dist = dist;
          pt_id = index;
        }
      }
      
      // Ищем два прилежащих ребра.
      let mut line_segments = vec![];
      if pt_id > 0usize {
        line_segments.push((subgraph[pt_id], subgraph[pt_id - 1]));
      }
      if pt_id < subgraph.len() - 2 {
        line_segments.push((subgraph[pt_id], subgraph[pt_id + 1]));
      }
      
      // Для каждого из рёбер:
      for segment in line_segments {
        let left = pos2(segment.0.x, segment.0.y);
        let right = pos2(segment.1.x, segment.1.y);
        
        // 1. Определяем, какой у нас треугольник
        let left_dist = left.distance(pos2(tx, ty));
        let right_dist = right.distance(pos2(tx, ty));
        let segment_dist = left.distance(right);
        
        let nearest_subgraph_pt: Pos2;
        
        // 1.1. Если треугольник тупоугольный...
        if f32::powi(left_dist, 2) >= f32::powi(right_dist, 2) + f32::powi(segment_dist, 2) {
          nearest_subgraph_pt = right; // Если левое ребро больше, то правая ближе
        } else if f32::powi(right_dist, 2) >= f32::powi(left_dist, 2) + f32::powi(segment_dist, 2) {
          nearest_subgraph_pt = left; // Если правое ребро больше, то левая вершина ближе
        }
        
        // 1.2. Иначе...
        else {
          let ac = vec2(tx - left.x, ty - left.y);
          let ab = vec2(right.x - left.x, right.y - left.y);
          nearest_subgraph_pt = left + ab * ac.dot(ab) / ab.dot(ab);
        }
        
        // 2. Проверяем, ближайшая ли это к нам точка
        let nearest_subgraph_dist = nearest_subgraph_pt.distance(tag.extract());
        if nearest_subgraph_dist < nearest_dist {
          nearest_dist = nearest_subgraph_dist;
          nearest_pt = nearest_subgraph_pt;
        }
      }
    }
    
    let check_msr = |max_sticking_radius: &f32| {
      // Если радиус слишком большой, то привязка местоположения к графу не осуществляется.
      if tag.extract().distance(nearest_pt) > *max_sticking_radius {
        return Some(tag.extract());
      }
      None
    };
    
    if let Ok(Some(pos)) = self.max_sticking_radius.ref_cx(check_msr) { nearest_pt = pos; }
    
    let icon_position_scaled = to_map(painter.clip_rect(), scale, nearest_pt, icon_size);
    let text_position = icon_position_scaled + icon_size / 2f32 + vec2(0f32, icon_size.y);
    
    painter.image(
      txr.id(),
      egui::Rect::from_min_max(icon_position_scaled, icon_position_scaled + icon_size),
      egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
      egui::Color32::WHITE,
    );
    
    painter.text(
      text_position,
      egui::Align2::CENTER_CENTER,
      tag.id.clone(),
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
      for anchor in anchors { anchors_pos_list.push(pos2(anchor.1.x, anchor.1.y)) }
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
        format!("Расс.: {:.2}", dist),
        egui::FontId::default(),
        egui::Color32::from_rgb(25, 200, 100)
      );
    }
    
    painter.extend(shapes);
    
    Ok(())
  }
  
  /// Отрисовывает вычисляемые пути до анкеров.
  pub fn draw_real_tag_distances(
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
      for anchor in anchors { anchors_pos_list.push((anchor.0.clone(), pos2(anchor.1.x, anchor.1.y))) }
      anchors_pos_list
    })?;
    
    let tag_center_pos = pos2(
      icon_position_scaled.x + icon_size.x / 2f32,
      icon_position_scaled.y + icon_size.y / 2f32
    );
    
    for anchor_pos in anchors_pos_list {
      let anchor_center_pos = pos2(
        painter.clip_rect().left() + anchor_pos.1.x * scale.x,
        painter.clip_rect().top() + anchor_pos.1.y * scale.y
      );
      
      shapes.extend(egui::Shape::dashed_line(
        &vec![tag_center_pos, anchor_center_pos],
        egui::Stroke::new(1.0, egui::Color32::from_rgb(25, 200, 100)),
        6.0,
        2.0
      ));
      
      if tag.dist.is_some() {
        let text_position = (tag_center_pos + anchor_center_pos.to_vec2()) / 2f32;
        
        let dists_ref = tag.dist.as_ref().unwrap();
        let mut real_dist = None;
        for dist in dists_ref {
          if anchor_pos.0.eq(&dist.aid) {
            real_dist = Some(dist.dist);
            break;
          }
        }
        
        if let Some(real_dist) = real_dist {
          painter.text(
            text_position,
            egui::Align2::CENTER_CENTER,
            format!("Расс.: {:.2}", real_dist),
            egui::FontId::default(),
            egui::Color32::from_rgb(25, 200, 100)
          );
        }
      }
    }
    
    painter.extend(shapes);
    
    Ok(())
  }
}
