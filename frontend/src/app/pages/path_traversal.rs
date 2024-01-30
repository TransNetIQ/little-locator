use crate::app::utils::{scale, to_map, from_map, load_texture};
use crate::app::app_data::LittleLocatorApp;
use crate::utils::MResult;

use egui::{pos2, Pos2, vec2};

impl LittleLocatorApp {
  /// Отображает интерфейс, связанный с тестовыми путямиц.
  pub fn show_path_traversal(&mut self, ui: &mut egui::Ui) -> MResult<()> {
    if ui.button("Очистить граф возможных путей").clicked() {
      self.last_pos = None;
      self.path_to_add = (false, None, false);
      self.path_traversal_graph.clear();
    }
    egui::Frame::canvas(ui.style()).show(ui, |ui2| { let _ = self.paint_path_traversal(ui2); });
    Ok(())
  }
  
  /// Отрисовывает страницу с тестовыми путями.
  pub fn paint_path_traversal(&mut self, ui: &mut egui::Ui) -> MResult<egui::Response> {
    let (response, painter) = ui.allocate_painter(ui.available_size_before_wrap(), egui::Sense::click());

    let location_size = self.location_size.get_cloned()?;
    let icon_size = vec2(10.0, 10.0);
    let scale = scale(painter.clip_rect(), location_size);
    let mut shapes = vec![];
    
    // 1. Рисуем здание
    egui::Image::from_bytes("bytes://location_map", self.location_image.get_cloned()?)
      .tint(egui::Color32::WHITE)
      .fit_to_original_size(1f32)
      .paint_at(ui, painter.clip_rect());
    
    // 2. Рисуем существующие тестовые пути
    let tag_txr = load_texture(ui, "tag", &self.tag_image_bytes)?;
    self.draw_path_traversals(&painter, scale, icon_size, &tag_txr);
      
    // 3. Если первое нажатие было...
    if self.path_to_add.0 && self.path_to_add.1.as_ref().is_some_and(|vec| !vec.is_empty()) {
      let path_vec = self.path_to_add.1.as_mut().unwrap();
      
      // 3.1. Рисуем весь путь из точек
      if path_vec.len() >= 2 {
        let points: Vec<Pos2> = path_vec
          .iter()
          .map(|p| to_map(painter.clip_rect(), scale, *p, vec2(0f32, 0f32)))
          .collect();
        for point in &points {
          painter.image(
            tag_txr.id(),
            egui::Rect::from_min_max(*point - icon_size / 2f32, *point + icon_size / 2f32),
            egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            egui::Color32::WHITE,
          );
        }
        shapes.push(egui::Shape::line(points, egui::Stroke::new(2.0, egui::Color32::from_rgb(84, 49, 224))));
      }
      
      // 3.2. Рисуем пунктирную линию до указателя мыши
      if let Some(pointer_pos) = response.hover_pos() {
        let last_pt = path_vec.last().unwrap();
        let last_pt = to_map(painter.clip_rect(), scale, *last_pt, vec2(0f32, 0f32));
        painter.image(
          tag_txr.id(),
          egui::Rect::from_min_max(last_pt - icon_size / 2f32, last_pt + icon_size / 2f32),
          egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
          egui::Color32::WHITE,
        );
        
        // 3.2.1. Если мышку отвели на 10 пикселей, тогда начинаем рисовать новые точки
        if pointer_pos.distance(last_pt) > 10f32 {
          self.path_to_add.2 = true;
          shapes.extend(egui::Shape::dotted_line(
            &vec![
              last_pt,
              pointer_pos
            ],
            egui::Color32::from_rgb(141, 49, 224), 6.0, 1.0)
          );
        }
      }
      
      // 3.3. Если пользователь в этот момент нажимает на точку...
      if let Some(pointer_pos) = response.interact_pointer_pos() && response.clicked() {
        self.last_pos = Some(pointer_pos);
        
        let last_pt = to_map(painter.clip_rect(), scale, *path_vec.last().unwrap(), vec2(0f32, 0f32));
        if pointer_pos.distance(last_pt) > 10f32 || self.path_to_add.2 {
          
          // 3.3.1. Он нажал на одну из вершин графа
          let mut clicked_to_pt = false;
          let mut pv = path_vec.clone();
          for point in &pv {
            let point_mapped = to_map(painter.clip_rect(), scale, *point, vec2(0f32, 0f32));
            if point_mapped.distance(pointer_pos) < 10f32 {
              clicked_to_pt = true;
              if pv.len() > 1 {
                pv.push(*point);
                self.path_traversal_graph.push(pv.clone());
              }
              break;
            }
          }
          
          // 3.3.2. Если это была не вершина графа, то добавляем точку в список
          if !clicked_to_pt {
            path_vec.push(from_map(painter.clip_rect(), scale, pointer_pos));
          } else {
            self.path_to_add.0 = false;
            self.path_to_add.1 = None;
            self.path_to_add.2 = false;
          }
        }
      }
    }
    
    // 4. Если первого нажатия не было...
    else {
      // 4.1. Если пользователь нажал на карту
      if let Some(pointer_pos) = response.interact_pointer_pos() {
        if 
          self.last_pos.is_none() ||
          self.last_pos.is_some_and(|lp| lp.distance(pointer_pos) > 10f32)
        {
          self.path_to_add.0 = true;
          self.path_to_add.1 = Some(vec![from_map(painter.clip_rect(), scale, pointer_pos)]);
        }
      } else {
        self.last_pos = None;
      }
    }
    
    painter.extend(shapes);
    Ok(response)
  }
}
