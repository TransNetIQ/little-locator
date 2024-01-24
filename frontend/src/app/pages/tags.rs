use crate::app::app_data::LittleLocatorApp;

impl LittleLocatorApp {
  /// Отображение списка тегов с возможностью выбрать опции для отрисовки.
  pub fn show_tags_list(&mut self, ui: &mut egui::Ui) {
    let mut keys = { self.tracked_tags_locations.keys().cloned().collect::<Vec<String>>() };
    keys.sort();
    for key in keys {
      ui.horizontal(|ui| {
        let tag = self.tracked_tags_locations.get_mut(&key).unwrap();

        ui.label(format!("{}", tag.0.back().unwrap()));
        ui.checkbox(&mut tag.1, "Отобразить метку");
        ui.checkbox(&mut tag.2, "Показать путь");
        ui.checkbox(&mut tag.4, "Показывать расстояние до анкеров");
      });
    }
  }
}
