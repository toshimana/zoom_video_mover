# API統合コンポーネント詳細設計書 - Zoom Video Mover

## 文書概要
**文書ID**: DES-API-001  
**コンポーネント名**: API統合コンポーネント（API Integration Component）  
**作成日**: 2025-08-03  
**作成者**: API設計者  
**レビューア**: システムアーキテクト  
**バージョン**: 1.0  

## コンポーネント概要

### 責任・役割
- **Zoom Cloud API連携**: 録画データ・メタデータの取得
- **レート制限管理**: API呼び出し頻度の制御・監視
- **ページネーション処理**: 大量データの効率的な取得
- **エラーハンドリング**: API固有エラーの分類・回復処理
- **JSON応答解析**: API レスポンスの構造化・検証

### アーキテクチャ位置
```
┌─────────────────────────────────────────────────────────────────┐
│                   Application Layer                             │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │              API Integration Component                       │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │ │
│  │  │   Zoom      │  │   Rate      │  │     Response        │ │ │
│  │  │  API Client │  │  Limiter    │  │     Parser          │ │ │
│  │  │             │  │             │  │                     │ │ │
│  │  └─────────────┘  └─────────────┘  └─────────────────────┘ │ │
│  └─────────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                 Infrastructure Layer                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │   HTTP      │  │   Auth      │  │    JSON                 │  │
│  │   Client    │  │ Integration │  │    Validator            │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## モジュール構造設計

### 内部モジュール構成
```rust
pub mod api {
    /// Zoom API クライアント実装
    pub mod zoom_client;
    
    /// レート制限管理
    pub mod rate_limiter;
    
    /// ページネーション処理
    pub mod pagination;
    
    /// レスポンス解析・検証
    pub mod response_parser;
    
    /// API エラーハンドリング
    pub mod error_handler;
    
    /// HTTP通信実装
    pub mod http_transport;
    
    /// JSON スキーマ検証
    pub mod schema_validator;
    
    /// API エンドポイント定義
    pub mod endpoints;
    
    /// エラー定義
    pub mod error;
    
    /// 設定・定数
    pub mod config;
}
```

### モジュール依存関係
```
zoom_client
    ├── → rate_limiter
    ├── → pagination
    ├── → response_parser
    ├── → error_handler
    ├── → http_transport
    └── → error

rate_limiter
    └── → error

pagination
    ├── → response_parser
    └── → error

response_parser
    ├── → schema_validator
    └── → error

error_handler
    └── → error

http_transport
    └── → error
```

## データ構造設計

### コアデータ構造

#### 1. 録画データ
```rust
/// Zoom 録画データ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoomRecording {
    /// 録画ID（Zoom固有）
    pub id: String,
    
    /// 会議ID
    pub meeting_id: String,
    
    /// 会議トピック
    pub topic: String,
    
    /// 録画開始時刻
    pub start_time: chrono::DateTime<chrono::Utc>,
    
    /// 録画終了時刻
    pub end_time: chrono::DateTime<chrono::Utc>,
    
    /// 録画時間（秒）
    pub duration: u32,
    
    /// 録画ファイル一覧
    pub recording_files: Vec<RecordingFile>,
    
    /// 会議ホスト情報
    pub host_info: HostInfo,
    
    /// 録画設定
    pub recording_settings: RecordingSettings,
    
    /// 共有権限
    pub sharing_info: Option<SharingInfo>,
    
    /// AI Companion 要約
    pub ai_summary: Option<AiSummary>,
}

/// 録画ファイル情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingFile {
    /// ファイルID
    pub id: String,
    
    /// ファイル名
    pub file_name: String,
    
    /// ファイル種別
    pub file_type: RecordingFileType,
    
    /// ファイルサイズ（バイト）
    pub file_size: u64,
    
    /// ダウンロードURL
    pub download_url: String,
    
    /// ダウンロードトークン
    pub download_token: Option<String>,
    
    /// ファイル作成時刻
    pub recording_start: chrono::DateTime<chrono::Utc>,
    
    /// ファイル終了時刻
    pub recording_end: chrono::DateTime<chrono::Utc>,
    
    /// ファイル形式（MP4, M4A, VTT, etc.）
    pub file_extension: String,
    
    /// ストリーミング情報
    pub play_url: Option<String>,
}

