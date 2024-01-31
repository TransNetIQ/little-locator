use crate::app::app_data::{LittleLocatorApp, ShowAnchorsDistOps, ShowTagOps};

impl LittleLocatorApp {
  /// Отображение списка тегов с возможностью выбрать опции для отрисовки.
  pub fn show_tags_list(&mut self, ui: &mut egui::Ui) {
    let mut keys = { self.tracked_tags_locations.keys().cloned().collect::<Vec<String>>() };
    keys.sort();
    for key in keys {
      ui.horizontal(|ui| {
        let tag = self.tracked_tags_locations.get_mut(&key).unwrap();

        ui.label(format!("{}", tag.locations.back().unwrap()));
        ui.label("Отобразить:");
        ui.checkbox(&mut tag.visible, "метку");
        if tag.visible {
          egui::ComboBox::from_id_source(2)
            .selected_text(format!("{}", match tag.visible_type {
              ShowTagOps::RealCoords => "реальные координаты",
              ShowTagOps::GraphSticked => "с учётом графа путей",
              ShowTagOps::Both => "оба варианта",
            }))
            .show_ui(ui, |ui| {
              ui.selectable_value(&mut tag.visible_type, ShowTagOps::RealCoords, "реальные координаты");
              ui.selectable_value(&mut tag.visible_type, ShowTagOps::GraphSticked, "с учётом графа путей");
              ui.selectable_value(&mut tag.visible_type, ShowTagOps::Both, "оба варианта");
            }
          );
        }
        ui.checkbox(&mut tag.show_path, "путь");
        ui.checkbox(&mut tag.show_anchor_distance, "расстояния до анкеров");
        if tag.show_anchor_distance {
          egui::ComboBox::from_id_source(3)
            .selected_text(format!("{}", match tag.anchor_distance_type {
              ShowAnchorsDistOps::RealDists => "по данным анкеров",
              ShowAnchorsDistOps::CoordsDists => "по координатам",
            }))
            .show_ui(ui, |ui| {
              ui.selectable_value(&mut tag.anchor_distance_type, ShowAnchorsDistOps::RealDists, "по данным анкеров");
              ui.selectable_value(&mut tag.anchor_distance_type, ShowAnchorsDistOps::CoordsDists, "по координатам");
            }
          );
        }
      });
    }
  }
}
