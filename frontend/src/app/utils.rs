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
  pub fn ref_cx<'a, F, R>(&self, func: F) -> MResult<R>
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
  pub fn mut_cx<'a, F, R>(&mut self, func: F) -> MResult<R>
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
