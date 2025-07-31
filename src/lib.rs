use std::fs;
use serde::{Deserialize, Serialize};
use reqwest::Client;
use chrono::{DateTime, Utc};
use eframe;

// カスタムエラー型の定義
#[derive(Debug)]
pub enum ZoomVideoMoverError {
    NetworkError(String),
    AuthenticationError(String),
    FileSystemError(String),
    ConfigError(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: Option<String>,
}

impl Config {
    /// 設定ファイルから設定を読み込む
    /// 
    /// # 事前条件
    /// - path は有効なファイルパスを指す
    /// - ファイルが存在し、読み取り可能である
    /// - ファイルの内容は有効な TOML 形式である
    /// 
    /// # 事後条件
    /// - 成功時: 有効な Config インスタンスを返す
    /// - client_id および client_secret は空でない
    /// - 失敗時: 適切なエラーを返す
    /// 
    /// # 不変条件
    /// - ファイルシステムの状態は変更されない
    /// - 入力パラメータは変更されない
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // 事前条件のassertion
        assert!(!path.is_empty(), "path must not be empty");
        debug_assert!(path.len() > 0, "path should have reasonable length");
        
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        
        // 事後条件のassertion
        debug_assert!(!config.client_id.is_empty(), "loaded config must have valid client_id");
        debug_assert!(!config.client_secret.is_empty(), "loaded config must have valid client_secret");
        
        Ok(config)
    }

    /// サンプル設定ファイルを作成する
    /// 
    /// # 副作用
    /// - ファイルシステムへの書き込み（指定されたパスにファイルを作成）
    /// 
    /// # 事前条件
    /// - path は有効なファイルパスを指す
    /// - ファイルの親ディレクトリが存在するか作成可能である
    /// - ファイルへの書き込み権限がある
    /// 
    /// # 事後条件
    /// - 成功時: サンプル設定ファイルが作成される
    /// - ファイルは有効な TOML 形式で保存される
    /// - 失敗時: 適切なエラーを返す
    /// 
    /// # 不変条件
    /// - 関数実行中にサンプル設定の内容は一定
    /// - 入力パラメータは変更されない
    pub fn create_sample_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // 事前条件のassertion
        assert!(!path.is_empty(), "path must not be empty");
        debug_assert!(path.len() > 0, "path should have reasonable length");
        
        let sample_config = Config {
            client_id: "your_zoom_client_id".to_string(),
            client_secret: "your_zoom_client_secret".to_string(),
            redirect_uri: Some("http://localhost:8080/callback".to_string()),
        };
        
        let content = toml::to_string_pretty(&sample_config)?;
        
        // 事後条件のassertion（書き込み前にコンテンツの妥当性確認）
        debug_assert!(!content.is_empty(), "generated TOML content must not be empty");
        debug_assert!(content.contains("client_id"), "TOML must contain client_id field");
        
        fs::write(path, content)?;
        
        // 事後条件のassertion（ファイル作成確認）
        debug_assert!(std::path::Path::new(path).exists(), "file should be created successfully");
        
        Ok(())
    }

    /// 設定をファイルに保存する
    /// 
    /// # 副作用
    /// - ファイルシステムへの書き込み（指定されたパスにファイルを保存）
    /// 
    /// # 事前条件
    /// - self は有効な Config インスタンスである
    /// - path は有効なファイルパスを指す
    /// - ファイルの親ディレクトリが存在するか作成可能である
    /// - ファイルへの書き込み権限がある
    /// 
    /// # 事後条件
    /// - 成功時: 設定が TOML 形式でファイルに保存される
    /// - 失敗時: 適切なエラーを返す
    /// 
    /// # 不変条件
    /// - self の内容は変更されない
    /// - 入力パラメータは変更されない
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // 事前条件のassertion
        assert!(!path.is_empty(), "path must not be empty");
        assert!(!self.client_id.is_empty(), "client_id must not be empty");
        assert!(!self.client_secret.is_empty(), "client_secret must not be empty");
        
        let content = toml::to_string_pretty(self)?;
        
        // 事後条件のassertion（書き込み前にコンテンツの妥当性確認）
        debug_assert!(!content.is_empty(), "generated TOML content must not be empty");
        debug_assert!(content.contains(&self.client_id), "TOML must contain the client_id");
        
        fs::write(path, content)?;
        
        // 事後条件のassertion（ファイル作成確認）
        debug_assert!(std::path::Path::new(path).exists(), "file should be created successfully");
        
        Ok(())
    }
}

// 標準エラー実装
impl std::fmt::Display for ZoomVideoMoverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZoomVideoMoverError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            ZoomVideoMoverError::AuthenticationError(msg) => write!(f, "Authentication error: {}", msg),
            ZoomVideoMoverError::FileSystemError(msg) => write!(f, "File system error: {}", msg),
            ZoomVideoMoverError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for ZoomVideoMoverError {}

