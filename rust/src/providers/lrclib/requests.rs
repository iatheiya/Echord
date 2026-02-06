// Файл: requests.rs
// Реализация FFI, HTTP-клиент и бизнес-логика
// [АУДИТ 5.3] Исправлены пути импорта

// [АУДИТ 5.3. ИСПРАВЛЕНО]
// Используем 'super::' для доступа к смежному модулю 'core'
use super::core::client::create_http_client;
use super::core::error::ApiError;
// [АУДИТ 5.3. КОНЕЦ ИСПРАВЛЕНИЯ]

use crate::models::{Lyrics, Track};
use log::debug;
use reqwest::header::{HeaderMap, HeaderValue}; // USER_AGENT удален, он в core
use std::sync::Arc;
use std::time::Duration;

// AGENT (User-Agent) перенесен в core/client.rs
// Это специфичный заголовок для Lrclib
const AGENT_HEADER: &str = "ViTune (https://github.com/25huizengek1/ViTune)";
const BASE_URL: &str = "https://lrclib.net";

/// FFI-объект, хранящий HTTP-клиент
#[derive(Debug, uniffi::Object)]
pub struct LrcLib {
    client: reqwest::Client,
}

#[uniffi::export]
impl LrcLib {
    /// Конструктор для UniFFI
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Реализация `bestLyrics` из Kotlin
    //
    // АУДИТ 2.5 (Интерфейс):
    // Это публичный FFI-контракт.
    // Возвращает 'ApiError' (из 'core.udl').
    #[uniffi::method]
    pub async fn best_lyrics(
        &self,
        artist: String,
        title: String,
        duration_ms: u64,
        album: Option<String>,
        synced: bool,
    ) -> Result<Option<Lyrics>, ApiError> {
        // АУДИТ 2.4 (Логирование): Уровень debug, не раскрывает PII.
        debug!(
            "Fetching best lyrics for: {} - {} (Album: {:?})",
            artist, title, album
        );

        // 1. Выполняем поиск (аналог `lyrics()` в Kotlin)
        let tracks = self
            .lyrics_by_meta(&artist, &title, album.as_deref(), synced)
            .await?;

        // 2. Находим лучший трек (аналог `bestMatchingFor()`)
        let duration = Duration::from_millis(duration_ms);
        let best_track = best_matching_for(&tracks, &title, duration);

        // 3. Формируем результат (аналог `mapCatching { ... }`)
        let lyrics = best_track.map(|track| {
            let text = if synced {
                track.synced_lyrics.clone()
            } else {
                track.plain_lyrics.clone()
            };
            
            // АУДИТ 4.2 (Performance): .clone() здесь необходим
            // для передачи владения String в новую структуру 'Lyrics',
            // так как 'best_track' является &Track.
            text.map(|t| Lyrics { text: t, synced })
        }).flatten(); // flatten() для Option<Option<Lyrics>> -> Option<Lyrics>

        Ok(lyrics)
    }
}

// ## Управление Ресурсами (Правило 1.4 / 2.0.2) ##
// Клиент создается один раз с использованием core-фабрики
impl Default for LrcLib {
    fn default() -> Self {
        let mut headers = HeaderMap::new();
        // [АУДИТ 1.2] Добавляем только специфичные заголовки
        headers.insert("Lrclib-Client", HeaderValue::from_static(AGENT_HEADER));

        // [АУДИТ 1.2] Используем core-фабрику для создания клиента
        // Общие заголовки (User-Agent) и таймаут будут применены в 'core'
        let client = create_http_client(Some(headers)); 

        Self { client }
    }
}

// ## Внутренняя Логика API ##

impl LrcLib {
    /// Приватный метод, аналог `queryLyrics(artist, title, album)`
    async fn query_lyrics_by_meta(
        &self,
        artist: &str,
        title: &str,
        album: Option<&str>,
    ) -> Result<Vec<Track>, ApiError> {
        let mut params = vec![
            ("track_name", title),
            ("artist_name", artist),
        ];
        if let Some(a) = album {
            params.push(("album_name", a));
        }

        let url = format!("{}/api/search", BASE_URL);
        debug!("Querying lrclib: {} with params {:?}", url, params);

        let response = self
            .client
            .get(url)
            .query(&params) // reqwest кодирует параметры (Безопасность)
            .send()
            .await? // -> ApiError::RequestError
            .error_for_status()?; // -> ApiError::RequestError

        // АУДИТ 2.3 (Семантика): .json() теперь вернет ApiError::ParseError
        // благодаря From<serde_json::Error> в 'core/error.rs'
        Ok(response.json::<Vec<Track>>().await?) // -> ApiError::ParseError
    }

