//! Сервер, управляющий данными.

use ll_data::Location;
use salvo::{Request, Response};
use salvo::handler;
use salvo::websocket::{Message, WebSocketUpgrade};

use crate::utils::{MResult, DATA_TX_QUEUE, DATA_RX_QUEUE};

/// Добавляет новые данные о местоположении.
#[handler]
pub async fn post_new_location(req: &mut Request) -> MResult<&'static str> {
  let data = req.parse_json::<Location>().await?;
  DATA_TX_QUEUE
    .get().ok_or::<String>("Не удалось подключиться к очереди данных (на запись).".into())?
    .send(data)?;
  Ok("Gotcha!")
}

/// Отправляет на фронтенд иконку позиционирования.
#[handler]
pub async fn get_position_img(req: &mut Request, res: &mut Response) {
  salvo::fs::NamedFile::builder("../frontend/assets/position.png").send(req.headers(), res).await;
}

/// Вебсокет для обновления местоположения на фронтенде.
#[handler]
pub async fn ws_location_sender(req: &mut Request, res: &mut Response) -> MResult<()> {
  WebSocketUpgrade::new()
    .upgrade(req, res, |mut ws| async move {
      while { tokio::time::sleep(tokio::time::Duration::from_millis(100)).await; true } {
        let queue = match DATA_RX_QUEUE.get().ok_or::<String>("Не удалось подключиться к очереди данных (на получение).".into()) {
          Ok(queue) => queue,
          Err(_) => return,
        };
        let guard = match queue.try_lock() {
          Ok(guard) => guard,
          Err(_) => return,
        };
        if let Ok(location) = guard.try_recv() {
          if ws.send(Message::text(serde_json::to_string(&location).unwrap())).await.is_err() { return; }
        }
      }
    })
    .await?;
  Ok(())
}