// テスト用の構造体定義

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingFile {
    pub id: String,
    pub file_type: String,
    pub file_size: u64,
    pub download_url: String,
    pub play_url: Option<String>,
    pub recording_start: DateTime<Utc>,
    pub recording_end: DateTime<Utc>,
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

#[derive(Debug, Clone)]
pub struct DownloadRequest {
    pub file_id: String,
    pub file_name: String,
    pub download_url: String,
    pub file_size: u64,
    pub output_path: std::path::PathBuf,
}

// OAuth Token構造体
#[derive(Debug, Clone)]
pub struct AuthToken {
    pub access_token: String,
    pub token_type: String,
    pub expires_at: DateTime<Utc>,
    pub refresh_token: Option<String>,
    pub scopes: Vec<String>,
}

// 旧Recording構造体（後方互換性のため）
#[derive(Debug, Serialize, Deserialize)]
pub struct LegacyRecording {
    pub id: String,
    pub download_url: String,
    pub file_type: String,
    pub file_size: u64,
    pub recording_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MeetingRecording {
    pub uuid: String,
    pub id: u64,
    pub topic: String,
    pub start_time: String,
    pub recording_files: Vec<LegacyRecording>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordingResponse {
    pub meetings: Vec<MeetingRecording>,
    pub page_count: u32,
    pub page_size: u32,
    pub total_records: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicSummary {
    pub name: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedSection {
    pub section_name: String,
    pub section_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryDetail {
    pub detail_type: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AISummaryResponse {
    pub meeting_uuid: String,
    pub summary_start_time: String,
    pub summary_end_time: String,
    pub summary_created_time: String,
    pub summary_last_modified_time: String,
    pub meeting_topic: String,
    pub meeting_id: String,
    pub meeting_host: String,
    pub summary_created_by: String,
    pub summary_title: String,
    pub summary_overview: String,
    pub summary_keywords: Vec<String>,
    pub summary_details: Vec<SummaryDetail>,
    pub detailed_sections: Vec<DetailedSection>,
    pub topic_summaries: Vec<TopicSummary>,
    pub next_steps: Vec<String>,
}

pub struct ZoomRecordingDownloader {
    client: Client,
    access_token: String,
    oauth_base_url: String,
    api_base_url: String,
    client_id: String,
    client_secret: String,
}

impl ZoomRecordingDownloader {
    /// 認証情報付きで新しいインスタンスを作成
    pub fn new(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            client: Client::new(),
            access_token: String::new(),
            oauth_base_url: "https://zoom.us".to_string(),
            api_base_url: "https://api.zoom.us".to_string(),
            client_id,
            client_secret,
        }
    }
    
    /// トークン付きで新しいインスタンスを作成
    pub fn new_with_token(client_id: String, client_secret: String, access_token: String) -> Self {
        Self {
            client: Client::new(),
            access_token,
            oauth_base_url: "https://zoom.us".to_string(),
            api_base_url: "https://api.zoom.us".to_string(),
            client_id,
            client_secret,
        }
    }
    
    /// OAuth ベースURLを設定（テスト用）
    pub fn set_oauth_base_url(&mut self, url: &str) {
        self.oauth_base_url = url.to_string();
    }
    
    /// API ベースURLを設定（テスト用）
    pub fn set_api_base_url(&mut self, url: &str) {
        self.api_base_url = url.to_string();
    }
    
    /// 認証URLを生成
    pub fn generate_auth_url(&self) -> Result<String, ZoomVideoMoverError> {
        let state = "test_state_12345"; // 本来はランダム生成
        let auth_url = format!(
            "{}/oauth/authorize?response_type=code&client_id={}&redirect_uri=http://localhost:8080/callback&state={}",
            self.oauth_base_url, self.client_id, state
        );
        Ok(auth_url)
    }
    
    /// 認証コードをトークンに交換
    pub async fn exchange_code(&self, auth_code: &str) -> Result<AuthToken, ZoomVideoMoverError> {
        let token_url = format!("{}/oauth/token", self.oauth_base_url);
        
        let response = self.client
            .post(&token_url)
            .header("content-type", "application/x-www-form-urlencoded")
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .body(format!(
                "grant_type=authorization_code&code={}&redirect_uri=http://localhost:8080/callback",
                auth_code
            ))
            .send()
            .await
            .map_err(|e| ZoomVideoMoverError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(ZoomVideoMoverError::AuthenticationError(
                format!("Token exchange failed: {}", response.status())
            ));
        }
        
        let token_data: serde_json::Value = response.json().await
            .map_err(|e| ZoomVideoMoverError::NetworkError(e.to_string()))?;
        
        Ok(AuthToken {
            access_token: token_data["access_token"].as_str().unwrap_or("").to_string(),
            token_type: token_data["token_type"].as_str().unwrap_or("Bearer").to_string(),
            expires_at: chrono::Utc::now() + chrono::Duration::seconds(token_data["expires_in"].as_i64().unwrap_or(3600)),
            refresh_token: token_data["refresh_token"].as_str().map(|s| s.to_string()),
            scopes: token_data["scope"].as_str().unwrap_or("").split(" ").map(|s| s.to_string()).collect(),
        })
    }
    
    /// 録画リストを取得
    pub async fn get_recordings(&self, from_date: &str, to_date: &str) -> Result<Vec<Recording>, ZoomVideoMoverError> {
        let url = format!("{}/v2/users/me/recordings?from={}&to={}", self.api_base_url, from_date, to_date);
        
        let response = self.client
            .get(&url)
            .bearer_auth(&self.access_token)
            .send()
            .await
            .map_err(|e| ZoomVideoMoverError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(ZoomVideoMoverError::NetworkError(
                format!("API request failed: {}", response.status())
            ));
        }
        
        let data: serde_json::Value = response.json().await
            .map_err(|e| ZoomVideoMoverError::NetworkError(e.to_string()))?;
        
        let mut recordings = Vec::new();
        if let Some(meetings) = data["meetings"].as_array() {
            for meeting in meetings {
                let recording = Recording {
                    meeting_id: meeting["id"].as_u64().unwrap_or(0).to_string(),
                    topic: meeting["topic"].as_str().unwrap_or("Unknown").to_string(),
                    start_time: chrono::DateTime::parse_from_rfc3339(
                        meeting["start_time"].as_str().unwrap_or("2024-01-01T00:00:00Z")
                    ).unwrap_or_default().with_timezone(&chrono::Utc),
                    duration: meeting["duration"].as_u64().unwrap_or(0) as u32,
                    recording_files: meeting["recording_files"].as_array().unwrap_or(&vec![]).iter().map(|f| {
                        RecordingFile {
                            id: f["id"].as_str().unwrap_or("").to_string(),
                            file_type: f["file_type"].as_str().unwrap_or("").to_string(),
                            file_size: f["file_size"].as_u64().unwrap_or(0),
                            download_url: f["download_url"].as_str().unwrap_or("").to_string(),
                            play_url: f["play_url"].as_str().map(|s| s.to_string()),
                            recording_start: chrono::DateTime::parse_from_rfc3339(
                                f["recording_start"].as_str().unwrap_or("2024-01-01T00:00:00Z")
                            ).unwrap_or_default().with_timezone(&chrono::Utc),
                            recording_end: chrono::DateTime::parse_from_rfc3339(
                                f["recording_end"].as_str().unwrap_or("2024-01-01T00:00:00Z")
                            ).unwrap_or_default().with_timezone(&chrono::Utc),
                        }
                    }).collect(),
                    ai_summary_available: false,
                };
                recordings.push(recording);
            }
        }
        
        Ok(recordings)
    }
    
    /// ファイルをダウンロード
    pub async fn download_file(
        &self, 
        request: DownloadRequest, 
        progress_sender: Option<tokio::sync::mpsc::Sender<u64>>
    ) -> Result<std::path::PathBuf, ZoomVideoMoverError> {
        use std::io::Write;
        
        let response = self.client
            .get(&request.download_url)
            .send()
            .await
            .map_err(|e| ZoomVideoMoverError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(ZoomVideoMoverError::NetworkError(
                format!("Download failed: {}", response.status())
            ));
        }
        
        let content = response.bytes().await
            .map_err(|e| ZoomVideoMoverError::NetworkError(e.to_string()))?;
        
        let mut file = std::fs::File::create(&request.output_path)
            .map_err(|e| ZoomVideoMoverError::FileSystemError(e.to_string()))?;
        
        file.write_all(&content)
            .map_err(|e| ZoomVideoMoverError::FileSystemError(e.to_string()))?;
        
        file.sync_all()
            .map_err(|e| ZoomVideoMoverError::FileSystemError(e.to_string()))?;
        
        // 進捗通知
        if let Some(sender) = progress_sender {
            let _ = sender.send(content.len() as u64).await;
        }
        
        Ok(request.output_path)
    }

}

// GUI関連の構造体（テスト用）
#[derive(Default)]
pub struct ZoomDownloaderApp {
    pub client_id: String,
    pub client_secret: String,
    pub output_dir: String,
    pub config_loaded: bool,
}

#[allow(dead_code)]
pub enum AppMessage {
    ConfigUpdated,
    DownloadStarted,
    DownloadCompleted,
    Error(String),
}

impl ZoomDownloaderApp {
    pub fn new() -> Self {
        Self::default()
    }
}

// eframeのApp traitの実装（テスト用）
impl eframe::App for ZoomDownloaderApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Zoom Recording Downloader");
            ui.label("GUI implementation placeholder");
        });
    }
}

pub mod windows_console {
    pub fn println_japanese(text: &str) {
        println!("{}", text);
    }
    
    pub fn setup_console_encoding() {
        // Simplified version for lib.rs - actual implementation in windows_console.rs
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}