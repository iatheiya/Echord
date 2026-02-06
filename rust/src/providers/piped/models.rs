=== FILE: src/piped/models.rs ===
// ИЗМЕНЕННЫЙ ФАЙЛ: Внесено исправление для надежного парсинга URL.
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use url::Url;
use uuid::Uuid;

// --- Ошибка ---

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum PipedError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("JSON parsing failed: {0}")]
    Parse(#[from] serde_json::Error),

    #[error("API error: {0}")]
    Api(String),

    #[error("URL parsing failed: {0}")]
    UrlParse(#[from] url::ParseError),
}

// --- Объект Сессии ---

#[derive(Debug, Clone, uniffi::Object)]
pub struct Session {
    // Владеет Arc для безопасного FFI-владения
    pub(crate) inner: Arc<InnerSession>,
}

#[derive(Debug)]
pub(crate) struct InnerSession {
    // reqwest::Client является дешево клонируемым
    pub(crate) client: reqwest::Client,
    pub(crate) api_base_url: Url,
    pub(crate) token: String,
}

// --- Модели API (для FFI, соответствуют UDL) ---

#[derive(Debug, Clone, uniffi::Record)]
pub struct Instance {
    pub name: String,
    pub api_base_url: String,
    pub locations_formatted: String,
    pub version: String,
    pub up_to_date: bool,
    pub is_cdn: bool,
    pub user_count: i64,
    pub last_checked: i64, // Unix timestamp (в секундах)
    pub has_cache: bool,
    pub uses_s3: bool,
    pub image_proxy_base_url: String,
    pub registration_disabled: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone, uniffi::Record)]
pub struct PlaylistPreview {
    pub id: String, // UUID в виде строки
    pub name: String,
    pub description: Option<String>,
    pub thumbnail_url: String,
    pub video_count: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone, uniffi::Record)]
pub struct CreatedPlaylist {
    pub id: String, // UUID в виде строки
}

#[derive(Debug, Deserialize, Serialize, Clone, uniffi::Record)]
pub struct Playlist {
    pub name: String,
    pub thumbnail_url: String,
    pub description: Option<String>,
    pub banner_url: Option<String>,
    pub video_count: i32,
    pub videos: Vec<PlaylistVideo>,
}

#[derive(Debug, Deserialize, Serialize, Clone, uniffi::Record)]
pub struct PlaylistVideo {
    pub url: String, // Относительный или полный URL
    pub title: String,
    pub thumbnail_url: String,
    pub uploader_name: String,
    pub uploader_url: String, // Относительный или полный URL
    pub uploader_avatar_url: String,
    pub duration_seconds: i64,
}

// Вспомогательная структура для десериализации (не FFI)
// Нужна для корректной обработки сложных типов, например, `DateTime<Utc>`
pub(crate) mod internal {
    use super::*;
    use chrono::{DateTime, Utc};
    
    #[derive(Debug, Deserialize)]
    pub struct ApiInstance {
        pub name: String,
        pub api_base_url: String,
        pub locations_formatted: String,
        pub version: String,
        pub up_to_date: bool,
        pub is_cdn: bool,
        pub user_count: i64,
        #[serde(with = "chrono::serde::ts_seconds")]
        pub last_checked: DateTime<Utc>,
        pub has_cache: bool,
        pub uses_s3: bool,
        pub image_proxy_base_url: String,
        pub registration_disabled: bool,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct TokenResponse {
        pub token: String,
    }
    
