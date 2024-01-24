use ewebsock::{WsEvent, WsMessage};
use ll_data::{Location, MAX_QUEUE_LEN};
use std::collections::VecDeque;

use crate::app::app_data::LittleLocatorApp;

impl LittleLocatorApp {
  /// Обрабатывает входящие местоположения
  pub fn handle_new_tags_locations(&mut self) {
    while let Some(event) = self.data_receiver.try_recv() {
      let message = match event {
        WsEvent::Message(message) => message,
        _ => continue,
      };
      let location_json = match message {
        WsMessage::Text(location_json) => location_json,
        _ => continue,
      };
      if let Ok(new_location) = serde_json::from_str::<Location>(&location_json) {
        if !self.tracked_tags_locations.contains_key(&new_location.id) {
          let mut new_vecdeque = VecDeque::new();
          let new_location_id = new_location.id.clone();
          new_vecdeque.push_back(new_location);
          self.tracked_tags_locations.insert(new_location_id, (new_vecdeque, true, false, 1usize, false));
        } else {
          let locations = self.tracked_tags_locations.get_mut(&new_location.id).unwrap();
          if locations.0.len() > MAX_QUEUE_LEN { locations.0.pop_front(); }
          locations.0.push_back(new_location);
          locations.3 += 1usize;
        };
      }
    }
  }
}
