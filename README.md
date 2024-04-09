# little-locator

![Отображение фиолетовой метки](frontend/assets/screenshot.png)

Приложение для простейшего проецирования геопозиций меток на карту здания.

**Возможности:**

1. Отображение меток
2. Отображение истории перемещений
3. Отсечение истории перемещений по времени
4. Поддержка конфигурирования
5. Отображение меток согласно графу возможных путей с максимальным радиусом прилипания

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

Чтобы отобразить карту с заранее выбранными параметрами, поместите в папку `backend` файл конфигурации `config.json` следующего вида:

```json
{
  "image_filepath": "assets/test_cropped.jpg",
  
  "length": 25.0,
  "width": 25.0,
  "max_sticking_radius": 0.5,
  "anchors": [
    {
      "id": "01:02:04:AC:C6",
      "x": 3.0,
      "y": 3.0,
      "z": 1.0
    },
    {
      "id": "01:02:06:AB:CC",
      "x": 22.0,
      "y": 3.0,
      "z": 1.0
    }
  ]
}
```

Чтобы загрузить карту автоматически, укажите данные ПК картирования в файле конфигурации:

```json
{
  "stnc_renaissance_username": "pe_admin",
  "stnc_renaissance_password": "pe_admin",
  "building_id": 5,
  "floor_id": 5,
  
  "length": 25.0,
  "width": 25.0,
  "max_sticking_radius": 0.5,
  "anchors": [
    {
      "id": "01:02:04:AC:C6",
      "x": 3.0,
      "y": 3.0,
      "z": 1.0
    },
    {
      "id": "01:02:06:AB:CC",
      "x": 22.0,
      "y": 3.0,
      "z": 1.0
    }
  ]
}
```

или:

```json
{
  "mss_backend_domain": "backend.domain",
  "django_api_login": "admin",
  "django_api_password": "password",
  "building_id": 5,
  "floor_id": 5,
  
  "length": 25.0,
  "width": 25.0,
  "max_sticking_radius": 0.5,
  "anchors": [
    {
      "id": "01:02:04:AC:C6",
      "x": 3.0,
      "y": 3.0,
      "z": 1.0
    },
    {
      "id": "01:02:06:AB:CC",
      "x": 22.0,
      "y": 3.0,
      "z": 1.0
    }
  ]
}
```

Путь указывается **относительно** папки `build`.

## Подготовка к сборке

Установите `rustup`, затем выполните:

```bash
curl https://sh.rustup.rs -sSf | sh
rustup toolchain install nightly
rustup default nightly
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
```

## Сборка

```bash
./prepare-product
```

## Запуск

```bash
cd build
./little-locator-backend
```

Сервер будет запущен на порту `5800`. [Переход в Web-UI](http://127.0.0.1:5800)