    /// Приватный метод, аналог `lyrics(artist, ...)`
    async fn lyrics_by_meta(
        &self,
        artist: &str,
        title: &str,
        album: Option<&str>,
        synced: bool,
    ) -> Result<Vec<Track>, ApiError> {
        let list = self
            .query_lyrics_by_meta(artist, title, album)
            .await?;

        // Фильтрация, как в Kotlin
        let filtered = list.into_iter().filter(|track| {
            if synced {
                track.synced_lyrics.is_some()
            } else {
                track.plain_lyrics.is_some()
            }
        }).collect();

        Ok(filtered)
    }
}

/// Порт `bestMatchingFor` из `Track.kt`
fn best_matching_for<'a>(
    tracks: &'a [Track],
    title: &str,
    duration: Duration,
) -> Option<&'a Track> {
    let target_secs = duration.as_secs();

    // 1. Поиск по точному совпадению длительности
    if let Some(track) = tracks.iter().find(|t| t.duration as u64 == target_secs) {
        return Some(track);
    }

    // 2. Поиск по минимальной разнице в длине названия
    // АУДИТ 2.3 (Семантика): Используем .chars().count() (как .length в Kotlin)
    // вместо .len() (байты UTF-8).
    let title_len = title.chars().count() as isize;
    tracks.iter().min_by_key(|t| {
        (t.track_name.chars().count() as isize - title_len).abs()
    })
}


// ## Тесты (Правило 1.4) ##
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Track;
    
    // ПРИМЕЧАНИЕ: Этим тестам потребуется 'mock' модуля 'core'
    // или включение в 'crate::core' при 'cfg(test)'.
    // Для данного аудита мы предполагаем, что 'super::core' недоступен
    // в 'cargo test', и фокусируемся на логике 'best_matching_for'.

    #[test]
    fn test_best_matching_for() {
        let tracks = vec![
            Track { id: 1, track_name: "Song Title".to_string(), artist_name: "Artist".to_string(), duration: 180.0, plain_lyrics: None, synced_lyrics: None },
            Track { id: 2, track_name: "Song Title (Remix)".to_string(), artist_name: "Artist".to_string(), duration: 240.0, plain_lyrics: None, synced_lyrics: None },
            Track { id: 3, track_name: "Song".to_string(), artist_name: "Artist".to_string(), duration: 185.0, plain_lyrics: None, synced_lyrics: None },
        ];

        let title = "Song Title";
        
        // Тест 1: Точное совпадение по длительности
        let duration1 = Duration::from_secs(180);
        assert_eq!(best_matching_for(&tracks, title, duration1).unwrap().id, 1);
        
        // Тест 2: Нет совпадения по длительности, ищем по названию
        let duration2 = Duration::from_secs(190);
        // "Song Title" (len 10) vs title "Song Title" (len 10) -> diff 0
        assert_eq!(best_matching_for(&tracks, title, duration2).unwrap().id, 1);
    }

    #[test]
    fn test_best_matching_for_utf8() {
        // "Beyoncé" (7 chars, 8 bytes)
        // "Beyonce" (7 chars, 7 bytes)
        let tracks = vec![
            Track { id: 1, track_name: "Beyoncé".to_string(), artist_name: "Artist".to_string(), duration: 180.0, plain_lyrics: None, synced_lyrics: None },
            Track { id: 2, track_name: "A".to_string(), artist_name: "Artist".to_string(), duration: 240.0, plain_lyrics: None, synced_lyrics: None },
        ];

        let title = "Beyonce"; // 7 chars
        let duration = Duration::from_secs(190); // Несовпадающая длительность

        // Rust (Fix): abs(7 - 7) = 0
        assert_eq!(best_matching_for(&tracks, title, duration).unwrap().id, 1);
    }
}
