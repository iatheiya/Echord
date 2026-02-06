// Файл: core/error.rs
// Регламент 2.0.1: Унифицированные типы ошибок (Синтез из ircliberror.rs и errors.rs)

use thiserror::Error;

// --- 1. Внутренняя Ошибка (CoreError) ---
// Используется всеми Rust-модулями (internal helpers).

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Network request failed: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Failed to parse URL: {0}")]
    UrlParse(#[from] url::ParseError),

    // Используется для ошибок serde_json::Error
    #[error("Failed to parse JSON response or payload: {0}")]
    Parse(#[from] serde_json::Error),

    #[error("Invalid HTTP method: {0}")]
    InvalidMethod(String),

    // Используется для ошибок парсинга заголовков (из JSON или их валидации reqwest)
    #[error("Invalid headers or header JSON: {0}")]
    InvalidHeader(String),
}

// --- 2. FFI-Ошибка (ApiError) ---
// Используется на FFI-границе (uniffi::export).
// Должна соответствовать 'core.udl'.
#[derive(Error, Debug, uniffi::Error)]
pub enum ApiError {
    #[error("Network or request error: {0}")]
    RequestError(String),
    #[error("Failed to parse response: {0}")]
    ParseError(String),
}

// --- 3. Мост (CoreError -> FFI-Ошибка) ---
// (АУДИТ 2.3): Гарантирует корректное преобразование.

impl From<CoreError> for ApiError {
    fn from(err: CoreError) -> Self {
        match err {
            // Сетевые ошибки, URL, Method, Header - это RequestError
            CoreError::Network(e) => ApiError::RequestError(e.to_string()),
            CoreError::UrlParse(e) => ApiError::RequestError(e.to_string()),
            CoreError::InvalidMethod(e) => ApiError::RequestError(format!("Invalid Method: {}", e)),
            CoreError::InvalidHeader(e) => ApiError::RequestError(format!("Header error: {}", e)),
            
            // Ошибки парсинга JSON - это ParseError
            CoreError::Parse(e) => ApiError::ParseError(e.to_string()),
        }
    }
}
