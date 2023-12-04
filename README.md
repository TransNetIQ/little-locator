# little-locator

![Отображение фиолетовой метки](arts/screenshot.png)

Приложение для простейшего проецирования геопозиций меток на карту здания.

Чтобы отобразить новую позицию, отправьте POST-запрос на `localhost:5800` с JSON вида:

```json
{
  "id": "Метка №1",
  "x": 16.0,
  "y": 12.0,
  "z": 1.0
}
```

**Убедитесь,** что масштаб при выборе картинки задан корректно.

## Подготовка к сборке

Установите `rustup`, затем выполните:

```bash
curl https://sh.rustup.rs -sSf | sh
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
```

## Сборка фронтенда

```bash
cd little-locator/frontend
trunk build --release
```

## Сборка бэкенда

```bash
cd little-locator/backend
cargo build --release
```

## Запуск

```bash
cd little-locator/backend
cargo run --release
```
