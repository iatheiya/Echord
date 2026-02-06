// Файл: core/request_impl.rs
// Регламент 1.2: Реализация FFI-контракта (NetworkRequest)
// Логика перемещена из commoncore.rs и инкапсулирована здесь.

use super::error::{ApiError, CoreError};
use super::http::HTTP_CLIENT;
use log::debug;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Method,
};
use std::collections::HashMap;
use std::str::FromStr;
use url::Url;

// --- FFI-структуры данных (Перемещены из commoncore.rs) ---

/// Структура запроса, получаемая через FFI.
#[derive(Debug, uniffi::Record)]
pub struct NetworkRequest {
    pub url: String,
    pub method: String,
    pub headers_json: Option<String>, 
    pub body: Option<String>,
}

/// Структура ответа, возвращаемая через FFI.
#[derive(Debug, uniffi::Record)]
pub struct RawResponse {
    pub status_code: i64,
    pub body: String,
}

// --- Приватный хелпер (Уровень 1: Функциональная унификация) ---

fn parse_headers_from_json(headers_json: Option<String>) -> Result<HeaderMap, CoreError> {
    let mut header_map_reqwest = HeaderMap::new();
    
    let Some(headers_str) = headers_json else {
        return Ok(header_map_reqwest);
    };
    
    debug!("Parsing headers from JSON...");
    
    // (Interfacing/Security) Валидация JSON
    let headers_map: HashMap<String, String> = serde_json::from_str(&headers_str)
        .map_err(|e| CoreError::InvalidHeader(format!("Failed to parse JSON headers: {}", e)))?; 

    for (k, v) in headers_map {
        // (Interfacing/Security) Валидация Имени
        let name = HeaderName::from_str(&k)
            .map_err(|e| CoreError::InvalidHeader(format!("Invalid header name ({}): {}", k, e)))?;
        // (Interfacing/Security) Валидация Значения
        let value = HeaderValue::from_str(&v)
            .map_err(|e| CoreError::InvalidHeader(format!("Invalid header value for {}: {}", k, e)))?;
        
        header_map_reqwest.append(name, value);
    }
    
    Ok(header_map_reqwest)
}

// --- Универсальная FFI-функция запроса (Граница взаимодействия) ---

/// (Rule 2.5) Публичный FFI-контракт.
/// Выполняет сырой HTTP-запрос.
///
/// Pre-conditions:
/// - 'request.url' должен быть валидным URL.
/// - 'request.method' должен быть валидным HTTP-методом.
/// - 'request.headers_json' (если 'Some') должен быть валидным JSON (String -> String map).
///
/// Post-conditions:
/// - Возвращает 'RawResponse' (включая 4xx/5xx статусы).
/// - Возвращает 'ApiError::RequestError' при ошибках сети, URL, Method, Headers.
/// - Возвращает 'ApiError::ParseError' (редко, если сам 'CoreError' не может быть создан).
#[uniffi::export]
pub async fn fetch_raw(request: NetworkRequest) -> Result<RawResponse, ApiError> {
    debug!(
        "Executing core::fetch_raw for [{}]: {}",
        request.method, request.url
    );

    // [ИСПРАВЛЕНО (Rule 2.3)]: Явное .map_err() для преобразования
    // url::ParseError -> CoreError::UrlParse,
    // который '?' затем преобразует в ApiError.
    let url = Url::parse(&request.url).map_err(CoreError::UrlParse)?;

    // [ИСПРАВЛЕНО (Rule 2.3)]: Явное .map_err()
    let method = Method::from_str(&request.method)
        .map_err(|_| CoreError::InvalidMethod(request.method.clone()))?;

    let mut req_builder = HTTP_CLIENT.request(method, url);

    // [ИСПРАВЛЕНО (Rule 2.3)]: '?' здесь корректен,
    // т.к. parse_headers_from_json уже возвращает CoreError.
    let headers = parse_headers_from_json(request.headers_json)?;
    req_builder = req_builder.headers(headers);

    if let Some(body) = request.body {
        req_builder = req_builder.body(body);
    }

    // [ИСПРАВЛЕНО (Rule 2.3)]: Явное .map_err() для
    // reqwest::Error -> CoreError::Network
    let response = req_builder.send().await.map_err(CoreError::Network)?;

    let status = response.status().as_u16() as i64;
    
    // (Semantic) .error_for_status() НЕ используется,
    // т.к. 'fetch_raw' должен возвращать сырой ответ.

    // [ИСПРАВЛЕНО (Rule 2.3)]: Явное .map_err()
    let body_text = response.text().await.map_err(CoreError::Network)?; 

    Ok(RawResponse {
        status_code: status,
        body: body_text,
    })
}


// --- Тесты (Rule 1.4) ---
#[cfg(test)]
mod tests {
    use super::*;

    // Тест: Невалидный URL
    #[tokio::test]
    async fn test_fetch_raw_invalid_url() {
        let request = NetworkRequest {
            url: "not a valid url".to_string(),
            method: "GET".to_string(),
            headers_json: None,
            body: None,
        };

        let result = fetch_raw(request).await;
        assert!(result.is_err());
        match result.err().unwrap() {
            ApiError::RequestError(s) => assert!(s.contains("url parse")), 
            e => panic!("Wrong error type: {:?}", e),
        }
    }

    // Тест: Невалидный JSON заголовков
    #[tokio::test]
    async fn test_fetch_raw_invalid_json_headers() {
        let request = NetworkRequest {
            url: "https://example.com".to_string(),
            method: "GET".to_string(),
            headers_json: Some("{ not valid json }".to_string()),
            body: None,
        };

        let result = fetch_raw(request).await;
        assert!(result.is_err());
        match result.err().unwrap() {
            ApiError::RequestError(s) => assert!(s.contains("Failed to parse JSON headers")), 
            e => panic!("Wrong error type: {:?}", e),
        }
    }
    
    // Тест: Невалидное значение заголовка
    #[tokio::test]
    async fn test_fetch_raw_invalid_header_value() {
        let request = NetworkRequest {
            url: "https://example.com".to_string(),
            method: "GET".to_string(),
            headers_json: Some(r#"{ "Valid-Name": "invalid\nvalue" }"#.to_string()), 
            body: None,
        };

        let result = fetch_raw(request).await;
        assert!(result.is_err());
        match result.err().unwrap() {
            ApiError::RequestError(s) => assert!(s.contains("Invalid header value")),
            e => panic!("Wrong error type: {:?}", e),
        }
    }
}
