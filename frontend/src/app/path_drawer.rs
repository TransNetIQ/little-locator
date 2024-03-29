use crate::app::app_data::LittleLocatorApp;

use egui::pos2;
use ll_data::MAX_QUEUE_LEN;
use log::debug;
use std::collections::VecDeque;

impl LittleLocatorApp {
  pub fn redraw_tag_path_with_time_limit(&mut self, tag_key: &String, limit_time: i64) {
    // Получаем вектор позиций, которые нам необходимо отрисовать
    let current_tag_line = match self.tracked_tags_paths.get_mut(tag_key) {
      Some(line) => line,
      None => {
        self.tracked_tags_paths.insert(tag_key.clone(), VecDeque::new());
        self.tracked_tags_paths.get_mut(tag_key).unwrap()
      }
    };
    
    // Получаем структуру-хранилище всех позиций
    let tag = self.tracked_tags_locations.get_mut(tag_key).unwrap();
    
    // 1. Если ограничение активно
    if self.limit_tag_path {
      // 1.1. Если перерисовка не была совершена
      if !self.limit_online {
        current_tag_line.clear();
        for pos in &tag.locations {
          if pos.ts >= limit_time { current_tag_line.push_back(pos2(pos.x, pos.y)); }
        }
        tag.redrawal_index = 0;
        debug!("Tag #{}. Tag line forced", &tag.locations.back().unwrap().id);
      }
      // 1.2. Если перерисовка уже совершена, и нужно только добавить новые элементы, если они старше ограничения
      else {
        let index = &mut tag.redrawal_index;
        while *index > 0 {
          let curr_tag = &tag.locations[tag.locations.len() - *index];
          if !self.limit_tag_path || curr_tag.ts >= limit_time {
            current_tag_line.push_back(pos2(curr_tag.x, curr_tag.y));
          }
          *index -= 1usize;
        }
        while current_tag_line.len() > MAX_QUEUE_LEN { current_tag_line.pop_front(); }
        debug!("Tag #{}. Ts sub: {}", &tag.locations.back().unwrap().id, tag.locations.back().unwrap().ts - limit_time);
      }
    }
    // 2. Если ограничение неактивно
    else {
      // 2.1. Если ограничение неактивно, но перерисовки не было совершено 
      // (более старые местоположения не отображаются)
      if !self.limit_online {
        current_tag_line.clear();
        tag.redrawal_index = tag.locations.len();
      }
      let index = &mut tag.redrawal_index;
      while *index > 0 {
        let curr_tag = &tag.locations[tag.locations.len() - *index];
        if !self.limit_tag_path || curr_tag.ts >= limit_time {
          current_tag_line.push_back(pos2(curr_tag.x, curr_tag.y));
        }
        *index -= 1usize;
      }
      while current_tag_line.len() > MAX_QUEUE_LEN { current_tag_line.pop_front(); }
      debug!("Tag #{}. Tag line cleared", &tag.locations.back().unwrap().id);
    }
  }
}