    #[derive(Debug, Deserialize, Serialize)]
    pub struct OkMessage {
        pub result: String,
    }
}

// --- FFI-методы для PlaylistVideo (логика, перенесенная из Kotlin) ---

/// Извлекает ID видео (параметр 'v') из поля URL.
#[uniffi::export]
pub fn video_get_id(video: PlaylistVideo) -> Option<String> {
    // Используем 'http://example.com' как базовый URL для парсинга относительных путей.
    let base = Url::parse("http://example.com").ok()?; 
    // Пытаемся объединить базовый URL с полем video.url
    let parsed_url = base.join(&video.url).ok()?;

    // Надежно ищем параметр 'v' среди всех пар запрос-значение.
    parsed_url.query_pairs()
        .find(|(key, _)| key == "v")
        .map(|(_, value)| value.to_string())
}

/// Извлекает ID канала/пользователя из поля uploader_url.
#[uniffi::export]
pub fn video_get_uploader_id(video: PlaylistVideo) -> Option<String> {
    // Используем 'http://example.com' как базовый URL для парсинга относительных путей.
    let base = Url::parse("http://example.com").ok()?;
    // Пытаемся объединить базовый URL с полем video.uploader_url
    let parsed_url = base.join(&video.uploader_url).ok()?;

    // Ожидаемый формат: /channel/UC_ID, /user/USER_ID.
    let path_segments: Vec<&str> = parsed_url
        .path_segments()?
        .filter(|s| !s.is_empty())
        .collect();

    // Проверяем, что у нас есть хотя бы два сегмента (например, ["channel", "UC_ID"])
    if path_segments.len() >= 2 {
        if path_segments[0] == "channel" || path_segments[0] == "user" {
            // Возвращаем второй сегмент
            return Some(path_segments[1].to_string());
        }
    }

    None
}

/// Переводит длительность из секунд в миллисекунды.
#[uniffi::export]
pub fn video_get_duration_ms(video: PlaylistVideo) -> i64 {
    video.duration_seconds * 1000
}

// --- Юнит-тесты (для покрытия краевых случаев парсинга URL) ---
#[cfg(test)]
mod tests {
    use super::*;

    fn mock_video(url: &str, uploader_url: &str) -> PlaylistVideo {
        PlaylistVideo {
            url: url.to_string(),
            title: "".to_string(),
            thumbnail_url: "".to_string(),
            uploader_name: "".to_string(),
            uploader_url: uploader_url.to_string(),
            uploader_avatar_url: "".to_string(),
            duration_seconds: 0,
        }
    }

    #[test]
    fn test_video_id_parser_robust() {
        // Тест 1: Относительный URL с query param
        let video1 = mock_video("/watch?v=dQw4w9WgXcQ", "");
        assert_eq!(video_get_id(video1), Some("dQw4w9WgXcQ".to_string()));

        // Тест 2: Полный URL с другими параметрами
        let video2 = mock_video("https://www.youtube.com/watch?list=...&v=abcdef123&t=1", "");
        assert_eq!(video_get_id(video2), Some("abcdef123".to_string()));
        
        // Тест 3: Некорректный URL
        let video3 = mock_video("/w?v=123", "");
        assert_eq!(video_get_id(video3), Some("123".to_string()));
        
        // Тест 4: Некорректный хост (должен вернуть None, так как v не найден)
        let video4 = mock_video("https://example.com", "");
        assert_eq!(video_get_id(video4), None);
        
        // Тест 5: URL без параметра 'v'
        let video5 = mock_video("/watch?list=123", "");
        assert_eq!(video_get_id(video5), None);
    }

    #[test]
    fn test_uploader_id_parser_robust() {
        // Тест 1: Относительный URL (channel)
        let video1 = mock_video("", "/channel/UC-lHJZR3Gqxm24_Vd_AJ5Yw");
        assert_eq!(
            video_get_uploader_id(video1),
            Some("UC-lHJZR3Gqxm24_Vd_AJ5Yw".to_string())
        );

        // Тест 2: Относительный URL (user)
        let video2 = mock_video("", "/user/Username123");
        assert_eq!(
            video_get_uploader_id(video2),
            Some("Username123".to_string())
        );
        
        // Тест 3: Полный URL (channel)
        let video3 = mock_video("", "https://www.youtube.com/channel/UC-lHJZR3Gqxm24_Vd_AJ5Yw/videos");
        assert_eq!(
            video_get_uploader_id(video3),
            Some("UC-lHJZR3Gqxm24_Vd_AJ5Yw".to_string())
        );

        // Тест 4: Только /channel
        let video4 = mock_video("", "/channel/");
        assert_eq!(video_get_uploader_id(video4), None);

        // Тест 5: Некорректный URL
        let video5 = mock_video("", "https://example.com/not_channel/id");
        assert_eq!(video_get_uploader_id(video5), None);
    }
}
