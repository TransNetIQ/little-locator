use egui::{TextureHandle, Ui, Pos2, Vec2, pos2, vec2};
use ll_data::MapSizes;
use std::sync::{Arc, Mutex};

use crate::utils::MResult;

/// Опциональная потокобезопасная структура данных.
pub struct OptionalRef<T: Clone> {
  inner_val: Arc<Mutex<Option<T>>>,
}

impl<T: Clone> OptionalRef<T> {
  pub fn new() -> Self {
    OptionalRef { inner_val: Arc::new(Mutex::new(None)) }
  }
  
  pub fn set(&self, val: T) -> MResult<()> {
    *self.inner_val.lock()? = Some(val);
    Ok(())
  }
  
  pub fn is_some(&self) -> bool {
    self.inner_val.lock().is_ok_and(|val| val.is_some())
  }
  
  /// Предоставляет контекст для иммутабельного доступа.
  /// 
  /// Пример:
  /// 
  /// ```rust
  /// optional_reference.ref_cx(|val| println!("{:?}", *val));
  /// ```
  pub fn ref_cx<F, R>(&self, func: F) -> MResult<R>
    where F: Fn(&T) -> R
  {
    let mutex_guard = self.inner_val.lock()?;
    let data_ref = mutex_guard.as_ref().ok_or::<String>("Option has None value.".into())?;
    Ok(func(data_ref))
  }
  
  /// Предоставляет контекст для мутабельного доступа.
  /// 
  /// Пример:
  /// 
  /// ```rust
  /// optional_reference.mut_cx(|val| *val = map_sizes);
  /// ```
  #[allow(dead_code)]
  pub fn mut_cx<F, R>(&mut self, func: F) -> MResult<R>
    where F: Fn(&mut T) -> R
  {
    let mut mutex_guard = self.inner_val.lock()?;
    let data_ref = mutex_guard.as_mut().ok_or::<String>("Option has None value.".into())?;
    Ok(func(data_ref))
  }
  
  pub fn clone(&self) -> Self {
    OptionalRef { inner_val: self.inner_val.clone() }
  }
  
  pub fn get_cloned(&self) -> MResult<T> {
    Ok((*self.inner_val.lock()?.as_ref().ok_or::<String>("Option has None value.".into())?).clone())
  }
}

pub type ImageBytesOptionalRef = OptionalRef<Vec<u8>>;

/// Загружает изображение из массива байтов.
pub fn load_image_from_memory(image_data: &[u8]) -> MResult<egui::ColorImage> {
  let image = image::load_from_memory(image_data)?;
  let size = [image.width() as _, image.height() as _];
  let image_buffer = image.to_rgba8();
  let pixels = image_buffer.as_flat_samples();
  Ok(egui::ColorImage::from_rgba_unmultiplied(
    size,
    pixels.as_slice(),
  ))
}

pub fn load_texture(ui: &mut Ui, name: &str, image_ref: &ImageBytesOptionalRef) -> MResult<TextureHandle> {
  Ok(ui.ctx().load_texture(
    name,
    egui::ImageData::Color(Arc::new(image_ref.ref_cx(|val| load_image_from_memory(val))??)),
    Default::default(),
  ))
}

/// Вычисляет масштаб картинки на экране.
pub fn scale(
  map_rect: egui::Rect,
  map_sizes: MapSizes,
) -> Vec2 {
  vec2(
    map_rect.width() / map_sizes.l,
    map_rect.height() / map_sizes.w,
  )
}

/// Вычисляет координаты для отображения их на карте.
pub fn to_map(
  map_rect: egui::Rect,
  scale: Vec2,
  position: Pos2,
  centering_val: Vec2,
) -> Pos2 {
  pos2(
    map_rect.left() + position.x * scale.x - centering_val.x / 2f32, // c = mr + p*s   =>   p = (c - mr) / s
    map_rect.top() + position.y * scale.y - centering_val.y / 2f32,
  )
}

/// Вычисляет координаты относительно истинных размеров местоположения.
pub fn from_map(
  map_rect: egui::Rect,
  scale: Vec2,
  coordinates: Pos2,
) -> Pos2 {
  pos2(
    (coordinates.x - map_rect.left()) / scale.x,
    (coordinates.y - map_rect.top()) / scale.y,
  )
}
