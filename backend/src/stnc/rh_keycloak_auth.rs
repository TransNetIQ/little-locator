use crate::utils::{AppConfig, MResult};
use ll_data::AnchorPos;
use log::{debug, info};
use reqwest::{Certificate, Client};
use tokio::io::AsyncWriteExt;
use std::process::Command;

pub const TOKEN_FILEPATH: &'static str = "token.txt";

/// Процесс авторизации при помощи KeyCloak через selenium-wire
pub async fn auth() -> MResult<String> {
  info!("Запускается процесс аутентификации в ПК картирования...");
  let mut handle = Command::new("ll_rhkc/bin/python").args(["rhkc/auth.py"]).spawn().unwrap();
  handle.wait()?;
  let token = std::fs::read_to_string(TOKEN_FILEPATH)?;
  info!("Получен токен.");
  debug!("Токен: {}", &token);
  Ok(token)
}

/// Получение изображения
pub async fn get_img(app_config: &mut AppConfig, bearer_token: String) -> MResult<()> {
  let domain = app_config.mss_backend_domain.clone().unwrap_or("https://api-plan-editor-demo.satellite-soft.ru".into());
  let origin_domain = app_config.mss_domain.clone().unwrap_or("https://plan-editor-demo.satellite-soft.ru".into());
  info!("Загружаем картинку с сервера...");
  let client = Client::builder().add_root_certificate(Certificate::from_pem(include_bytes!("../rp-transnetiq-ru-chain.pem")).unwrap()).use_rustls_tls().build()?;
  let mut building_req_hs = crate::stnc::get_headers()?;
  // let bearer_token = auth_with_keycloak().await?;
  building_req_hs.insert("Accept", "application/json, text/plain, */*".parse()?);
  building_req_hs.insert("Authorization", (&bearer_token).parse()?);
  building_req_hs.insert("Origin", origin_domain.parse()?);
  building_req_hs.insert("Referer", origin_domain.parse()?);
  let building_data: serde_json::Value = client
    .get(format!(
      "{}/floor_plans/?building={}",
      domain,
      app_config.building_id.ok_or::<String>("Не указан building_id".into())?
    ))
    .headers(building_req_hs)
    .send()
    .await?
    .json()
    .await?;
  let link = building_data
    .as_array()  .unwrap()
    .get(0)      .unwrap()
    .as_object() .unwrap()
    .get("image").unwrap()
    .as_str()    .unwrap();
  debug!("Ссылка на изображение: {}", link);
  let link = link.replace("http", "https")  + "/";
  let mut image_file = tokio::fs::File::options().create(true).write(true).open(crate::stnc::IMAGE_FILEPATH).await?;
  let image_bytes = client.get(link)
    .send()
    .await?
    .bytes()
    .await?;
  image_file.write(&image_bytes.to_vec()).await?;
  app_config.image_filepath = Some(crate::stnc::IMAGE_FILEPATH.to_owned());
  tokio::fs::write("config.json", serde_json::to_string(&app_config)?).await?;
  info!("Картинка загружена. Запуск сервера...");
  Ok(())
}

pub async fn get_anchors(app_config: &mut AppConfig, bearer_token: String) -> MResult<()> {
  let domain = app_config.mss_backend_domain.clone().unwrap_or("https://api-plan-editor-demo.satellite-soft.ru".into());
  let origin_domain = app_config.mss_domain.clone().unwrap_or("https://plan-editor-demo.satellite-soft.ru".into());
  debug!("Запрос анкеров.");
  let client = Client::builder().add_root_certificate(Certificate::from_pem(include_bytes!("../rp-transnetiq-ru-chain.pem")).unwrap()).use_rustls_tls().build()?;
  let mut building_req_hs = crate::stnc::get_headers()?;
  // let bearer_token = auth_with_keycloak().await?;
  building_req_hs.insert("Accept", "application/json, text/plain, */*".parse()?);
  building_req_hs.insert("Authorization", (&bearer_token).parse()?);
  building_req_hs.insert("Origin", origin_domain.parse()?);
  building_req_hs.insert("Referer", origin_domain.parse()?);
  let anchors_data: serde_json::Value = client
    .get(format!(
      "{}/devices/?organization={}",
      domain,
      app_config.organization_id.ok_or::<String>("Не указан organization_id".into())?
    ))
    .headers(building_req_hs)
    .send()
    .await?
    .json()
    .await?;

  debug!("Содержимое ответа: {:?}", anchors_data);
  let anchors = anchors_data.as_array().unwrap();
  let mut anchors_vec = vec![];
  
  for anchor in anchors {
    let anchor = anchor.as_object().unwrap();
    let mac = anchor.get("mac").unwrap().as_str().unwrap();
    let x = anchor.get("latitude").unwrap().as_f64().unwrap();
    let y = anchor.get("longitude").unwrap().as_f64().unwrap();
    let z = anchor.get("local_z").unwrap().as_f64().unwrap() + 3.0;
    anchors_vec.push(AnchorPos {
      id: mac.to_owned(),
      x: x as f32,
      y: y as f32,
      z: z as f32,
    });
  }
  
  let mut probesm_config = tokio::fs::read_to_string(crate::stnc::PROBESM_CONFIG_FILEPATH).await?
    .split("\n")
    .map(|v| v.to_owned())
    .filter(|s| !s.starts_with("g=2"))
    .filter(|s| !s.is_empty())
    .collect::<Vec<String>>();
  anchors_vec.into_iter().for_each(|a| probesm_config.insert(0, format!(r#"g=2;{};{};{};{};"{}""#, a.id, a.x, a.y, a.z, format!("uwbv3-{}", a.id))));
  tokio::fs::write(crate::stnc::PROBESM_CONFIG_FILEPATH, probesm_config.join("\n")).await?;
  info!("Анкеры загружены.");
  Ok(())
}
