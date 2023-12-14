use crate::app::app_data::LittleLocatorApp;

use chrono::Local;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

impl LittleLocatorApp {
  /// Создаёт приложение.
  pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
    #[cfg(not(target_arch = "wasm32"))]
    let server_origin = "127.0.0.1";
    #[cfg(target_arch = "wasm32")]
    let server_origin = {
      let window = web_sys::window().expect("no global `window` exists");
      let document = window.document().expect("should have a document on window");
      let location = document.location().expect("no location in the document");
      location.hostname().expect("no hostname in document")
    };

    cc.egui_ctx.set_visuals(egui::Visuals::light());
    egui_extras::install_image_loaders(&cc.egui_ctx);

    let pos_img = Arc::new(Mutex::new(Option::None));
    {
      let pos_img = pos_img.clone();
      let request = ehttp::Request::get(format!("http://{}:5800/position_img", server_origin));
      ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
        *pos_img.lock().unwrap() = Some(result.unwrap().bytes.clone());
      });
    }

    let (data_tx, data_rx) = ewebsock::connect(format!("ws://{}:5800/ws_updater", server_origin)).unwrap();

    Self {
      _server_origin: server_origin.to_owned(),
      _data_sender: data_tx,
      data_receiver: data_rx,
      l_input: "25.0".into(),
      w_input: "25.0".into(),
      position_image_bytes: pos_img,
      location_image: Arc::new(Mutex::new(Option::None)),
      location_size: None,
      done: false,
      tracked_tags_locations: HashMap::new(),
      tracked_tags_paths: HashMap::new(),
      limit_tag_path: false,
      limited: false,
      current_limit: (Local::now().date_naive(), 0, 0),
      previous_limit: (Local::now().date_naive(), 0, 0),
    }
  }
}
