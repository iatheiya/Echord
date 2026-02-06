// ------------------------------------
// Файл: src/sponsorblock/models.rs
// ------------------------------------
use crate::core::CoreError; // [MOVED_TO_MODULE] Зависимость от 'core'
use serde::{Deserialize, Serialize};
use thiserror::Error;

// 1. FFI Ошибка
#[derive(Debug, Error, uniffi::Error)]
pub enum SponsorBlockError {
    #[error("Failed to build request")]
    Request,
    #[error("Failed to (de)serialize data")]
    Serialization,
    #[error("Network request failed")]
    Network,
    #[error("API returned invalid segment data")]
    InvalidSegmentData,
    #[error("Failed to parse base URL")]
    UrlParsing,
}

// (Правило 2.3 Семантика): Преобразование общих ошибок в доменные
impl From<CoreError> for SponsorBlockError {
    fn from(err: CoreError) -> Self {
        match err {
            CoreError::Request => SponsorBlockError::Request,
            CoreError::Serialization => SponsorBlockError::Serialization,
            CoreError::Network => SponsorBlockError::Network,
            CoreError::UrlParsing => SponsorBlockError::UrlParsing,
        }
    }
}

// 2. FFI Enums (с serde для внутреннего использования)
#[derive(Debug, Serialize, Deserialize, uniffi::Enum, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    #[serde(rename = "sponsor")]
    Sponsor,
    #[serde(rename = "selfpromo")]
    SelfPromotion,
    #[serde(rename = "interaction")]
    Interaction,
    #[serde(rename = "intro")]
    Intro,
    #[serde(rename = "outro")]
    Outro,
    #[serde(rename = "preview")]
    Preview,
    #[serde(rename = "music_offtopic")]
    OfftopicMusic,
    #[serde(rename = "filler")]
    Filler,
    #[serde(rename = "poi_highlight")]
    PoiHighlight,
}

impl Category {
    /// Возвращает строковое представление, ожидаемое API
    pub(crate) fn serial_name(&self) -> &'static str {
        match self {
            Category::Sponsor => "sponsor",
            Category::SelfPromotion => "selfpromo",
            Category::Interaction => "interaction",
            Category::Intro => "intro",
            Category::Outro => "outro",
            Category::Preview => "preview",
            Category::OfftopicMusic => "music_offtopic",
            Category::Filler => "filler",
            Category::PoiHighlight => "poi_highlight",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, uniffi::Enum, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    #[serde(rename = "skip")]
    Skip,
    #[serde(rename = "mute")]
    Mute,
    #[serde(rename = "full")]
    Full,
    #[serde(rename = "poi")]
    POI,
    #[serde(rename = "chapter")]
    Chapter,
}

impl Action {
    /// Возвращает строковое представление, ожидаемое API
    pub(crate) fn serial_name(&self) -> &'static str {
        match self {
            Action::Skip => "skip",
            Action::Mute => "mute",
            Action::Full => "full",
            Action::POI => "poi",
            Action::Chapter => "chapter",
        }
    }
}

// 3. FFI Record (публичная модель)
#[derive(Debug, uniffi::Record, Clone, PartialEq)]
pub struct Segment {
    pub start_time: f64,
    pub end_time: f64,
    pub uuid: Option<String>,
    pub category: Category,
    pub action: Action,
    pub description: String,
}

// 4. Внутренняя структура для Deserialization (ApiSegment)
#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Поля используются `serde`
pub(crate) struct ApiSegment {
    segment: Vec<f64>,
    #[serde(rename = "UUID")]
    uuid: Option<String>,
    category: Category,
    #[serde(rename = "actionType")]
    action: Action,
    description: String,
}

// 5. Безопасное преобразование из ApiSegment -> Segment
impl TryFrom<ApiSegment> for Segment {
    type Error = SponsorBlockError;

    fn try_from(api_segment: ApiSegment) -> Result<Self, Self::Error> {
        // (Правило 2.3 Семантика): Безопасная проверка границ массива
        let start_time = *api_segment
            .segment
            .get(0)
            .ok_or(SponsorBlockError::InvalidSegmentData)?;
        let end_time = *api_segment
            .segment
            .get(1)
            .ok_or(SponsorBlockError::InvalidSegmentData)?;

        Ok(Segment {
            start_time,
            end_time,
            uuid: api_segment.uuid,
            category: api_segment.category,
            action: api_segment.action,
            description: api_segment.description,
        })
    }
}

// 6. Тесты
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // (Правило 1.4 Тесты): Тест на корректность парсинга
    fn test_segment_parsing_ok() {
        let api_seg = ApiSegment {
            segment: vec![10.5, 20.0],
            uuid: Some("uuid-123".to_string()),
            category: Category::Sponsor,
            action: Action::Skip,
            description: "Test".to_string(),
        };

        let segment = Segment::try_from(api_seg).unwrap();
        assert_eq!(segment.start_time, 10.5);
        assert_eq!(segment.end_time, 20.0);
        assert_eq!(segment.uuid, Some("uuid-123".to_string()));
    }

    #[test]
    // (Правило 1.4 Тесты): Тест на краевой случай (короткий массив)
    fn test_invalid_segment_parsing_short() {
        let api_seg = ApiSegment {
            segment: vec![10.5], // Слишком короткий
            uuid: None,
            category: Category::Sponsor,
            action: Action::Skip,
            description: "Test".to_string(),
        };

        let result = Segment::try_from(api_seg);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            SponsorBlockError::InvalidSegmentData
        ));
    }

    #[test]
    // (Правило 1.4 Тесты): Тест на краевой случай (пустой массив)
    fn test_invalid_segment_parsing_empty() {
        let api_seg = ApiSegment {
            segment: vec![], // Пустой
            uuid: None,
            category: Category::Sponsor,
            action: Action::Skip,
            description: "Test".to_string(),
        };

        let result = Segment::try_from(api_seg);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            SponsorBlockError::InvalidSegmentData
        ));
    }
}
