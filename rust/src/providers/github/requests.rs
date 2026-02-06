// Файл: requests.rs
// Логика выполнения запросов к GitHub API

use crate::models::{GitHubError, Release, ReleaseList};
use once_cell::sync::Lazy;
use reqwest::{header, Client};
use std::time::Duration; // [ДОБАВЛЕНО] для таймаута

const API_VERSION: &str = "2022-11-28";
const CONTENT_TYPE: &str = "application";
const CONTENT_SUBTYPE: &str = "vnd.github+json";
const BASE_URL: &str = "https://api.github.com";
const USER_AGENT: &str = "rust-github-client"; // Рекомендуется для GitHub API

// Создаем статический HTTP-клиент, имитируя GitHub.httpClient by lazy
static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    let content_type = format!("{}/{}", CONTENT_TYPE, CONTENT_SUBTYPE);

    let mut headers = header::HeaderMap::new();
    headers.insert(
        "X-GitHub-Api-Version",
        header::HeaderValue::from_static(API_VERSION),
    );
    headers.insert(
        header::ACCEPT,
        header::HeaderValue::from_str(&content_type).unwrap(),
    );
    headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/json"),
    );
    headers.insert(
        header::USER_AGENT,
        header::HeaderValue::from_static(USER_AGENT)
    );

    Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(15)) // [ИСПРАВЛЕНО] Добавлен таймаут (Риск 4.2)
        .build()
        .expect("Failed to build reqwest client")
});

/// Запрашивает список релизов для репозитория.
#[uniffi::export(async)]
pub async fn releases(
    owner: String,
    repo: String,
    page: i32,
    page_size: i32,
) -> Result<ReleaseList, GitHubError> {
    // Воспроизводим логику require из GitHub.kt
    if page <= 0 || page_size <= 0 {
        log::warn!(
            "GitHub error: invalid pagination (page: {}, size: {})",
            page, page_size
        );
        return Err(GitHubError::InvalidPagination);
    }

    let url = format!("{}/repos/{}/{}/releases", BASE_URL, owner, repo);

    log::debug!("Fetching releases from: {}", url);

    let response = HTTP_CLIENT
        .get(&url)
        .query(&[
            ("per_page", page_size.to_string()),
            ("page", page.to_string()),
        ])
        .send()
        .await?;

    // Проверяем на ошибки (4xx, 5xx)
    let releases = response.error_for_status()?.json::<Vec<Release>>().await?;

    Ok(ReleaseList { items: releases })
}

// --- Тесты ---
#[cfg(test)]
mod tests {
    use super::*;

    // Тест требует живого интернета и может быть нестабильным
    // Используем tokio::test для асинхронного теста
    #[tokio::test]
    #[ignore] // Игнорируем, так как это сетевой вызов
    async fn test_fetch_releases_real() {
        // Используем популярный репозиторий для теста
        let owner = "rust-lang".to_string();
        let repo = "rust".to_string();
        let page = 1;
        let page_size = 2;

        let result = releases(owner, repo, page, page_size).await;

        assert!(result.is_ok());
        let release_list = result.unwrap();
        assert_eq!(release_list.items.len(), 2);
        assert!(release_list.items[0].tag.contains("1.")); // У Rust теги вида "1.X.X"
    }

    #[tokio::test]
    async fn test_invalid_pagination() {
        let owner = "test".to_string();
        let repo = "test".to_string();
        
        let result = releases(owner.clone(), repo.clone(), 0, 30).await;
        assert!(matches!(result, Err(GitHubError::InvalidPagination)));

        let result = releases(owner.clone(), repo.clone(), 1, -1).await;
        assert!(matches!(result, Err(GitHubError::InvalidPagination)));
    }
}
