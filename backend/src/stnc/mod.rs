pub mod django_auth;
pub mod rh_keycloak_auth;

use crate::utils::{AppConfig, MResult};
use log::info;
use reqwest::cookie::Jar;
use salvo::http::HeaderMap;
use std::sync::Arc;

const IMAGE_FILEPATH: &'static str = "assets/loaded.png";
const PROBESM_CONFIG_FILEPATH: &'static str = "probesm.ini";

/// Получение общих заголовков
fn get_headers() -> MResult<HeaderMap> {
  let mut building_req_hs = HeaderMap::new();
  building_req_hs.insert("Accept-Language", "ru-RU,ru;q=0.9,en-US;q=0.8,en;q=0.7".parse()?);
  building_req_hs.insert("Connection", "keep-alive".parse()?);
  building_req_hs.insert("Sec-Fetch-Dest", "iframe".parse()?);
  building_req_hs.insert("Sec-Fetch-Mode", "navigate".parse()?);
  building_req_hs.insert("Sec-Fetch-Site", "same-site".parse()?);
  building_req_hs.insert("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36".parse()?);
  building_req_hs.insert("sec-ch-ua", "\"Not(A:Brand\";v=\"24\", \"Chromium\";v=\"122\"".parse()?);
  building_req_hs.insert("sec-ch-ua-mobile", "?0".parse()?);
  building_req_hs.insert("sec-ch-ua-platform", "\"Linux\"".parse()?);
  Ok(building_req_hs)
}

/// Получение изображения
pub async fn get_img(token: Option<String>, cookie_store: Option<Arc<Jar>>) -> MResult<()> {
  let mut app_config = serde_json::from_str::<AppConfig>(&tokio::fs::read_to_string("config.json").await?)?;
  if app_config.image_filepath.is_some()
  { return Ok(()) }
  else if
    app_config.image_filepath.is_none() &&
    app_config.mss_backend_domain.is_some() &&
    app_config.stnc_renaissance_username.is_some() &&
    app_config.stnc_renaissance_password.is_some() &&
    app_config.org_name.is_some() &&
    app_config.building_id.is_some()
  { return rh_keycloak_auth::get_img(&mut app_config, token.unwrap()).await }
  else if
    app_config.image_filepath.is_none() &&
    app_config.mss_backend_domain.is_some() &&
    app_config.django_api_login.is_some() &&
    app_config.django_api_password.is_some() &&
    app_config.org_name.is_some() &&
    app_config.building_id.is_some()
  { return django_auth::get_img(&mut app_config, cookie_store.unwrap()).await }
  else { return Err("Не найдено необходимых параметров в конфигурации! Пожалуйста, прочтите инструкцию и заполните файл `config.json` в соответствии с ней.".into()) }
}

/// Получение местоположений анкеров
pub async fn get_anchors(token: Option<String>, cookie_store: Option<Arc<Jar>>) -> MResult<()> {
  let mut app_config = serde_json::from_str::<AppConfig>(&tokio::fs::read_to_string("config.json").await?)?;
  if
    app_config.mss_backend_domain.is_some() &&
    app_config.stnc_renaissance_username.is_some() &&
    app_config.stnc_renaissance_password.is_some() &&
    app_config.org_name.is_some() &&
    app_config.organization_id.is_some()
  { return rh_keycloak_auth::get_anchors(&mut app_config, token.unwrap()).await }
  else if
    app_config.mss_backend_domain.is_some() &&
    app_config.django_api_login.is_some() &&
    app_config.django_api_password.is_some() &&
    app_config.org_name.is_some() &&
    app_config.organization_id.is_some()
  { return django_auth::get_anchors(&mut app_config, cookie_store.unwrap()).await }
  else { return Err("Не найдено необходимых параметров в конфигурации! Пожалуйста, прочтите инструкцию и заполните файл `config.json` в соответствии с ней.".into()) }
}

pub async fn update_data() -> MResult<()> {
  // Удаляем картинку из конфига
  let mut app_config = serde_json::from_str::<AppConfig>(&tokio::fs::read_to_string("config.json").await?)?;
  app_config.image_filepath = None;
  app_config.anchors.clear();
  
  let (mut token, mut cookie_store) = (None, None);
  if app_config.stnc_renaissance_username.is_some() && app_config.stnc_renaissance_password.is_some() {
    token = Some(rh_keycloak_auth::auth().await?);
  }
  else if app_config.django_api_login.is_some() && app_config.django_api_password.is_some() {
    cookie_store = Some(django_auth::auth(&mut app_config).await?);
  }
  
  tokio::fs::write("config.json", serde_json::to_string(&app_config)?).await?;
  let _ = tokio::fs::remove_file(crate::stnc::rh_keycloak_auth::TOKEN_FILEPATH).await;
  
  crate::stnc::get_img(token.clone(), cookie_store.clone()).await?;
  crate::stnc::get_anchors(token, cookie_store).await?;
  info!("Всё готово.");
  Ok(())
}