/// 録画ファイル種別
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecordingFileType {
    /// 動画ファイル（MP4）
    Video,
    
    /// 音声ファイル（M4A）
    Audio,
    
    /// チャットファイル（TXT）
    Chat,
    
    /// トランスクリプト（VTT/SRT）
    Transcript,
    
    /// 共有画面録画
    SharedScreen,
    
    /// ホワイトボード
    Whiteboard,
    
    /// AI要約ファイル
    Summary,
    
    /// その他
    Other(String),
}
```

#### 2. API リクエスト・レスポンス
```rust
/// 録画検索リクエスト
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingSearchRequest {
    /// 検索対象ユーザーID
    pub user_id: String,
    
    /// 検索開始日
    pub from: chrono::NaiveDate,
    
    /// 検索終了日
    pub to: chrono::NaiveDate,
    
    /// ページサイズ（1-300）
    pub page_size: Option<u32>,
    
    /// ページトークン
    pub next_page_token: Option<String>,
    
    /// 録画種別フィルタ
    pub recording_type: Option<RecordingType>,
    
    /// AI 要約を含むかどうか
    pub include_ai_summary: Option<bool>,
}

/// 録画検索レスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingSearchResponse {
    /// 会議一覧
    pub meetings: Vec<MeetingWithRecordings>,
    
    /// 次ページトークン
    pub next_page_token: Option<String>,
    
    /// ページサイズ
    pub page_size: u32,
    
    /// 合計件数
    pub total_records: Option<u32>,
}

/// 会議と録画データセット
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingWithRecordings {
    /// 会議情報
    pub meeting_info: MeetingInfo,
    
    /// 録画データ一覧
    pub recordings: Vec<ZoomRecording>,
}
```

#### 3. レート制限・監視
```rust
/// レート制限状況
#[derive(Debug, Clone)]
pub struct RateLimitStatus {
    /// 制限値（requests/second）
    pub limit: u32,
    
    /// 現在の使用量
    pub current_usage: u32,
    
    /// 残り使用可能回数
    pub remaining: u32,
    
    /// リセット時刻
    pub reset_time: chrono::DateTime<chrono::Utc>,
    
    /// 制限期間（秒）
    pub window_size: u32,
}

/// API 呼び出し統計
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCallMetrics {
    /// 総API呼び出し数
    pub total_calls: u64,
    
    /// 成功呼び出し数
    pub successful_calls: u64,
    
    /// エラー呼び出し数
    pub error_calls: u64,
    
    /// レート制限エラー数
    pub rate_limit_errors: u64,
    
    /// 平均レスポンス時間（ミリ秒）
    pub avg_response_time: f64,
    
    /// 最大レスポンス時間（ミリ秒）
    pub max_response_time: u64,
    
    /// 統計開始時刻
    pub start_time: chrono::DateTime<chrono::Utc>,
    
    /// 統計終了時刻
    pub end_time: chrono::DateTime<chrono::Utc>,
}
```

## インターフェース設計

### 公開API

#### 1. Zoom API クライアント
```rust
/// Zoom API クライアント - コンポーネントのメインインターフェース
#[async_trait]
pub trait ZoomApiClient: Send + Sync {
    /// 録画データの検索・取得
    async fn search_recordings(&self, request: RecordingSearchRequest) -> Result<Vec<ZoomRecording>, ApiError>;
    
    /// ページネーション付き録画データ取得
    async fn search_recordings_paginated(&self, request: RecordingSearchRequest) -> Result<PaginatedResult<ZoomRecording>, ApiError>;
    
    /// 特定録画の詳細情報取得
    async fn get_recording_details(&self, recording_id: &str) -> Result<ZoomRecording, ApiError>;
    
    /// AI要約の取得
    async fn get_ai_summary(&self, meeting_id: &str) -> Result<Option<AiSummary>, ApiError>;
    
    /// レート制限状況取得
    fn get_rate_limit_status(&self) -> RateLimitStatus;
    
    /// API呼び出し統計取得
    fn get_api_metrics(&self) -> ApiCallMetrics;
    
