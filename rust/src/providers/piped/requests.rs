=== FILE: src/piped/requests.rs ===
// ИЗМЕНЕННЫЙ ФАЙЛ: Удалена локальная декларация HTTP_CLIENT и обновлены импорты.

use crate::models::{
    CreatedPlaylist, InnerSession, Instance, OkMessage, PipedError, Playlist, PlaylistPreview,
    Session, TokenResponse,
};
// ИЗМЕНЕНИЕ: Импортируем внутреннюю модель для десериализации
use crate::models::internal::{ApiInstance, TokenResponse, OkMessage};
// ИЗМЕНЕНИЕ (РЕФАКТОРИНГ CORE): Импортируем общего клиента из core
use crate::core::HTTP_CLIENT; // ИСПОЛЬЗУЕТСЯ ОБЩИЙ КЛИЕНТ
// use once_cell::sync::Lazy; // УДАЛЕНО: Перенесено в core
use reqwest::{Method, Url};
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

// --- Вспомогательная функция запроса (как в Piped.kt) ---

/// Выполняет аутентифицированный запрос
async fn request(
    session: &InnerSession,
    method: Method,
    endpoint: &str,
    body: Option<Value>,
) -> Result<reqwest::Response, PipedError> {
    let url = session.api_base_url.join(endpoint)?;
    log::debug!("Piped Request: {} {}", method, url);

    let mut request_builder = session
        .client
        .request(method, url)
        .header("Authorization", &session.token)
        .header("Accept", "application/json");

    if let Some(body) = body {
        request_builder = request_builder
            .json(&body) // .json() корректно обработает `Value`
            .header("Content-Type", "application/json");
    }

    Ok(request_builder.send().await?.error_for_status()?)
}

// --- Статические (неаутентифицированные) функции ---

#[uniffi::export]
pub async fn get_instances() -> Result<Vec<Instance>, PipedError> {
    log::debug!("Fetching Piped instances...");
    
    // Шаг 1. Десериализуем во *внутреннюю* структуру `ApiInstance`
    let api_instances = HTTP_CLIENT // ИСПОЛЬЗУЕТ `core::HTTP_CLIENT`
        .get("https://piped-instances.kavin.rocks/")
        .send()
        .await?
        .error_for_status()?
        .json::<Vec<ApiInstance>>() // Используем ApiInstance
        .await?;

    // Шаг 2. Преобразуем (маппим) внутреннюю структуру в публичную FFI-структуру `Instance`
    let instances = api_instances
        .into_iter()
        .map(|api_inst| Instance {
            name: api_inst.name,
            api_base_url: api_inst.api_base_url,
            locations_formatted: api_inst.locations_formatted,
            version: api_inst.version,
            up_to_date: api_inst.up_to_date,
            is_cdn: api_inst.is_cdn,
            user_count: api_inst.user_count,
            last_checked: api_inst.last_checked.timestamp(), // Преобразуем DateTime<Utc> в i64
            has_cache: api_inst.has_cache,
            uses_s3: api_inst.uses_s3,
            image_proxy_base_url: api_inst.image_proxy_base_url,
            registration_disabled: api_inst.registration_disabled,
        })
        .collect();

    log::info!("Fetched {} Piped instances", instances.len());
    Ok(instances)
}

#[uniffi::export]
pub async fn login(
    api_base_url: String,
    username: String,
    password: String,
) -> Result<Arc<Session>, PipedError> {
    let base_url = Url::parse(&api_base_url)?;
    let login_url = base_url.join("login")?;

    let body = json!({
        "username": username,
        "password": password
    });

    log::debug!("Logging in to {}", api_base_url);

    let response = HTTP_CLIENT // ИСПОЛЬЗУЕТ `core::HTTP_CLIENT`
        .post(login_url)
        .json(&body)
        .send()
        .await?
        .error_for_status()?
        .json::<TokenResponse>()
        .await?;

    log::info!("Successfully logged in, creating session.");

    Ok(Arc::new(Session {
        inner: Arc::new(InnerSession {
            // Клонируем клиента, это дешевая операция, так как reqwest::Client
            // использует внутренний Arc для пула соединений.
            client: HTTP_CLIENT.clone(), 
            api_base_url: base_url,
            token: response.token,
        }),
    }))
}

