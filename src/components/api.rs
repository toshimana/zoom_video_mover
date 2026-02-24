//! API統合コンポーネント
//!
//! # 責任
//! - Zoom API との通信
//! - レート制限管理
//! - リクエスト/レスポンス処理
//! - ページネーション処理
//! - エラーハンドリング

use crate::errors::{AppError, AppResult};
use crate::components::{ComponentLifecycle, Configurable};
use crate::components::auth::AuthToken;
use async_trait::async_trait;
use chrono::NaiveDate;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use std::time::{Duration, Instant};
use std::collections::VecDeque;
use log;

/// API設定
#[derive(Debug, Clone)]
pub struct ApiConfig {
    /// ベースURL
    pub base_url: String,
    /// レート制限設定
    pub rate_limit: RateLimitConfig,
    /// タイムアウト設定
    pub timeout: Duration,
    /// 最大リトライ回数
    pub max_retries: u32,
    /// ページサイズ（Zoom APIは最大300件/ページをサポート）
    pub default_page_size: u32,
    /// 最大ページ取得数（安全制限）
    pub max_pages: u32,
    /// ページ間待機時間（ミリ秒）
    pub page_interval_ms: u64,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.zoom.us/v2".to_string(),
            rate_limit: RateLimitConfig::default(),
            timeout: Duration::from_secs(30),
            max_retries: 3,
            default_page_size: 300,
            max_pages: 100,
            page_interval_ms: 100,
        }
    }
}

/// レート制限設定
///
/// Zoom APIプラン別上限: Free=2, Pro=20, Business+=60 req/sec
/// デフォルト10はPro以上で安全なマージン付きの値
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// 秒あたりのリクエスト数
    /// Zoom APIプラン別上限: Free=2, Pro=20, Business+=60
    pub requests_per_second: u32,
    /// バースト容量
    pub burst_size: u32,
    /// 最大待機時間
    pub max_wait_time: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 10,
            burst_size: 20,
            max_wait_time: Duration::from_secs(60),
        }
    }
}

/// Zoom録画ファイルタイプ
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordingFileType {
    MP4,
    M4A,
    #[serde(rename = "TRANSCRIPT")]
    Transcript,
    #[serde(rename = "CHAT")]
    Chat,
    #[serde(rename = "CC")]
    ClosedCaption,
    #[serde(rename = "TIMELINE")]
    Timeline,
    #[serde(rename = "SUMMARY")]
    Summary,
    #[serde(other)]
    Unknown,
}

impl std::fmt::Display for RecordingFileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MP4 => write!(f, "MP4"),
            Self::M4A => write!(f, "M4A"),
            Self::Transcript => write!(f, "TRANSCRIPT"),
            Self::Chat => write!(f, "CHAT"),
            Self::ClosedCaption => write!(f, "CC"),
            Self::Timeline => write!(f, "TIMELINE"),
            Self::Summary => write!(f, "SUMMARY"),
            Self::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

impl RecordingFileType {
    /// ファイルタイプに対応する拡張子を返す
    pub fn extension(&self) -> &str {
        match self {
            Self::MP4 => "mp4",
            Self::M4A => "m4a",
            Self::Transcript => "vtt",
            Self::Chat => "txt",
            Self::ClosedCaption => "vtt",
            Self::Timeline => "json",
            Self::Summary => "json",
            Self::Unknown => "dat",
        }
    }
}

/// 録画ファイル情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingFile {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub meeting_id: String,
    #[serde(default)]
    pub recording_start: String,
    #[serde(default)]
    pub recording_end: String,
    pub file_type: RecordingFileType,
    #[serde(default)]
    pub file_extension: String,
    #[serde(default)]
    pub file_size: u64,
    pub play_url: Option<String>,
    #[serde(default)]
    pub download_url: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub recording_type: String,
}

impl RecordingFile {
    /// 安定した識別子を返す（idが空の場合はfile_typeからフォールバック生成）
    pub fn stable_id(&self) -> String {
        if !self.id.is_empty() {
            self.id.clone()
        } else {
            format!("auto_{}", self.file_type.to_string().to_lowercase())
        }
    }
}

/// 会議録画情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingRecording {
    pub uuid: String,
    pub id: u64,
    #[serde(default)]
    pub account_id: String,
    pub host_id: String,
    pub topic: String,
    #[serde(rename = "type", default)]
    pub meeting_type: u32,
    pub start_time: String,
    #[serde(default)]
    pub timezone: String,
    pub duration: u32,
    #[serde(default)]
    pub total_size: u64,
    #[serde(default)]
    pub recording_count: u32,
    #[serde(default)]
    pub recording_files: Vec<RecordingFile>,
}

