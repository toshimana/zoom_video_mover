use std::fs;
use serde::{Deserialize, Serialize};
use reqwest::Client;
use chrono::{DateTime, Utc};
use eframe;
use std::time::Duration;
use tokio::time::{sleep, Instant};

/// ファイル名のサニタイズ
/// 
/// # 事前条件
/// - input は空でない文字列である
/// 
/// # 事後条件
/// - Windows/Linux/macOSで使用可能なファイル名が返される
/// - 特殊文字が適切に置換される
/// 
/// # 不変条件
/// - 入力文字列の意味は保たれる
fn sanitize_filename(input: &str) -> String {
    assert!(!input.is_empty(), "input must not be empty");
    
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
    
    // 長すぎる場合の切り詰め
    if result.len() > 200 {
        result.truncate(200);
        result = result.trim_end().to_string();
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
fn parse_datetime(datetime_str: &str) -> DateTime<Utc> {
    assert!(!datetime_str.is_empty(), "datetime_str must not be empty");
    
    chrono::DateTime::parse_from_rfc3339(datetime_str)
        .unwrap_or_else(|_| {
            chrono::DateTime::parse_from_str("2025-01-01T00:00:00Z", "%Y-%m-%dT%H:%M:%SZ")
                .expect("Default datetime should be valid")
        })
        .with_timezone(&chrono::Utc)
}

// カスタムエラー型の定義
#[derive(Debug)]
pub enum ZoomVideoMoverError {
    NetworkError(String),
    AuthenticationError(String),
    FileSystemError(String),
    ConfigError(String),
    RateLimitError(String),
    InvalidTokenError(String),
    ApiError { code: u16, message: String },
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
            ZoomVideoMoverError::RateLimitError(msg) => write!(f, "Rate limit error: {}", msg),
            ZoomVideoMoverError::InvalidTokenError(msg) => write!(f, "Invalid token error: {}", msg),
            ZoomVideoMoverError::ApiError { code, message } => write!(f, "API error {}: {}", code, message),
        }
    }
}

impl std::error::Error for ZoomVideoMoverError {}

// テスト用の構造体定義

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

#[derive(Debug, Clone)]
pub struct DownloadRequest {
    pub file_id: String,
    pub file_name: String,
    pub download_url: String,
    pub file_size: u64,
    pub output_path: std::path::PathBuf,
}

// OAuth Token構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub access_token: String,
    pub token_type: String,
    pub expires_at: DateTime<Utc>,
    pub refresh_token: Option<String>,
    pub scopes: Vec<String>,
}

impl AuthToken {
    /// トークンが有効かどうか確認
    pub fn is_valid(&self) -> bool {
        chrono::Utc::now() < self.expires_at && !self.access_token.is_empty()
    }
    
    /// 必要なスコープが含まれているか確認
    pub fn has_scope(&self, required_scope: &str) -> bool {
        self.scopes.iter().any(|scope| scope == required_scope)
    }
    
