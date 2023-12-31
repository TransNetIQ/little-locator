//! Сервер, управляющий данными.

use crate::utils::{MResult, DATA_TX_QUEUE, WS_TX_QUEUE, AppConfig};

use ll_data::{Location, MapSizes, MAX_QUEUE_LEN};
use log::debug;
use salvo::{Request, Response};
use salvo::handler;
use salvo::websocket::{Message, WebSocketUpgrade};
use tokio::sync::mpsc;
use tokio::fs;

/// Добавляет новые данные о местоположении.
#[handler]
pub async fn post_new_location(req: &mut Request) -> MResult<&'static str> {
  let data = req.parse_json::<Location>().await?;
  DATA_TX_QUEUE
    .get().ok_or::<String>("Не удалось подключиться к очереди данных (на запись).".into())?
    .send(data)
    .await?;
  debug!("Got new location, inserted into DATA_QUEUE");
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
      let (tx, mut rx) = mpsc::channel(MAX_QUEUE_LEN);
      {
        let ws_tx_queue = match WS_TX_QUEUE.get()
          .ok_or::<String>("".into())
        {
          Ok(queue) => queue,
          Err(_) => return,
        };
        let ws_tx_guard = ws_tx_queue.lock().await;
        if ws_tx_guard.send(tx).await.is_err() { return }
        debug!("Sent new client to WS_QUEUE");
      }
      // Ожидаем новые местоположения и отправляем клиенту.
      while let Some(location) = rx.recv().await {
        if ws.send(Message::text(serde_json::to_string(&location).unwrap())).await.is_err() { return; }
        debug!("Sent new location from DATA_QUEUE over WebSocket");
      }
    })
    .await?;
  Ok(())
}

/// Отправляет на фронтенд данные о наличии конфигурации.
#[handler]
pub async fn get_config(res: &mut Response) -> MResult<()> {
  let app_config = serde_json::from_str::<AppConfig>(&fs::read_to_string("config.json").await?)?;
  res.render(salvo::writing::Json(MapSizes { l: app_config.length, w: app_config.width }));
  Ok(())
}

/// Отправляет на фронтенд карту расположения.
#[handler]
pub async fn get_location_img(req: &mut Request, res: &mut Response) -> MResult<()> {
  let app_config = serde_json::from_str::<AppConfig>(&fs::read_to_string("config.json").await?)?;
  salvo::fs::NamedFile::builder(app_config.image_filepath).send(req.headers(), res).await;
  Ok(())
}
