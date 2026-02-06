// ------------------------------------
// Файл: src/sponsorblock/requests.rs (УЛУЧШЕН)
// ------------------------------------
use crate::models::{Action, ApiSegment, Category, Segment, SponsorBlockError};
// [MOVED_TO_MODULE] Импортируем HTTP_CLIENT и CoreError из `core`
use crate::core::{CoreError, HTTP_CLIENT};
use reqwest::Url;
// [УЛУЧШЕНИЕ] (Правило 5.6) Импорт Cow для оптимизации аллокаций
use std::borrow::Cow;

const BASE_URL: &str = "https://sponsor.ajay.app";

// [MOVED_TO_MODULE] `HTTP_CLIENT` и `USER_AGENT` удалены.
// Они теперь находятся в `src/core/mod.rs`

// 2. FFI Объект
#[derive(Debug, uniffi::Object)]
pub struct SponsorBlockApi;

#[uniffi::export]
impl SponsorBlockApi {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self
    }

    /// Получает сегменты для видео, повторяя логику Segments.kt
    #[uniffi::method]
    pub async fn get_segments(
        &self,
        video_id: String,
        categories: Option<Vec<Category>>,
        actions: Option<Vec<Action>>,
        required_segments: Option<Vec<String>>,
    ) -> Result<Vec<Segment>, SponsorBlockError> {
        let base_url =
            Url::parse(BASE_URL).map_err(|_| SponsorBlockError::UrlParsing)?;
        let url = base_url
            .join("/api/skipSegments")
            .map_err(|_| SponsorBlockError::UrlParsing)?;

        // 1. Применяем значения по умолчанию
        let categories_list = categories.unwrap_or_else(|| {
            vec![
                Category::Sponsor,
                Category::OfftopicMusic,
                Category::PoiHighlight,
            ]
        });

        let actions_list = actions.unwrap_or_else(|| vec![Action::Skip, Action::POI]);

        // 2. Собираем query-параметры.
        // [УЛУЧШЕНИЕ] (Правило 5.6) Используем Cow для RVO (Return Value Optimization)
        // на статических строках (Cow::Borrowed) и владения
        // динамическими (Cow::Owned).
        let mut query_params: Vec<(&str, Cow<'_, str>)> = Vec::new();
        query_params.push(("videoID", Cow::Owned(video_id)));
        query_params.push(("service", Cow::Borrowed("YouTube")));

        if !categories_list.is_empty() {
            for category in categories_list {
                query_params.push(("category", Cow::Borrowed(category.serial_name())));
            }
        }

        if !actions_list.is_empty() {
            for action in actions_list {
                query_params.push(("action", Cow::Borrowed(action.serial_name())));
            }
        }

        if let Some(segments) = required_segments {
            if !segments.is_empty() {
                for segment_id in segments {
                    // segment_id - это String, мы должны передать владение
                    query_params.push(("requiredSegment", Cow::Owned(segment_id)));
                }
            }
        }

        // 3. Выполняем запрос
        // [MOVED_TO_MODULE] Используем `HTTP_CLIENT` из `core`
        let response = HTTP_CLIENT
            .get(url)
            .query(&query_params)
            .send()
            .await
            .map_err(|e| {
                // (Правило 2.4 Логирование):
                log::warn!("SponsorBlock request failed: {:?}", e);
                CoreError::Network
            })?; // '?' автоматически вызовет .into() для CoreError -> SponsorBlockError

        if !response.status().is_success() {
            log::warn!(
                "SponsorBlock request returned non-success status: {}",
                response.status()
            );
            // (Правило 2.3 Семантика): API-ошибка, а не сетевая
            return Err(SponsorBlockError::Network);
        }

        // 4. Десериализуем внутреннюю модель
        let api_segments = response
            .json::<Vec<ApiSegment>>()
            .await
            .map_err(|e| {
                log::warn!("SponsorBlock JSON deserialization failed: {:?}", e);
                CoreError::Serialization
            })?; // '?' также преобразует CoreError

        // 5. Безопасно конвертируем во FFI-модель
        api_segments
            .into_iter()
            .map(Segment::try_from)
            .collect::<Result<Vec<Segment>, SponsorBlockError>>()
    }
}

// 6. Тесты
#[cfg(test)]
mod tests {
    use super::*;

    // (Правило 1.4 Тесты): Интеграционный тест (требует сети)
    #[tokio::test]
    #[ignore = "network test"] // Игнорируем по умолчанию, чтобы не ходить в сеть
    async fn test_get_segments_network_call() {
        // Убедимся, что логирование включено для теста
        let _ = env_logger::builder().is_test(true).try_init();

        let api = SponsorBlockApi::new();
        let video_id = "jNQXAC9IVRw"; // (Rick Astley)

        let result = api
            .get_segments(video_id.to_string(), None, None, None)
            .await;

        assert!(result.is_ok());
        let segments = result.unwrap();
        assert!(!segments.is_empty());
        log::debug!("Found {} segments", segments.len());
    }
}