    /// 接続テスト
    async fn test_connection(&self) -> Result<(), ApiError>;
}
```

#### 2. 実装クラス
```rust
/// Zoom API クライアント実装
pub struct ZoomCloudApiClient {
    /// HTTP クライアント
    http_client: Arc<HttpClient>,
    
    /// 認証管理
    auth_client: Arc<dyn AuthenticationClient>,
    
    /// レート制限管理
    rate_limiter: Arc<TokenBucketRateLimiter>,
    
    /// レスポンス解析
    response_parser: Arc<ResponseParser>,
    
    /// エラーハンドラー
    error_handler: Arc<ApiErrorHandler>,
    
    /// API設定
    config: ApiConfig,
    
    /// メトリクス収集
    metrics_collector: Arc<ApiMetricsCollector>,
}

impl ZoomCloudApiClient {
    /// 新しいAPIクライアントを作成
    pub fn new(
        auth_client: Arc<dyn AuthenticationClient>,
        config: ApiConfig,
    ) -> Result<Self, ApiError> {
        let http_client = Arc::new(HttpClient::new(config.clone())?);
        let rate_limiter = Arc::new(TokenBucketRateLimiter::new(
            config.rate_limit.requests_per_second,
            config.rate_limit.burst_size,
        )?);
        let response_parser = Arc::new(ResponseParser::new()?);
        let error_handler = Arc::new(ApiErrorHandler::new());
        let metrics_collector = Arc::new(ApiMetricsCollector::new());
        
        Ok(Self {
            http_client,
            auth_client,
            rate_limiter,
            response_parser,
            error_handler,
            config,
            metrics_collector,
        })
    }
}

#[async_trait]
impl ZoomApiClient for ZoomCloudApiClient {
    async fn search_recordings(&self, request: RecordingSearchRequest) -> Result<Vec<ZoomRecording>, ApiError> {
        // 1. 入力パラメータ検証
        self.validate_search_request(&request)?;
        
        // 2. レート制限チェック
        self.rate_limiter.acquire().await?;
        
        // 3. 認証トークン取得
        let access_token = self.auth_client.get_valid_access_token().await
            .map_err(|e| ApiError::AuthenticationError(e))?;
        
        // 4. APIリクエスト実行
        let start_time = Instant::now();
        let response = self.execute_recordings_request(&request, &access_token).await;
        let elapsed = start_time.elapsed();
        
        // 5. メトリクス記録
        self.metrics_collector.record_api_call(elapsed, response.is_ok()).await;
        
        match response {
            Ok(recordings) => {
                // 6. レスポンス解析・検証
                let validated_recordings = self.response_parser.parse_recordings(recordings)?;
                Ok(validated_recordings)
            },
            Err(error) => {
                // 7. エラー処理・回復
                self.error_handler.handle_api_error(error).await
            }
        }
    }
    
    async fn search_recordings_paginated(&self, request: RecordingSearchRequest) -> Result<PaginatedResult<ZoomRecording>, ApiError> {
        let mut all_recordings = Vec::new();
        let mut current_request = request;
        let mut total_pages = 0;
        
        loop {
            // 1. ページネーション対応リクエスト実行
            let response = self.execute_paginated_request(&current_request).await?;
            
            // 2. 録画データを結果に追加
            all_recordings.extend(response.recordings);
            total_pages += 1;
            
            // 3. 次ページの確認
            if let Some(next_token) = response.next_page_token {
                current_request.next_page_token = Some(next_token);
                
                // 4. 最大ページ数制限チェック
                if total_pages >= self.config.max_pages_per_request {
                    return Err(ApiError::TooManyPages(total_pages));
                }
                
                // 5. ページ間隔制御
                tokio::time::sleep(self.config.page_interval).await;
            } else {
                break;
            }
        }
        
        Ok(PaginatedResult {
            data: all_recordings,
            total_pages,
            total_items: all_recordings.len(),
        })
    }
}
```

### 内部インターフェース

#### 1. レート制限管理
```rust
/// レート制限インターフェース
#[async_trait]
pub trait RateLimiter: Send + Sync {
    /// リクエスト実行許可を取得
    async fn acquire(&self) -> Result<RateLimitPermit, RateLimitError>;
    
    /// 現在のレート制限状況取得
    fn get_status(&self) -> RateLimitStatus;
    
