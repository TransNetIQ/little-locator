//! Модуль диалога выбора картинки.
#[cfg(target_arch = "wasm32")]
use rfd::AsyncFileDialog;
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;

use crate::app::utils::ImageBytesOptionalRef;
use crate::utils::MResult;

/// Вызывает диалог выбора картинки.
/// 
/// Функция требуется, когда у Little Locator нет серверного конфига.
pub fn select_image_dialog(image_ref: ImageBytesOptionalRef) -> MResult<()> {
  #[cfg(target_arch = "wasm32")] {
    wasm_bindgen_futures::spawn_local(async move {
      let file = AsyncFileDialog::new()
        .add_filter("image", &["png", "jpg"])
        .set_directory("/")
        .pick_file()
        .await;
      let data = file.unwrap().read().await;
      image_ref.set(data.clone()).unwrap();
    });
  }
  #[cfg(not(target_arch = "wasm32"))] {
    let file = FileDialog::new()
      .add_filter("image", &["png", "jpg"])
      .set_directory("/")
      .pick_file();
    let data = std::fs::read(file.ok_or::<String>("File was not selected.".into())?)?;
    image_ref.set(data)?;
  }
  Ok(())
}
