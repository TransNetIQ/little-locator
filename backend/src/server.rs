//! Сервер, управляющий данными.

use ll_data::Location;
use salvo::Request;
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