/// 録画検索レスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingSearchResponse {
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub page_count: u32,
    pub page_size: u32,
    pub total_records: u32,
    #[serde(default)]
    pub next_page_token: Option<String>,
    #[serde(default)]
    pub meetings: Vec<MeetingRecording>,
}

/// 録画検索リクエスト
#[derive(Debug, Clone)]
pub struct RecordingSearchRequest {
    pub user_id: Option<String>,
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub page_size: Option<u32>,
    pub next_page_token: Option<String>,
}

/// Token Bucket レート制限実装
#[derive(Debug)]
struct TokenBucket {
    capacity: u32,
    tokens: f64,
    refill_rate: f64,
    last_refill: Instant,
}

impl TokenBucket {
    fn new(requests_per_second: u32, burst_capacity: u32) -> Self {
        Self {
            capacity: burst_capacity,
            tokens: burst_capacity as f64,
            refill_rate: requests_per_second as f64,
            last_refill: Instant::now(),
        }
    }
    
    fn refill_tokens(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        
        let tokens_to_add = elapsed * self.refill_rate;
        self.tokens = (self.tokens + tokens_to_add).min(self.capacity as f64);
        self.last_refill = now;
    }
    
    fn try_consume_token(&mut self) -> bool {
        self.refill_tokens();
        
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
    
    fn time_until_next_token(&self) -> Duration {
        if self.tokens >= 1.0 {
            Duration::ZERO
        } else {
            let tokens_needed = 1.0 - self.tokens;
            let seconds_to_wait = tokens_needed / self.refill_rate;
            Duration::from_secs_f64(seconds_to_wait)
        }
    }
}

/// API統合コンポーネント
pub struct ApiComponent {
    /// API設定
    config: ApiConfig,
    /// HTTPクライアント
    http_client: Client,
    /// レート制限管理
    rate_limiter: Arc<Mutex<TokenBucket>>,
    /// 現在のトークン
    current_token: Arc<RwLock<Option<AuthToken>>>,
    /// API呼び出し統計
    metrics: Arc<Mutex<ApiCallMetrics>>,
}

/// API呼び出し統計
#[derive(Debug, Clone)]
pub struct ApiCallMetrics {
    pub total_calls: u64,
    pub successful_calls: u64,
    pub error_calls: u64,
    pub rate_limit_errors: u64,
    pub response_times: VecDeque<Duration>,
}

impl ApiCallMetrics {
    fn new() -> Self {
        Self {
            total_calls: 0,
            successful_calls: 0,
            error_calls: 0,
            rate_limit_errors: 0,
            response_times: VecDeque::with_capacity(100),
        }
    }
    
    fn record_call(&mut self, duration: Duration, success: bool) {
        self.total_calls += 1;
        if success {
            self.successful_calls += 1;
        } else {
            self.error_calls += 1;
        }
        
        self.response_times.push_back(duration);
        if self.response_times.len() > 100 {
            self.response_times.pop_front();
        }
    }
    
    fn average_response_time(&self) -> Option<Duration> {
        if self.response_times.is_empty() {
            return None;
        }
        
        let total: Duration = self.response_times.iter().sum();
        Some(total / self.response_times.len() as u32)
    }
}

impl ApiComponent {
    /// 新しいAPIコンポーネントを作成
    /// 
    /// # 事前条件
    /// - config は有効なAPI設定である
    /// 
    /// # 事後条件
    /// - ApiComponentインスタンスが作成される
    /// - 内部状態が適切に初期化される
    pub fn new(config: ApiConfig) -> Self {
        let http_client = Client::builder()
            .timeout(config.timeout)
            .build()
            .expect("Failed to create HTTP client");
        
        let rate_limiter = Arc::new(Mutex::new(TokenBucket::new(
            config.rate_limit.requests_per_second,
            config.rate_limit.burst_size,
        )));
        
        Self {
            config,
            http_client,
            rate_limiter,
            current_token: Arc::new(RwLock::new(None)),
            metrics: Arc::new(Mutex::new(ApiCallMetrics::new())),
        }
    }
    
