use crate::utils::{AppConfig, MResult};
use log::{debug, info};
use salvo::http::HeaderMap;
use tokio::io::AsyncWriteExt;
use std::process::Command;

const IMAGE_FILEPATH: &'static str = "assets/loaded.png";

pub async fn auth() -> MResult<String> {
  info!("Запускается процесс аутентификации в ПК картирования...");
  let mut handle = Command::new("ll_rhkc/bin/python").args(["rhkc/auth.py"]).spawn().unwrap();
  handle.wait()?;
  let token = std::fs::read_to_string("token.txt")?;
  info!("Получен токен.");
  Ok(token)
}

pub async fn get_img() -> MResult<()> {
  let mut app_config = serde_json::from_str::<AppConfig>(&tokio::fs::read_to_string("config.json").await?)?;
  if !(
    app_config.image_filepath.is_none() &&
    app_config.stnc_renaissance_username.is_some() &&
    app_config.stnc_renaissance_password.is_some() &&
    app_config.org_name.is_some() &&
    app_config.building_id.is_some() &&
    app_config.floor_id.is_some()
  ) { return Ok(()) }
  let client = reqwest::Client::new();
  let mut building_req_hs = HeaderMap::new();
  let bearer_token = auth().await?;
  building_req_hs.insert("Accept", "application/json, text/plain, */*".parse()?);
  building_req_hs.insert("Accept-Language", "ru-RU,ru;q=0.9,en-US;q=0.8,en;q=0.7".parse()?);
  building_req_hs.insert("Authorization", (&bearer_token).parse()?);
  building_req_hs.insert("Connection", "keep-alive".parse()?);
  building_req_hs.insert("Origin", "https://plan-editor-demo.satellite-soft.ru".parse()?);
  building_req_hs.insert("Referer", "https://plan-editor-demo.satellite-soft.ru".parse()?);
  building_req_hs.insert("Sec-Fetch-Dest", "iframe".parse()?);
  building_req_hs.insert("Sec-Fetch-Mode", "navigate".parse()?);
  building_req_hs.insert("Sec-Fetch-Site", "same-site".parse()?);
  building_req_hs.insert("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36".parse()?);
  building_req_hs.insert("sec-ch-ua", "\"Not(A:Brand\";v=\"24\", \"Chromium\";v=\"122\"".parse()?);
  building_req_hs.insert("sec-ch-ua-mobile", "?0".parse()?);
  building_req_hs.insert("sec-ch-ua-platform", "\"Linux\"".parse()?);
  let building_data = client.get(format!("https://api-plan-editor-demo.satellite-soft.ru/floor_plans/?building={}", app_config.building_id.ok_or::<String>("Не указан building_id".into())?))
    .headers(building_req_hs)
    .send()
    .await?
    .bytes()
    .await?;
  debug!("Данные с сервера: {:?}", building_data);
  let building_data = String::from_utf8(building_data.to_vec())?;
  let start_index = building_data.find("http://api-plan-editor-demo.satellite-soft.ru/uploads").ok_or::<String>("Не удалось найти ссылку".into())?;
  let end_index = building_data.find("\",\"image_settings").ok_or::<String>("Не удалось найти окончание ссылки".into())?;
  let link = building_data[start_index..end_index].to_owned();
  debug!("Ссылка на изображение: {}", link);
  let mut image_file = tokio::fs::File::options().create_new(true).write(true).open(IMAGE_FILEPATH).await?;
  let image_bytes = reqwest::get(link)
    .await?
    .bytes()
    .await?;
  image_file.write(&image_bytes.to_vec()).await?;
  app_config.image_filepath = Some(IMAGE_FILEPATH.to_owned());
  tokio::fs::write("config.json", serde_json::to_string(&app_config)?).await?;
  Ok(())
}
