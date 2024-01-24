pub mod cfg;
pub mod map;
pub mod tags;
pub mod path_traversal;

use std::sync::atomic::Ordering as AtomicOrdering;

use crate::app::app_data::LittleLocatorApp;
use crate::utils::{HOURS, MINUTES, MResult};

pub const MENU: [&'static str; 3] = [
  "Карта",
  "Метки",
  "Тестовый граф путей"
];

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

impl LittleLocatorApp {
  /// Показывает главную страницу.
  pub fn show_main_page(&mut self, ui: &mut egui::Ui) -> MResult<()> {
    // Показываем меню
    ui.horizontal(|ui| {
      ui.label("Меню:");
      egui::ComboBox::from_label("").show_index(ui, &mut self.menu, 3usize, |i| MENU[i]);
    });
    // Показываем весь остальной интерфейс
    match self.menu {
      0usize => { self.show_map(ui); },
      1usize => { self.show_tags_list(ui); },
      2usize => { self.show_path_traversal(ui); },
      _ => return Err("Такого пункта меню не существует.".into())
    }
    Ok(())
  }
}