    /// 認証トークンを設定
    /// 
    /// # 副作用
    /// - 内部のトークン状態を更新
    /// 
    /// # 事前条件
    /// - token は有効なAuthTokenである
    /// 
    /// # 事後条件
    /// - トークンが設定される
    pub async fn set_auth_token(&self, token: AuthToken) {
        assert!(!token.access_token.is_empty(), "access_token must not be empty");
        
        let mut current_token = self.current_token.write().await;
        *current_token = Some(token);
        
        log::info!("Auth token set for API component");
    }
    
    /// 録画データを検索
    /// 
    /// # 副作用
    /// - HTTPリクエストの送信
    /// - レート制限の消費
    /// - メトリクスの記録
    /// 
    /// # 事前条件
    /// - request は有効な検索リクエストである
    /// - 認証トークンが設定されている
    /// 
    /// # 事後条件
    /// - 成功時: 録画データのレスポンスが返される
    /// - 失敗時: 適切なエラーが返される
    pub async fn search_recordings(&self, request: RecordingSearchRequest) -> AppResult<RecordingSearchResponse> {
        // 事前条件の検証
        if request.from > request.to {
            return Err(AppError::validation("'from' date must be before or equal to 'to' date", None));
        }
        
        // レート制限チェック
        self.wait_for_rate_limit().await?;
        
        // 認証トークン取得
        let token = self.get_valid_token().await?;
        
        // API URL構築
        let user_id = request.user_id.as_deref().unwrap_or("me");
        let url = format!("{}/users/{}/recordings", self.config.base_url, user_id);
        
        // クエリパラメータ構築
        let mut query_params = vec![
            ("from", request.from.format("%Y-%m-%d").to_string()),
            ("to", request.to.format("%Y-%m-%d").to_string()),
        ];
        
        let page_size = request.page_size.unwrap_or(self.config.default_page_size);
        query_params.push(("page_size", page_size.to_string()));
        
        if let Some(next_page_token) = &request.next_page_token {
            query_params.push(("next_page_token", next_page_token.clone()));
        }
        
        // HTTPリクエスト実行
        let start_time = Instant::now();
        let response = self.http_client
            .get(&url)
            .bearer_auth(&token.access_token)
            .query(&query_params)
            .send()
            .await
            .map_err(|e| AppError::network("Failed to send API request", Some(e)))?;
        
        let duration = start_time.elapsed();
        
        // ステータスコードチェック
        let status = response.status();
        if !status.is_success() {
            self.record_api_call(duration, false).await;
            
            return match status {
                StatusCode::UNAUTHORIZED => Err(AppError::authentication("Unauthorized API access", None::<std::io::Error>)),
                StatusCode::TOO_MANY_REQUESTS => {
                    self.record_rate_limit_error().await;
                    let retry_after = response.headers()
                        .get(reqwest::header::RETRY_AFTER)
                        .and_then(|v| v.to_str().ok())
                        .and_then(|v| v.parse::<u64>().ok());
                    Err(AppError::rate_limit_with_retry("API rate limit exceeded", retry_after))
                },
                StatusCode::NOT_FOUND => Err(AppError::not_found("User or resource not found")),
                _ => {
                    let error_body = response.text().await.unwrap_or_default();
                    Err(AppError::external_service(format!("API error: {} - {}", status, error_body)))
                }
            };
        }
        
        // レスポンス解析（raw JSONをログ出力してからパース）
        let response_text = response.text().await
            .map_err(|e| AppError::network("Failed to read API response body", Some(e)))?;

        log::debug!("[DL-DIAG] API raw response (truncated): {}",
            if response_text.len() > 2000 { &response_text[..2000] } else { &response_text });

        // SUMMARYファイルのdownload_url有無を個別にINFOログ出力
        if let Ok(raw) = serde_json::from_str::<serde_json::Value>(&response_text) {
            if let Some(meetings) = raw.get("meetings").and_then(|m| m.as_array()) {
                for mtg in meetings {
                    let topic = mtg.get("topic").and_then(|t| t.as_str()).unwrap_or("?");
                    let uuid = mtg.get("uuid").and_then(|u| u.as_str()).unwrap_or("?");
                    if let Some(files) = mtg.get("recording_files").and_then(|f| f.as_array()) {
                        for f in files {
                            let ft = f.get("file_type").and_then(|v| v.as_str()).unwrap_or("?");
                            let has_url = f.get("download_url")
                                .and_then(|u| u.as_str())
                                .map(|u| !u.is_empty())
                                .unwrap_or(false);
                            log::info!("[DL-DIAG] API file: meeting='{}' uuid={} type={} has_download_url={}",
                                topic, uuid, ft, has_url);
                        }
                    }
                }
            }
        }

        let response_body: RecordingSearchResponse = serde_json::from_str(&response_text)
            .map_err(|e| AppError::data_format("Failed to parse API response", Some(e)))?;
        
        self.record_api_call(duration, true).await;
        
        // 事後条件の検証
        debug_assert!(response_body.page_size > 0, "page_size must be positive");
        
        Ok(response_body)
    }
    