    /// レート制限設定の動的更新
    async fn update_limits(&self, new_limit: u32) -> Result<(), RateLimitError>;
}

/// Token Bucket レート制限実装
pub struct TokenBucketRateLimiter {
    /// トークンバケット（同期プリミティブ）
    bucket: Arc<Mutex<TokenBucket>>,
    
    /// 制限設定
    config: RateLimitConfig,
    
    /// 統計情報
    metrics: Arc<RateLimitMetrics>,
}

impl TokenBucketRateLimiter {
    pub fn new(requests_per_second: u32, burst_size: u32) -> Result<Self, RateLimitError> {
        let bucket = TokenBucket::new(requests_per_second, burst_size)?;
        let config = RateLimitConfig {
            requests_per_second,
            burst_size,
            window_duration: Duration::seconds(1),
        };
        let metrics = Arc::new(RateLimitMetrics::new());
        
        Ok(Self {
            bucket: Arc::new(Mutex::new(bucket)),
            config,
            metrics,
        })
    }
}

#[async_trait]
impl RateLimiter for TokenBucketRateLimiter {
    async fn acquire(&self) -> Result<RateLimitPermit, RateLimitError> {
        let start_time = Instant::now();
        
        loop {
            {
                let mut bucket = self.bucket.lock().await;
                
                // 1. トークンバケットの更新
                bucket.refill_tokens();
                
                // 2. トークン取得試行
                if bucket.try_consume_token() {
                    self.metrics.record_successful_acquire(start_time.elapsed()).await;
                    return Ok(RateLimitPermit::new());
                }
            }
            
            // 3. トークン不足時の待機
            let wait_time = self.calculate_wait_time().await?;
            self.metrics.record_rate_limit_wait(wait_time).await;
            
            tokio::time::sleep(wait_time).await;
            
            // 4. 最大待機時間チェック
            if start_time.elapsed() > self.config.max_wait_time {
                return Err(RateLimitError::WaitTimeExceeded);
            }
        }
    }
    
    fn get_status(&self) -> RateLimitStatus {
        let bucket = self.bucket.blocking_lock();
        RateLimitStatus {
            limit: self.config.requests_per_second,
            current_usage: bucket.current_usage(),
            remaining: bucket.available_tokens(),
            reset_time: bucket.next_refill_time(),
            window_size: self.config.window_duration.num_seconds() as u32,
        }
    }
}
```

#### 2. レスポンス解析・検証
```rust
/// レスポンス解析インターフェース
#[async_trait]
pub trait ResponseParser: Send + Sync {
    /// 録画データレスポンス解析
    async fn parse_recordings(&self, raw_response: serde_json::Value) -> Result<Vec<ZoomRecording>, ParseError>;
    
    /// ページネーションレスポンス解析
    async fn parse_paginated_response(&self, raw_response: serde_json::Value) -> Result<RecordingSearchResponse, ParseError>;
    
    /// AI要約レスポンス解析
    async fn parse_ai_summary(&self, raw_response: serde_json::Value) -> Result<Option<AiSummary>, ParseError>;
    
    /// レスポンス整合性検証
    async fn validate_response(&self, response: &ZoomRecording) -> Result<(), ValidationError>;
}

/// レスポンス解析実装
pub struct ZoomResponseParser {
    /// JSON スキーマ検証
    schema_validator: Arc<JsonSchemaValidator>,
    
    /// データ正規化
    data_normalizer: Arc<DataNormalizer>,
    
    /// フィールド検証ルール
    validation_rules: Vec<Box<dyn FieldValidator>>,
}

impl ZoomResponseParser {
    pub fn new() -> Result<Self, ParseError> {
        let schema_validator = Arc::new(JsonSchemaValidator::new()?);
        let data_normalizer = Arc::new(DataNormalizer::new());
        let validation_rules = vec![
            Box::new(DateTimeValidator::new()),
            Box::new(UrlValidator::new()),
            Box::new(FileSizeValidator::new()),
            Box::new(DurationValidator::new()),
        ];
        
        Ok(Self {
            schema_validator,
            data_normalizer,
            validation_rules,
        })
    }
}

