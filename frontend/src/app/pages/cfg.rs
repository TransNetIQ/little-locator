use ll_data::MapSizes;
use std::sync::atomic::Ordering as AtomicOrdering;

use crate::app::app_data::LittleLocatorApp;
use crate::app::image_selector::select_image_dialog;
use crate::utils::MResult;

impl LittleLocatorApp {
  /// Показывает страницу выбора карты.
  pub fn show_cfg_page(&mut self, ui: &mut egui::Ui) -> MResult<()> {
    
    // 1. Необходимо указать размеры помещения
    ui.horizontal(|ui| {
      ui.label("Длина здания:");
      ui.text_edit_singleline(&mut self.l_input);
    });
    ui.horizontal(|ui| {
      ui.label("Ширина здания:");
      ui.text_edit_singleline(&mut self.w_input);
    });

    // 2. Необходимо выбрать карту помещения
    let map_selection_text = {
      let map_image = self.location_image.clone();
      if map_image.is_some() { "Карта выбрана!" }
      else { "Выбор карты..." }
    };
    if ui.button(map_selection_text).clicked() { select_image_dialog(self.location_image.clone())?; }

    // 3. По кнопке "Готово" записывается конфиг и загружается карта
    if ui.button("Готово").clicked() {
      let l = self.l_input.parse::<f32>();
      let w = self.w_input.parse::<f32>();
      if l.is_err() || w.is_err() { return Err("Не удалось распарсить значения длины и ширины.".into()) }
      self.location_size.set(MapSizes { l: l.unwrap(), w: w.unwrap() })?;
      self.done.store(true, AtomicOrdering::Relaxed);
    }
    Ok(())
  }
}
