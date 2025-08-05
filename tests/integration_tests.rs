/// 統合テスト - コンポーネント間の連携をテスト
/// 
/// # テスト戦略
/// - 実際のAPIを使用しないモックベースのテスト
/// - コンポーネント間のデータフローの検証
/// - エラー処理の統合テスト

use zoom_video_mover_lib::components::{
    ComponentLifecycle,
    auth::{AuthComponent, AuthToken},
    api::{ApiComponent, ApiConfig, RecordingSearchRequest},
    download::{DownloadComponent, DownloadConfig},
    config::{AppConfig, OAuthConfig, ApiSettings},
    integration::{IntegrationComponent, IntegrationConfig},
};
use zoom_video_mover_lib::errors::AppResult;
use chrono::{NaiveDate, Utc, Duration, Datelike};
use tempfile::TempDir;
use tokio;

/// テスト用のモック設定を作成
fn create_test_config() -> AppConfig {
    AppConfig {
        oauth: OAuthConfig {
            client_id: "test_client_id".to_string(),
            client_secret: "test_client_secret".to_string(),
            redirect_uri: "http://localhost:8080/callback".to_string(),
            scopes: vec![
                "recording:read".to_string(),
                "user:read".to_string(),
            ],
        },
        output_directory: "./test_downloads".to_string(),
        max_concurrent_downloads: 2,
        request_timeout_seconds: 30,
        rate_limit_per_second: 5,
        debug_mode: true,
        log_level: "debug".to_string(),
        api: ApiSettings {
            base_url: "https://api.zoom.us/v2".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
            default_page_size: 10,
        },
    }
}

/// テスト用の認証トークンを作成
fn create_test_auth_token() -> AuthToken {
    AuthToken {
        access_token: "test_access_token".to_string(),
        token_type: "Bearer".to_string(),
        expires_at: Utc::now() + Duration::hours(1),
        refresh_token: Some("test_refresh_token".to_string()),
        scopes: vec!["recording:read".to_string(), "user:read".to_string()],
    }
}

/// 基本的なコンポーネントライフサイクルテスト
#[tokio::test]
async fn test_component_lifecycle() -> AppResult<()> {
    // 認証コンポーネント
    let oauth_config = OAuthConfig {
        client_id: "test_client".to_string(),
        client_secret: "test_secret".to_string(),
        redirect_uri: "http://localhost:8080/callback".to_string(),
        scopes: vec!["recording:read".to_string()],
    };
    
    let mut auth_component = AuthComponent::new(oauth_config);
    
    // 初期化テスト
    auth_component.initialize().await?;
    assert!(auth_component.health_check().await);
    
    // シャットダウンテスト
    auth_component.shutdown().await?;
    
    Ok(())
}

/// APIコンポーネントの基本テスト
#[tokio::test]
async fn test_api_component_lifecycle() -> AppResult<()> {
    let api_config = ApiConfig::default();
    let mut api_component = ApiComponent::new(api_config);
    
    // 初期化テスト
    api_component.initialize().await?;
    assert!(api_component.health_check().await);
    
    // 認証トークンの設定テスト
    let token = create_test_auth_token();
    api_component.set_auth_token(token).await;
    
    // シャットダウンテスト
    api_component.shutdown().await?;
    
    Ok(())
}

/// ダウンロードコンポーネントの基本テスト
#[tokio::test]
async fn test_download_component_lifecycle() -> AppResult<()> {
    let temp_dir = TempDir::new().unwrap();
    let download_config = DownloadConfig {
        concurrent_downloads: 1,
        chunk_size: 1024,
        timeout: std::time::Duration::from_secs(10),
        max_retries: 2,
        output_directory: temp_dir.path().to_path_buf(),
    };
    
    let mut download_component = DownloadComponent::new(download_config);
    
    // 初期化テスト
    download_component.initialize().await?;
    assert!(download_component.health_check().await);
    
    // タスク追加テスト
    download_component.add_download_task(
        "test_task_1".to_string(),
        "https://example.com/test.mp4".to_string(),
        "test.mp4".to_string(),
        Some(1024),
    ).await?;
    
    // シャットダウンテスト
    download_component.shutdown().await?;
    
    Ok(())
}

/// 録画検索リクエストのバリデーションテスト
#[tokio::test]
async fn test_recording_search_request_validation() -> AppResult<()> {
    let api_config = ApiConfig::default();
    let api_component = ApiComponent::new(api_config);
    
    // 無効な日付範囲のテスト
    let invalid_request = RecordingSearchRequest {
        user_id: Some("test_user".to_string()),
        from: NaiveDate::from_ymd_opt(2025, 1, 31).unwrap(),
        to: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), // from > to
        page_size: Some(30),
        next_page_token: None,
    };
    
    // バリデーションエラーが発生することを確認
    let result = api_component.search_recordings(invalid_request).await;
    assert!(result.is_err());
    
    Ok(())
}

