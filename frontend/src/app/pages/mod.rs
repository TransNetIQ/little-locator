pub mod cfg;
pub mod map;
pub mod tags;
pub mod path_traversal;

use std::sync::atomic::Ordering as AtomicOrdering;

use crate::app::app_data::LittleLocatorApp;
use crate::utils::{Ignore, MResult};

use super::app_data::MenuOps;

impl eframe::App for LittleLocatorApp {
  /// Отрисовывает приложение.
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    ctx.request_repaint_after(std::time::Duration::from_millis(200));
    egui::CentralPanel::default().show(ctx, |ui| {
      if !self.done.load(AtomicOrdering::Relaxed) { let _ = self.show_cfg_page(ui); }
      else {
        self.handle_new_tags_locations();
        let _ = self.show_main_page(ui);
      }
    });
  }
}

const MENU: [&str; 3] = [
  "Карта",
  "Метки",
  "Граф путей",
];

impl LittleLocatorApp {
  /// Показывает главную страницу.
  pub fn show_main_page(&mut self, ui: &mut egui::Ui) -> MResult<()> {
    // Показываем меню
    ui.horizontal(|ui| {
      ui.label("Меню:");
      egui::ComboBox::from_id_source(1)
        .selected_text(format!("{}", match self.menu {
          MenuOps::Map => MENU[0],
          MenuOps::Tags => MENU[1],
          MenuOps::Graph => MENU[2],
        }))
        .show_ui(ui, |ui| {
          ui.selectable_value(&mut self.menu, MenuOps::Map, MENU[0]);
          ui.selectable_value(&mut self.menu, MenuOps::Tags, MENU[1]);
          ui.selectable_value(&mut self.menu, MenuOps::Graph, MENU[2]);
        }
      );
      if self.menu == MenuOps::Map { ui.checkbox(&mut self.show_path_traversal_graph, "Отображать граф"); }
      if self.menu == MenuOps::Map || self.menu == MenuOps::Graph {
        if ui.button("Принудительно обновить изображение и анкера").clicked() {
          self.updating.store(true, AtomicOrdering::Relaxed);
          let _ = crate::app::startup_requests::update_location_image(self.location_size.clone(), self.location_image.clone(), self.updating.clone());
        }
      }
    });
    // Показываем весь остальной интерфейс
    match self.menu {
      MenuOps::Map => { self.show_map(ui).ignore(); },
      MenuOps::Tags => { self.show_tags_list(ui); },
      MenuOps::Graph => { self.show_path_traversal(ui).ignore(); },
    }
    Ok(())
  }
}
