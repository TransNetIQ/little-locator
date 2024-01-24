use crate::app::app_data::LittleLocatorApp;
use crate::utils::MResult;

impl LittleLocatorApp {
  /// Отображает интерфейс, связанный с тестовыми путямиц.
  pub fn show_path_traversal(&mut self, ui: &mut egui::Ui) -> MResult<()> {
    egui::Frame::canvas(ui.style()).show(ui, |ui2| { let _ = self.paint_path_traversal(ui2); });
    Ok(())
  }
  
  /// Отрисовывает страницу с тестовыми путями.
  pub fn paint_path_traversal(&mut self, ui: &mut egui::Ui) -> MResult<egui::Response> {
    let (response, painter) = ui.allocate_painter(ui.available_size_before_wrap(), egui::Sense::drag());

    // 1. Рисуем здание
    egui::Image::from_bytes("bytes://location_map", self.location_image.get_cloned()?)
      .tint(egui::Color32::WHITE)
      .fit_to_original_size(1f32)
      .paint_at(ui, painter.clip_rect());
    
    Ok(response)
  }
}
