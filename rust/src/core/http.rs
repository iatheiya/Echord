// Файл: core/http.rs
// Регламент 2.0.1: Унифицированный HTTP-клиент и внутренние хелперы.
// (Синтез из http.rs, client.rs, innertubecore.rs)

use super::error::CoreError;
use once_cell::sync::Lazy;
use reqwest::{header, Client, Url};
use std::time::Duration;

const APP_USER_AGENT: &str = "ViTune-Core/1.0 (https://github.com/ViTune/Core)"; // Единый User-Agent
const DEFAULT_TIMEOUT_SEC: u64 = 30;

// --- Управление Ресурсами (Rule 1.4) ---
// Глобальный, потокобезопасный и лениво-инициализируемый HTTP-клиент.
// 'reqwest::Client' использует Arc внутри и безопасен для 'static Lazy'.
// Его 'drop' не требуется, т.к. он живет до конца программы.
pub static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::USER_AGENT,
        header::HeaderValue::from_static(APP_USER_AGENT),
    );

    Client::builder()
        .default_headers(headers)
        .gzip(true)
        .brotli(true)
        .deflate(true)
        .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SEC))
        .connect_timeout(Duration::from_secs(DEFAULT_TIMEOUT_SEC))
        .build()
        .expect("Failed to build core HTTP_CLIENT")
});

// --- Внутренние (Rust-only) асинхронные хелперы (Rule 1.1) ---

/// Универсальный хелпер для выполнения GET-запроса и парсинга JSON-ответа.
/// Используется внутренними провайдерами Rust, не FFI.
pub async fn fetch_json<T: for<'de> serde::Deserialize<'de>>(
    url: Url,
) -> Result<T, CoreError> {
    let response = HTTP_CLIENT
        .get(url)
        .send()
        .await? // -> CoreError::Network
        .error_for_status()? // -> CoreError::Network (HTTP status)
        .json::<T>()
        .await?; // -> CoreError::Parse
    Ok(response)
}

/// Универсальный хелпер для выполнения GET-запроса и получения текстового ответа.
pub async fn fetch_text(url: Url) -> Result<String, CoreError> {
    let response_text = HTTP_CLIENT
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    Ok(response_text)
}
