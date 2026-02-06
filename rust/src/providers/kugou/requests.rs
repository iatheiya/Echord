// kugou/requests.rs

use super::models::{
    Candidate, DownloadLyricsResponse, KuGouError, Lyrics, SearchLyricsResponse, SearchSongResponse,
    SongInfo,
};
// MOVED_TO_MODULE: `CLIENT` и `parse_json_response`
// были перемещены в `core::http` и `core::json`.
use crate::core::{http, json};
use reqwest::Url;
use std::convert::TryFrom;

/// (Правило 2.5) Публичный FFI-интерфейс (контракт не изменен).
#[doc(hidden)] // Скрываем от `cargo doc`
pub async fn lyrics(
    artist: String,
    title: String,
    duration: u64,
) -> Result<String, KuGouError> {
    // (Правило 2.4) Логирование безопасно (только входные данные).
    log::debug!(
        "Rust: Запрос текста для {} - {} ({}ms)",
        artist,
        title,
        duration
    );

    let result = internal_lyrics_logic(&artist, &title, duration).await;

    match result {
        Ok(Some(lyrics)) => Ok(lyrics.0),
        Ok(None) => Err(KuGouError::NotFound),
        Err(e) => Err(e),
    }
}

/// (Правило 2.3) Внутренняя логика (семантика сохранена).
async fn internal_lyrics_logic(
    artist: &str,
    title: &str,
    duration: u64,
) -> Result<Option<Lyrics>, KuGouError> {
    let keyword = keyword(artist, title);
    let info_by_keyword = search_song(&keyword).await?;

    let ffi_duration = i64::try_from(duration).map_err(|_| {
        KuGouError::ParsingError(format!("Duration value is too large: {}", duration))
    })?;

    if !info_by_keyword.is_empty() {
        let mut tolerance = 0i64;

        while tolerance <= 5 {
            for info in &info_by_keyword {
                if info.duration >= ffi_duration - tolerance
                    && info.duration <= ffi_duration + tolerance
                {
                    if let Some(candidate) = search_lyrics_by_hash(&info.hash).await?.first() {
                        let lyrics =
                            download_lyrics(candidate.id, &candidate.access_key).await?;
                        return Ok(Some(lyrics.normalize()));
                    }
                }
            }
            tolerance += 1;
        }
    }

    if let Some(candidate) = search_lyrics_by_keyword(&keyword).await?.first() {
        let lyrics = download_lyrics(candidate.id, &candidate.access_key).await?;
        return Ok(Some(lyrics.normalize()));
    }

    Ok(None)
}

/// Загружает сам текст песни.
async fn download_lyrics(id: i64, access_key: &str) -> Result<Lyrics, KuGouError> {
    let base_url = Url::parse_with_params(
        "https://krcs.kugou.com/download",
        &[
            ("ver", "1"),
            ("man", "yes"),
            ("client", "pc"),
            ("fmt", "lrc"),
            ("id", &id.to_string()),
            ("accesskey", access_key),
        ],
    )
    .unwrap();

    // [РЕФАКТОРИНГ] Используем `core::http::fetch_json`
    let response = http::fetch_json::<DownloadLyricsResponse>(base_url).await?;

    // Логика, специфичная для KuGou (Base64 + UTF8)
    let decoded_bytes = base64::decode(&response.content)?;
    
    // (Правило 2.3) Семантика обработки FromUtf8Error сохранена
    let lyrics_str =
        String::from_utf8(decoded_bytes).map_err(|e| KuGouError::ParsingError(e.to_string()))?;

    Ok(Lyrics(lyrics_str))
}

/// Ищет кандидатов на текст по хэшу песни.
async fn search_lyrics_by_hash(hash: &str) -> Result<Vec<Candidate>, KuGouError> {
    let base_url = Url::parse_with_params(
        "https://krcs.kugou.com/search",
        &[
            ("ver", "1"),
            ("man", "yes"),
            ("client", "mobi"),
            ("hash", hash), // "hash" считается доверенным (из API)
        ],
    )
    .unwrap();

    // [РЕФАКТОРИНГ] Используем `core::http::fetch_text`
    let response_text = http::fetch_text(base_url).await?;

    // [РЕФАКТОРИНГ] Используем `core::json::parse_json_from_text`
    let parsed: SearchLyricsResponse =
        json::parse_json_from_text(&response_text, "search_lyrics_by_hash")?;
    Ok(parsed.candidates)
}

