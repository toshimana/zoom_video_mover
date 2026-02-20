/// Zoom Video Mover ライブラリ - レイヤードアーキテクチャ実装
/// 
/// # アーキテクチャ構成
/// - components: ビジネスロジック・コンポーネント層
/// - errors: 統一エラーハンドリング層 
/// - gui: プレゼンテーション層
/// - windows_console: プラットフォーム固有処理層

pub mod errors;
pub mod components;
pub mod services;
pub mod services_impl;
pub mod gui;
pub mod windows_console;

// レガシーAPIの互換性維持
pub use components::config::{AppConfig, OAuthConfig};
pub use components::auth::AuthToken;
pub use errors::{AppError, AppResult};
pub use gui::{ZoomDownloaderApp, AppMessage};

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// ファイル名のサニタイズ
/// 
/// # 事前条件
/// - なし（空文字列も許可）
/// 
/// # 事後条件
/// - Windows/Linux/macOSで使用可能なファイル名が返される
/// - 特殊文字が適切に置換される
/// 
/// # 不変条件
/// - 入力文字列の意味は可能な限り保たれる
pub fn sanitize_filename(input: &str) -> String {
    // 空文字列の早期処理
    if input.is_empty() {
        return "unnamed".to_string();
    }
    
    let mut result = input
        .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
        .replace("  ", " ")
        .trim()
        .to_string();
    
    // Windows予約名の回避
    let reserved_names = ["CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9"];
    if reserved_names.iter().any(|&name| result.to_uppercase() == name) {
        result = format!("_{}", result);
    }
    
    // 空の場合のフォールバック
    if result.is_empty() {
        result = "unnamed".to_string();
    }
    
    // 長すぎる場合の切り詰め（文字境界を考慮）
    if result.len() > 200 {
        // 文字境界を考慮した切り詰め
        let mut end = 200;
        while end > 0 && !result.is_char_boundary(end) {
            end -= 1;
        }
        result.truncate(end);
        result = result.trim_end().to_string();
        
        // 切り詰め後に空になった場合の処理
        if result.is_empty() {
            result = "unnamed".to_string();
        }
    }
    
    debug_assert!(!result.is_empty(), "sanitized filename must not be empty");
    debug_assert!(!result.contains('/'), "sanitized filename must not contain /");
    
    result
}

/// 日時文字列をパース
/// 
/// # 事前条件
/// - datetime_str は空でない文字列である
/// 
/// # 事後条件
/// - 有効なDateTime<Utc>が返される
/// - パース失敗時はデフォルト値が返される
/// 
/// # 不変条件
/// - 入力文字列は変更されない
pub fn parse_datetime(datetime_str: &str) -> DateTime<Utc> {
    assert!(!datetime_str.is_empty(), "datetime_str must not be empty");
    
    chrono::DateTime::parse_from_rfc3339(datetime_str)
        .unwrap_or_else(|_| {
            // フォールバック: 2025年1月1日のUTC時刻
            chrono::DateTime::parse_from_rfc3339("2025-01-01T00:00:00Z")
                .expect("Default datetime should be valid")
        })
        .with_timezone(&chrono::Utc)
}

// レガシー型の互換性維持のため、旧Config構造体を実装
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: Option<String>,
}

impl Config {
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        use std::fs;
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
    
    pub fn create_sample_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs;
        let sample_config = Config {
            client_id: "your_zoom_client_id".to_string(),
            client_secret: "your_zoom_client_secret".to_string(),
            redirect_uri: Some("http://localhost:8080/callback".to_string()),
        };
        let content = toml::to_string_pretty(&sample_config)?;
        fs::write(path, content)?;
        Ok(())
    }
    
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs;
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}

// レガシーAPI構造体（最小限）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingFile {
    pub id: String,
    pub meeting_id: String,
    pub recording_start: DateTime<Utc>,
    pub recording_end: DateTime<Utc>,
    pub file_type: String,
    pub file_extension: String,
    pub file_size: u64,
    pub play_url: Option<String>,
    pub download_url: String,
    pub status: String,
    pub recording_type: String,
    pub filename: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recording {
    pub meeting_id: String,
    pub topic: String,
    pub start_time: DateTime<Utc>,
    pub duration: u32,
    pub recording_files: Vec<RecordingFile>,
    pub ai_summary_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingResponse {
    pub from: String,
    pub to: String,
    pub page_count: u32,
    pub page_size: u32,
    pub total_records: u32,
    pub meetings: Vec<MeetingRecording>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingRecording {
    pub uuid: String,
    pub id: u64,
    pub account_id: String,
    pub host_id: String,
    pub topic: String,
    pub meeting_type: u32,
    pub start_time: String,
    pub timezone: String,
    pub duration: u32,
    pub total_size: u64,
    pub recording_count: u32,
    pub recording_files: Vec<RecordingFile>,
}

#[derive(Debug, Clone)]
pub struct DownloadRequest {
    pub file_id: String,
    pub file_name: String,
    pub download_url: String,
    pub file_size: u64,
    pub output_path: std::path::PathBuf,
}

// レガシー互換のためのZoomRecordingDownloader（最小限のスタブ）
pub struct ZoomRecordingDownloader {
    // スタブ実装
}

impl ZoomRecordingDownloader {
    pub fn new(_client_id: String, _client_secret: String, _redirect_uri: String) -> Self {
        Self {}
    }
    
    pub fn new_with_token(_client_id: String, _client_secret: String, _access_token: String) -> Self {
        Self {}
    }
    
    pub async fn get_recordings(&mut self, _user_id: Option<&str>, _from_date: &str, _to_date: &str, _page_size: Option<u32>) -> Result<RecordingResponse, Box<dyn std::error::Error + Send + Sync>> {
        // スタブ実装 - 空のレスポンスを返す
        Ok(RecordingResponse {
            from: _from_date.to_string(),
            to: _to_date.to_string(),
            page_count: 1,
            page_size: 30,
            total_records: 0,
            meetings: vec![],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("test"), "test");
        assert_eq!(sanitize_filename("test/file"), "test_file");
        assert_eq!(sanitize_filename("CON"), "_CON");
    }
    
    #[test]
    fn test_parse_datetime() {
        let dt = parse_datetime("2025-01-01T00:00:00Z");
        assert_eq!(dt.year(), 2025);
    }
}