//! Сервер, управляющий данными.

use ll_data::Location;
use salvo::{Request, Response};
use salvo::handler;

use crate::utils::{MResult, DATA_QUEUE};

/// Добавляет новые данные о местоположении.
#[handler]
pub async fn post_new_location(req: &mut Request) -> MResult<&'static str> {
  let data = req.parse_json::<Location>().await?;
  println!("Sending...");
  DATA_QUEUE
    .get().ok_or::<String>("Не удалось подключиться к очереди данных.".into())?
    .send(data)?;
  println!("Sent.");
  Ok("Gotcha!")
}

/// Отправляет на фронтенд иконку позиционирования.
#[handler]
pub async fn get_position_img(req: &mut Request, res: &mut Response) {
  salvo::fs::NamedFile::builder("../frontend/assets/position.png").send(req.headers(), res).await;
}
