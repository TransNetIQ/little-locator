use ewebsock::{WsEvent, WsMessage};
use ll_data::{Location, MAX_QUEUE_LEN};
use std::collections::VecDeque;

use crate::app::app_data::{LittleLocatorApp, TagSettings};

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
          self.tracked_tags_locations.insert(
            new_location_id,
            TagSettings {
              locations: new_vecdeque,
              redrawal_index: 1usize,
              visible: true,
              show_path: false,
              show_anchor_calculated_distance: false,
              show_anchor_real_distance: false,
            }
          );
        } else {
          let tag = self.tracked_tags_locations.get_mut(&new_location.id).unwrap();
          if tag.locations.len() > MAX_QUEUE_LEN { tag.locations.pop_front(); }
          tag.locations.push_back(new_location);
          tag.redrawal_index += 1usize;
        };
      }
    }
  }
}