#[async_trait]
impl ResponseParser for ZoomResponseParser {
    async fn parse_recordings(&self, raw_response: serde_json::Value) -> Result<Vec<ZoomRecording>, ParseError> {
        // 1. JSON スキーマ検証
        self.schema_validator.validate(&raw_response, "recordings_response.json")?;
        
        // 2. 基本デシリアライゼーション
        let response: RecordingSearchResponse = serde_json::from_value(raw_response)
            .map_err(|e| ParseError::DeserializationError(e))?;
        
        // 3. データ正規化・検証
        let mut validated_recordings = Vec::new();
        for meeting in response.meetings {
            for recording in meeting.recordings {
                // 4. 個別録画データの検証
                self.validate_recording(&recording).await?;
                
                // 5. データ正規化
                let normalized_recording = self.data_normalizer.normalize_recording(recording)?;
                validated_recordings.push(normalized_recording);
            }
        }
        
        Ok(validated_recordings)
    }
    
    async fn validate_response(&self, recording: &ZoomRecording) -> Result<(), ValidationError> {
        // 1. 必須フィールド検証
        if recording.id.is_empty() {
            return Err(ValidationError::MissingRequiredField("id"));
        }
        
        // 2. カスタム検証ルール適用
        for validator in &self.validation_rules {
            validator.validate(recording)?;
        }
        
        // 3. 録画ファイル整合性検証
        for file in &recording.recording_files {
            self.validate_recording_file(file)?;
        }
        
        Ok(())
    }
}
```

## アルゴリズム設計

### レート制限アルゴリズム

#### Token Bucket 実装
```rust
/// Token Bucket レート制限アルゴリズム
pub struct TokenBucket {
    /// バケット容量（最大トークン数）
    capacity: u32,
    
    /// 現在のトークン数
    tokens: f64,
    
    /// トークン補充レート（tokens/second）
    refill_rate: f64,
    
    /// 最後の補充時刻
    last_refill: Instant,
}

impl TokenBucket {
    pub fn new(requests_per_second: u32, burst_capacity: u32) -> Result<Self, RateLimitError> {
        Ok(Self {
            capacity: burst_capacity,
            tokens: burst_capacity as f64,
            refill_rate: requests_per_second as f64,
            last_refill: Instant::now(),
        })
    }
    
    /// トークン補充処理
    pub fn refill_tokens(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        
        // 経過時間に基づくトークン追加
        let tokens_to_add = elapsed * self.refill_rate;
        self.tokens = (self.tokens + tokens_to_add).min(self.capacity as f64);
        self.last_refill = now;
    }
    
    /// トークン消費試行
    pub fn try_consume_token(&mut self) -> bool {
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
    
    /// 利用可能トークン数
    pub fn available_tokens(&self) -> u32 {
        self.tokens.floor() as u32
    }
    
    /// 次回トークン利用可能時刻
    pub fn time_until_next_token(&self) -> Duration {
        if self.tokens >= 1.0 {
            Duration::ZERO
        } else {
            let tokens_needed = 1.0 - self.tokens;
            let seconds_to_wait = tokens_needed / self.refill_rate;
            Duration::from_secs_f64(seconds_to_wait)
        }
    }
}
```

### ページネーション最適化

#### 適応的ページサイズ調整
```rust
/// 適応的ページネーション管理
pub struct AdaptivePagination {
    /// 初期ページサイズ
    initial_page_size: u32,
    
    /// 最大ページサイズ
    max_page_size: u32,
    
    /// 最小ページサイズ
    min_page_size: u32,
    
    /// 応答時間履歴
    response_time_history: VecDeque<Duration>,
    
