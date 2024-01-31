use crate::app::app_data::LittleLocatorApp;

pub const TAG_SHOW_MENU: [&str; 3] = [
  "Реальные координаты",
  "С учётом графа путей",
  "Оба варианта"
];

pub const ANCHOR_DISTS_SHOW_MENU: [&str; 2] = [
  "по данным анкеров",
  "по координатам"
];

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
          egui::ComboBox::from_label("").show_index(ui, &mut tag.visible_type, 3usize, |i| TAG_SHOW_MENU[i]);
        }
        ui.checkbox(&mut tag.show_path, "путь");
        ui.checkbox(&mut tag.show_path, "расстояния до анкеров");
        if tag.show_path {
          egui::ComboBox::from_label("").show_index(ui, &mut tag.anchor_distance_type, 2usize, |i| ANCHOR_DISTS_SHOW_MENU[i]);
        }
      });
    }
  }
}
