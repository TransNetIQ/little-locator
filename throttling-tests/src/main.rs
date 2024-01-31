use ll_data::{Location, LocationType};

fn send_request_with_location(loc: Location) -> () {
  reqwest::blocking::Client::new()
    .post("http://127.0.0.1:5800/")
    .json(&loc)
    .send().unwrap();
}

fn main() {
  loop {
    // 1
    send_request_with_location(Location {
      id: "MovableHelper1".into(),
      loc_type: LocationType::Tag,
      x: 15.65,
      y: 17.7,
      z: 1.0,
      ts: ll_data::curr_ts(),
      dist: None,
    });
    send_request_with_location(Location {
      id: "MovableHelper2".into(),
      loc_type: LocationType::Tag,
      x: 12.8,
      y: 19.6,
      z: 1.0,
      ts: ll_data::curr_ts(),
      dist: None,
    });
    send_request_with_location(Location {
      id: "MovableHelper3".into(),
      loc_type: LocationType::Tag,
      x: 10.0,
      y: 17.0,
      z: 1.0,
      ts: ll_data::curr_ts(),
      dist: None,
    });
    std::thread::sleep(std::time::Duration::from_millis(500));
    // 2
    send_request_with_location(Location {
      id: "MovableHelper1".into(),
      loc_type: LocationType::Tag,
      x: 17.65,
      y: 17.7,
      z: 1.0,
      ts: ll_data::curr_ts(),
      dist: None,
    });
    send_request_with_location(Location {
      id: "MovableHelper2".into(),
      loc_type: LocationType::Tag,
      x: 15.8,
      y: 19.6,
      z: 1.0,
      ts: ll_data::curr_ts(),
      dist: None,
    });
    send_request_with_location(Location {
      id: "MovableHelper3".into(),
      loc_type: LocationType::Tag,
      x: 15.0,
      y: 17.0,
      z: 1.0,
      ts: ll_data::curr_ts(),
      dist: None,
    });
    std::thread::sleep(std::time::Duration::from_millis(500));
    // 3
    send_request_with_location(Location {
      id: "MovableHelper1".into(),
      loc_type: LocationType::Tag,
      x: 15.65,
      y: 20.7,
      z: 1.0,
      ts: ll_data::curr_ts(),
      dist: None,
    });
    send_request_with_location(Location {
      id: "MovableHelper2".into(),
      loc_type: LocationType::Tag,
      x: 12.8,
      y: 21.6,
      z: 1.0,
      ts: ll_data::curr_ts(),
      dist: None,
    });
    send_request_with_location(Location {
      id: "MovableHelper3".into(),
      loc_type: LocationType::Tag,
      x: 10.0,
      y: 14.0,
      z: 1.0,
      ts: ll_data::curr_ts(),
      dist: None,
    });
    std::thread::sleep(std::time::Duration::from_millis(500));
  }
}