    /// エラー率履歴
    error_rate_history: VecDeque<f64>,
}

impl AdaptivePagination {
    /// レスポンス性能に基づくページサイズ調整
    pub fn adjust_page_size(&mut self, response_time: Duration, had_error: bool) -> u32 {
        // 1. 応答時間履歴更新
        self.response_time_history.push_back(response_time);
        if self.response_time_history.len() > 10 {
            self.response_time_history.pop_front();
        }
        
        // 2. エラー率履歴更新
        let error_value = if had_error { 1.0 } else { 0.0 };
        self.error_rate_history.push_back(error_value);
        if self.error_rate_history.len() > 10 {
            self.error_rate_history.pop_front();
        }
        
        // 3. 平均応答時間計算
        let avg_response_time = self.response_time_history.iter()
            .map(|d| d.as_secs_f64())
            .sum::<f64>() / self.response_time_history.len() as f64;
        
        // 4. エラー率計算
        let error_rate = self.error_rate_history.iter().sum::<f64>() / self.error_rate_history.len() as f64;
        
        // 5. ページサイズ調整ロジック
        let current_size = self.initial_page_size;
        
        if error_rate > 0.1 {
            // エラー率高：ページサイズ削減
            (current_size / 2).max(self.min_page_size)
        } else if avg_response_time < 1.0 && error_rate < 0.05 {
            // 高性能：ページサイズ増加
            (current_size * 2).min(self.max_page_size)
        } else {
            // 現状維持
            current_size
        }
    }
    
    /// 最適化されたページネーション実行
    pub async fn execute_optimized_pagination<T>(
        &mut self,
        initial_request: RecordingSearchRequest,
        api_client: &dyn ZoomApiClient,
    ) -> Result<Vec<T>, ApiError> {
        let mut all_results = Vec::new();
        let mut current_request = initial_request;
        let mut current_page_size = self.initial_page_size;
        
        loop {
            // 1. 動的ページサイズ設定
            current_request.page_size = Some(current_page_size);
            
            // 2. APIリクエスト実行（性能測定付き）
            let start_time = Instant::now();
            let response = api_client.search_recordings_paginated(current_request.clone()).await;
            let elapsed = start_time.elapsed();
            
            match response {
                Ok(page_result) => {
                    // 3. 成功時の処理
                    all_results.extend(page_result.data);
                    
                    // 4. ページサイズ最適化
                    current_page_size = self.adjust_page_size(elapsed, false);
                    
                    // 5. 次ページ処理
                    if let Some(next_token) = page_result.next_page_token {
                        current_request.next_page_token = Some(next_token);
                    } else {
                        break;
                    }
                },
                Err(error) => {
                    // 6. エラー時の処理
                    current_page_size = self.adjust_page_size(elapsed, true);
                    
                    if current_page_size < self.min_page_size {
                        return Err(error);
                    }
                    
                    // 7. リトライ（小さなページサイズで）
                    continue;
                }
            }
        }
        
        Ok(all_results)
    }
}
```

## エラー処理設計

### エラー階層構造
```rust
/// API統合エラー定義
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// Zoom API固有エラー
    #[error("Zoom API error: {status_code} - {message}")]
    ZoomApiError {
        status_code: u16,
        error_code: String,
        message: String,
        details: Option<serde_json::Value>,
    },
    
    /// レート制限エラー
    #[error("Rate limit exceeded: {current_usage}/{limit} requests")]
    RateLimitExceeded {
        current_usage: u32,
        limit: u32,
        reset_time: chrono::DateTime<chrono::Utc>,
        retry_after: Duration,
    },
    
    /// ページネーションエラー
    #[error("Pagination error: {message}")]
    PaginationError {
        message: String,
        page_token: Option<String>,
        total_pages_processed: u32,
    },
    
    /// JSON解析エラー
    #[error("JSON parsing error: {source}")]
    JsonParseError {
        #[from]
        source: serde_json::Error,
        raw_data: Option<String>,
    },
    
    /// スキーマ検証エラー
    #[error("Schema validation error: {field} - {message}")]
    SchemaValidationError {
        field: String,
        message: String,
        expected_type: String,
        actual_value: Option<serde_json::Value>,
    },
    
    /// ネットワークエラー
    #[error("Network error: {source}")]
    NetworkError {
        #[from]
        source: reqwest::Error,
    },
    
    /// 認証エラー
    #[error("Authentication error: {source}")]
    AuthenticationError {
        #[from]
        source: super::auth::AuthError,
    },
    
    /// 設定エラー
    #[error("Configuration error: {message}")]
    ConfigurationError {
        message: String,
        parameter: String,
    },
}

/// エラー回復戦略実装
pub struct ApiErrorRecoveryStrategy {
    /// リトライ設定
    retry_config: RetryConfig,
    
    /// 回復アクション定義
    recovery_actions: HashMap<ApiErrorType, RecoveryAction>,
}