    /// 複数のスコープがすべて含まれているか確認
    pub fn has_all_scopes(&self, required_scopes: &[&str]) -> bool {
        required_scopes.iter().all(|&scope| self.has_scope(scope))
    }
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordingResponse {
    pub from: String,
    pub to: String,
    pub page_count: u32,
    pub page_size: u32,
    pub total_records: u32,
    pub meetings: Vec<MeetingRecording>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicSummary {
    #[serde(default)]
    pub topic_title: String,
    #[serde(default)]
    pub topic_content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedSection {
    #[serde(default)]
    pub section_title: String,
    #[serde(default)]
    pub section_content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryDetail {
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AISummaryResponse {
    #[serde(default)]
    pub meeting_uuid: String,
    #[serde(default)]
    pub summary_start_time: String,
    #[serde(default)]
    pub summary_end_time: String,
    #[serde(default)]
    pub summary_created_time: String,
    #[serde(default)]
    pub summary_last_modified_time: String,
    #[serde(default)]
    pub summary_title: String,
    #[serde(default)]
    pub summary_overview: String,
    #[serde(default)]
    pub summary_details: Vec<SummaryDetail>,
    #[serde(default)]
    pub summary_content: String,
    #[serde(default)]
    pub next_steps: Vec<String>,
    #[serde(default)]
    pub summary_keyword: Vec<String>,
    
    // Alternative field names that Zoom might use
    #[serde(default, alias = "summary")]
    pub summary: String,
    #[serde(default, alias = "key_points")]
    pub key_points: Vec<String>,
    #[serde(default, alias = "action_items")]
    pub action_items: Vec<String>,
    #[serde(default, alias = "meeting_id")]
    pub meeting_id: String,
    
    // Additional structured content fields
    #[serde(default, alias = "topic_summaries")]
    pub topic_summaries: Vec<TopicSummary>,
    #[serde(default, alias = "detailed_sections")]
    pub detailed_sections: Vec<DetailedSection>,
}

// Alternative structure for unknown AI summary formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericAISummary {
    #[serde(flatten)]
    pub data: serde_json::Map<String, serde_json::Value>,
}

pub struct ZoomRecordingDownloader {
    client: Client,
    access_token: String,
    oauth_base_url: String,
    api_base_url: String,
    client_id: String,
    client_secret: String,
    last_request_time: Option<Instant>,
    rate_limit_remaining: Option<u32>,
}

impl ZoomRecordingDownloader {
    /// レート制限をチェックして必要に応じて待機
    /// 
    /// # 副作用
    /// - 必要に応じてスレッドをスリープさせる
    /// - last_request_time を更新
    /// 
    /// # 事前条件
    /// - self は有効なインスタンスである
    /// 
    /// # 事後条件
    /// - レート制限に適合した状態でリクエスト可能
    /// - last_request_time が更新される
    /// 
    /// # 不変条件
    /// - 他のフィールドは変更されない
    async fn rate_limit_check(&mut self) {
        let now = Instant::now();
        
        if let Some(last_time) = self.last_request_time {
            let elapsed = now.duration_since(last_time);
            let min_interval = Duration::from_millis(50); // 20 requests/second = 50ms interval
            
            if elapsed < min_interval {
                let sleep_duration = min_interval - elapsed;
                sleep(sleep_duration).await;
            }
        }
        
        self.last_request_time = Some(now);
    }
    
    /// HTTPレスポンスからレート制限情報を取得
    /// 
    /// # 副作用
    /// - self.rate_limit_remaining を更新
    /// 
    /// # 事前条件
    /// - response は有効なHTTPレスポンスである
    /// 
    /// # 事後条件
    /// - レート制限情報が更新される
    /// - 429エラーの場合はRateLimitErrorが返される
    /// 
    /// # 不変条件
    /// - 他のフィールドは変更されない
    fn handle_rate_limit_response(&mut self, response: &reqwest::Response) -> Result<(), ZoomVideoMoverError> {
        // レート制限情報を取得
        if let Some(remaining) = response.headers().get("X-RateLimit-Remaining") {
            if let Ok(remaining_str) = remaining.to_str() {
                if let Ok(remaining_count) = remaining_str.parse::<u32>() {
                    self.rate_limit_remaining = Some(remaining_count);
                }
            }
        }
        
        // 429 Too Many Requestsのチェック
        if response.status() == 429 {
            let retry_after = response.headers()
                .get("Retry-After")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(60); // デフォルトは60秒
            
            return Err(ZoomVideoMoverError::RateLimitError(
                format!("Rate limit exceeded. Retry after {} seconds", retry_after)
            ));
        }
        
        Ok(())
    }
    
    /// 指数バックオフでリトライを実行
    /// 
    /// # 副作用
    /// - 指定された関数を実行し、必要に応じてリトライ
    /// - 失敗時には指数的に待機時間を延長
    /// 
    /// # 事前条件
    /// - operation は有効な非同期関数である
    /// - max_retries は0以上である
    /// 
    /// # 事後条件
    /// - 成功時: 関数の結果が返される
    /// - 失敗時: 最後のエラーが返される
    /// 
    /// # 不変条件
    /// - self の状態は関数実行によってのみ変更される
    async fn retry_with_exponential_backoff<F, T, Fut>(
        &mut self,
        mut operation: F,
        max_retries: u32,
    ) -> Result<T, ZoomVideoMoverError>
    where
        F: FnMut(&mut Self) -> Fut,
        Fut: std::future::Future<Output = Result<T, ZoomVideoMoverError>>,
    {
        assert!(max_retries > 0, "max_retries must be greater than 0");
        
        let mut last_error = None;
        
        for attempt in 0..=max_retries {
            match operation(self).await {
                Ok(result) => return Ok(result),
                Err(err) => {
                    last_error = Some(err);
                    
                    if attempt < max_retries {
                        let delay_ms = 1000 * (1 << attempt); // 1s, 2s, 4s, 8s...
                        let delay_duration = Duration::from_millis(delay_ms.min(30000)); // 最大3秒
                        sleep(delay_duration).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap())
    }
    /// 認証情報付きで新しいインスタンスを作成
    pub fn new(client_id: String, client_secret: String, _redirect_uri: String) -> Self {
        Self {
            client: Client::new(),
            access_token: String::new(),
            oauth_base_url: "https://zoom.us".to_string(),
            api_base_url: "https://api.zoom.us".to_string(),
            client_id,
            client_secret,
            last_request_time: None,
            rate_limit_remaining: None,
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
            last_request_time: None,
            rate_limit_remaining: None,
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
    /// 
    /// # 事前条件
    /// - client_id が設定されている
    /// - redirect_uri が有効なURLである
    /// 
    /// # 事後条件
    /// - 成功時: 有効な認証URLが返される
    /// - URLに必要なパラメータがすべて含まれる
    /// 
    /// # 不変条件
    /// - self の状態は変更されない
    pub fn generate_auth_url(&self, redirect_uri: &str, state: Option<&str>) -> Result<String, ZoomVideoMoverError> {
        // 事前条件のassertion
        assert!(!self.client_id.is_empty(), "client_id must be set");
        assert!(!redirect_uri.is_empty(), "redirect_uri must not be empty");
        
        let state_param = state.unwrap_or("default_state");
        let required_scopes = "recording:read user:read meeting:read";
        
        let auth_url = format!(
            "{}/oauth/authorize?response_type=code&client_id={}&redirect_uri={}&state={}&scope={}",
            self.oauth_base_url, 
            urlencoding::encode(&self.client_id),
            urlencoding::encode(redirect_uri),
            urlencoding::encode(state_param),
            urlencoding::encode(required_scopes)
        );
        
        // 事後条件のassertion
        debug_assert!(auth_url.contains("response_type=code"), "URL must contain response_type");
        debug_assert!(auth_url.contains(&self.client_id), "URL must contain client_id");
        
        Ok(auth_url)
    }
    
    /// 認証コードをトークンに交換
    /// 
    /// # 副作用
    /// - HTTPリクエストの送信
    /// 
    /// # 事前条件
    /// - auth_code は有効な認証コードである
    /// - client_id と client_secret が設定されている
    /// - インターネット接続が利用可能
    /// 
    /// # 事後条件
    /// - 成功時: 有効なAuthTokenが返される
    /// - トークンの有効期限が適切に設定される
    /// - 失敗時: 適切なエラーが返される
    /// 
    /// # 不変条件
    /// - self の認証情報は変更されない
    pub async fn exchange_code(&self, auth_code: &str, redirect_uri: &str) -> Result<AuthToken, ZoomVideoMoverError> {
        // 事前条件のassertion
        assert!(!auth_code.is_empty(), "auth_code must not be empty");
        assert!(!redirect_uri.is_empty(), "redirect_uri must not be empty");
        assert!(!self.client_id.is_empty(), "client_id must be set");
        assert!(!self.client_secret.is_empty(), "client_secret must be set");
        
        let token_url = format!("{}/oauth/token", self.oauth_base_url);
        
        let response = self.client
            .post(&token_url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .body(format!(
                "grant_type=authorization_code&code={}&redirect_uri={}",
                auth_code, redirect_uri
            ))
            .send()
            .await
            .map_err(|e| ZoomVideoMoverError::NetworkError(e.to_string()))?;
        
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ZoomVideoMoverError::AuthenticationError(
                format!("Token exchange failed: {} - {}", status, error_text)
            ));
        }
        
        let token_data: serde_json::Value = response.json().await
            .map_err(|e| ZoomVideoMoverError::NetworkError(e.to_string()))?;
        
        let expires_in = token_data["expires_in"].as_i64().unwrap_or(3600);
        let expires_at = chrono::Utc::now() + chrono::Duration::seconds(expires_in);
        
        let token = AuthToken {
            access_token: token_data["access_token"].as_str().unwrap_or("").to_string(),
            token_type: token_data["token_type"].as_str().unwrap_or("bearer").to_string(),
            expires_at,
            refresh_token: token_data["refresh_token"].as_str().map(|s| s.to_string()),
            scopes: token_data["scope"].as_str().unwrap_or("")
                .split(' ')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect(),
        };
        
        // 事後条件のassertion
        debug_assert!(!token.access_token.is_empty(), "access_token must not be empty");
        debug_assert!(token.expires_at > chrono::Utc::now(), "token must not be expired");
        
        Ok(token)
    }
    
    /// 録画リストを取得
    /// 
    /// # 副作用
    /// - Zoom APIへのHTTPリクエスト送信
    /// 
    /// # 事前条件
    /// - access_token が有効である
    /// - from_date と to_date が YYYY-MM-DD 形式である
    /// - from_date <= to_date である
    /// - インターネット接続が利用可能
    /// 
    /// # 事後条件
    /// - 成功時: 有効な録画リストが返される
    /// - 各録画の情報が正しくilenameと共に設定される
    /// - 失敗時: 適切なエラーが返される
    /// 
    /// # 不変条件
    /// - self の状態は変更されない
    /// - 入力パラメータは変更されない
    pub async fn get_recordings(&mut self, user_id: Option<&str>, from_date: &str, to_date: &str, page_size: Option<u32>) -> Result<RecordingResponse, ZoomVideoMoverError> {
        // 事前条件のassertion
        assert!(!self.access_token.is_empty(), "access_token must be set");
        assert!(!from_date.is_empty(), "from_date must not be empty");
        assert!(!to_date.is_empty(), "to_date must not be empty");
        debug_assert!(from_date.len() == 10, "from_date should be YYYY-MM-DD format");
        debug_assert!(to_date.len() == 10, "to_date should be YYYY-MM-DD format");
        
        let user_param = user_id.unwrap_or("me");
        let page_size_param = page_size.unwrap_or(30).min(300); // 最大300に制限
        
        let url = format!(
            "{}/v2/users/{}/recordings?from={}&to={}&page_size={}",
            self.api_base_url, user_param, from_date, to_date, page_size_param
        );
        
        // レート制限チェック
        self.rate_limit_check().await;
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Content-Type", "application/json")
            .header("User-Agent", "ZoomVideoMover/1.0")
            .send()
            .await
            .map_err(|e| ZoomVideoMoverError::NetworkError(e.to_string()))?;
        
        // レート制限レスポンス処理
        self.handle_rate_limit_response(&response)?;
        
        let status_code = response.status().as_u16();
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            
            // エラーコード別の処理
            match status_code {
                401 => return Err(ZoomVideoMoverError::InvalidTokenError(
                    "Access token is invalid or expired".to_string()
                )),
                403 => return Err(ZoomVideoMoverError::AuthenticationError(
                    "Insufficient permissions or invalid scopes".to_string()
                )),
                404 => return Err(ZoomVideoMoverError::ApiError {
                    code: 404,
                    message: "User not found or no recordings available".to_string(),
                }),
                _ => return Err(ZoomVideoMoverError::ApiError {
                    code: status_code,
                    message: format!("API request failed: {}", error_text),
                }),
            }
        }
        
        let data: serde_json::Value = response.json().await
            .map_err(|e| ZoomVideoMoverError::NetworkError(e.to_string()))?;
        
        let mut meetings = Vec::new();
        if let Some(meetings_array) = data["meetings"].as_array() {
            for meeting in meetings_array {
                let mut recording_files = Vec::new();
                if let Some(files_array) = meeting["recording_files"].as_array() {
                    for file in files_array {
                        // ファイル名を生成
                        let file_type = file["file_type"].as_str().unwrap_or("UNKNOWN");
                        let recording_type = file["recording_type"].as_str().unwrap_or("unknown");
                        let extension = match file_type {
                            "MP4" => "mp4",
                            "M4A" => "m4a", 
                            "CHAT" => "txt",
                            "TRANSCRIPT" => "vtt",
                            _ => "bin",
                        };
                        
                        let topic = meeting["topic"].as_str().unwrap_or("Meeting");
                        let meeting_id = meeting["id"].as_u64().unwrap_or(0);
                        let start_time = meeting["start_time"].as_str().unwrap_or("2025-01-01T00:00:00Z");
                        
                        // ファイル名をサニタイズして生成
                        let sanitized_topic = sanitize_filename(topic);
                        let filename = format!("{}_{}_{}_{}.{}", 
                            sanitized_topic,
                            meeting_id,
                            start_time[..10].replace("-", ""), // YYYYMMDD
                            recording_type,
                            extension
                        );
                        
                        let recording_file = RecordingFile {
                            id: file["id"].as_str().unwrap_or("").to_string(),
                            meeting_id: meeting["uuid"].as_str().unwrap_or("").to_string(),
                            recording_start: parse_datetime(
                                file["recording_start"].as_str().unwrap_or("2025-01-01T00:00:00Z")
                            ),
                            recording_end: parse_datetime(
                                file["recording_end"].as_str().unwrap_or("2025-01-01T00:00:00Z")
                            ),
                            file_type: file_type.to_string(),
                            file_extension: file["file_extension"].as_str().unwrap_or(extension).to_string(),
                            file_size: file["file_size"].as_u64().unwrap_or(0),
                            play_url: file["play_url"].as_str().map(|s| s.to_string()),
                            download_url: file["download_url"].as_str().unwrap_or("").to_string(),
                            status: file["status"].as_str().unwrap_or("unknown").to_string(),
                            recording_type: recording_type.to_string(),
                            filename,
                        };
                        recording_files.push(recording_file);
                    }
                }
                
                let meeting_recording = MeetingRecording {
                    uuid: meeting["uuid"].as_str().unwrap_or("").to_string(),
                    id: meeting["id"].as_u64().unwrap_or(0),
                    account_id: meeting["account_id"].as_str().unwrap_or("").to_string(),
                    host_id: meeting["host_id"].as_str().unwrap_or("").to_string(),
                    topic: meeting["topic"].as_str().unwrap_or("Unknown").to_string(),
                    meeting_type: meeting["type"].as_u64().unwrap_or(0) as u32,
                    start_time: meeting["start_time"].as_str().unwrap_or("2025-01-01T00:00:00Z").to_string(),
                    timezone: meeting["timezone"].as_str().unwrap_or("UTC").to_string(),
                    duration: meeting["duration"].as_u64().unwrap_or(0) as u32,
                    total_size: meeting["total_size"].as_u64().unwrap_or(0),
                    recording_count: meeting["recording_count"].as_u64().unwrap_or(0) as u32,
                    recording_files,
                };
                meetings.push(meeting_recording);
            }
        }
        
        let response = RecordingResponse {
            from: data["from"].as_str().unwrap_or(from_date).to_string(),
            to: data["to"].as_str().unwrap_or(to_date).to_string(),
            page_count: data["page_count"].as_u64().unwrap_or(1) as u32,
            page_size: data["page_size"].as_u64().unwrap_or(30) as u32,
            total_records: data["total_records"].as_u64().unwrap_or(0) as u32,
            meetings,
        };
        
        // 事後条件のassertion
        debug_assert!(response.page_size <= 300, "page_size should not exceed 300");
        debug_assert!(response.meetings.len() <= response.page_size as usize, "meetings count should not exceed page_size");
        
        Ok(response)
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

    /// ミーティングUUIDを使用してAI要約を取得する
    /// 
    /// # 副作用
    /// - HTTPリクエストの送信
    /// - デバッグレスポンスファイルの保存
    /// 
    /// # 事前条件
    /// - meeting_uuid は有効なZoomミーティングUUIDである
    /// - アクセストークンが有効である
    /// - 必要なスコープ（meeting:read）が許可されている
    /// 
    /// # 事後条件
    /// - 成功時: AI要約が利用可能な場合は Some(AISummaryResponse) を返す
    /// - AI要約が利用不可の場合は None を返す
    /// - 失敗時: 適切なエラーメッセージと共にエラーを返す
    /// 
    /// # 不変条件
    /// - self の access_token は変更されない
    pub async fn get_ai_summary(&mut self, meeting_uuid: &str) -> Result<Option<AISummaryResponse>, ZoomVideoMoverError> {
        println!("Requesting AI summary for UUID: {}", meeting_uuid);
        
        let uuid_variants = self.generate_uuid_variants(meeting_uuid);
        
        for (variant_idx, uuid_variant) in uuid_variants.iter().enumerate() {
            println!("Trying UUID variant {}/{}: {}", variant_idx + 1, uuid_variants.len(), uuid_variant);
            
            let endpoints = self.generate_ai_summary_endpoints(uuid_variant);
            
            for (endpoint_idx, url) in endpoints.iter().enumerate() {
                if let Some(summary) = self.try_single_endpoint(url, meeting_uuid, variant_idx, endpoint_idx).await? {
                    return Ok(Some(summary));
                }
            }
        }
        
        println!("ℹ No AI summary available for meeting {} (this is normal if AI Companion was not enabled)", meeting_uuid);
        println!("ℹ AI summaries require: 1) AI Companion enabled, 2) Meeting host access, 3) Processing time (up to 24h)");
        Ok(None)
    }

    /// ミーティングIDを使用してAI要約を取得する
    /// 
    /// # 副作用
    /// - HTTPリクエストの送信
    /// - デバッグレスポンスファイルの保存
    /// 
    /// # 事前条件
    /// - meeting_id は有効なZoomミーティングIDである
    /// - アクセストークンが有効である
    /// - 必要なスコープ（meeting:read）が許可されている
    /// 
    /// # 事後条件
    /// - 成功時: AI要約が利用可能な場合は Some(AISummaryResponse) を返す
    /// - AI要約が利用不可の場合は None を返す
    /// - 失敗時: 適切なエラーメッセージと共にエラーを返す
    pub async fn get_ai_summary_by_meeting_id(&mut self, meeting_id: u64) -> Result<Option<AISummaryResponse>, ZoomVideoMoverError> {
        println!("Requesting AI summary for Meeting ID: {}", meeting_id);
        
        let endpoints = vec![
            format!("https://api.zoom.us/v2/meetings/{}/batch_summary", meeting_id),
            format!("https://api.zoom.us/v2/meetings/{}/summary", meeting_id),
            format!("https://api.zoom.us/v2/meetings/{}/ai_companion_summary", meeting_id),
            format!("https://api.zoom.us/v2/meetings/{}/recording_summary", meeting_id),
            format!("https://api.zoom.us/v2/meetings/{}/meeting_summary", meeting_id),
            format!("https://api.zoom.us/v2/ai_companion/meetings/{}/summary", meeting_id),
            format!("https://api.zoom.us/v2/ai_companion/summary/{}", meeting_id),
            format!("https://api.zoom.us/v2/meetings/{}/ai_summary", meeting_id),
            format!("https://api.zoom.us/v2/meetings/{}/detailed_summary", meeting_id),
            format!("https://api.zoom.us/v2/meetings/{}/content_summary", meeting_id),
            format!("https://api.zoom.us/v2/meetings/{}/companion_summary", meeting_id),
            format!("https://api.zoom.us/v2/ai/meetings/{}/summary", meeting_id),
            format!("https://api.zoom.us/v2/ai/summary/meetings/{}", meeting_id),
            format!("https://api.zoom.us/v2/meetings/{}/analysis", meeting_id),
            format!("https://api.zoom.us/v2/meetings/{}/insights", meeting_id),
        ];
        
        for (i, url) in endpoints.iter().enumerate() {
            println!("Trying endpoint {}: {}", i + 1, url);
            
            self.rate_limit_check().await;
            
            let response = self.client
                .get(url)
                .bearer_auth(&self.access_token)
                .send()
                .await
                .map_err(|e| ZoomVideoMoverError::NetworkError(e.to_string()))?;

            match response.status().as_u16() {
                200 => {
                    println!("AI summary response received via Meeting ID!");
                    let response_text = response.text().await
                        .map_err(|e| ZoomVideoMoverError::NetworkError(e.to_string()))?;
                    println!("Response length: {} chars", response_text.len());
                    
                    // Save debug response
                    self.save_debug_response(&response_text, &format!("meeting_id_{}_endpoint_{}", meeting_id, i+1)).await;
                    
                    if let Ok(summary) = serde_json::from_str::<AISummaryResponse>(&response_text) {
                        println!("Successfully parsed AI summary!");
                        return Ok(Some(summary));
                    } else if let Ok(generic_json) = serde_json::from_str::<serde_json::Value>(&response_text) {
                        println!("Received valid JSON, converting to AI summary format");
                        let converted_summary = self.convert_generic_to_ai_summary(generic_json, &meeting_id.to_string());
                        return Ok(Some(converted_summary));
                    } else {
                        println!("Response is not valid JSON");
                        self.save_debug_response(&response_text, &format!("meeting_id_{}_endpoint_{}_invalid", meeting_id, i+1)).await;
                        continue;
                    }
                },
                404 => {
                    println!("Meeting ID endpoint {} returned 404 (not found)", i+1);
                },
                401 => {
                    println!("Meeting ID endpoint {} returned 401 (Unauthorized)", i+1);
                    println!("ℹ Note: Access token may be expired or insufficient scopes");
                },
                403 => {
                    println!("Meeting ID endpoint {} returned 403 (Forbidden)", i+1);
                    println!("ℹ Note: You may need to be the meeting host to access summaries");
                },
                429 => {
                    println!("Meeting ID endpoint {} returned 429 (Rate limit exceeded)", i+1);
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                },
                422 => {
                    println!("Meeting ID endpoint {} returned 422 (Unprocessable Entity)", i+1);
                    println!("ℹ This may indicate the summary is still being processed");
                },
                500..=599 => {
                    println!("Meeting ID endpoint {} returned {} (Server error)", i+1, response.status());
                },
                _ => {
                    println!("Meeting ID endpoint {} returned {} (Unknown error)", i+1, response.status());
                    if let Ok(error_text) = response.text().await {
                        if !error_text.is_empty() {
                            println!("Error details: {}", 
                                if error_text.len() > 300 { 
                                    format!("{}...", &error_text[..300]) 
                                } else { 
                                    error_text 
                                }
                            );
                        }
                    }
                }
            }
        }
        
        println!("No AI summary found via Meeting ID {}", meeting_id);
        Ok(None)
    }

    /// UUID形式のバリエーションを生成する（純粋関数）
    /// 
    /// # 事前条件
    /// - meeting_uuid は空でない文字列である
    /// 
    /// # 事後条件
    /// - UUIDのエンコーディング変形リストを返す
    /// - 副作用なし
    /// 
    /// # 不変条件
    /// - 入力パラメータを変更しない
    fn generate_uuid_variants(&self, meeting_uuid: &str) -> Vec<String> {
        vec![
            meeting_uuid.to_string(),
            meeting_uuid.replace("/", "%2F").replace("=", "%3D"),
        ]
    }

    /// AI要約エンドポイントのリストを生成する（純粋関数）
    /// 
    /// # 事前条件
    /// - uuid_variant は空でない文字列である
    /// 
    /// # 事後条件
    /// - エンドポイントURLのリストを返す
    /// - 副作用なし
    /// 
    /// # 不変条件
    /// - 入力パラメータを変更しない
    fn generate_ai_summary_endpoints(&self, uuid_variant: &str) -> Vec<String> {
        vec![
            format!("https://api.zoom.us/v2/meetings/{}/meeting_summary", uuid_variant),
            format!("https://api.zoom.us/v2/meetings/{}/recordings", uuid_variant),
            format!("https://api.zoom.us/v2/meetings/{}/summary", uuid_variant),
            format!("https://api.zoom.us/v2/meetings/{}/batch_summary", uuid_variant),
            format!("https://api.zoom.us/v2/meetings/{}/ai_companion_summary", uuid_variant),
            format!("https://api.zoom.us/v2/meetings/{}/ai_summary", uuid_variant),
            format!("https://api.zoom.us/v2/meetings/{}/detailed_summary", uuid_variant),
            format!("https://api.zoom.us/v2/meetings/{}/content_summary", uuid_variant),
            format!("https://api.zoom.us/v2/meetings/{}/companion_summary", uuid_variant),
            format!("https://api.zoom.us/v2/ai_companion/meetings/{}/summary", uuid_variant),
            format!("https://api.zoom.us/v2/ai_companion/summary/{}", uuid_variant),
            format!("https://api.zoom.us/v2/ai/meetings/{}/summary", uuid_variant),
            format!("https://api.zoom.us/v2/ai/summary/meetings/{}", uuid_variant),
            format!("https://api.zoom.us/v2/meetings/{}/analysis", uuid_variant),
            format!("https://api.zoom.us/v2/meetings/{}/insights", uuid_variant),
            format!("https://api.zoom.us/v2/meetings/{}/recording_summary", uuid_variant),
        ]
    }

    /// 単一のエンドポイントでAI要約を試行する
    /// 
    /// # 副作用
    /// - HTTPリクエストの送信
    /// - デバッグレスポンスファイルの保存
    /// 
    /// # 事前条件
    /// - url は有効なURLである
    /// - meeting_uuid は空でない文字列である
    /// - variant_idx と endpoint_idx は有効なインデックスである
    /// 
    /// # 事後条件
    /// - 成功時: AISummaryResponseを返す
    /// - 失敗時: None を返す
    /// 
    /// # 不変条件
    /// - ネットワークエラーは上位に伝播される
    async fn try_single_endpoint(&mut self, url: &str, meeting_uuid: &str, variant_idx: usize, endpoint_idx: usize) -> Result<Option<AISummaryResponse>, ZoomVideoMoverError> {
        println!("Trying endpoint {}: {}", endpoint_idx + 1, url);

        self.rate_limit_check().await;

        let response = self.client
            .get(url)
            .bearer_auth(&self.access_token)
            .send()
            .await
            .map_err(|e| ZoomVideoMoverError::NetworkError(e.to_string()))?;

        match response.status().as_u16() {
            200 => {
                println!("✓ Received 200 response!");
                let response_text = response.text().await
                    .map_err(|e| ZoomVideoMoverError::NetworkError(e.to_string()))?;
                println!("Response length: {} chars", response_text.len());
                
                // Save debug response
                self.save_debug_response(&response_text, &format!("uuid_{}_variant_{}_endpoint_{}", meeting_uuid.replace("/", "_").replace("=", "_"), variant_idx + 1, endpoint_idx + 1)).await;
                
                return Ok(self.extract_ai_summary_from_response(&response_text, url, meeting_uuid).await);
            },
            404 => {
                println!("Endpoint {} returned 404 (not found)", endpoint_idx + 1);
            },
            401 => {
                println!("Endpoint {} returned 401 (Unauthorized)", endpoint_idx + 1);
                println!("ℹ Note: Access token may be expired or insufficient scopes");
            },
            403 => {
                println!("Endpoint {} returned 403 (Forbidden)", endpoint_idx + 1);
                println!("ℹ Note: You may need to be the meeting host to access summaries");
            },
            429 => {
                println!("Endpoint {} returned 429 (Rate limit exceeded)", endpoint_idx + 1);
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            },
            422 => {
                println!("Endpoint {} returned 422 (Unprocessable Entity)", endpoint_idx + 1);
                println!("ℹ This may indicate the summary is still being processed");
            },
            500..=599 => {
                println!("Endpoint {} returned {} (Server error)", endpoint_idx + 1, response.status());
            },
            _ => {
                println!("Endpoint {} returned {} (Unknown error)", endpoint_idx + 1, response.status());
                if let Ok(error_text) = response.text().await {
                    if !error_text.is_empty() {
                        println!("Error details: {}", 
                            if error_text.len() > 300 { 
                                format!("{}...", &error_text[..300]) 
                            } else { 
                                error_text 
                            }
                        );
                    }
                }
            }
        }
        Ok(None)
    }

    /// HTTPレスポンスからAI要約を抽出する
    /// 
    /// # 事前条件
    /// - response_text は有効なHTTPレスポンステキストである
    /// - url は有効なURLである
    /// - meeting_uuid は空でない文字列である
    /// 
    /// # 事後条件
    /// - 成功時: AISummaryResponseを返す
    /// - 失敗時: None を返す
    /// 
    /// # 不変条件
    /// - 入力パラメータを変更しない
    async fn extract_ai_summary_from_response(&self, response_text: &str, url: &str, meeting_uuid: &str) -> Option<AISummaryResponse> {
        // Check if this is a recordings endpoint response
        if url.contains("/recordings") {
            if let Ok(recordings_data) = serde_json::from_str::<serde_json::Value>(response_text) {
                // Look for SUMMARY file type in recording files
                if let Some(recording_files) = recordings_data.get("recording_files").and_then(|v| v.as_array()) {
                    for file in recording_files {
                        if let Some(file_type) = file.get("file_type").and_then(|v| v.as_str()) {
                            if file_type == "SUMMARY" {
                                println!("✓ Found SUMMARY file in recordings!");
                                let converted_summary = self.convert_generic_to_ai_summary(file.clone(), meeting_uuid);
                                return Some(converted_summary);
                            }
                        }
                    }
                }
                println!("No SUMMARY file found in recordings");
                return None;
            }
        } else {
            // Try to parse as meeting summary response
            if let Ok(summary) = serde_json::from_str::<AISummaryResponse>(response_text) {
                println!("✓ Successfully parsed AI summary!");
                return Some(summary);
            } else if let Ok(generic_json) = serde_json::from_str::<serde_json::Value>(response_text) {
                println!("✓ Received valid JSON, converting to AI summary format");
                let converted_summary = self.convert_generic_to_ai_summary(generic_json, meeting_uuid);
                return Some(converted_summary);
            } else {
                println!("Response is not valid JSON");
                return None;
            }
        }
        None
    }

    /// 汎用JSON形式をAISummaryResponseに変換する
    /// 
    /// # 事前条件
    /// - json は有効なJSON Valueである
    /// - meeting_uuid は空でない文字列である
    /// 
    /// # 事後条件
    /// - 常に有効なAISummaryResponseを返す
    /// - 不明なフィールドは適切なデフォルト値で初期化される
    /// 
    /// # 不変条件
    /// - 入力パラメータを変更しない
    fn convert_generic_to_ai_summary(&self, json: serde_json::Value, meeting_uuid: &str) -> AISummaryResponse {
        // Extract common fields that might exist in various formats
        let summary_text = json.get("summary")
            .or_else(|| json.get("overview"))
            .or_else(|| json.get("content"))
            .or_else(|| json.get("brief_summary"))
            .or_else(|| json.get("executive_summary"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let key_points = json.get("key_points")
            .or_else(|| json.get("highlights"))
            .or_else(|| json.get("main_points"))
            .or_else(|| json.get("important_points"))
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        let action_items: Vec<String> = json.get("action_items")
            .or_else(|| json.get("next_steps"))
            .or_else(|| json.get("todos"))
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        // Extract summary_content (detailed markdown content)
        let summary_content = json.get("summary_content")
            .or_else(|| json.get("detailed_content"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        AISummaryResponse {
            meeting_uuid: meeting_uuid.to_string(),
            summary_start_time: json.get("start_time").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            summary_end_time: json.get("end_time").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            summary_created_time: json.get("created_time").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            summary_last_modified_time: json.get("modified_time").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            summary_title: json.get("title").and_then(|v| v.as_str()).unwrap_or("AI Generated Summary").to_string(),
            summary_overview: summary_text.clone(),
            summary_details: Vec::new(), // TODO: Extract from nested structures
            summary_content: summary_content,
            next_steps: action_items.clone(),
            summary_keyword: json.get("keywords")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
            summary: summary_text,
            key_points: key_points,
            action_items: action_items,
            meeting_id: json.get("meeting_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            topic_summaries: Vec::new(), // TODO: Extract topic summaries
            detailed_sections: Vec::new(), // TODO: Extract detailed sections
        }
    }

    /// デバッグレスポンスをファイルに保存
    /// 
    /// # 副作用
    /// - ファイルシステムへの書き込み
    /// 
    /// # 事前条件
    /// - response_text は空でない文字列である
    /// - suffix は有効なファイル名接尾辞である
    /// 
    /// # 事後条件
    /// - デバッグファイルが作成される（エラー時は警告出力のみ）
    /// 
    /// # 不変条件
    /// - self の状態は変更されない
    async fn save_debug_response(&self, response_text: &str, suffix: &str) {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S_%3f").to_string();
        let filename = format!("debug_responses/ai_response_{}_{}_{}.json", suffix, timestamp, std::process::id());
        
        // Create debug directory if it doesn't exist
        if let Err(e) = std::fs::create_dir_all("debug_responses") {
            println!("Warning: Could not create debug directory: {}", e);
            return;
        }
        
        if let Err(e) = std::fs::write(&filename, response_text) {
            println!("Warning: Could not save debug response to {}: {}", filename, e);
        } else {
            println!("Debug response saved to: {}", filename);
        }
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