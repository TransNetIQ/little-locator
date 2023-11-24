//! Сервер, управляющий данными.

use salvo::Request;
use salvo::handler;

use crate::utils::{Location, MResult, DATA_QUEUE};

/// Добавляет новые данные о местоположении.
#[handler]
pub async fn post_new_location(req: &mut Request) -> MResult<&'static str> {
  let data = req.parse_json::<Location>().await?;
  DATA_QUEUE
    .get().ok_or::<String>("Не удалось подключиться к очереди данных.".into())?
    .lock()?
    .send(data)?;
  Ok("Gotcha!")
}
