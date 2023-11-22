#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use once_cell::sync::OnceCell;
use salvo::http::ParseError;
use salvo::prelude::*;
use serde::Deserialize;
use std::collections::{VecDeque, HashMap};
use std::sync::{Arc, Mutex};

/// Запускает программу.
#[tokio::main]
async fn main() {
  tracing_subscriber::fmt().init();
  tokio::task::spawn(async {
    let router = Router::new().post(post_new_location);
    let acceptor = TcpListener::new("127.0.0.1:5800").bind().await;
    Server::new(acceptor).serve(router).await;
  });
  let native_options = eframe::NativeOptions::default();
  let _ = eframe::run_native("Little Locator", native_options, Box::new(|cc| Box::new(LittleLocatorApp::new(cc))));
}

/// Данные о местоположении.
#[derive(Deserialize, Clone)]
struct Location {
  pub id: String,
  pub x: f64,
  pub y: f64,
  pub z: f64,
}

impl std::fmt::Display for Location {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&format!("Location of object #{}: x - {}, y - {}, z - {}", self.id, self.x, self.y, self.z))
  }
}

/// Ошибки сервера.
pub struct ServerError {
  msg: String
}

#[async_trait]
impl Writer for ServerError {
  async fn write(mut self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
    res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
    res.render(self.msg);
  }
}

impl From<String> for ServerError {
  fn from(string: String) -> Self {
    Self { msg: string }
  }
}

impl From<&str> for ServerError {
  fn from(str: &str) -> Self {
    str.to_owned().into()
  }
}

impl From<ParseError> for ServerError {
  fn from(value: ParseError) -> Self {
    value.to_string().into()
  }
}

impl<T> From<std::sync::PoisonError<T>> for ServerError {
  fn from(value: std::sync::PoisonError<T>) -> Self {
    value.to_string().into()
  }
}

type MResult<T> = Result<T, ServerError>;

// Ячейка для обмена данными между бэкендом и фронтендом.
static DATA_QUEUE: OnceCell<Arc<Mutex<VecDeque<Location>>>> = OnceCell::new();

/// Добавляет новые данные о местоположении.
#[handler]
async fn post_new_location(req: &mut Request) -> MResult<&'static str> {
  let data = req.parse_json::<Location>().await?;
  DATA_QUEUE
    .get().ok_or::<String>("Не удалось подключиться к очереди данных.".into())?
    .lock()?
    .push_back(data);
  Ok("Gotcha!")
}

/// Структура приложения.
#[derive(Default, Clone)]
struct LittleLocatorApp {
  location_image: Option<String>,
  location_sizes: Option<[f32; 2]>,
  data_flow: Arc<Mutex<VecDeque<Location>>>,
  current_locations: HashMap<String, Location>,
}

impl LittleLocatorApp {
  fn new(cc: &eframe::CreationContext<'_>) -> Self {
    egui_extras::install_image_loaders(&cc.egui_ctx);
    let data_flow = Arc::new(Mutex::new(VecDeque::new()));
    DATA_QUEUE.set(data_flow.clone()).ok();
    Self {
      data_flow,
      ..Default::default()
    }
  }

  pub fn ui_content(&mut self, ui: &mut egui::Ui) -> egui::Response {
    let (mut response, painter) = ui.allocate_painter(ui.available_size_before_wrap(), egui::Sense::drag());

    let mut position_rect = egui::Rect::from_min_size(egui::Pos2::ZERO, response.rect.size());

    egui::Image::from_uri(format!("file://{}", self.location_image.as_ref().unwrap().clone()))
      .tint(egui::Color32::WHITE)
      .fit_to_original_size(1f32)
      .paint_at(ui, painter.clip_rect());
    response
  }
}

impl eframe::App for LittleLocatorApp {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    egui::CentralPanel::default().show(ctx, |ui| {
      if let Some(image_path) = &self.location_image {
        ui.horizontal(|ui| {
          ui.label("Picked file:");
          ui.monospace(image_path);
        });

        {
          let mut data_flow_guard = self.data_flow.lock().unwrap();
          while let Some(new_location) = data_flow_guard.pop_front() {
            self.current_locations.insert(new_location.id.clone(), new_location);
          }
        }
        for key in self.current_locations.keys() {
          ui.label(format!("{}", self.current_locations.get(key).unwrap()));
        }
        egui::Frame::canvas(ui.style()).show(ui, |ui2| { self.ui_content(ui2); });
      } else if ui.button("Open file…").clicked() {
        if let Some(path) = rfd::FileDialog::new().pick_file() {
          self.location_image = Some(path.display().to_string());
        }
      }
    });
  }
}
