use crate::utils::{MResult, DATA_RX_QUEUE, WS_TX_QUEUE};

use std::sync::Arc;
use std::sync::mpsc;
use tokio::sync::Mutex;

/// Управляет отправкой местоположений всем клиентам.
pub async fn start_threaded_location_sender() -> MResult<()> {
  let (tx, rx) = mpsc::channel();
  WS_TX_QUEUE.set(Arc::new(Mutex::new(tx))).unwrap();
  
  tokio::spawn(async move {
    let mut channels = vec![];
    // Блокируем очередь данных здесь и только здесь.
    let data_rx_queue_lock = match DATA_RX_QUEUE.get().ok_or::<String>("Не удалось получить очередь данных".into()) {
      Err(_) => return,
      Ok(queue) => queue.lock().await,
    };
    
    loop {
      // Пока можем добавлять новые каналы в список рассылки, делаем это
      while let Ok(channel) = rx.try_recv() {
        channels.push(channel);
      }
      // Получив новое местоположение, отсылаем его по всем каналам WebSocket-клиентов.
      //
      // Если клиент умер, мы его убираем из списка.
      if let Ok(location) = data_rx_queue_lock.try_recv() {
        let mut i: usize = 0;
        while i < channels.len() {
          if channels[i].send(location.clone()).is_err() {
            channels.remove(i);
          } else {
            i += 1;
          }
        }
      }
    }
  });
  Ok(())
}
