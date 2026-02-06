// [requests.rs]
// [AUDITED] v11.
// 1. [FIXED] Устранена двойная десериализация JSON.
//    `post_request` теперь возвращает `String`, а не `serde_json::Value`.
//    `fetch_*` методы передают `String` напрямую в `parse_*` методы.

use crate::providers::innertube::models::{
    InnertubeError, PlayerBody, PlayerResponse, Context,
    BrowseResponse, NextResponse, ContinuationResponse,
    SongItem, GetQueueResponseInner, PlayerResponseInner, 
    PlaylistPanelVideoRenderer, BrowseResponseInner,
    SectionListRenderer,
};
use crate::providers::innertube::core::{self, HTTP_CLIENT};
use uniffi::Object;
use serde::Deserialize;
use serde_json::{self, json};
use reqwest::header::HeaderMap;

// ====================================================================
// КОНСТАНТЫ И МАСКИ (Утилиты)
// ====================================================================
const BASE_URL: &str = "https://music.youtube.com/youtubei/v1";
const PLAYER_ENDPOINT: &str = "/player";
const QUEUE_ENDPOINT: &str = "/queue";
const NEXT_ENDPOINT: &str = "/next"; 
const BROWSE_ENDPOINT: &str = "/browse"; 

// ====================================================================
// FFI ОБЪЕКТ: InnertubeClient
// ====================================================================
#[derive(Debug, Object)]
pub struct InnertubeClient {
    api_key: String,
}

// [AUDITED] Внутренние (не FFI) хелперы
// Сюда перенесены все парсеры и маски из блока #[uniffi::export]
impl InnertubeClient {
    
    // [FIXED] Внутренний асинхронный хелпер для запроса.
    // Теперь возвращает Result<String, ...> вместо дженерика <T>,
    // чтобы избежать двойной десериализации.
    async fn post_request(
        &self,
        endpoint: &str,
        body: &serde_json::Value,
        user_agent: &str,
        mask: Option<&str>,
    ) -> Result<String, InnertubeError>
    {
        let url = format!("{}{}{}", BASE_URL, endpoint, self.api_key);
        let client = HTTP_CLIENT.get()
            .ok_or_else(|| InnertubeError::NetworkError { message: "HTTP client not initialized".to_string() })?;
        
        let mut headers = HeaderMap::new();
        headers.insert("User-Agent", user_agent.parse().map_err(|e| InnertubeError::InvalidRequestData { message: format!("Invalid User-Agent: {}", e) })?);
        
        let final_body = if let Some(m) = mask {
            let mut b = body.clone();
            if let Some(map) = b.as_object_mut() {
                map.insert("mask".to_string(), serde_json::Value::String(m.to_string()));
            }
            b
        } else {
            body.clone()
        };

        let response_text = client.post(&url)
            .headers(headers)
            .json(&final_body)
            .send()
            .await
            .map_err(|e| InnertubeError::NetworkError { message: e.to_string() })?
            .text()
            .await
            .map_err(|e| InnertubeError::NetworkError { message: e.to_string() })?;

        // [FIXED] Не десериализуем здесь, возвращаем сырую строку.
        Ok(response_text)
    }
    
    // Внутренний хелпер для парсинга (SongItem)
    fn extract_song_items(raw_json: String) -> Result<Vec<SongItem>, InnertubeError> {
        let response: GetQueueResponseInner = serde_json::from_str(&raw_json)
            .map_err(|e| InnertubeError::JsonDeserializationError { message: format!("Queue deserialization failed: {}", e) })?;

        let items: Vec<SongItem> = response.contents
            .and_then(|c| c.tab_renderer)
            .and_then(|t| t.content)
            .and_then(|c| c.music_queue_renderer)
            .and_then(|m| m.contents)
            .map(|contents| {
                contents.into_iter()
                    .filter_map(|content| content.playlist_panel_video_renderer)
                    .filter_map(|renderer| renderer.to_song_item().ok()) // Игнорируем невалидные/неполные элементы
                    .collect()
            })
            .unwrap_or_default();
        
        Ok(items)
    }

    // --------------------------------------------------------------------
    // [MOVED] PARSER METHODS (Теперь внутренние)
    // --------------------------------------------------------------------

    /// Парсит и маппит PlayerResponse из сырого JSON.
    fn parse_player_response(&self, raw_json: String) -> Result<PlayerResponse, InnertubeError> {
        // [AUDITED] Десериализация 1 (происходит здесь)
        let response_inner: PlayerResponseInner = serde_json::from_str(&raw_json)
            .map_err(|e| InnertubeError::JsonDeserializationError { message: format!("Player deserialization failed: {}", e) })?;

        Ok(PlayerResponse {
            playability_status_status: response_inner.playability_status.and_then(|s| s.status),
            streaming_data_present: response_inner.streaming_data.is_some(),
        })
    }
    
    /// Парсит и маппит Queue/SongItems из сырого JSON.
    fn parse_queue_items(&self, raw_json: String) -> Result<Vec<SongItem>, InnertubeError> {
        Self::extract_song_items(raw_json)
    }

