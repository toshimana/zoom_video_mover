//! Zoom Video Mover ライブラリ - レイヤードアーキテクチャ実装
//!
//! # アーキテクチャ構成
//! - components: ビジネスロジック・コンポーネント層
//! - errors: 統一エラーハンドリング層
//! - gui: プレゼンテーション層
//! - windows_console: プラットフォーム固有処理層

pub mod components;
pub mod errors;
pub mod gui;
pub mod services;
pub mod services_impl;
pub mod windows_console;

// 公開API
pub use components::api::{
    MeetingRecording, MeetingSummaryResponse, RecordingFile, RecordingFileType,
    RecordingSearchResponse, SummaryDetail,
};
pub use components::auth::AuthToken;
pub use components::config::{AppConfig, OAuthConfig};
pub use errors::{AppError, AppResult};
pub use gui::{AppMessage, ZoomDownloaderApp};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
    let reserved_names = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    if reserved_names
        .iter()
        .any(|&name| result.to_uppercase() == name)
    {
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
    debug_assert!(
        !result.contains('/'),
        "sanitized filename must not contain /"
    );

    result
}

/// 会議録画ファイルの保存パスを生成する
///
/// # 事前条件
/// - meeting.start_time はISO 8601形式の文字列である
///
/// # 事後条件
/// - "YYYY-MM-DD/YYYY-MM-DD_HH-MM_topic_filetype.ext" 形式のパスが返される
/// - 拡張子は1つだけ付与される
pub fn generate_file_path(meeting: &MeetingRecording, recording_file: &RecordingFile) -> String {
    let date_str = meeting.start_time.split('T').next().unwrap_or("unknown");
    let topic_safe = sanitize_filename(&meeting.topic);
    let file_type_label = recording_file.file_type.to_string().to_lowercase();
    let extension = if !recording_file.file_extension.is_empty() {
        recording_file.file_extension.to_lowercase()
    } else {
        recording_file.file_type.extension().to_string()
    };

    // start_timeからHH-MMを抽出
    let time_str = meeting.start_time.split('T').nth(1).unwrap_or("00:00:00");
    let time_clean = time_str
        .split('Z')
        .next()
        .unwrap_or(time_str)
        .split('+')
        .next()
        .unwrap_or(time_str);
    let time_parts: Vec<&str> = time_clean.split(':').collect();
    let time_hhmm = format!(
        "{}-{}",
        time_parts.first().unwrap_or(&"00"),
        time_parts.get(1).unwrap_or(&"00")
    );

    let folder_name = date_str.to_string();
    let file_name = format!(
        "{}_{}_{}_{}.{}",
        date_str, time_hhmm, topic_safe, file_type_label, extension
    );

    format!("{}/{}", folder_name, file_name)
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

// Config構造体（設定ファイル読み書き用）
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

    fn make_test_meeting(start_time: &str, topic: &str) -> MeetingRecording {
        MeetingRecording {
            uuid: "test-uuid".to_string(),
            id: 1234567890,
            account_id: String::new(),
            host_id: "host".to_string(),
            topic: topic.to_string(),
            meeting_type: 2,
            start_time: start_time.to_string(),
            timezone: "UTC".to_string(),
            duration: 3600,
            total_size: 0,
            recording_count: 0,
            recording_files: vec![],
        }
    }

    fn make_test_file(file_type: RecordingFileType, file_extension: &str) -> RecordingFile {
        RecordingFile {
            id: "file1".to_string(),
            meeting_id: String::new(),
            recording_start: String::new(),
            recording_end: String::new(),
            file_type,
            file_extension: file_extension.to_string(),
            file_size: 0,
            play_url: None,
            download_url: "https://example.com/dl".to_string(),
            status: String::new(),
            recording_type: String::new(),
        }
    }

    #[test]
    fn test_generate_file_path_mp4() {
        let meeting = make_test_meeting("2025-02-24T10:30:00Z", "Project Discussion");
        let file = make_test_file(RecordingFileType::MP4, "MP4");
        let path = generate_file_path(&meeting, &file);
        assert_eq!(
            path,
            "2025-02-24/2025-02-24_10-30_Project Discussion_mp4.mp4"
        );
    }

    #[test]
    fn test_generate_file_path_summary_empty_extension() {
        let meeting = make_test_meeting("2025-02-24T10:30:00Z", "Project Discussion");
        let file = make_test_file(RecordingFileType::Summary, "");
        let path = generate_file_path(&meeting, &file);
        assert_eq!(
            path,
            "2025-02-24/2025-02-24_10-30_Project Discussion_summary.json"
        );
    }

    #[test]
    fn test_generate_file_path_no_double_extension() {
        let meeting = make_test_meeting("2025-02-24T10:30:00Z", "Test");
        let file = make_test_file(RecordingFileType::MP4, "MP4");
        let path = generate_file_path(&meeting, &file);
        // ファイル名に拡張子が1つだけであること
        assert!(path.ends_with(".mp4"));
        assert!(!path.ends_with(".mp4.mp4"));
    }

    #[test]
    fn test_generate_file_path_folder_is_date_only() {
        let meeting = make_test_meeting("2025-02-24T10:30:00Z", "Test");
        let file = make_test_file(RecordingFileType::MP4, "MP4");
        let path = generate_file_path(&meeting, &file);
        let folder = path.split('/').next().unwrap();
        assert_eq!(folder, "2025-02-24");
    }
}