/// Ищет кандидатов на текст по ключевому слову.
async fn search_lyrics_by_keyword(keyword: &str) -> Result<Vec<Candidate>, KuGouError> {
    let base_url = Url::parse("https://krcs.kugou.com/search").unwrap();

    let mut url = base_url;
    url.query_pairs_mut()
        .append_pair("ver", "1")
        .append_pair("man", "yes")
        .append_pair("client", "mobi")
         // (Правило 4.2 Interfacing) `append_pair` корректно %-кодирует keyword
        .append_pair("keyword", keyword);

    // [РЕФАКТОРИНГ] Используем `core::http::fetch_text`
    let response_text = http::fetch_text(url).await?;

    // [РЕФАКТОРИНГ] Используем `core::json::parse_json_from_text`
    let parsed: SearchLyricsResponse =
        json::parse_json_from_text(&response_text, "search_lyrics_by_keyword")?;
    Ok(parsed.candidates)
}

/// Ищет информацию о песне (хэш) по ключевому слову.
async fn search_song(keyword: &str) -> Result<Vec<SongInfo>, KuGouError> {
    let base_url = Url::parse("https://mobileservice.kugou.com/api/v3/search/song").unwrap();

    let mut url = base_url;
    url.query_pairs_mut()
        .append_pair("version", "9108")
        .append_pair("plat", "0")
        .append_pair("pagesize", "8")
        .append_pair("showtype", "0")
        // (Правило 4.2 Interfacing) `append_pair` корректно %-кодирует keyword
        .append_pair("keyword", keyword);

    // [РЕФАКТОРИНГ] Используем `core::http::fetch_json`
    let response = http::fetch_json::<SearchSongResponse>(url).await?;

    Ok(response.data.info)
}

// --- Вспомогательные функции (специфичные для KuGou) ---

/// Эквивалент `String.extract` из Kotlin.
fn extract(s: &str, start_delimiter: &str, end_delimiter: char) -> (String, String) {
    if let Some(start_index) = s.find(start_delimiter) {
        if let Some(end_index_rel) = s[start_index..].find(end_delimiter) {
            let end_index_abs = start_index + end_index_rel;

            let mut main_part = String::with_capacity(s.len());
            main_part.push_str(&s[..start_index]);
            main_part.push_str(&s[end_index_abs + 1..]);

            let extracted_part = s[start_index + start_delimiter.len()..end_index_abs].to_string();

            return (main_part, extracted_part);
        }
    }
    (s.to_string(), "".to_string())
}

/// Эквивалент `keyword` из Kotlin.
fn keyword(artist: &str, title: &str) -> String {
    let (new_title, featuring) = extract(title, " (feat. ", ')');

    let new_artist = if featuring.is_empty() {
        artist.to_string()
    } else {
        format!("{}, {}", artist, featuring)
    };

    let new_artist = new_artist
        .replace(", ", "、")
        .replace(" & ", "、")
        .replace('.', "");

    format!("{} - {}", new_artist, new_title)
}

// (Правило 1.4) Секция тестов
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_simple() {
        let artist = "Artist";
        let title = "Title";
        assert_eq!(keyword(artist, title), "Artist - Title");
    }

    #[test]
    fn test_keyword_featuring() {
        let artist = "Artist";
        let title = "Title (feat. Other)";
        assert_eq!(keyword(artist, title), "Artist、Other - Title");
    }

    #[test]
    fn test_keyword_multiple_artists() {
        let artist = "Artist1, Artist2 & Artist3";
        let title = "Title (feat. Other1, Other2)";
        assert_eq!(
            keyword(artist, title),
            "Artist1、Artist2、Artist3、Other1、Other2 - Title"
        );
    }
    
    #[test]
    fn test_extract() {
        let (main, extracted) = extract("Title (feat. Other)", " (feat. ", ')');
        assert_eq!(main, "Title");
        assert_eq!(extracted, "Other");
        
        let (main, extracted) = extract("Title (no feat)", " (feat. ", ')');
        assert_eq!(main, "Title (no feat)");
        assert_eq!(extracted, "");
    }
}
