use crate::utils::{MResult, DATA_RX_QUEUE, WS_TX_QUEUE};

use ll_data::MAX_QUEUE_LEN;
use log::debug;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;

/// Управляет отправкой местоположений всем клиентам.
pub async fn start_threaded_location_sender() -> MResult<()> {
  let (tx, mut rx) = mpsc::channel(MAX_QUEUE_LEN);
  WS_TX_QUEUE.set(Arc::new(Mutex::new(tx))).unwrap();
  let channels = Arc::new(Mutex::new(Vec::new()));
  
  // Пока можем добавлять новые каналы в список рассылки, делаем это
  tokio::spawn({
    let channels = channels.clone();
    
    async move {
      while let Some(channel) = rx.recv().await {
        channels.lock().await.push(channel);
        debug!("Added new channel");
      }
    }
  });
  
  tokio::spawn({
    let channels = channels.clone();
    
    async move {
      // Блокируем очередь данных здесь и только здесь.
      let mut data_rx_queue_lock = match DATA_RX_QUEUE.get()
        .ok_or::<String>("Не удалось получить очередь данных".into())
      {
        Err(_) => return,
        Ok(queue) => queue.lock().await,
      };
      debug!("Got DATA_RX_QUEUE");
      
      // Получив новое местоположение, отсылаем его по всем каналам WebSocket-клиентов.
      //
      // Если клиент умер, мы его убираем из списка.
      while let Some(location) = data_rx_queue_lock.recv().await {
        debug!("Got new location, going to send it over all channels");
        {
          let mut channels_guard = channels.lock().await;
          let mut i: usize = 0;
          while i < channels_guard.len() {
            if channels_guard[i].send(location.clone()).await.is_err() {
              channels_guard.remove(i);
              debug!("Removed flawless channel");
            } else {
              i += 1;
              debug!("Sent successfully");
            }
          }
        }
        debug!("Sent over all channels successfully and dropped channels' guard");
      }
    }
  });
  Ok(())
}
