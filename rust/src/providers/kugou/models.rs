// kugou/models.rs

// MOVED_TO_MODULE: `From<reqwest::Error>` и `From<serde_json::Error>`
// были перемещены в `core::error::CoreError`.
use crate::core::error::CoreError;
use serde::Deserialize;

// ### Модели данных для Serde ###
#[derive(Debug, Deserialize)]
pub(super) struct SearchLyricsResponse {
    pub candidates: Vec<Candidate>,
}

#[derive(Debug, Deserialize)]
pub(super) struct Candidate {
    pub id: i64,
    #[serde(rename = "accesskey")]
    pub access_key: String,
    pub duration: i64,
}

#[derive(Debug, Deserialize)]
pub(super) struct DownloadLyricsResponse {
    pub content: String, // Это Base64
}

#[derive(Debug, Deserialize)]
pub(super) struct SearchSongResponse {
    pub data: SearchSongData,
}

#[derive(Debug, Deserialize)]
pub(super) struct SearchSongData {
    pub info: Vec<SongInfo>,
}

#[derive(Debug, Deserialize)]
pub(super) struct SongInfo {
    pub duration: i64,
    pub hash: String,
}

// ### Структура для текстов и нормализации ###
#[derive(Debug)]
pub(super) struct Lyrics(pub String);

impl Lyrics {
    /// (Правило 2.3: Семантика)
    /// Логика нормализации полностью сохранена.
    #[allow(clippy::all)]
    pub(super) fn normalize(self) -> Lyrics {
        let mut to_drop = 0;
        let mut maybe_to_drop = 0;

        let text = self.0.replace("\r\n", "\n");
        let text = text.trim();

        for line in text.lines() {
            let line_len = line.len() + 1; // +1 для \n

            if line.starts_with("[ti:")
                || line.starts_with("[ar:")
                || line.starts_with("[al:")
                || line.starts_with("[by:")
                || line.starts_with("[hash:")
                || line.starts_with("[sign:")
                || line.starts_with("[qq:")
                || line.starts_with("[total:")
                || line.starts_with("[offset:")
                || line.starts_with("[id:")
                || contains_at(line, "]Written by：", 9)
                || contains_at(line, "]Lyrics by：", 9)
                || contains_at(line, "]Composed by：", 9)
                || contains_at(line, "]Producer：", 9)
                || contains_at(line, "]作曲 : ", 9)
                || contains_at(line, "]作词 : ", 9)
            {
                to_drop += line_len + maybe_to_drop;
                maybe_to_drop = 0;
            } else if maybe_to_drop == 0 {
                maybe_to_drop = line_len;
            } else {
                maybe_to_drop = 0;
                break;
            }
        }

        let final_text = text
            .get(to_drop + maybe_to_drop..)
            .unwrap_or_default()
            .replace("&apos;", "'");

        Lyrics(final_text)
    }
}

/// Вспомогательная функция (семантика сохранена).
fn contains_at(s: &str, pattern: &str, index: usize) -> bool {
    s.get(index..)
        .map_or(false, |suffix| suffix.starts_with(pattern))
}

// ### Определение FFI Ошибки ###

/// (Правило 2.5) Публичный контракт FFI-ошибки.
#[derive(Debug, thiserror::Error, uniffi::Error)]
#[uniffi(flat_error)]
pub enum KuGouError {
    #[error("Lyrics not found")]
    NotFound,
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Parsing error: {0}")]
    ParsingError(String),
    #[error("Base64 decoding error: {0}")]
    Base64Error(String),
}

// (Правило 1.2) Конверсия из `CoreError`.
impl From<CoreError> for KuGouError {
    fn from(err: CoreError) -> Self {
        match err {
            CoreError::NetworkError(s) => KuGouError::NetworkError(s),
            CoreError::ParsingError(s) => KuGouError::ParsingError(s),
        }
    }
}

// Специфичная для KuGou ошибка
impl From<base64::DecodeError> for KuGouError {
    fn from(err: base64::DecodeError) -> Self {
        KuGouError::Base64Error(err.to_string())
    }
}