// --- Методы Объекта Сессии (Аутентифицированные) ---

#[uniffi::export]
impl Session {
    pub fn get_api_base_url(&self) -> String {
        self.inner.api_base_url.to_string()
    }

    pub fn get_token(&self) -> String {
        self.inner.token.clone()
    }

    // --- Playlists ---

    pub async fn playlist_list(&self) -> Result<Vec<PlaylistPreview>, PipedError> {
        let response = request(&self.inner, Method::GET, "user/playlists", None).await?;
        Ok(response.json::<Vec<PlaylistPreview>>().await?)
    }

    pub async fn playlist_create(&self, name: String) -> Result<CreatedPlaylist, PipedError> {
        let body = json!({ "name": name });

        let response =
            request(&self.inner, Method::POST, "user/playlists/create", Some(body)).await?;
        Ok(response.json::<CreatedPlaylist>().await?)
    }

    pub async fn playlist_rename(&self, id: String, name: String) -> Result<bool, PipedError> {
        // КЛЮЧЕВОЕ ПРАВИЛО: Валидация FFI-входа (UUID)
        let _ = Uuid::parse_str(&id).map_err(|e| PipedError::Api(e.to_string()))?;

        let body = json!({
            "playlistId": id,
            "newName": name
        });

        let response =
            request(&self.inner, Method::POST, "user/playlists/rename", Some(body)).await?;
        Ok(response.json::<OkMessage>().await?.is_ok())
    }

    pub async fn playlist_delete(&self, id: String) -> Result<bool, PipedError> {
        // КЛЮЧЕВОЕ ПРАВИЛО: Валидация FFI-входа (UUID)
        let _ = Uuid::parse_str(&id).map_err(|e| PipedError::Api(e.to_string()))?;

        let body = json!({ "playlistId": id });

        let response =
            request(&self.inner, Method::POST, "user/playlists/delete", Some(body)).await?;
        Ok(response.json::<OkMessage>().await?.is_ok())
    }

    pub async fn playlist_add(&self, id: String, videos: Vec<String>) -> Result<bool, PipedError> {
        // КЛЮЧЕВОЕ ПРАВИЛО: Валидация FFI-входа (UUID)
        let _ = Uuid::parse_str(&id).map_err(|e| PipedError::Api(e.to_string()))?;

        let body = json!({
            "playlistId": id,
            "videoIds": videos
        });

        let response =
            request(&self.inner, Method::POST, "user/playlists/add", Some(body)).await?;
        Ok(response.json::<OkMessage>().await?.is_ok())
    }

    pub async fn playlist_remove(&self, id: String, index: i32) -> Result<bool, PipedError> {
        // КЛЮЧЕВОЕ ПРАВИЛО: Валидация FFI-входа (UUID)
        let _ = Uuid::parse_str(&id).map_err(|e| PipedError::Api(e.to_string()))?;

        let body = json!({
            "playlistId": id,
            "index": index
        });

        let response =
            request(&self.inner, Method::POST, "user/playlists/remove", Some(body)).await?;
        Ok(response.json::<OkMessage>().await?.is_ok())
    }

    pub async fn playlist_songs(&self, id: String) -> Result<Playlist, PipedError> {
        // КЛЮЧЕВОЕ ПРАВИЛО: Валидация FFI-входа (UUID)
        let _ = Uuid::parse_str(&id).map_err(|e| PipedError::Api(e.to_string()))?;

        let endpoint = format!("playlists/{}", id);
        let response = request(&self.inner, Method::GET, &endpoint, None).await?;
        Ok(response.json::<Playlist>().await?)
    }
}
