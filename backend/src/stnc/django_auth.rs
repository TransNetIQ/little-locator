use crate::utils::{AppConfig, MResult};
use ll_data::AnchorPos;
use log::{debug, info};
use regex::Regex;
use reqwest::{cookie::Jar, redirect::Policy, Certificate, Client};
use tokio::io::AsyncWriteExt;
use std::sync::Arc;

const CSRF_MIDDLEWARE_RE: &'static str = r#"<input type="hidden" name="csrfmiddlewaretoken" value="(?<value>\S{64})">"#;

/// Процесс авторизации при помощи Django
pub async fn auth(app_config: &mut AppConfig) -> MResult<Arc<Jar>> {
  let domain = app_config.mss_backend_domain.clone().unwrap_or("https://api-plan-editor-demo.satellite-soft.ru".into());
  let username = app_config.django_api_login.as_ref().unwrap();
  let password = app_config.django_api_password.as_ref().unwrap();
  
  debug!("Подготовка хранилища Cookie...");
  let cookie_store = Arc::new(Jar::default());
  
  debug!("Подготовка клиента...");
  let cli = Client::builder()
    .add_root_certificate(Certificate::from_pem(include_bytes!("../rp-transnetiq-ru-chain.pem")).unwrap())
    .use_rustls_tls()
    .cookie_provider(cookie_store.clone())
    .build()?;
  let building_req_hs = crate::stnc::get_headers()?;
  
  // 1. Запрос страницы авторизации
  debug!("Запрос страницы авторизации.");
  let req1 = cli.get(format!("{}/api-auth/login", domain)).headers(building_req_hs.clone()).send().await?;
  
  debug!("Формирование парсера регулярного выражения...");
  let csrf_middleware_regex = Regex::new(CSRF_MIDDLEWARE_RE)?;
  let req1_response = req1.text().await?;
  let csrf_middleware = csrf_middleware_regex
    .captures(&req1_response)
    .ok_or::<String>("Не удалось найти регулярное выражение.".into())?
    .name("value")
    .ok_or::<String>("Не удалось найти middleware.".into())?
    .as_str();
  debug!("Найден middleware: {}", csrf_middleware);
  
  // 2. Авторизация
  debug!("Подготовка нового клиента.");
  let cli = Client::builder()
    .add_root_certificate(Certificate::from_pem(include_bytes!("../rp-transnetiq-ru-chain.pem")).unwrap())
    .use_rustls_tls()
    .cookie_provider(cookie_store.clone())
    .redirect(Policy::none())
    .build()?;
  let mut building_req_hs = crate::stnc::get_headers()?;
  building_req_hs.insert("Accept", "Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8".parse()?);
  building_req_hs.insert("Connection", "close".parse()?);
  building_req_hs.insert("Content-Type", "application/x-www-form-urlencoded".parse()?);
  building_req_hs.insert("Origin", domain.parse()?);
  building_req_hs.insert("Referer", format!("{}/api-auth/login", domain).parse()?);
  
  debug!("Запрос авторизации.");
  cli.post(format!("{}/api-auth/login/", domain))
    .headers(building_req_hs.clone())
    .body(format!("csrfmiddlewaretoken={}&next=&username={}&password={}&submit=Log+in", csrf_middleware, username, password))
    .send()
    .await?;
  
  Ok(cookie_store)
}

/// Получение картинки
pub async fn get_img(app_config: &mut AppConfig, cookie_store: Arc<Jar>) -> MResult<()> {
  let domain = app_config.mss_backend_domain.clone().unwrap_or("https://api-plan-editor-demo.satellite-soft.ru".into());
  
  let cli = Client::builder()
    .add_root_certificate(Certificate::from_pem(include_bytes!("../rp-transnetiq-ru-chain.pem")).unwrap())
    .use_rustls_tls()
    .cookie_provider(cookie_store.clone())
    .redirect(Policy::none())
    .build()?;
  
  let mut building_req_hs = crate::stnc::get_headers()?;
  building_req_hs.insert("Accept", "Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8".parse()?);
  building_req_hs.insert("Connection", "close".parse()?);
  building_req_hs.insert("Content-Type", "application/x-www-form-urlencoded".parse()?);
  building_req_hs.insert("Origin", domain.parse()?);
  building_req_hs.insert("Referer", format!("{}/api-auth/login", domain).parse()?);
  
  // Получение изображения
  debug!("Запрос изображения.");
  let building_data: serde_json::Value = cli
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
  debug!("Содержимое ответа: {:?}", building_data);
  let link = building_data
    .as_array()  .unwrap()
    .get(0)      .unwrap()
    .as_object() .unwrap()
    .get("image").unwrap()
    .as_str()    .unwrap();
  debug!("Ссылка на изображение: {}", link);
  let link = link.replace("http", "https") + "/";
  let mut image_file = tokio::fs::File::options().create(true).write(true).open(crate::stnc::IMAGE_FILEPATH).await?;
  let image_bytes = cli.get(link)
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

/// Получение анкеров
pub async fn get_anchors(app_config: &mut AppConfig, cookie_store: Arc<Jar>) -> MResult<()> {
  let domain = app_config.mss_backend_domain.clone().unwrap_or("https://api-plan-editor-demo.satellite-soft.ru".into());
  
  let cli = Client::builder()
    .add_root_certificate(Certificate::from_pem(include_bytes!("../rp-transnetiq-ru-chain.pem")).unwrap())
    .use_rustls_tls()
    .cookie_provider(cookie_store.clone())
    .redirect(Policy::none())
    .build()?;
  
  let mut building_req_hs = crate::stnc::get_headers()?;
  building_req_hs.insert("Accept", "Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8".parse()?);
  building_req_hs.insert("Connection", "close".parse()?);
  building_req_hs.insert("Content-Type", "application/x-www-form-urlencoded".parse()?);
  building_req_hs.insert("Origin", domain.parse()?);
  building_req_hs.insert("Referer", domain.parse()?);
  
  // Получение изображения
  debug!("Запрос анкеров.");
  let anchors_data: serde_json::Value = cli
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