    /// すべての録画データを取得（ページネーション対応）
    /// 
    /// # 副作用
    /// - 複数のHTTPリクエストの送信
    /// - レート制限の消費
    /// - メトリクスの記録
    /// 
    /// # 事前条件
    /// - request は有効な検索リクエストである
    /// - 認証トークンが設定されている
    /// 
    /// # 事後条件
    /// - 成功時: すべての録画データが返される
    /// - 失敗時: 適切なエラーが返される
    pub async fn get_all_recordings(&self, request: RecordingSearchRequest) -> AppResult<Vec<MeetingRecording>> {
        let mut all_meetings = Vec::new();
        let mut current_request = request;
        let mut total_pages = 0;
        
        loop {
            // ページ取得
            let response = self.search_recordings(current_request.clone()).await?;
            
            // 録画データを結果に追加
            all_meetings.extend(response.meetings);
            total_pages += 1;
            
            // 次ページの確認
            if let Some(next_token) = response.next_page_token {
                current_request.next_page_token = Some(next_token);
                
                // 最大ページ数制限チェック（安全のため）
                if total_pages >= self.config.max_pages {
                    log::warn!("Reached maximum page limit ({})", self.config.max_pages);
                    break;
                }

                // ページ間隔制御（レート制限対策）
                tokio::time::sleep(Duration::from_millis(self.config.page_interval_ms)).await;
            } else {
                break;
            }
        }
        
        log::info!("Retrieved {} meetings across {} pages", all_meetings.len(), total_pages);
        Ok(all_meetings)
    }
    
    /// 有効な認証トークンを取得
    async fn get_valid_token(&self) -> AppResult<AuthToken> {
        let token_guard = self.current_token.read().await;
        
        match token_guard.as_ref() {
            Some(token) if token.is_valid() => Ok(token.clone()),
            _ => Err(AppError::authentication("No valid auth token available", None::<std::io::Error>)),
        }
    }
    
    /// レート制限の待機
    async fn wait_for_rate_limit(&self) -> AppResult<()> {
        let start_time = Instant::now();
        
        loop {
            {
                let mut rate_limiter = self.rate_limiter.lock().await;
                if rate_limiter.try_consume_token() {
                    return Ok(());
                }
            }
            
            // 次のトークンまでの待機時間を計算
            let wait_time = {
                let rate_limiter = self.rate_limiter.lock().await;
                rate_limiter.time_until_next_token()
            };
            
            // 最大待機時間チェック
            if start_time.elapsed() + wait_time > self.config.rate_limit.max_wait_time {
                return Err(AppError::rate_limit("Rate limit wait time exceeded"));
            }
            
            tokio::time::sleep(wait_time).await;
        }
    }
    
    /// API呼び出し統計を記録
    async fn record_api_call(&self, duration: Duration, success: bool) {
        let mut metrics = self.metrics.lock().await;
        metrics.record_call(duration, success);
    }
    
    /// レート制限エラーを記録
    async fn record_rate_limit_error(&self) {
        let mut metrics = self.metrics.lock().await;
        metrics.rate_limit_errors += 1;
    }
    
    /// API統計情報を取得
    /// 
    /// # 副作用
    /// - 内部メトリクスデータの読み取り（非破壊的）
    /// 
    /// # 事前条件
    /// - コンポーネントが初期化されている
    /// 
    /// # 事後条件
    /// - 現在のAPI統計情報が返される
    /// - メトリクスの値は呼び出し時点でのスナップショット
    /// 
    /// # 不変条件
    /// - 内部のメトリクスデータは変更されない
    /// - コンポーネントの状態は保持される
    pub async fn get_metrics(&self) -> ApiCallMetrics {
        let metrics = self.metrics.lock().await;
        metrics.clone()
    }
}

#[async_trait]
impl ComponentLifecycle for ApiComponent {
    async fn initialize(&mut self) -> AppResult<()> {
        log::info!("Initializing ApiComponent");
        
        // 接続テスト（オプション）
        // 初期化時には認証トークンがない可能性があるため、スキップ
        
        log::info!("ApiComponent initialized successfully");
        Ok(())
    }
    