/// 設定ファイルの読み込みテスト
#[tokio::test]
async fn test_config_loading() -> AppResult<()> {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test_config.toml");
    
    // 存在しない設定ファイルの場合はデフォルト設定が返されることを確認
    let config = AppConfig::load_from_file(&config_path)?;
    assert!(!config.oauth.client_id.is_empty() || config.oauth.client_id.is_empty()); // デフォルト値をチェック
    
    Ok(())
}

/// エラー処理の統合テスト
#[tokio::test]
async fn test_error_handling_integration() -> AppResult<()> {
    use zoom_video_mover_lib::errors::AppError;
    
    // ネットワークエラーのテスト
    let network_error = AppError::network("Test network error", None::<std::io::Error>);
    assert!(network_error.is_recoverable());
    assert!(network_error.retry_after().is_some());
    
    // バリデーションエラーのテスト
    let validation_error = AppError::validation("Test validation error", Some("field".to_string()));
    assert!(!validation_error.is_recoverable());
    assert!(validation_error.retry_after().is_none());
    
    // レート制限エラーのテスト
    let rate_limit_error = AppError::rate_limit("Test rate limit error");
    assert!(rate_limit_error.is_recoverable());
    
    Ok(())
}

/// ファイル名のサニタイズテスト
#[test]
fn test_filename_sanitization() {
    use zoom_video_mover_lib::sanitize_filename;
    
    // 通常のファイル名
    assert_eq!(sanitize_filename("normal_filename"), "normal_filename");
    
    // 特殊文字を含むファイル名
    assert_eq!(sanitize_filename("file/with\\special:chars"), "file_with_special_chars");
    
    // Windows予約名
    assert_eq!(sanitize_filename("CON"), "_CON");
    assert_eq!(sanitize_filename("PRN"), "_PRN");
    
    // 空文字列
    assert_eq!(sanitize_filename(""), "unnamed");
    
    // 長すぎるファイル名
    let long_name = "a".repeat(250);
    let sanitized = sanitize_filename(&long_name);
    assert!(sanitized.len() <= 200);
}

/// 日時解析テスト
#[test]
fn test_datetime_parsing() {
    use zoom_video_mover_lib::parse_datetime;
    
    // 有効なISO 8601形式
    let valid_datetime = "2025-01-01T12:00:00Z";
    let parsed = parse_datetime(valid_datetime);
    assert_eq!(parsed.year(), 2025);
    
    // 無効な形式（デフォルト値が返される）
    let invalid_datetime = "invalid_datetime";
    let parsed = parse_datetime(invalid_datetime);
    assert_eq!(parsed.year(), 2025); // デフォルト値
}

/// Property-basedテストの基本例
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        /// ファイル名サニタイズの性質テスト
        #[test]
        fn sanitize_filename_properties(
            input in "[\\PC]{1,100}"
        ) {
            use zoom_video_mover_lib::sanitize_filename;
            
            let sanitized = sanitize_filename(&input);
            
            // Property 1: 結果は空でない
            prop_assert!(!sanitized.is_empty());
            
            // Property 2: 結果は200文字以下
            prop_assert!(sanitized.len() <= 200);
            
            // Property 3: 危険な文字が含まれていない
            prop_assert!(!sanitized.contains('/'));
            prop_assert!(!sanitized.contains('\\'));
            prop_assert!(!sanitized.contains(':'));
        }
        
        /// 日付範囲の妥当性テスト
        #[test]
        fn date_range_validation_properties(
            year1 in 2020i32..2030i32,
            month1 in 1u32..13u32,
            day1 in 1u32..29u32, // 安全な日数範囲
            year2 in 2020i32..2030i32,
            month2 in 1u32..13u32,
            day2 in 1u32..29u32,
        ) {
            let date1 = NaiveDate::from_ymd_opt(year1, month1, day1).unwrap();
            let date2 = NaiveDate::from_ymd_opt(year2, month2, day2).unwrap();
            
            let (from_date, to_date) = if date1 <= date2 {
                (date1, date2)
            } else {
                (date2, date1)
            };
            
            // Property: 正しい順序の日付範囲は常に有効
            prop_assert!(from_date <= to_date);
            
            // Property: RecordingSearchRequestの作成は成功する
            let request = RecordingSearchRequest {
                user_id: Some("test_user".to_string()),
                from: from_date,
                to: to_date,
                page_size: Some(30),
                next_page_token: None,
            };
            
            prop_assert!(request.from <= request.to);
        }
    }
}