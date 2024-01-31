use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};

use ll_data::{AnchorPos, MapSizes, MaxStickingRadius};

use crate::app::utils::{ImageBytesOptionalRef, OptionalRef};
use crate::utils::MResult;

/// Получение имени хоста для создания запросов к бэкенду.
pub fn get_server_origin() -> MResult<String> {
  #[cfg(not(target_arch = "wasm32"))]
  let server_origin = "127.0.0.1".to_owned();
  #[cfg(target_arch = "wasm32")]
  let server_origin = {
    let window = web_sys::window().ok_or::<String>("no global `window` exists".into())?;
    let document = window.document().ok_or::<String>("should have a document on window".into())?;
    let location = document.location().ok_or::<String>("no location in the document".into())?;
    location.hostname()?
  };
  Ok(server_origin)
}

/// Получение пиктограмм меток и анкеров.
pub fn get_pics(
  tag_img: ImageBytesOptionalRef,
  anchor_img: ImageBytesOptionalRef,
) -> MResult<()> {
  let server_origin = get_server_origin()?;
  
  let tag_img_request = ehttp::Request::get(format!("http://{}:5800/tag_img", server_origin));
  let anchor_img_request = ehttp::Request::get(format!("http://{}:5800/anchor_img", server_origin));
  
  ehttp::fetch(tag_img_request, move |result| { let _ = tag_img.set(result.unwrap().bytes.clone()); });
  ehttp::fetch(anchor_img_request, move |result| { let _ = anchor_img.set(result.unwrap().bytes.clone()); });
  
  Ok(())
}

/// Получение конфигурации приложения с сервера.
pub fn request_config(
  location_size: OptionalRef<MapSizes>,
  location_image: ImageBytesOptionalRef,
  anchors: OptionalRef<HashMap<String, AnchorPos>>,
  max_sticking_radius: OptionalRef<f32>,
  done: Arc<AtomicBool>,
) -> MResult<()> {
  let server_origin = get_server_origin()?;
  
  let loc_size_request = ehttp::Request::get(format!("http://{}:5800/config", server_origin));
  let loc_img_request = ehttp::Request::get(format!("http://{}:5800/location_img", server_origin));
  let loc_anchors_request = ehttp::Request::get(format!("http://{}:5800/anchors", server_origin));
  let max_sticking_radius_request = ehttp::Request::get(format!("http://{}:5800/msr", server_origin));
  
  // Запрос на получение размеров картинки 
  // 
  // P.S. Если есть размеры, - есть всё остальное.
  ehttp::fetch(loc_size_request, move |result| {
    match result {
      Err(_) => (),
      Ok(resp) => {
        let bytes = resp.bytes.clone();
        let map_sizes = match serde_json::from_slice::<MapSizes>(&bytes) {
          Err(_) => return,
          Ok(v) => v,
        };
        let _ = location_size.set(map_sizes);
        // Запрос на получение картинки
        ehttp::fetch(loc_img_request, move |result| {
          match result {
            Err(_) => (),
            Ok(resp) => if resp.status == 200 {
              let _ = location_image.set(resp.bytes.clone());
              done.store(true, AtomicOrdering::Relaxed);
            },
          }
        });
        // Запрос на получение списка анкеров
        ehttp::fetch(loc_anchors_request, move |result| {
          match result {
            Err(_) => (),
            Ok(resp) => {
              let bytes = resp.bytes.clone();
              let anchors_vec = match serde_json::from_slice::<Vec<AnchorPos>>(&bytes) {
                Err(_) => return,
                Ok(v) => v,
              };
              let mut hm = HashMap::new();
              for anchor in anchors_vec { hm.insert(anchor.id.clone(), anchor); }
              let _ = anchors.set(hm);
            },
          }
        });
        // Запрос на получение максимального радиуса прилипания
        ehttp::fetch(max_sticking_radius_request, move |result| {
          match result {
            Err(_) => (),
            Ok(resp) => {
              let bytes = resp.bytes.clone();
              let msr = match serde_json::from_slice::<MaxStickingRadius>(&bytes) {
                Err(_) => return,
                Ok(v) => v,
              };
              if msr.max_sticking_radius >= 0f32 { let _ = max_sticking_radius.set(msr.max_sticking_radius); }
            }
          }
        });
      }
    }
  });
  Ok(())
}
