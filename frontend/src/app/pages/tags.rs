use crate::app::app_data::LittleLocatorApp;

impl LittleLocatorApp {
  /// Отображение списка тегов с возможностью выбрать опции для отрисовки.
  pub fn show_tags_list(&mut self, ui: &mut egui::Ui) {
    let mut keys = { self.tracked_tags_locations.keys().cloned().collect::<Vec<String>>() };
    keys.sort();
    for key in keys {
      ui.horizontal(|ui| {
        let tag = self.tracked_tags_locations.get_mut(&key).unwrap();

        ui.label(format!("{}", tag.locations.back().unwrap()));
        ui.checkbox(&mut tag.visible, "Отобразить метку");
        ui.checkbox(&mut tag.show_path, "Показать путь");
        ui.checkbox(&mut tag.show_anchor_calculated_distance, "Показывать расстояния до анкеров");
        ui.checkbox(&mut tag.show_anchor_real_distance, "Расстояния передаются метками")
      });
    }
  }
}
