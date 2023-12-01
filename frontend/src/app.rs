//! Приложение для отрисовки местоположений.

use egui::{pos2, vec2};
use ll_data::DATA_QUEUE;
use ll_data::Location;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

#[cfg(target_arch = "wasm32")]
use rfd::AsyncFileDialog;
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;

// use web_sys::{ErrorEvent, MessageEvent, WebSocket};

/// Загружает изображение из файла.
// fn load_image_from_path(path: &std::path::Path) -> Result<egui::ColorImage, image::ImageError> {
//   let image = image::io::Reader::open(path)?.decode()?;
//   let size = [image.width() as _, image.height() as _];
//   let image_buffer = image.to_rgba8();
//   let pixels = image_buffer.as_flat_samples();
//   Ok(egui::ColorImage::from_rgba_unmultiplied(
//     size,
//     pixels.as_slice(),
//   ))
// }

fn load_image_from_memory(image_data: &[u8]) -> Result<egui::ColorImage, image::ImageError> {
  let image = image::load_from_memory(image_data)?;
  let size = [image.width() as _, image.height() as _];
  let image_buffer = image.to_rgba8();
  let pixels = image_buffer.as_flat_samples();
  Ok(egui::ColorImage::from_rgba_unmultiplied(
    size,
    pixels.as_slice(),
  ))
}

/// Структура приложения.
pub struct LittleLocatorApp {
  // Поля страницы выбора карты
  l_input: String,
  w_input: String,
  done: bool,
  // Данные картинок
  position_image_bytes: Arc<Mutex<Option<Vec<u8>>>>,
  // Данные о местоположениях и карте
  location_image: Arc<Mutex<Option<Vec<u8>>>>,
  location_size: Option<[f32; 2]>,
  data_receiver: mpsc::Receiver<Location>,
  current_locations: HashMap<String, Location>,
}

impl LittleLocatorApp {
  /// Создаёт приложение.
  pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
    egui_extras::install_image_loaders(&cc.egui_ctx);
    let (data_tx, data_rx) = mpsc::channel::<Location>();
    DATA_QUEUE.set(Arc::new(Mutex::new(data_tx))).ok();

    let pos_img = Arc::new(Mutex::new(Option::None));
    {
      let pos_img = pos_img.clone();
      let request = ehttp::Request::get("http://127.0.0.1:5800/position_img");
      ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
        log::debug!("{:?}", result);
        *pos_img.lock().unwrap() = Some(result.unwrap().bytes.clone());
      });
    }

    Self {
      data_receiver: data_rx,
      l_input: "25.0".into(),
      w_input: "25.0".into(),
      position_image_bytes: pos_img,
      location_image: Arc::new(Mutex::new(Option::None)),
      location_size: None,
      done: false,
      current_locations: HashMap::new(),
    }
  }

  /// Отображает карту здания и текущее местоположение объектов.
  pub fn paint_location(&mut self, ui: &mut egui::Ui) -> egui::Response {
    let (response, painter) = ui.allocate_painter(ui.available_size_before_wrap(), egui::Sense::drag());

    // Рисуем здание
    let loc_img = self.location_image.lock().unwrap();
    egui::Image::from_bytes("bytes://location_map", loc_img.as_ref().unwrap().clone())
      .tint(egui::Color32::WHITE)
      .fit_to_original_size(1f32)
      .paint_at(ui, painter.clip_rect());

    // Рисуем местоположения объектов
    let pos_img = self.position_image_bytes.lock().unwrap();
    let tag_txr = ui.ctx().load_texture(
      "tag",
      egui::ImageData::Color(Arc::new(load_image_from_memory(pos_img.as_ref().unwrap()).unwrap())),
      Default::default(),
    );

    let location_size = self.location_size.as_ref().unwrap();
    let icon_size = vec2(20.0, 20.0);

    let scale = vec2(painter.clip_rect().width() / location_size[0], painter.clip_rect().height() / location_size[1]);

    for key in self.current_locations.keys() {
      let icon_position_scaled = pos2(
        painter.clip_rect().left() + self.current_locations[key].x * scale.x,
        painter.clip_rect().top() + self.current_locations[key].y * scale.y
      );

      painter.image(
        tag_txr.id(),
        egui::Rect::from_min_max(icon_position_scaled, icon_position_scaled + icon_size),
        egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
        egui::Color32::WHITE,
      );
    }
    response
  }

  /// Показывает страницу выбора карты.
  pub fn show_map_selection_page(&mut self, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
      ui.label("Длина здания:");
      ui.text_edit_singleline(&mut self.l_input);
    });
    ui.horizontal(|ui| {
      ui.label("Ширина здания:");
      ui.text_edit_singleline(&mut self.w_input);
    });

    if ui.button("Выбор карты…").clicked() {
      let image_path = self.location_image.clone();
      #[cfg(target_arch = "wasm32")] {
        wasm_bindgen_futures::spawn_local(async move {
          let file = AsyncFileDialog::new()
            .add_filter("image", &["png", "jpg"])
            .set_directory("/")
            .pick_file()
            .await;
          let data = file.unwrap().read().await;
          *image_path.lock().unwrap() = Some(data.clone());
        });
      }
      #[cfg(not(target_arch = "wasm32"))] {
        let file = FileDialog::new()
          .add_filter("image", &["png", "jpg"])
          .set_directory("/")
          .pick_file();
        let data = std::fs::read(file.unwrap()).unwrap();
        *image_path.lock().unwrap() = Some(data);
      }
    }

    if ui.button("Готово").clicked() {
      let l = self.l_input.parse::<f32>();
      let w = self.w_input.parse::<f32>();
      if l.is_err() || w.is_err() { return }
      self.location_size = Some([l.unwrap(), w.unwrap()]);
      self.done = true;
    }
  }

  /// Показывает страницу с картой.
  pub fn show_map_page(&mut self, ui: &mut egui::Ui) {
    while let Ok(new_location) = self.data_receiver.try_recv() {
      self.current_locations.insert(new_location.id.clone(), new_location);
    }

    for key in self.current_locations.keys() {
      ui.label(format!("{}", self.current_locations.get(key).unwrap()));
    }
    egui::Frame::canvas(ui.style()).show(ui, |ui2| { self.paint_location(ui2); });
  }
}

impl eframe::App for LittleLocatorApp {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    egui::CentralPanel::default().show(ctx, |ui| {
      if self.done { self.show_map_page(ui); }
      else { self.show_map_selection_page(ui); }
    });
  }
}