    async fn shutdown(&mut self) -> AppResult<()> {
        log::info!("Shutting down ApiComponent");
        
        // メトリクスのログ出力
        let metrics = self.get_metrics().await;
        log::info!("API call statistics: total={}, success={}, errors={}, rate_limit_errors={}",
            metrics.total_calls, metrics.successful_calls, metrics.error_calls, metrics.rate_limit_errors);
        
        if let Some(avg_response_time) = metrics.average_response_time() {
            log::info!("Average API response time: {:?}", avg_response_time);
        }
        
        log::info!("ApiComponent shut down successfully");
        Ok(())
    }
    
    async fn health_check(&self) -> bool {
        // HTTPクライアントの状態確認
        // 設定の妥当性確認
        !self.config.base_url.is_empty() && 
        self.config.rate_limit.requests_per_second > 0 &&
        self.config.default_page_size > 0
    }
}

impl Configurable<ApiConfig> for ApiComponent {
    fn update_config(&mut self, config: ApiConfig) -> AppResult<()> {
        self.config = config;
        
        // HTTPクライアントの再構築
        self.http_client = Client::builder()
            .timeout(self.config.timeout)
            .build()
            .map_err(|e| AppError::configuration("Failed to create HTTP client", Some(e)))?;
        
        // レート制限の更新
        let new_rate_limiter = TokenBucket::new(
            self.config.rate_limit.requests_per_second,
            self.config.rate_limit.burst_size,
        );
        self.rate_limiter = Arc::new(Mutex::new(new_rate_limiter));
        
        log::info!("ApiComponent configuration updated");
        Ok(())
    }
    
    fn get_config(&self) -> &ApiConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    
    #[test]
    fn test_token_bucket() {
        let mut bucket = TokenBucket::new(10, 20);
        
        // 初期状態では20トークン利用可能
        for _ in 0..20 {
            assert!(bucket.try_consume_token());
        }
        
        // トークンが枯渇
        assert!(!bucket.try_consume_token());
        
        // 待機時間が必要
        let wait_time = bucket.time_until_next_token();
        assert!(wait_time > Duration::ZERO);
    }
    
    #[tokio::test]
    async fn test_api_component_lifecycle() {
        let config = ApiConfig::default();
        let mut api_component = ApiComponent::new(config);
        
        // 初期化テスト
        assert!(api_component.initialize().await.is_ok());
        assert!(api_component.health_check().await);
        
        // 終了処理テスト
        assert!(api_component.shutdown().await.is_ok());
    }
    
    #[test]
    fn test_recording_search_request_validation() {
        let valid_request = RecordingSearchRequest {
            user_id: Some("test_user".to_string()),
            from: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            to: NaiveDate::from_ymd_opt(2025, 1, 31).unwrap(),
            page_size: Some(30),
            next_page_token: None,
        };
        
        // 日付範囲が有効
        assert!(valid_request.from <= valid_request.to);
    }

    #[test]
    fn test_recording_file_type_serialization() {
        // 各バリアントのシリアライズ確認
        assert_eq!(serde_json::to_string(&RecordingFileType::MP4).unwrap(), "\"MP4\"");
        assert_eq!(serde_json::to_string(&RecordingFileType::M4A).unwrap(), "\"M4A\"");
        assert_eq!(serde_json::to_string(&RecordingFileType::Transcript).unwrap(), "\"TRANSCRIPT\"");
        assert_eq!(serde_json::to_string(&RecordingFileType::Chat).unwrap(), "\"CHAT\"");
        assert_eq!(serde_json::to_string(&RecordingFileType::ClosedCaption).unwrap(), "\"CC\"");
        assert_eq!(serde_json::to_string(&RecordingFileType::Timeline).unwrap(), "\"TIMELINE\"");
        assert_eq!(serde_json::to_string(&RecordingFileType::Summary).unwrap(), "\"SUMMARY\"");
    }

