// [models.rs]
// [AUDITED] Без изменений.

use uniffi::Object;
use serde::{Deserialize, Serialize};

// ====================================================================
// 1. ОШИБКИ FFI
// ====================================================================

// Соответствует #[Error] InnertubeError в innertube.udl
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum InnertubeError {
    #[error("JSON serialization failed: {message}")]
    JsonSerializationError { message: String },
    #[error("JSON deserialization failed: {message}")]
    JsonDeserializationError { message: String },
    // [FIX] Удалено: ChallengeFailed
    #[error("Invalid request data: {message}")]
    InvalidRequestData { message: String },
    #[error("Logic error: Missing critical data field: {message}")]
    LogicError { message: String }, // Добавлено для ошибок, связанных с отсутствием критичных полей (videoId, browseId)
    #[error("Network error: {message}")]
    NetworkError { message: String }, // Добавлено для ошибок IO (reqwest)
    #[error("Initialization failed: {message}")]
    InitializationFailed { message: String }, // Добавлено для ошибок инициализации (HTTP Client)
}

// ====================================================================
// 2. СТРУКТУРЫ, СОЗДАННЫЕ UNIFFI (MODELS)
// ====================================================================

// --- FFI Records/Enums (Полная реализация) ---

// Соответствует Context в innertube.udl (Используется для сериализации в запросах)
#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Enum)]
pub enum Context {
    // В оригинальном контексте было 25+ вариантов. Оставляю только те, что используются в Kotlin
    DefaultAndroidMusic,
    DefaultIOS,
    DefaultWeb, 
    DefaultTV,
}

impl Context {
    /// Получить userAgent, необходимый для запроса (ВНУТРЕННИЙ)
    pub fn user_agent(&self) -> &'static str {
        match self {
            Context::DefaultAndroidMusic => "com.google.android.apps.youtube.music/5.11.51",
            Context::DefaultIOS => "com.google.ios.youtube.music/5.11.51",
            Context::DefaultWeb => "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/117.0",
            Context::DefaultTV => "com.google.android.apps.youtube.music.tv/1.0.0",
        }
    }
}


// --- Serde-структуры для внутреннего парсинга JSON ---
// Все эти структуры не экспортируются через FFI, а используются внутри Rust
// для десериализации сырого JSON и последующего маппинга в FFI Records.

