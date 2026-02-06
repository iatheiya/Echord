// requests.rs
use std::time::Duration;
use futures::FutureExt; // [НОВОЕ] (Rule 2.1) Для .catch_unwind().await
use tokio;
use log::{debug, warn};

// [ИЗМЕНЕНО] Импорт `core` и ошибок/моделей
use crate::models::{Language, TranslateError};
use crate::core::core::{self, CoreError};

// --- Константы (без изменений) ---
const DT_PARAMS: [&str; 10] = ["at", "bd", "ex", "ld", "md", "qca", "rw", "rm", "ss", "t"];
const DEFAULT_HOST: &str = "translate.googleapis.com";
const MAX_RETRIES: u32 = 2;

// --- MOVED_TO_MODULE (core) ---
// HTTP_CLIENT, TOKIO_RUNTIME, TIMEOUTS, USER_AGENT
// ---------------------------------


// Основная асинхронная логика перевода
async fn translate_inner(
    text: String,
    from: Language,
    to: Language,
) -> Result<String, TranslateError> {
    if to == Language::Auto {
        return Err(TranslateError::TargetLanguageAuto); // (Rule 2.3) Семантика сохранена
    }

    let url = format!(
        "https://{}/translate_a/single",
        DEFAULT_HOST
    );

    let mut query_params = vec![
        ("client", "gtx"),
        ("ie", "utf-8"),
        ("oe", "utf-8"),
        ("otf", "1"),
        ("ssel", "0"),
        ("tsel", "0"),
        ("sl", from.code()),
        ("tl", to.code()),
        ("hl", to.code()),
        ("q", text.as_str()),
    ];

    for dt in DT_PARAMS.iter() {
        query_params.push(("dt", dt));
    }

    let mut retries = 0;
    let mut last_error: Option<TranslateError> = None;

    // Цикл ретраев (Rule 2.3) - семантика Ktor retry сохранена
    loop {
        // [ИЗМЕНЕНО] Вызов общего `core` хелпера
        match core::http_get_text(&url, &query_params).await {
            // 1. Успешный запрос
            Ok(response_text) => {
                // `http_get_text` в `core` уже проверил статусы 4xx/5xx
                return Ok(response_text);
            }
            
            // 5. Ошибка сети/IO/статуса из `core`
            Err(e) => {
                let core_error_msg = e.to_string();
                warn!("Ошибка Core: {}. Попытка #{}", core_error_msg, retries);
                
                // (Rule 2.3) Проверяем семантику ретраев
                let should_retry = if let CoreError::Network(req_err) = e {
                    // Не повторять, если это ошибка статуса (4xx/5xx)
                    // ИЛИ если это ошибка сервера (5xx) - тогда повторять
                    !req_err.is_status() || req_err.status().map_or(true, |s| s.is_server_error())
                } else {
                    true // Повторяем ошибки парсинга (маловероятно)
                };

                last_error = Some(TranslateError::CoreError { message: core_error_msg });

                if !should_retry {
                    debug!("Ошибка клиента (4xx) или невосстановимая ошибка. Прерывание ретраев.");
                    break;
                }
            }
        }
        
        if retries >= MAX_RETRIES {
            debug!("Достигнут лимит ретраев ({})", MAX_RETRIES);
            break;
        }
        retries += 1;
        
        let delay_ms = 100 * (1 << retries);
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    }
    
    Err(last_error.unwrap_or(TranslateError::RequestError { 
        message: "Запрос провалился после всех ретраев".to_string() 
    }))
}

// --- Граница взаимодействия (FFI) ---

// [ИЗМЕНЕНО] FFI-функция стала `async` (Rule 2.5)
#[uniffi::export(async_runtime = "tokio")]
pub async fn translate(
    text: String,
    from: Language,
    to: Language,
) -> Result<String, TranslateError> {
    
    // (Rule 2.2 / 4.2)
    // Обертка для перехвата паники в async FFI.
    // Используем AssertUnwindSafe, т.к. future может не быть UnwindSafe.
    let future = std::panic::AssertUnwindSafe(
        translate_inner(text, from, to)
    );

    future.catch_unwind().await.map_err(|e| {
        // Перехват паники и преобразование ее в ошибку FFI
        let message = if let Some(s) = e.downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = e.downcast_ref::<String>() {
            s.clone()
        } else {
            "Неизвестная FFI паника".to_string()
        };
        warn!("FFI паника перехвачена (async): {}", message);
        TranslateError::Panic { message }
    })? // '?' #1: Распространяет ошибку паники (TranslateError::Panic)
}


// --- Тесты (Rule 1.4) ---
#[cfg(test)]
mod tests {
    use super::*;

    // Тесты обновлены для использования `tokio::test`

    #[test]
    fn test_language_code() {
        assert_eq!(Language::English.code(), "en");
        assert_eq!(Language::ChineseSimplified.code(), "zh-cn");
        assert_eq!(Language::Auto.code(), "auto");
    }

    #[tokio::test]
    async fn test_target_language_auto_is_error() {
        let result = translate_inner(
            "Hello".to_string(),
            Language::English,
            Language::Auto,
        ).await;
        // (Rule 1.4) Тест на краевой случай (ошибка логики)
        assert!(matches!(result, Err(TranslateError::TargetLanguageAuto)));
    }

    #[tokio::test]
    #[ignore] // Отключаем по умолчанию, т.к. требует сети
    async fn test_successful_translation() {
        let result = translate_inner(
            "Hello, world".to_string(),
            Language::English,
            Language::French,
        ).await;
        // (Rule 1.4) Интеграционный тест
        assert!(result.is_ok(), "Перевод не удался: {:?}", result.err());
        assert!(!result.unwrap().is_empty());
    }
}