impl ApiErrorRecoveryStrategy {
    /// エラー種別に基づく自動回復
    pub async fn attempt_recovery(&self, error: &ApiError) -> RecoveryResult {
        match error {
            ApiError::RateLimitExceeded { retry_after, .. } => {
                // レート制限: 指定時間待機後リトライ
                RecoveryResult::RetryAfter(*retry_after)
            },
            
            ApiError::NetworkError { source } => {
                // ネットワークエラー: 指数バックオフリトライ
                if self.should_retry_network_error(source) {
                    let backoff_delay = self.calculate_backoff_delay();
                    RecoveryResult::RetryAfter(backoff_delay)
                } else {
                    RecoveryResult::Unrecoverable
                }
            },
            
            ApiError::AuthenticationError { .. } => {
                // 認証エラー: トークン更新試行
                RecoveryResult::RequiresReAuth
            },
            
            ApiError::ZoomApiError { status_code, error_code, .. } => {
                match (*status_code, error_code.as_str()) {
                    (404, _) => RecoveryResult::Unrecoverable,  // Not Found
                    (429, _) => {
                        // Too Many Requests: 適応的待機
                        let wait_time = self.calculate_adaptive_wait_time();
                        RecoveryResult::RetryAfter(wait_time)
                    },
                    (500..=599, _) => {
                        // Server Error: 短時間リトライ
                        RecoveryResult::RetryAfter(Duration::seconds(5))
                    },
                    _ => RecoveryResult::Unrecoverable,
                }
            },
            
            _ => RecoveryResult::Unrecoverable,
        }
    }
}
```

## テスト設計

### 単体テスト戦略
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    
    // HTTPサーバーモック
    async fn setup_mock_zoom_server() -> MockServer {
        let mock_server = MockServer::start().await;
        
        // 正常な録画検索レスポンス
        Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::path("/v2/users/me/recordings"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(create_mock_recordings_response()))
            .mount(&mock_server)
            .await;
        
        // レート制限レスポンス
        Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::header("X-Rate-Limit-Test", "true"))
            .respond_with(ResponseTemplate::new(429)
                .set_body_json(json!({
                    "code": 429,
                    "message": "Too Many Requests"
                }))
                .insert_header("Retry-After", "60"))
            .mount(&mock_server)
            .await;
        
        mock_server
    }
    
    #[tokio::test]
    async fn test_successful_recordings_search() {
        // Arrange
        let mock_server = setup_mock_zoom_server().await;
        let api_client = create_test_api_client(&mock_server.uri()).await;
        
        let request = RecordingSearchRequest {
            user_id: "test_user".to_string(),
            from: chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            to: chrono::NaiveDate::from_ymd_opt(2025, 1, 31).unwrap(),
            page_size: Some(30),
            next_page_token: None,
            recording_type: None,
            include_ai_summary: Some(true),
        };
        
        // Act
        let result = api_client.search_recordings(request).await;
        
        // Assert
        assert!(result.is_ok());
        let recordings = result.unwrap();
        assert!(!recordings.is_empty());
        assert_eq!(recordings[0].topic, "Test Meeting");
    }
    
    #[tokio::test]
    async fn test_rate_limit_handling() {
        // Arrange
        let mock_server = setup_mock_zoom_server().await;
        let api_client = create_test_api_client(&mock_server.uri()).await;
        
        // Rate limit エラーを発生させるリクエスト
        let request = RecordingSearchRequest {
            user_id: "rate_limit_test".to_string(),
            from: chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            to: chrono::NaiveDate::from_ymd_opt(2025, 1, 31).unwrap(),
            page_size: Some(30),
            next_page_token: None,
            recording_type: None,
            include_ai_summary: None,
        };
        
        // Act & Assert
        let result = api_client.search_recordings(request).await;
        
        match result {
            Err(ApiError::RateLimitExceeded { retry_after, .. }) => {
                assert!(retry_after.as_secs() > 0);
            },
            _ => panic!("Expected RateLimitExceeded error"),
        }
    }
}
```

