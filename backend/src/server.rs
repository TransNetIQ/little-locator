//! Сервер, управляющий данными.

use crate::utils::{MResult, DATA_TX_QUEUE, WS_TX_QUEUE};

use ll_data::Location;
use salvo::{Request, Response};
use salvo::handler;
use salvo::websocket::{Message, WebSocketUpgrade};
use std::sync::mpsc;

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
      // Регистрируем клиента для получения местоположений.
      let (tx, rx) = mpsc::channel();
      {
        let ws_tx_queue = match WS_TX_QUEUE.get().ok_or::<String>("".into()) {
          Ok(queue) => queue,
          Err(_) => return,
        };
        let ws_tx_guard = ws_tx_queue.lock().await;
        if ws_tx_guard.send(tx).is_err() { return }
      }
      // Ожидаем новые местоположения и отправляем клиенту.
      loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        while let Ok(location) = rx.try_recv() {
          if ws.send(Message::text(serde_json::to_string(&location).unwrap())).await.is_err() { return; }
        }
      }
    })
    .await?;
  Ok(())
}
