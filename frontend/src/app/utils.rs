/// Загружает изображение из массива байтов.
pub fn load_image_from_memory(image_data: &[u8]) -> Result<egui::ColorImage, image::ImageError> {
  let image = image::load_from_memory(image_data)?;
  let size = [image.width() as _, image.height() as _];
  let image_buffer = image.to_rgba8();
  let pixels = image_buffer.as_flat_samples();
  Ok(egui::ColorImage::from_rgba_unmultiplied(
    size,
    pixels.as_slice(),
  ))
}
