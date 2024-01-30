use crate::app::app_data::LittleLocatorApp;
use crate::app::startup_requests::{get_server_origin, get_pics, request_config};
use crate::app::utils::OptionalRef;
use crate::utils::MResult;

use chrono::Local;
use std::collections::HashMap;
use std::sync::{Arc, atomic::AtomicBool};

impl LittleLocatorApp {
  /// Создаёт приложение.
  pub fn new(cc: &eframe::CreationContext<'_>) -> MResult<Self> {
    cc.egui_ctx.set_visuals(egui::Visuals::dark());
    egui_extras::install_image_loaders(&cc.egui_ctx);

    let tag_img = OptionalRef::new();
    let anchor_img = OptionalRef::new();
    get_pics(tag_img.clone(), anchor_img.clone())?;
    
    let done = Arc::new(AtomicBool::new(false));
    let loc_size = OptionalRef::new();
    let anchors = OptionalRef::new();
    let loc_img = OptionalRef::new();
    let max_sticking_radius = OptionalRef::new();
    
    let _ = request_config(
      loc_size.clone(),
      loc_img.clone(),
      anchors.clone(),
      max_sticking_radius.clone(),
      done.clone(),
    );
    
    let server_origin = get_server_origin()?;
    let (data_tx, data_rx) = ewebsock::connect(format!("ws://{}:5800/ws_updater", server_origin))?;

    Ok(Self {
      _server_origin: server_origin,
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
      menu: 0,
      path_traversal_graph: Vec::new(),
      path_to_add: (false, None, false),
      last_pos: None,
      max_sticking_radius,
    })
  }
}
