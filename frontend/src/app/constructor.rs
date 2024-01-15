use crate::app::app_data::LittleLocatorApp;

use chrono::Local;
use ll_data::{MapSizes, Pos3};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering as AtomicOrdering}};

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

    cc.egui_ctx.set_visuals(egui::Visuals::dark());
    egui_extras::install_image_loaders(&cc.egui_ctx);

    let tag_img = Arc::new(Mutex::new(Option::None));
    {
      let tag_img = tag_img.clone();
      let request = ehttp::Request::get(format!("http://{}:5800/tag_img", server_origin));
      ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
        *tag_img.lock().unwrap() = Some(result.unwrap().bytes.clone());
      });
    }

    let anchor_img = Arc::new(Mutex::new(Option::None));
    {
      let anchor_img = anchor_img.clone();
      let request = ehttp::Request::get(format!("http://{}:5800/anchor_img", server_origin));
      ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
        *anchor_img.lock().unwrap() = Some(result.unwrap().bytes.clone());
      });
    }
    
    let done = Arc::new(AtomicBool::new(false));
    let loc_size = Arc::new(Mutex::new(Option::None));
    let loc_img = Arc::new(Mutex::new(Option::None));
    let anchors = Arc::new(Mutex::new(Option::None));
    {
      let loc_size = loc_size.clone();
      let loc_img = loc_img.clone();
      let anchors = anchors.clone();
      let done = done.clone();
      
      let loc_size_request = ehttp::Request::get(format!("http://{}:5800/config", server_origin));
      let loc_img_request = ehttp::Request::get(format!("http://{}:5800/location_img", server_origin));
      let loc_anchors_request = ehttp::Request::get(format!("http://{}:5800/anchors", server_origin));
      ehttp::fetch(loc_size_request, move |result: ehttp::Result<ehttp::Response>| {
        match result {
          Err(_) => return,
          Ok(resp) => {
            let bytes = resp.bytes.clone();
            let map_sizes = match serde_json::from_slice::<MapSizes>(&bytes) {
              Err(_) => return,
              Ok(v) => v,
            };
            *loc_size.lock().unwrap() = Some(map_sizes);
            // Запрос на получение картинки
            ehttp::fetch(loc_img_request, move |result: ehttp::Result<ehttp::Response>| {
              match result {
                Err(_) => return,
                Ok(resp) => if resp.status == 200 {
                  *loc_img.lock().unwrap() = Some(resp.bytes.clone());
                  done.store(true, AtomicOrdering::Relaxed);
                },
              }
            });
            // Запрос на получение списка анкеров
            ehttp::fetch(loc_anchors_request, move |result: ehttp::Result<ehttp::Response>| {
              match result {
                Err(_) => return,
                Ok(resp) => {
                  let bytes = resp.bytes.clone();
                  let anchors_vec = match serde_json::from_slice::<Vec<Pos3>>(&bytes) {
                    Err(_) => return,
                    Ok(v) => v,
                  };
                  *anchors.lock().unwrap() = Some(anchors_vec);
                },
              }
            });
          }
        }
      });
    }
    
    let (data_tx, data_rx) = ewebsock::connect(format!("ws://{}:5800/ws_updater", server_origin)).unwrap();

    Self {
      _server_origin: server_origin.to_owned(),
      _data_sender: data_tx,
      data_receiver: data_rx,
      l_input: "25.0".into(),
      w_input: "25.0".into(),
      tag_image_bytes: tag_img,
      anchor_image_bytes: anchor_img,
      location_image: loc_img,
      location_size: loc_size,
      done,
      tracked_tags_locations: HashMap::new(),
      tracked_tags_paths: HashMap::new(),
      anchors,
      limit_tag_path: false,
      limit_online: false,
      current_limit: (Local::now().date_naive(), 0, 0),
      previous_limit: (Local::now().date_naive(), 0, 0),
      show_only_tags_list: false,
    }
  }
}