    #[test]
    fn test_recording_file_type_deserialization() {
        // 既知のタイプのデシリアライズ確認
        assert_eq!(serde_json::from_str::<RecordingFileType>("\"MP4\"").unwrap(), RecordingFileType::MP4);
        assert_eq!(serde_json::from_str::<RecordingFileType>("\"M4A\"").unwrap(), RecordingFileType::M4A);
        assert_eq!(serde_json::from_str::<RecordingFileType>("\"TRANSCRIPT\"").unwrap(), RecordingFileType::Transcript);
        assert_eq!(serde_json::from_str::<RecordingFileType>("\"CHAT\"").unwrap(), RecordingFileType::Chat);
        assert_eq!(serde_json::from_str::<RecordingFileType>("\"CC\"").unwrap(), RecordingFileType::ClosedCaption);
        assert_eq!(serde_json::from_str::<RecordingFileType>("\"TIMELINE\"").unwrap(), RecordingFileType::Timeline);
        assert_eq!(serde_json::from_str::<RecordingFileType>("\"SUMMARY\"").unwrap(), RecordingFileType::Summary);

        // 未知のタイプはUnknownにフォールバック
        assert_eq!(serde_json::from_str::<RecordingFileType>("\"UNKNOWN_TYPE\"").unwrap(), RecordingFileType::Unknown);
    }

    #[test]
    fn test_recording_file_type_display() {
        assert_eq!(RecordingFileType::MP4.to_string(), "MP4");
        assert_eq!(RecordingFileType::ClosedCaption.to_string(), "CC");
        assert_eq!(RecordingFileType::Timeline.to_string(), "TIMELINE");
        assert_eq!(RecordingFileType::Summary.to_string(), "SUMMARY");
        assert_eq!(RecordingFileType::Unknown.to_string(), "UNKNOWN");
    }

    #[test]
    fn test_recording_file_summary_missing_fields() {
        // SUMMARYファイルはid, status, file_size, recording_type, play_urlが欠落する可能性がある
        let json = r#"{
            "file_type": "SUMMARY",
            "download_url": "https://example.com/download/summary.json"
        }"#;
        let file: RecordingFile = serde_json::from_str(json).unwrap();
        assert_eq!(file.file_type, RecordingFileType::Summary);
        assert_eq!(file.download_url, "https://example.com/download/summary.json");
        assert!(file.id.is_empty());
        assert!(file.status.is_empty());
        assert_eq!(file.file_size, 0);
        assert!(file.play_url.is_none());
    }

    #[test]
    fn test_api_config_defaults() {
        let config = ApiConfig::default();
        assert_eq!(config.default_page_size, 300);
        assert_eq!(config.max_pages, 100);
        assert_eq!(config.page_interval_ms, 100);
    }

    #[test]
    fn test_extension_method() {
        assert_eq!(RecordingFileType::MP4.extension(), "mp4");
        assert_eq!(RecordingFileType::M4A.extension(), "m4a");
        assert_eq!(RecordingFileType::Transcript.extension(), "vtt");
        assert_eq!(RecordingFileType::Chat.extension(), "txt");
        assert_eq!(RecordingFileType::ClosedCaption.extension(), "vtt");
        assert_eq!(RecordingFileType::Timeline.extension(), "json");
        assert_eq!(RecordingFileType::Summary.extension(), "json");
        assert_eq!(RecordingFileType::Unknown.extension(), "dat");
    }

    #[test]
    fn test_stable_id_with_id() {
        let file = RecordingFile {
            id: "abc123".to_string(),
            meeting_id: String::new(),
            recording_start: String::new(),
            recording_end: String::new(),
            file_type: RecordingFileType::MP4,
            file_extension: "mp4".to_string(),
            file_size: 0,
            play_url: None,
            download_url: "https://example.com/dl".to_string(),
            status: String::new(),
            recording_type: String::new(),
        };
        assert_eq!(file.stable_id(), "abc123");
    }

    #[test]
    fn test_stable_id_empty_id() {
        let file = RecordingFile {
            id: String::new(),
            meeting_id: String::new(),
            recording_start: String::new(),
            recording_end: String::new(),
            file_type: RecordingFileType::Summary,
            file_extension: String::new(),
            file_size: 0,
            play_url: None,
            download_url: "https://example.com/dl".to_string(),
            status: String::new(),
            recording_type: String::new(),
        };
        assert_eq!(file.stable_id(), "auto_summary");
    }

    #[test]
    fn test_recording_file_summary_missing_download_url() {
        // SUMMARYファイルでdownload_urlが省略された場合のデシリアライズ
        let json = r#"{
            "file_type": "SUMMARY"
        }"#;
        let file: RecordingFile = serde_json::from_str(json).unwrap();
        assert_eq!(file.file_type, RecordingFileType::Summary);
        assert!(file.download_url.is_empty());
        assert!(file.id.is_empty());
        assert_eq!(file.stable_id(), "auto_summary");
    }
}