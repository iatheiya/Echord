// Файл: models.rs
// Модели данных для GitHub API, с поддержкой Serde и UniFFI

use chrono::{DateTime, Utc};
use serde::Deserialize;

// --- Ошибка ---

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum GitHubError {
    #[error("GitHub error: invalid pagination. Page and size must be > 0.")]
    #[uniffi(flat_error)]
    InvalidPagination,

    #[error("GitHub request failed: {0}")]
    #[uniffi(flat_error)]
    RequestFailed { message: String },
}

// Конвертер из reqwest::Error в наш GitHubError
impl From<reqwest::Error> for GitHubError {
    fn from(e: reqwest::Error) -> Self {
        GitHubError::RequestFailed {
            message: e.to_string(),
        }
    }
}

// --- Модели (из Reactions.kt) ---

#[derive(Debug, Deserialize, uniffi::Record)]
pub struct Reactions {
    pub url: String,
    #[serde(rename = "total_count")]
    pub count: i32,
    #[serde(rename = "+1")]
    pub likes: i32,
    #[serde(rename = "-1")]
    pub dislikes: i32,
    #[serde(rename = "laugh")]
    pub laughs: i32,
    pub confused: i32,
    #[serde(rename = "heart")]
    pub hearts: i32,
    #[serde(rename = "hooray")]
    pub hoorays: i32,
    pub eyes: i32,
    #[serde(rename = "rocket")]
    pub rockets: i32,
}

// --- Модели (из SimpleUser.kt) ---

#[derive(Debug, Deserialize, uniffi::Record)]
pub struct SimpleUser {
    pub name: Option<String>,
    pub email: Option<String>,
    pub login: String,
    pub id: i32,
    pub node_id: String,
    pub avatar_url: String,
    pub gravatar_id: Option<String>,
    pub url: String,
    #[serde(rename = "html_url")]
    pub frontend_url: String,
    pub followers_url: String,
    pub following_url: String,
    pub gists_url: String,
    pub starred_url: String,
    pub subscriptions_url: String,
    pub organizations_url: String,
    pub repos_url: String,
    pub events_url: String,
    pub received_events_url: String,
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(rename = "site_admin")]
    pub admin: bool,
}

// --- Модели (из Release.kt) ---

#[derive(Debug, Deserialize, uniffi::Enum)]
#[serde(rename_all = "lowercase")]
pub enum AssetState {
    Uploaded,
    Open,
}

#[derive(Debug, Deserialize, uniffi::Record)]
pub struct Asset {
    pub url: String,
    #[serde(rename = "browser_download_url")]
    pub download_url: String,
    pub id: i32,
    pub node_id: String,
    pub name: String,
    pub label: Option<String>,
    pub state: AssetState,
    pub content_type: String,
    pub size: i64,
    #[serde(rename = "download_count")]
    pub downloads: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub uploader: Option<SimpleUser>,
}

#[derive(Debug, Deserialize, uniffi::Record)]
pub struct Release {
    pub id: i32,
    pub node_id: String,
    pub url: String,
    #[serde(rename = "html_url")]
    pub frontend_url: String,
    pub assets_url: String,
    #[serde(rename = "tag_name")]
    pub tag: String,
    pub name: Option<String>,
    #[serde(rename = "body")]
    pub markdown: Option<String>,
    pub draft: bool,
    #[serde(rename = "prerelease")]
    pub pre_release: bool,
    pub created_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
    pub author: SimpleUser,
    
    // [УЛУЧШЕНИЕ] Семантически соответствует `List<Asset> = emptyList()` из Kotlin
    #[serde(default)] 
    pub assets: Vec<Asset>,
    
    #[serde(rename = "body_html")]
    pub html: Option<String>,
    #[serde(rename = "body_text")]
    pub text: Option<String>,
    pub discussion_url: Option<String>,
    pub reactions: Option<Reactions>,
}

// --- Вспомогательный тип для UniFFI ---

#[derive(uniffi::Record)]
pub struct ReleaseList {
    pub items: Vec<Release>,
}