### Property-basedテスト
```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        /// レート制限アルゴリズムの性質検証
        #[test]
        fn test_rate_limiter_properties(
            requests_per_second in 1u32..100u32,
            burst_size in 1u32..50u32,
            request_count in 1usize..200usize,
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let rate_limiter = TokenBucketRateLimiter::new(requests_per_second, burst_size).unwrap();
                
                let start_time = Instant::now();
                let mut successful_requests = 0;
                
                for _ in 0..request_count {
                    if rate_limiter.acquire().await.is_ok() {
                        successful_requests += 1;
                    }
                }
                
                let elapsed = start_time.elapsed();
                let expected_max_requests = (elapsed.as_secs() as u32 * requests_per_second) + burst_size;
                
                // Property: レート制限を超えたリクエストは許可されない
                prop_assert!(successful_requests <= expected_max_requests as usize);
                
                // Property: 適切な時間経過後は新しいリクエストが許可される
                tokio::time::sleep(Duration::seconds(2)).await;
                prop_assert!(rate_limiter.acquire().await.is_ok());
            });
        }
        
        /// ページネーション処理の完全性検証
        #[test]
        fn test_pagination_completeness(
            total_items in 1usize..1000usize,
            page_size in 1u32..100u32,
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                // モックページネーションレスポンス生成
                let mock_responses = create_mock_paginated_responses(total_items, page_size);
                
                let mut collected_items = Vec::new();
                let mut current_page = 0;
                
                for response in mock_responses {
                    let parsed = parse_mock_response(response).unwrap();
                    collected_items.extend(parsed.recordings);
                    current_page += 1;
                }
                
                // Property: 全アイテムが重複なく取得される
                prop_assert_eq!(collected_items.len(), total_items);
                
                // Property: ページ数が期待値と一致
                let expected_pages = (total_items as f64 / page_size as f64).ceil() as usize;
                prop_assert_eq!(current_page, expected_pages);
                
                // Property: アイテムの順序が保持される
                for (i, item) in collected_items.iter().enumerate() {
                    prop_assert_eq!(item.sequence_number, i);
                }
            });
        }
        
        /// JSON解析の冪等性検証
        #[test]
        fn test_json_parsing_idempotency(
            recording in arb_zoom_recording()
        ) {
            let parser = ZoomResponseParser::new().unwrap();
            
            // Serialize → Parse → Serialize → Parse
            let json1 = serde_json::to_value(&recording).unwrap();
            let parsed1 = parser.parse_single_recording(json1.clone()).unwrap();
            let json2 = serde_json::to_value(&parsed1).unwrap();
            let parsed2 = parser.parse_single_recording(json2).unwrap();
            
            // Property: 複数回の解析で結果が変わらない
            prop_assert_eq!(parsed1, parsed2);
        }
    }
    
    /// 任意のZoom録画データ生成
    fn arb_zoom_recording() -> impl Strategy<Value = ZoomRecording> {
        (
            "[a-zA-Z0-9]{10,20}",  // id
            "[a-zA-Z0-9]{10,20}",  // meeting_id
            "[\\w\\s]{5,50}",      // topic
            1u32..7200u32,         // duration
            prop::collection::vec(arb_recording_file(), 1..5),  // files
        ).prop_map(|(id, meeting_id, topic, duration, files)| {
            ZoomRecording {
                id,
                meeting_id,
                topic,
                start_time: chrono::Utc::now() - chrono::Duration::hours(1),
                end_time: chrono::Utc::now(),
                duration,
                recording_files: files,
                host_info: create_default_host_info(),
                recording_settings: create_default_settings(),
                sharing_info: None,
                ai_summary: None,
            }
        })
    }
}
```

## 性能・セキュリティ考慮事項

### 性能最適化
1. **HTTP接続プール**: Keep-Alive接続の再利用
2. **レスポンスキャッシュ**: 頻繁に取得されるメタデータのキャッシュ
3. **並列処理**: 独立したAPIリクエストの並列実行
4. **適応的ページサイズ**: 性能に基づく動的調整

### セキュリティ強化
1. **HTTPS強制**: 全API通信のTLS 1.3暗号化
2. **トークン保護**: アクセストークンのメモリ保護
3. **入力検証**: 全API パラメータの検証
4. **ログサニタイゼーション**: 機密情報の除去

---

**承認**:  
API設計者: [ ] 承認  
システムアーキテクト: [ ] 承認  
**承認日**: ___________