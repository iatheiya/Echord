// Файл: core/json.rs
// Регламент 1.2, Уровень 1: Унифицированный хелпер для парсинга JSON.

use super::error::CoreError;

/// Унифицированная функция парсинга JSON.
/// @param response_text Сырая строка ответа.
/// @param context Контекст для логирования (например, "YouTube Response").
pub fn parse_json_from_text<T: for<'de> serde::Deserialize<'de>>(
    response_text: &str,
    context: &str, 
) -> Result<T, CoreError> {
    serde_json::from_str(response_text).map_err(|e| {
        // (Rule 2.4) Логирование безопасно.
        log::warn!(
            "Failed to parse JSON for {}: {}",
            context,
            e // Ошибка serde безопасна для логирования
        );
        CoreError::from(e)
    })
}

// --- Тесты (Rule 1.4) ---
#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize, Debug, PartialEq)]
    struct TestStruct {
        id: i32,
        name: String,
    }

    #[test]
    fn test_parse_json_success() {
        let json = r#"{"id": 1, "name": "Test"}"#;
        let result = parse_json_from_text::<TestStruct>(json, "test_success");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_json_error() {
        let json = r#"{id: 1, name: "Test"}"#; // Невалидный JSON
        let result = parse_json_from_text::<TestStruct>(json, "test_error");
        assert!(result.is_err());
    }
}
