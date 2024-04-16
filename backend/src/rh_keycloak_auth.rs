use crate::utils::{AppConfig, MResult};
use log::{debug, info};
use regex::Regex;
use reqwest::{cookie::Jar, redirect::Policy, Certificate, Client};
use salvo::http::HeaderMap;
use tokio::io::AsyncWriteExt;
use std::{process::Command, sync::Arc};

const TOKEN_FILEPATH: &'static str = "token.txt";
const IMAGE_FILEPATH: &'static str = "assets/loaded.png";

const CSRF_MIDDLEWARE_RE: &'static str = r#"<input type="hidden" name="csrfmiddlewaretoken" value="(?<value>\S{64})">"#;

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

pub async fn auth_with_keycloak() -> MResult<String> {
  info!("Запускается процесс аутентификации в ПК картирования...");
  let mut handle = Command::new("ll_rhkc/bin/python").args(["rhkc/auth.py"]).spawn().unwrap();
  handle.wait()?;
  let token = std::fs::read_to_string(TOKEN_FILEPATH)?;
  std::fs::remove_file(TOKEN_FILEPATH)?;
  info!("Получен токен.");
  debug!("Токен: {}", &token);
  Ok(token)
}

pub async fn get_img() -> MResult<()> {
  let mut app_config = serde_json::from_str::<AppConfig>(&tokio::fs::read_to_string("config.json").await?)?;
  info!("{:?}", app_config);
  if app_config.image_filepath.is_some()
  { return Ok(()) }
  else if
    app_config.image_filepath.is_none() &&
    app_config.mss_backend_domain.is_some() &&
    app_config.stnc_renaissance_username.is_some() &&
    app_config.stnc_renaissance_password.is_some() &&
    app_config.org_name.is_some() &&
    app_config.building_id.is_some()
  { return get_img_with_keycloak(&mut app_config).await }
  else if
    app_config.image_filepath.is_none() &&
    app_config.mss_backend_domain.is_some() &&
    app_config.django_api_login.is_some() &&
    app_config.django_api_password.is_some() &&
    app_config.org_name.is_some() &&
    app_config.building_id.is_some()
  { return get_img_with_django_cookies(&mut app_config).await }
  else { return Err("Не найдено необходимых параметров в конфигурации! Пожалуйста, прочтите инструкцию и заполните файл `config.json` в соответствии с ней.".into()) }
}

pub async fn get_img_with_keycloak(app_config: &mut AppConfig) -> MResult<()> {
  let domain = app_config.mss_backend_domain.clone().unwrap_or("https://api-plan-editor-demo.satellite-soft.ru".into());
  let origin_domain = app_config.mss_domain.clone().unwrap_or("https://plan-editor-demo.satellite-soft.ru".into());
  info!("Загружаем картинку с сервера...");
  let client = Client::builder().add_root_certificate(Certificate::from_pem(include_bytes!("rp-transnetiq-ru-chain.pem")).unwrap()).use_rustls_tls().build()?;
  let mut building_req_hs = get_headers()?;
  let bearer_token = auth_with_keycloak().await?;
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
  let mut image_file = tokio::fs::File::options().create(true).write(true).open(IMAGE_FILEPATH).await?;
  let image_bytes = client.get(link)
    .send()
    .await?
    .bytes()
    .await?;
  image_file.write(&image_bytes.to_vec()).await?;
  app_config.image_filepath = Some(IMAGE_FILEPATH.to_owned());
  tokio::fs::write("config.json", serde_json::to_string(&app_config)?).await?;
  info!("Картинка загружена. Запуск сервера...");
  Ok(())
}

pub async fn get_img_with_django_cookies(app_config: &mut AppConfig) -> MResult<()> {
  let domain = app_config.mss_backend_domain.clone().unwrap_or("https://api-plan-editor-demo.satellite-soft.ru".into());
  let username = app_config.django_api_login.as_ref().unwrap();
  let password = app_config.django_api_password.as_ref().unwrap();
  
  debug!("Подготовка хранилища Cookie...");
  let cookie_store = Arc::new(Jar::default());
  
  debug!("Подготовка клиента...");
  let cli = Client::builder()
    .add_root_certificate(Certificate::from_pem(include_bytes!("rp-transnetiq-ru-chain.pem")).unwrap())
    .use_rustls_tls()
    .cookie_provider(cookie_store.clone())
    .build()?;
  let building_req_hs = get_headers()?;
  
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
    .add_root_certificate(Certificate::from_pem(include_bytes!("rp-transnetiq-ru-chain.pem")).unwrap())
    .use_rustls_tls()
    .cookie_provider(cookie_store)
    .redirect(Policy::none())
    .build()?;
  let mut building_req_hs = get_headers()?;
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
  
  // 3. Получение изображения
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
  let mut image_file = tokio::fs::File::options().create(true).write(true).open(IMAGE_FILEPATH).await?;
  let image_bytes = cli.get(link)
    .send()
    .await?
    .bytes()
    .await?;
  image_file.write(&image_bytes.to_vec()).await?;
  app_config.image_filepath = Some(IMAGE_FILEPATH.to_owned());
  tokio::fs::write("config.json", serde_json::to_string(&app_config)?).await?;
  info!("Картинка загружена. Запуск сервера...");
  Ok(())
}