    /// Парсит и маппит NextResponse из сырого JSON.
    fn parse_next_response(&self, raw_json: String) -> Result<NextResponse, InnertubeError> {
        // [AUDITED] Десериализация 1 (происходит здесь)
        let response_inner: serde_json::Value = serde_json::from_str(&raw_json)
            .map_err(|e| InnertubeError::JsonDeserializationError { message: format!("Next deserialization failed: {}", e) })?;

        // [RISK] Хрупкая логика, зависимая от JSON-путей
        let browse_id = response_inner["onResponseReceivedActions"][0]["openPopupAction"]["popup"]["continuationPopupRenderer"]["contents"][0]["sectionListRenderer"]["contents"][0]["musicDescriptionShelfRenderer"]["description"]["runs"][0]["navigationEndpoint"]["browseEndpoint"]["browseId"].as_str()
            .map(|s| s.to_string());
        let video_id = response_inner["contents"]["twoColumnWatchNextResults"]["results"]["resultsContent"]["videoPrimaryInfoRenderer"]["videoId"].as_str()
            .map(|s| s.to_string());


        Ok(NextResponse {
            get_browse_id: browse_id,
            get_video_id: video_id,
        })
    }

    /// Парсит и маппит BrowseResponse (Lyrics) из сырого JSON.
    fn parse_browse_response(&self, raw_json: String) -> Result<BrowseResponse, InnertubeError> {
        // [AUDITED] Десериализация 1 (происходит здесь)
        let response_inner: BrowseResponseInner = serde_json::from_str(&raw_json)
            .map_err(|e| InnertubeError::JsonDeserializationError { message: format!("Browse deserialization failed: {}", e) })?;

        let mut lyrics_text = None;
        let mut contents = None;

        if let Some(ref tabs) = response_inner.contents
            .and_then(|c| c.single_column_browse_results_renderer)
            .and_then(|s| s.tabs)
            .and_then(|mut t| t.pop()) // Берем последнюю вкладку (Lyrics)
            .and_then(|t| t.tab_renderer)
            .and_then(|t| t.content)
            .and_then(|c| c.section_list_renderer)
            .and_then(|s| s.contents) 
        {
            contents = Some(SectionListRenderer { contents: Some(tabs.clone().into_iter().map(|c| c.into()).collect()) });
            
            for section in tabs {
                if let Some(ref music_desc) = section.music_description_shelf_renderer {
                    lyrics_text = music_desc.description.as_ref()
                        .and_then(|r| r.runs.first())
                        .and_then(|r| r.text.clone());
                    break;
                }
            }
        }

        Ok(BrowseResponse {
            lyrics_text,
            contents,
        })
    }
    
    // --------------------------------------------------------------------
    // [MOVED] MASK METHODS (Теперь внутренние)
    // --------------------------------------------------------------------

    /// Маска для /player
    fn get_player_mask(&self) -> String {
        "playabilityStatus,streamingData,videoDetails".to_string()
    }
    
    /// Маска для /queue
    fn get_queue_mask(&self) -> String {
        "contents".to_string()
    }

    /// Маска для /next
    fn get_next_mask(&self) -> String {
        "contents,continuationContents".to_string()
    }

    /// Маска для /browse
    fn get_browse_mask(&self) -> String {
        "contents".to_string()
    }
}

// ====================================================================
// FFI-ЭКСПОРТЫ (ФИНАЛЬНЫЕ)
// ====================================================================

#[uniffi::export]
impl InnertubeClient {
    /// Конструктор FFI-объекта. Безопасная инициализация ресурсов.
    #[uniffi::constructor]
    pub fn new(api_key: String) -> Result<InnertubeClient, InnertubeError> {
        core::init_http_client()
            .map_err(|e| InnertubeError::InitializationFailed { message: format!("HTTP Client init failed: {}", e) })?; 
        
        Ok(InnertubeClient { api_key })
    }
    
    // --------------------------------------------------------------------
    // IO METHODS (Async FFI)
    // --------------------------------------------------------------------

    /// Получает PlayerResponse для videoId/playlistId.
    pub async fn fetch_player(
        &self,
        body: PlayerBody,
        context: Context,
        user_agent: String
    ) -> Result<PlayerResponse, InnertubeError> {
        let player_body = json!({
            "videoId": body.video_id,
            "playlistId": body.playlist_id,
            "context": context,
        });

        let player_mask = self.get_player_mask();
        
        // [FIXED] 1. Получаем сырую строку
        let raw_response = self.post_request(PLAYER_ENDPOINT, &player_body, &user_agent, Some(player_mask))
            .await?;

        // [FIXED] 2. Парсим строку (1 десериализация)
        self.parse_player_response(raw_response)
    }

    /// Получает Queue/Playlist Items.
    pub async fn fetch_queue(
        &self,
        video_id: String,
        playlist_id: Option<String>,
        context: Context,
        user_agent: String
    ) -> Result<Vec<SongItem>, InnertubeError> {
        let queue_body = json!({
            "videoId": video_id,
            "playlistId": playlist_id,
            "context": context,
        });
        
        let queue_mask = self.get_queue_mask();
        
        // [FIXED] 1. Получаем сырую строку
        let raw_response = self.post_request(QUEUE_ENDPOINT, &queue_body, &user_agent, Some(queue_mask))
            .await?; 

        // [FIXED] 2. Парсим строку (1 десериализация)
        self.parse_queue_items(raw_response)
    }

