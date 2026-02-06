/*
 * ФАЙЛ: models.rs
 * (ИСПРАВЛЕН ПОСЛЕ АУДИТА: Устранены семантическая ошибка и ошибка производительности)
 */
use std::fmt::{Display, Formatter};
use uuid::Uuid;
use url::Url;
// Используем time::PrimitiveDateTime для точного соответствия kotlinx.datetime.LocalDateTime
use time::{PrimitiveDateTime, format_description::FormatItem};
// (FIXED: [MEDIUM] Добавлен Lazy для кеширования парсинга формата)
use once_cell::sync::Lazy;


// --- Implementations for SerializableUUID ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[allow(dead_code)] // Разрешаем, если тип не используется в запросах
pub struct SerializableUUID(pub Uuid);

impl Display for SerializableUUID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_hyphenated().to_string())
    }
}

// FFI-взаимодействие (граница): из String в NewType
impl TryFrom<String> for SerializableUUID {
    type Error = uuid::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // [SAFETY]: Uuid::parse_str - безопасная операция парсинга.
        Ok(SerializableUUID(Uuid::parse_str(&value)?))
    }
}

// FFI-взаимодействие (граница): из NewType в String
impl From<SerializableUUID> for String {
    fn from(value: SerializableUUID) -> Self {
        value.to_string()
    }
}

// --- Implementations for SerializableUrl ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub struct SerializableUrl(pub Url);

impl Display for SerializableUrl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.as_str())
    }
}

impl TryFrom<String> for SerializableUrl {
    type Error = url::ParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // [SAFETY]: Url::parse - безопасная операция парсинга.
        Ok(SerializableUrl(Url::parse(&value)?))
    }
}

impl From<SerializableUrl> for String {
    fn from(value: SerializableUrl) -> Self {
        value.to_string()
    }
}

// --- Implementations for SerializableIso8601Date (LocalDateTime) ---

// (FIXED: [MEDIUM] (Rule 4.2) Кешируем парсинг формата)
static FALLBACK_FORMAT_SEC: Lazy<Vec<FormatItem<'static>>> = Lazy::new(|| {
    time::format_description::parse("[year]-[month]-[day]T[hour]:[minute]:[second]")
        .expect("Static format string failed to parse")
});

// Семантическое соответствие (Правило 2.3):
// Kotlin: LocalDateTime.parse(str.removeSuffix("Z"))
// Rust:   time::PrimitiveDateTime (локальное время без смещения)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub struct SerializableIso8601Date(pub PrimitiveDateTime);

impl Display for SerializableIso8601Date {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Формат по умолчанию, похожий на kotlinx (e.g., 2025-11-11T13:22:27.0)
        write!(f, "{}", self.0.to_string())
    }
}

impl TryFrom<String> for SerializableIso8601Date {
    type Error = time::error::Parse;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        
        // (FIXED: [HIGH] (Rule 2.3) Репликация семантики Kotlin `removeSuffix("Z")`)
        let value = value.strip_suffix('Z').unwrap_or(&value);

        // [SAFETY]: time::PrimitiveDateTime::parse - безопасная операция парсинга.
        let default_iso_format = time::format_description::well_known::Iso8601::DEFAULT;

        match PrimitiveDateTime::parse(value, &default_iso_format) {
            Ok(dt) => Ok(SerializableIso8601Date(dt)),
            Err(e) => {
                // Фоллбэк для форматов без долей секунд
                // (FIXED: [MEDIUM] Используем кешированный формат)
                match PrimitiveDateTime::parse(value, &FALLBACK_FORMAT_SEC) {
                    Ok(dt) => Ok(SerializableIso8601Date(dt)),
                    Err(_) => Err(e) // Возвращаем оригинальную ошибку парсинга
                }
            }
        }
    }
}

impl From<SerializableIso8601Date> for String {
    fn from(value: SerializableIso8601Date) -> Self {
        value.to_string()
    }
}

// --- Тесты (Правило 1.4) ---
#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_uuid_conversion() {
        let uuid_str = "a1b2c3d4-e5f6-7890-1234-567890abcdef";
        let newtype: SerializableUUID = String::from(uuid_str).try_into().unwrap();
        assert_eq!(newtype.0, Uuid::from_str(uuid_str).unwrap());
        let converted_str: String = newtype.into();
        assert_eq!(converted_str, uuid_str);
    }

    #[test]
    fn test_url_conversion() {
        let url_str = "https://example.com/path";
        let newtype: SerializableUrl = String::from(url_str).try_into().unwrap();
        assert_eq!(newtype.0, Url::parse(url_str).unwrap());
        let converted_str: String = newtype.into();
        assert_eq!(converted_str, url_str);
    }

    #[test]
    fn test_datetime_conversion() {
        // Формат с долями секунд
        let dt_str_nano = "2025-11-11T13:22:27.123456789";
        let newtype_nano: SerializableIso8601Date = String::from(dt_str_nano).try_into().unwrap();
        assert_eq!(newtype_nano.0.year(), 2025);
        assert_eq!(newtype_nano.0.nanosecond(), 123456789);

        // Формат без долей секунд (проверяется фоллбэком)
        let dt_str_sec = "2025-11-11T13:22:27";
        let newtype_sec: SerializableIso8601Date = String::from(dt_str_sec).try_into().unwrap();
        assert_eq!(newtype_sec.0.year(), 2025);
        assert_eq!(newtype_sec.0.nanosecond(), 0);
    }

    // (NEW: Тест, покрывающий исправление семантической ошибки [HIGH])
    #[test]
    fn test_datetime_conversion_with_z() {
        // Строка с 'Z' должна парситься так же, как строка без 'Z',
        // согласно семантике Kotlin (removeSuffix)
        let dt_str_z = "2025-11-11T13:22:27.123Z";
        let newtype_z: SerializableIso8601Date = String::from(dt_str_z).try_into().unwrap();

        let dt_str_no_z = "2025-11-11T13:22:27.123";
        let newtype_no_z: SerializableIso8601Date = String::from(dt_str_no_z).try_into().unwrap();

        // Они должны быть идентичны
        assert_eq!(newtype_z, newtype_no_z);
        assert_eq!(newtype_z.0.year(), 2025);
        assert_eq!(newtype_z.0.nanosecond(), 123000000); // .123
    }

    #[test]
    fn test_datetime_bad_format() {
        let dt_str = "not a date";
        assert!(String::from(dt_str).try_into::<SerializableIso8601Date>().is_err());
    }
}
