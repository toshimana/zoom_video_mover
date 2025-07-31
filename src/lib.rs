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