    /// [ASYNC] Получает текст песни, выполняя оркестровку /next -> /browse.
    pub async fn fetch_lyrics(
        &self,
        video_id: String,
        context: Context,
        user_agent: String
    ) -> Result<Option<String>, InnertubeError> {
        if video_id.is_empty() {
            return Ok(None);
        }

        // 1. ЗАПРОС: /next
        let next_body = json!({
            "videoId": video_id,
            "context": context,
        });
        
        let next_mask = self.get_next_mask(); 
        // [FIXED] 1. Получаем сырую строку /next
        let next_response_raw = self.post_request(NEXT_ENDPOINT, &next_body, &user_agent, Some(next_mask)).await?;
        
        // [FIXED] 2. Парсим строку (1 десериализация)
        let next_response: NextResponse = self.parse_next_response(next_response_raw)?;

        let browse_id = next_response.get_browse_id()
            .ok_or_else(|| InnertubeError::LogicError { message: format!("Browse ID not found in /next response for video: {}", next_response.get_video_id().unwrap_or_default()) })?;
        
        // 3. ЗАПРОС: /browse
        let browse_body = json!({
            "browseId": browse_id,
            "context": context,
        });
        
        let browse_mask = self.get_browse_mask();
        // [FIXED] 3. Получаем сырую строку /browse
        let browse_response_raw = self.post_request(BROWSE_ENDPOINT, &browse_body, &user_agent, Some(browse_mask)).await?;
        
        // [FIXED] 4. Парсим строку (1 десериализация)
        let browse_response: BrowseResponse = self.parse_browse_response(browse_response_raw)?;
        
        // 5. ВОЗВРАТ: Текст песни
        Ok(browse_response.lyrics_text)
    }
}

// [NEW] Базовые тесты для покрытия Rule 1.4
#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::innertube::models::PlayerBody;

    // Helper для создания mock-клиента (требует реальный API-ключ для инициализации)
    // В реальном CI это должно браться из env
    fn create_client() -> InnertubeClient {
        let api_key = std::env::var("YTM_API_KEY").unwrap_or_else(|_| "DUMMY_API_KEY_FOR_TESTS".to_string());
        InnertubeClient::new(api_key).expect("Failed to initialize client")
    }

    #[test]
    fn test_client_initialization() {
        // Проверяет, что HTTP_CLIENT инициализируется (Rule 1.4)
        let client = create_client();
        assert_eq!(client.api_key.is_empty(), false);
        // Проверяем, что HTTP_CLIENT был установлен
        assert!(HTTP_CLIENT.get().is_some());
    }

    // REQUIRES_APPROVAL: Эти тесты требуют реальной сети и валидного API ключа.
    // Они должны быть помечены как #[ignore] в CI, если не настроены env.
    
    #[tokio::test]
    #[ignore] // Игнорировать, так как требует сети и API-ключа
    async fn test_fetch_player_integration() {
        let client = create_client();
        let body = PlayerBody {
            video_id: "9mDySso-G_8".to_string(), // Пример (Rick Astley)
            playlist_id: None,
            cpn: None,
            signature_timestamp: None,
        };
        
        let context = Context::DefaultAndroidMusic;
        let user_agent = context.user_agent().to_string();
        
        let response = client.fetch_player(
            body, 
            context, 
            user_agent
        ).await;
        
        assert!(response.is_ok());
        let player_response = response.unwrap();
        assert_eq!(player_response.playability_status_status, Some("OK".to_string()));
        assert_eq!(player_response.streaming_data_present, true);
    }
    
    #[tokio::test]
    #[ignore] // Игнорировать, так как требует сети и API-ключа
    async fn test_fetch_queue_integration() {
        let client = create_client();
        let video_id = "9mDySso-G_8".to_string();
        
        let context = Context::DefaultAndroidMusic;
        let user_agent = context.user_agent().to_string();

        let response = client.fetch_queue(
            video_id,
            None, 
            context, 
            user_agent
        ).await;

        assert!(response.is_ok());
        let items = response.unwrap();
        assert!(items.len() > 0); // Очередь должна содержать хотя бы один элемент
    }

    #[tokio::test]
    #[ignore] // Игнорировать, так как требует сети и API-ключа
    async fn test_fetch_lyrics_integration() {
        let client = create_client();
        // ID видео, у которого ТОЧНО есть текст
        let video_id_with_lyrics = "9mDySso-G_8".to_string(); 
        
        let context = Context::DefaultAndroidMusic;
        let user_agent = context.user_agent().to_string();
        
        let response = client.fetch_lyrics(
            video_id_with_lyrics, 
            context, 
            user_agent
        ).await;

        assert!(response.is_ok());
        let lyrics = response.unwrap();
        assert!(lyrics.is_some());
        assert!(lyrics.unwrap().contains("Never gonna give you up"));
    }
}