// Внутренняя структура для парсинга /queue и /next.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetQueueResponseInner {
    pub contents: Option<QueueContents>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueContents {
    pub tab_renderer: Option<TabRenderer>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TabRenderer {
    pub content: Option<TabContent>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TabContent {
    pub music_queue_renderer: Option<MusicQueueRenderer>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicQueueRenderer {
    pub contents: Option<Vec<MusicQueueContent>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicQueueContent {
    pub playlist_panel_video_renderer: Option<PlaylistPanelVideoRenderer>,
}

// Внутренняя структура для парсинга PlayerResponse.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerResponseInner {
    pub playability_status: Option<PlayabilityStatus>,
    pub streaming_data: Option<StreamingData>,
    pub video_details: Option<VideoDetails>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayabilityStatus {
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamingData {} // Placeholder, если данные не нужны

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoDetails {} // Placeholder, если данные не нужны


// Внутренняя структура для парсинга /next.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NextResponseInner {
    pub contents: Option<NextContents>,
    // Используется для получения Contination
    pub continuation_contents: Option<ContinuationContents>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NextContents {
    pub music_player_page_renderer: Option<MusicPlayerPageRenderer>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinuationContents {
    pub music_continuation_renderer: Option<MusicContinuationRenderer>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicContinuationRenderer {
    pub continuation_endpoint: Option<ContinuationEndpoint>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinuationEndpoint {
    pub continuation_command: Option<ContinuationCommand>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinuationCommand {
    pub token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicPlayerPageRenderer {
    pub content: Option<MusicPlayerPageContent>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicPlayerPageContent {
    pub music_queue_renderer: Option<MusicQueueRenderer>, // Переиспользуем структуру
    pub section_list_renderer: Option<SectionListRendererInner>, // Для Lyrics
}

// Внутренняя структура для /browse (Lyrics).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowseResponseInner {
    pub contents: Option<BrowseContents>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowseContents {
    pub single_column_browse_results_renderer: Option<SingleColumnBrowseResultsRenderer>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SingleColumnBrowseResultsRenderer {
    pub tabs: Option<Vec<Tab>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tab {
    pub tab_renderer: Option<TabRendererInner>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TabRendererInner {
    pub content: Option<TabContentInner>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TabContentInner {
    pub section_list_renderer: Option<SectionListRendererInner>,
}


// --- Общие структуры для Lyrics и SongItem ---

// Соответствует SectionListRenderer в innertube.udl
#[derive(Debug, Clone, Deserialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct SectionListRenderer {
    pub contents: Option<Vec<SectionListRendererContent>>,
}

// Соответствует SectionListRendererContent в innertube.udl
#[derive(Debug, Clone, Deserialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct SectionListRendererContent {
    pub music_description_shelf_renderer: Option<MusicDescriptionShelfRenderer>,
    pub music_shelf_renderer: Option<MusicShelfRenderer>,
    // Добавьте другие типы рендереров по мере необходимости
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicDescriptionShelfRenderer {
    pub description: Option<RunsInner>,
    pub footer: Option<RunsInner>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicShelfRenderer {
    pub title: Option<RunsInner>,
    pub contents: Option<Vec<PlaylistPanelVideoRenderer>>, // Используем для /browse (Related songs)
}

// Внутренняя (Serde) версия SectionListRenderer для вложенности
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SectionListRendererInner {
    pub contents: Option<Vec<SectionListRendererContentInner>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SectionListRendererContentInner {
    pub music_description_shelf_renderer: Option<MusicDescriptionShelfRenderer>,
    pub music_shelf_renderer: Option<MusicShelfRenderer>,
}

// Соответствует Runs в innertube.udl
#[derive(Debug, Clone, Deserialize, uniffi::Record)]
pub struct Runs {
    pub runs: Vec<Run>,
}

// Внутренняя (Serde) версия Runs
#[derive(Debug, Deserialize)]
pub struct RunsInner {
    pub runs: Vec<RunInner>,
}

// Соответствует Run в innertube.udl
#[derive(Debug, Clone, Deserialize, uniffi::Record)]
pub struct Run {
    pub text: String,
}

// Внутренняя (Serde) версия Run
#[derive(Debug, Deserialize)]
pub struct RunInner {
    pub text: Option<String>,
    pub navigation_endpoint: Option<NavigationEndpointInner>,
}


// --- Структуры для SongItem ---

// Соответствует SongItem в innertube.udl
#[derive(Debug, Clone, uniffi::Record)]
pub struct SongItem {
    pub key: String, // Композитный ключ: videoId_playlistId
    pub video_id: String,
    pub explicit: bool,
    pub authors: Option<Vec<Info>>,
    pub info: Info,
    pub album: Option<Info>,
    pub duration_text: Option<String>,
    pub thumbnail: Option<Thumbnail>,
}

// Соответствует Info в innertube.udl
#[derive(Debug, Clone, Deserialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    pub name: Option<String>,
    pub endpoint: Option<NavigationEndpoint>,
}

// Соответствует NavigationEndpoint в innertube.udl
#[derive(Debug, Clone, Deserialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct NavigationEndpoint {
    pub video_id: Option<String>,
    pub playlist_id: Option<String>,
    pub browse_id: Option<String>,
    pub params: Option<String>,
}

// Соответствует Thumbnail в innertube.udl
#[derive(Debug, Clone, Deserialize, uniffi::Record)]
pub struct Thumbnail {
    pub url: String,
    pub width: u32,
    pub height: u32,
}


// --- Внутренние Serde-структуры для SongItem (PlaylistPanelVideoRenderer) ---
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistPanelVideoRenderer {
    pub video_id: Option<String>,
    pub title: Option<RunsInner>,
    pub long_byline_text: Option<RunsInner>,
    pub short_byline_text: Option<RunsInner>,
    pub length_text: Option<RunsInner>,
    pub navigation_endpoint: Option<NavigationEndpointInner>,
    pub thumbnail: Option<ThumbnailDetails>,
    pub badges: Option<Vec<Badge>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailDetails {
    pub music_thumbnail_renderer: Option<MusicThumbnailRenderer>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicThumbnailRenderer {
    pub thumbnail: Option<ThumbnailInner>,
}

#[derive(Debug, Deserialize)]
pub struct ThumbnailInner {
    pub thumbnails: Vec<ThumbnailRaw>,
}

#[derive(Debug, Deserialize)]
pub struct ThumbnailRaw {
    pub url: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Badge {
    pub music_inline_badge_renderer: Option<MusicInlineBadgeRenderer>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicInlineBadgeRenderer {
    pub icon: Option<Icon>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Icon {
    pub icon_type: String,
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigationEndpointInner {
    pub watch_endpoint: Option<WatchEndpoint>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchEndpoint {
    pub video_id: Option<String>,
    pub playlist_id: Option<String>,
    pub params: Option<String>,
}


// ====================================================================
// 3. МЕХАНИЗМ МАППИНГА В FFI (SongItem::from)
// ====================================================================

impl PlaylistPanelVideoRenderer {
    /// Маппинг внутренней Serde-структуры в экспортируемую FFI Record SongItem
    pub fn to_song_item(self) -> Result<SongItem, InnertubeError> {
        let video_id = self.video_id
            .ok_or_else(|| InnertubeError::LogicError { message: "Song item missing videoId".to_string() })?;

        // 1. Имя
        let title_text = self.title.as_ref()
            .and_then(|r| r.runs.first())
            .and_then(|r| r.text.clone());
            
        // 2. Авторы (long_byline_text)
        let authors = self.long_byline_text.as_ref()
            .map(|r| r.runs.iter()
                .filter_map(|run| {
                    if let Some(ref text) = run.text {
                        if text != &" • " && text != &" · " {
                            let endpoint = run.navigation_endpoint.as_ref()
                                .and_then(|ne| ne.watch_endpoint.as_ref())
                                .map(|we| NavigationEndpoint {
                                    video_id: we.video_id.clone(),
                                    playlist_id: we.playlist_id.clone(),
                                    browse_id: None,
                                    params: we.params.clone(),
                                });
                            Some(Info { name: Some(text.clone()), endpoint })
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect::<Vec<Info>>()
            );

        // 3. Длительность
        let duration_text = self.length_text.as_ref()
            .and_then(|r| r.runs.first())
            .and_then(|r| r.text.clone());

        // 4. Обложка
        let thumbnail = self.thumbnail.as_ref()
            .and_then(|r| r.music_thumbnail_renderer.as_ref())
            .and_then(|r| r.thumbnail.as_ref())
            .and_then(|r| r.thumbnails.first().cloned())
            .map(|t| Thumbnail {
                url: t.url,
                width: t.width,
                height: t.height,
            });

        // 5. Явный контент (Explicit)
        let explicit = self.badges.as_ref().map_or(false, |badges| {
            badges.iter().any(|b| {
                b.music_inline_badge_renderer.as_ref()
                    .map_or(false, |m| m.icon.as_ref().map_or(false, |i| i.icon_type == "MUSIC_EXPLICIT_BADGE"))
            })
        });
        
        // 6. Конечная точка
        let navigation_endpoint = self.navigation_endpoint.as_ref()
            .and_then(|ne| ne.watch_endpoint.as_ref());
            
        let endpoint_ffi = navigation_endpoint.map(|e| NavigationEndpoint {
            video_id: e.video_id.clone(),
            playlist_id: e.playlist_id.clone(),
            browse_id: None,
            params: e.params.clone(),
        });
            
        let info = Info {
            name: title_text,
            endpoint: endpoint_ffi,
        };
        
        let playlist_id = navigation_endpoint.as_ref().and_then(|e| e.playlist_id.as_ref());

        let key = format!("{}_{}", video_id, playlist_id.map_or("", |id| id));

        Ok(SongItem {
            key,
            video_id,
            explicit,
            authors,
            info,
            album: None, // Альбом не парсится на этом уровне
            duration_text,
            thumbnail,
        })
    }
}


// --- FFI Records (Промежуточные/Итоговые) ---

// Соответствует PlayerResponse в innertube.udl
#[derive(Debug, Clone, uniffi::Record)]
pub struct PlayerResponse {
    pub playability_status_status: Option<String>, 
    pub streaming_data_present: bool, 
}

// Соответствует NextResponse в innertube.udl
#[derive(Debug, Clone, uniffi::Record)]
pub struct NextResponse {
    pub get_browse_id: Option<String>,
    pub get_video_id: Option<String>,
}

impl NextResponse {
    // Вспомогательный метод для извлечения browseId
    pub fn get_browse_id(&self) -> Option<String> {
        self.get_browse_id.clone()
    }
}

// Соответствует BrowseResponse в innertube.udl
#[derive(Debug, Clone, uniffi::Record)]
pub struct BrowseResponse {
    pub lyrics_text: Option<String>,
    pub contents: Option<SectionListRenderer>, 
}

// Соответствует ContinuationResponse в innertube.udl
#[derive(Debug, Clone, uniffi::Record)]
pub struct ContinuationResponse {
    pub continuation: Option<String>,
}

// Соответствует PlayerBody в innertube.udl
#[derive(Debug, Clone, Serialize, uniffi::Record)]
pub struct PlayerBody {
    pub video_id: String,
    pub playlist_id: Option<String>,
    pub cpn: Option<String>,
    pub signature_timestamp: Option<String>, 
}

// Соответствует QueueBody в innertube.udl
// NOTE: Не экспортируется, но используется для тела запроса.
#[derive(Debug, Clone, Serialize)]
pub struct QueueBody {
    #[serde(rename = "videoId")]
    pub video_id: Option<String>,
    #[serde(rename = "playlistId")]
    pub playlist_id: Option<String>,
}
