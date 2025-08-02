# 実装方針 - Zoom Video Mover

## 実装の基本方針

### 実装哲学・原則
- **Safety First**: Rustの型安全性・メモリ安全性を最大限活用
- **Performance by Design**: 非同期処理・ゼロコスト抽象化の活用
- **Fail Fast**: 早期エラー検出・明確なエラーメッセージ
- **Self-Documenting Code**: コード自体が仕様書となる明確性
- **Defensive Programming**: 想定外の状況に対する堅牢性
- **Incremental Development**: 小さな単位での継続的改善

### 品質基準
- **Zero Warnings**: `cargo clippy` 警告0件を維持
- **Type Safety**: 型システムによる不正状態の防止
- **Memory Safety**: 手動メモリ管理の排除・Rustの所有権活用
- **Thread Safety**: データ競合の完全排除
- **Error Handling**: すべてのエラーケースの明示的処理

## Rust実装規約

### コーディングスタイル

#### 命名規約
```rust
// モジュール・クレート名: snake_case
mod oauth_client;
use zoom_api_client;

// 構造体・列挙型・トレイト: PascalCase
struct ZoomRecordingDownloader;
enum AuthenticationState;
trait ApiClient;

// 関数・変数・フィールド: snake_case
fn authenticate_user();
let access_token = get_token();
struct Config { client_id: String }

// 定数・静的変数: SCREAMING_SNAKE_CASE
const MAX_CONCURRENT_DOWNLOADS: usize = 5;
static DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

// ライフタイム: 短い小文字
fn process_data<'a>(data: &'a str) -> &'a str;

// 型パラメータ: 大文字1文字から開始
fn generic_function<T, E>() where T: Clone, E: Error;
```

#### コード構造
```rust
// ファイル内構造順序
// 1. モジュール宣言
mod config;
mod oauth;

// 2. use宣言（グループ化・アルファベット順）
use std::collections::HashMap;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::config::Config;
use crate::oauth::AuthToken;

// 3. 型定義（構造体・列挙型・エイリアス）
type Result<T> = std::result::Result<T, ZoomVideoMoverError>;

// 4. 定数定義
const API_BASE_URL: &str = "https://api.zoom.us/v2";

// 5. 実装
impl ZoomRecordingDownloader {
    // コンストラクタ
    pub fn new() -> Self { }
    
    // パブリックメソッド
    pub async fn authenticate(&self) -> Result<AuthToken> { }
    
    // プライベートメソッド
    async fn make_api_request(&self) -> Result<Response> { }
}
```

### エラーハンドリング実装

#### カスタムエラータイプ定義
```rust
// thiserror を使用した包括的エラー定義
#[derive(Debug, thiserror::Error)]
pub enum ZoomVideoMoverError {
    // 認証関連エラー
    #[error("Authentication failed: {message}")]
    Authentication { message: String },
    
    #[error("Token expired at {expired_at}")]
    TokenExpired { expired_at: DateTime<Utc> },
    
    // ネットワーク関連エラー
    #[error("Network error: {source}")]
    Network { 
        #[from]
        source: reqwest::Error 
    },
    
    #[error("HTTP {status}: {message}")]
    Http { status: u16, message: String },
    
    // ファイルシステム関連エラー
    #[error("File operation failed: {operation} on {path}")]
    FileSystem { 
        operation: String, 
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    
    // 設定関連エラー
    #[error("Configuration error: {field} is {issue}")]
    Configuration { field: String, issue: String },
    
    // データ検証エラー
    #[error("Validation failed: {field} must {requirement}")]
    Validation { field: String, requirement: String },
}

// Result型エイリアス
pub type Result<T> = std::result::Result<T, ZoomVideoMoverError>;

// エラー変換の実装
impl From<std::io::Error> for ZoomVideoMoverError {
    fn from(error: std::io::Error) -> Self {
        ZoomVideoMoverError::FileSystem {
            operation: "unknown".to_string(),
            path: PathBuf::new(),
            source: error,
        }
    }
}
```

#### エラー処理パターン
```rust
// 1. 早期リターンパターン
pub async fn download_file(&self, url: &str, path: &Path) -> Result<u64> {
    // 事前検証
    if url.is_empty() {
        return Err(ZoomVideoMoverError::Validation {
            field: "url".to_string(),
            requirement: "not be empty".to_string(),
        });
    }
    
    // HTTP リクエスト
    let response = self.client
        .get(url)
        .send()
        .await
        .map_err(|e| ZoomVideoMoverError::Network { source: e })?;
    
    // ステータス確認
    if !response.status().is_success() {
        return Err(ZoomVideoMoverError::Http {
            status: response.status().as_u16(),
            message: format!("Failed to download from {}", url),
        });
    }
    
    // ファイル保存
    let bytes = response.bytes().await?;
    tokio::fs::write(path, bytes).await
        .map_err(|e| ZoomVideoMoverError::FileSystem {
            operation: "write".to_string(),
            path: path.to_path_buf(),
            source: e,
        })?;
    
    Ok(bytes.len() as u64)
}

// 2. エラー文脈追加パターン
pub async fn process_with_context<F, T>(&self, operation: &str, task: F) -> Result<T>
where
    F: Future<Output = Result<T>>,
{
    task.await.map_err(|e| {
        log::error!("Operation '{}' failed: {}", operation, e);
        e // エラーをそのまま伝播、ログに記録
    })
}

// 3. エラー回復パターン
pub async fn download_with_retry(&self, url: &str, max_attempts: u32) -> Result<Vec<u8>> {
    let mut last_error = None;
    
    for attempt in 1..=max_attempts {
        match self.download_file_once(url).await {
            Ok(data) => return Ok(data),
            Err(e) => {
                log::warn!("Download attempt {} failed: {}", attempt, e);
                last_error = Some(e);
                
                // 指数バックオフ
                if attempt < max_attempts {
                    let delay = Duration::from_secs(2u64.pow(attempt.min(6)));
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
    
    Err(last_error.unwrap())
}
```

### 非同期処理実装

#### tokio実装パターン
```rust
// 1. 基本的な非同期関数
pub async fn authenticate_user(&self, client_id: &str, client_secret: &str) -> Result<AuthToken> {
    // 事前条件チェック
    debug_assert!(!client_id.is_empty(), "client_id must not be empty");
    debug_assert!(!client_secret.is_empty(), "client_secret must not be empty");
    
    // OAuth認証フロー
    let auth_url = self.generate_auth_url(client_id).await?;
    
    // ユーザー認証（外部ブラウザ）
    self.open_browser(&auth_url)?;
    
    // 認証コード待機
    let auth_code = self.wait_for_auth_code().await?;
    
    // トークン交換
    let token = self.exchange_code_for_token(client_id, client_secret, &auth_code).await?;
    
    // 事後条件チェック
    debug_assert!(!token.access_token.is_empty(), "access_token must not be empty");
    debug_assert!(token.expires_at > Utc::now(), "token must not be expired");
    
    Ok(token)
}

// 2. 並列処理の実装
pub async fn download_multiple_files(&self, requests: Vec<DownloadRequest>) -> Result<Vec<PathBuf>> {
    use tokio::sync::Semaphore;
    
    // 同時ダウンロード数制限
    let semaphore = Arc::new(Semaphore::new(self.config.max_concurrent));
    let mut tasks = Vec::new();
    
    for request in requests {
        let permit = semaphore.clone().acquire_owned().await?;
        let downloader = self.clone();
        
        let task = tokio::spawn(async move {
            let _permit = permit; // permitをタスク終了まで保持
            downloader.download_single_file(request).await
        });
        
        tasks.push(task);
    }
    
    // すべてのタスクの完了を待機
    let results = futures::future::try_join_all(tasks).await?;
    let paths: Result<Vec<_>> = results.into_iter().collect();
    
    paths
}

// 3. タイムアウト付き処理
pub async fn api_call_with_timeout<T>(&self, operation: impl Future<Output = Result<T>>) -> Result<T> {
    let timeout_duration = Duration::from_secs(self.config.api_timeout_seconds);
    
    match tokio::time::timeout(timeout_duration, operation).await {
        Ok(result) => result,
        Err(_) => Err(ZoomVideoMoverError::Timeout {
            operation: "API call".to_string(),
            timeout_seconds: self.config.api_timeout_seconds,
        }),
    }
}
```

#### チャネル通信実装
```rust
// 1. 非同期メッセージング
#[derive(Debug, Clone)]
pub enum AppMessage {
    // コマンド（UI → ビジネスロジック）
    StartAuthentication,
    SearchRecordings { from: String, to: String },
    DownloadFiles(Vec<DownloadRequest>),
    CancelOperation,
    
    // イベント（ビジネスロジック → UI）
    AuthenticationCompleted(AuthToken),
    RecordingsFound(Vec<Recording>),
    DownloadProgress { file_id: String, progress: f64 },
    OperationCompleted,
    ErrorOccurred(ZoomVideoMoverError),
}

// 2. メッセージハンドラー
pub struct MessageHandler {
    command_receiver: mpsc::Receiver<AppMessage>,
    event_sender: broadcast::Sender<AppMessage>,
    downloader: ZoomRecordingDownloader,
}

impl MessageHandler {
    pub async fn run(&mut self) {
        while let Some(message) = self.command_receiver.recv().await {
            match message {
                AppMessage::StartAuthentication => {
                    self.handle_authentication().await;
                }
                AppMessage::SearchRecordings { from, to } => {
                    self.handle_search_recordings(&from, &to).await;
                }
                AppMessage::DownloadFiles(requests) => {
                    self.handle_download_files(requests).await;
                }
                AppMessage::CancelOperation => {
                    self.handle_cancellation().await;
                }
                _ => {
                    log::warn!("Unexpected command message: {:?}", message);
                }
            }
        }
    }
    
    async fn handle_authentication(&mut self) {
        match self.downloader.authenticate().await {
            Ok(token) => {
                let _ = self.event_sender.send(AppMessage::AuthenticationCompleted(token));
            }
            Err(error) => {
                let _ = self.event_sender.send(AppMessage::ErrorOccurred(error));
            }
        }
    }
}

// 3. 進捗通知の実装
pub struct ProgressReporter {
    sender: mpsc::Sender<ProgressUpdate>,
}

impl ProgressReporter {
    pub async fn report_progress(&self, file_id: &str, current: u64, total: u64) -> Result<()> {
        let progress = if total > 0 {
            (current as f64) / (total as f64)
        } else {
            0.0
        };
        
        let update = ProgressUpdate {
            file_id: file_id.to_string(),
            bytes_current: current,
            bytes_total: total,
            progress_ratio: progress,
            timestamp: Utc::now(),
        };
        
        self.sender.send(update).await
            .map_err(|_| ZoomVideoMoverError::ChannelClosed)?;
        
        Ok(())
    }
}
```

### 型安全性実装

#### 型による不正状態防止
```rust
// 1. NewType パターンによる型安全性
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MeetingId(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MeetingUuid(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileId(String);

impl MeetingId {
    pub fn new(id: String) -> Result<Self> {
        if id.is_empty() {
            return Err(ZoomVideoMoverError::Validation {
                field: "meeting_id".to_string(),
                requirement: "not be empty".to_string(),
            });
        }
        
        // Meeting IDは数値のみ
        if !id.chars().all(char::is_numeric) {
            return Err(ZoomVideoMoverError::Validation {
                field: "meeting_id".to_string(),
                requirement: "contain only digits".to_string(),
            });
        }
        
        Ok(MeetingId(id))
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for MeetingId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// 2. Builder パターンによる段階的構築
#[derive(Debug)]
pub struct DownloadRequestBuilder {
    file_id: Option<FileId>,
    url: Option<String>,
    output_path: Option<PathBuf>,
    file_size: Option<u64>,
}

impl DownloadRequestBuilder {
    pub fn new() -> Self {
        Self {
            file_id: None,
            url: None,
            output_path: None,
            file_size: None,
        }
    }
    
    pub fn file_id(mut self, id: FileId) -> Self {
        self.file_id = Some(id);
        self
    }
    
    pub fn url(mut self, url: String) -> Self {
        self.url = Some(url);
        self
    }
    
    pub fn output_path(mut self, path: PathBuf) -> Self {
        self.output_path = Some(path);
        self
    }
    
    pub fn file_size(mut self, size: u64) -> Self {
        self.file_size = Some(size);
        self
    }
    
    pub fn build(self) -> Result<DownloadRequest> {
        Ok(DownloadRequest {
            file_id: self.file_id.ok_or_else(|| ZoomVideoMoverError::Validation {
                field: "file_id".to_string(),
                requirement: "be specified".to_string(),
            })?,
            url: self.url.ok_or_else(|| ZoomVideoMoverError::Validation {
                field: "url".to_string(),
                requirement: "be specified".to_string(),
            })?,
            output_path: self.output_path.ok_or_else(|| ZoomVideoMoverError::Validation {
                field: "output_path".to_string(),
                requirement: "be specified".to_string(),
            })?,
            file_size: self.file_size.unwrap_or(0),
        })
    }
}

// 3. 状態型による不正遷移防止
#[derive(Debug)]
pub struct AuthenticationFlow<State> {
    client_id: String,
    client_secret: String,
    _state: PhantomData<State>,
}

// 状態マーカー型
#[derive(Debug)]
pub struct Unconfigured;
#[derive(Debug)]
pub struct Configured;
#[derive(Debug)]
pub struct UrlGenerated;
#[derive(Debug)]
pub struct Authenticated;

impl AuthenticationFlow<Unconfigured> {
    pub fn new() -> Self {
        Self {
            client_id: String::new(),
            client_secret: String::new(),
            _state: PhantomData,
        }
    }
    
    pub fn configure(self, client_id: String, client_secret: String) -> Result<AuthenticationFlow<Configured>> {
        if client_id.is_empty() || client_secret.is_empty() {
            return Err(ZoomVideoMoverError::Configuration {
                field: "oauth_credentials".to_string(),
                issue: "missing or empty".to_string(),
            });
        }
        
        Ok(AuthenticationFlow {
            client_id,
            client_secret,
            _state: PhantomData,
        })
    }
}

impl AuthenticationFlow<Configured> {
    pub async fn generate_auth_url(self) -> Result<(AuthenticationFlow<UrlGenerated>, String)> {
        let auth_url = format!(
            "https://zoom.us/oauth/authorize?response_type=code&client_id={}&redirect_uri={}",
            self.client_id,
            "http://localhost:8080/callback"
        );
        
        let new_state = AuthenticationFlow {
            client_id: self.client_id,
            client_secret: self.client_secret,
            _state: PhantomData,
        };
        
        Ok((new_state, auth_url))
    }
}

impl AuthenticationFlow<UrlGenerated> {
    pub async fn exchange_code(self, auth_code: &str) -> Result<(AuthenticationFlow<Authenticated>, AuthToken)> {
        // トークン交換の実装
        let token = self.perform_token_exchange(auth_code).await?;
        
        let new_state = AuthenticationFlow {
            client_id: self.client_id,
            client_secret: self.client_secret,
            _state: PhantomData,
        };
        
        Ok((new_state, token))
    }
}
```

### テスト支援実装

#### 依存性注入による抽象化
```rust
// 1. トレイトベースの抽象化
#[async_trait::async_trait]
pub trait HttpClient: Send + Sync {
    async fn get(&self, url: &str) -> Result<Response>;
    async fn post(&self, url: &str, body: Vec<u8>) -> Result<Response>;
}

#[async_trait::async_trait]
pub trait FileStorage: Send + Sync {
    async fn read(&self, path: &Path) -> Result<Vec<u8>>;
    async fn write(&self, path: &Path, data: &[u8]) -> Result<()>;
    async fn exists(&self, path: &Path) -> bool;
}

// 2. 本番実装
pub struct ReqwestHttpClient {
    client: reqwest::Client,
}

#[async_trait::async_trait]
impl HttpClient for ReqwestHttpClient {
    async fn get(&self, url: &str) -> Result<Response> {
        let response = self.client.get(url).send().await?;
        Ok(Response::from_reqwest(response).await?)
    }
    
    async fn post(&self, url: &str, body: Vec<u8>) -> Result<Response> {
        let response = self.client.post(url).body(body).send().await?;
        Ok(Response::from_reqwest(response).await?)
    }
}

// 3. テスト用モック実装
pub struct MockHttpClient {
    responses: HashMap<String, Result<Response>>,
}

#[async_trait::async_trait]
impl HttpClient for MockHttpClient {
    async fn get(&self, url: &str) -> Result<Response> {
        self.responses.get(url)
            .cloned()
            .unwrap_or_else(|| Err(ZoomVideoMoverError::Network {
                source: reqwest::Error::from(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Mock response not found"
                ))
            }))
    }
    
    async fn post(&self, url: &str, _body: Vec<u8>) -> Result<Response> {
        self.get(url).await
    }
}

// 4. 依存性注入のコンストラクタ
impl ZoomRecordingDownloader {
    // 本番用コンストラクタ
    pub fn new() -> Self {
        Self {
            http_client: Box::new(ReqwestHttpClient::new()),
            file_storage: Box::new(TokioFileStorage::new()),
            config: Config::default(),
        }
    }
    
    // テスト用コンストラクタ
    pub fn new_with_dependencies(
        http_client: Box<dyn HttpClient>,
        file_storage: Box<dyn FileStorage>,
        config: Config,
    ) -> Self {
        Self {
            http_client,
            file_storage,
            config,
        }
    }
}
```

#### テストユーティリティ
```rust
// テスト用のビルダー・ファクトリー
pub mod test_utils {
    use super::*;
    
    pub struct TestConfigBuilder {
        client_id: String,
        client_secret: String,
        output_dir: PathBuf,
    }
    
    impl TestConfigBuilder {
        pub fn new() -> Self {
            Self {
                client_id: "test_client_id".to_string(),
                client_secret: "test_client_secret".to_string(),
                output_dir: PathBuf::from("./test_output"),
            }
        }
        
        pub fn client_id(mut self, id: &str) -> Self {
            self.client_id = id.to_string();
            self
        }
        
        pub fn build(self) -> Config {
            Config {
                client_id: self.client_id,
                client_secret: self.client_secret,
                redirect_uri: Some("http://localhost:8080/callback".to_string()),
                output_directory: self.output_dir,
                max_concurrent_downloads: 3,
                timeout_seconds: 30,
            }
        }
    }
    
    pub fn create_test_recording() -> Recording {
        Recording {
            meeting_id: MeetingId::new("123456789".to_string()).unwrap(),
            meeting_uuid: MeetingUuid::new("test-uuid-123".to_string()).unwrap(),
            topic: "Test Meeting".to_string(),
            start_time: Utc.ymd(2024, 1, 15).and_hms(10, 0, 0),
            duration: 60,
            recording_files: vec![
                RecordingFile {
                    id: FileId::new("file1".to_string()).unwrap(),
                    file_type: FileType::Video,
                    file_size: 1073741824, // 1GB
                    download_url: "https://example.com/video.mp4".to_string(),
                    recording_start: Utc.ymd(2024, 1, 15).and_hms(10, 0, 0),
                    recording_end: Utc.ymd(2024, 1, 15).and_hms(11, 0, 0),
                }
            ],
            ai_summary_available: true,
        }
    }
    
    pub async fn create_mock_downloader() -> ZoomRecordingDownloader {
        let mut mock_http = MockHttpClient::new();
        mock_http.add_response(
            "https://api.zoom.us/v2/users/me/recordings",
            Ok(Response::json(json!({
                "meetings": [create_test_recording()],
                "page_count": 1,
                "total_records": 1
            })))
        );
        
        ZoomRecordingDownloader::new_with_dependencies(
            Box::new(mock_http),
            Box::new(MockFileStorage::new()),
            TestConfigBuilder::new().build(),
        )
    }
}
```

## 関数設計・実装規約

### 関数コメント規約（必須）

#### コメント必須要素
```rust
/// OAuth認証フローを実行してアクセストークンを取得する
/// 
/// # 副作用
/// - ウェブブラウザの起動（デフォルトブラウザでauth_urlを開く）
/// - HTTPリクエストの送信（Zoom OAuth サーバーへ）
/// - ファイルシステムへの書き込み（取得したトークンの保存）
/// 
/// # 事前条件
/// - client_id が空でなく、有効なZoom Client IDである
/// - client_secret が空でなく、対応するClient Secretである
/// - インターネット接続が利用可能である
/// - デフォルトブラウザが利用可能である
/// 
/// # 事後条件
/// - 成功時：有効なAuthTokenが返される
/// - access_token フィールドが空でない
/// - expires_at が現在時刻より未来である
/// - 取得したトークンがファイルに保存される
/// - 失敗時：適切なエラーメッセージと共にErrorが返される
/// 
/// # 不変条件
/// - 実行中にclient_idとclient_secretは変更されない
/// - OAuth仕様（RFC 6749）に準拠した処理が実行される
/// - トークンの有効性が保証される（期限・スコープ）
/// 
/// # Examples
/// ```rust
/// let downloader = ZoomRecordingDownloader::new();
/// let token = downloader.authenticate("your_client_id", "your_client_secret").await?;
/// assert!(!token.access_token.is_empty());
/// assert!(token.expires_at > chrono::Utc::now());
/// ```
/// 
/// # Errors
/// - `AuthenticationError::InvalidCredentials` - 無効なclient_id/secret
/// - `AuthenticationError::NetworkError` - ネットワーク接続エラー
/// - `AuthenticationError::OAuthFailed` - OAuth認証失敗
/// - `AuthenticationError::BrowserError` - ブラウザ起動失敗
pub async fn authenticate(&self, client_id: &str, client_secret: &str) -> Result<AuthToken> {
    // 事前条件のassertion
    assert!(!client_id.is_empty(), "client_id must not be empty");
    assert!(!client_secret.is_empty(), "client_secret must not be empty");
    debug_assert!(client_id.len() > 5, "client_id should be reasonable length");
    
    // 実装
    let auth_url = self.generate_auth_url(client_id).await?;
    self.open_browser(&auth_url)?;
    let auth_code = self.wait_for_auth_code().await?;
    let token = self.exchange_code(client_id, client_secret, &auth_code).await?;
    
    // 事後条件のassertion
    debug_assert!(!token.access_token.is_empty(), "access_token must be valid");
    debug_assert!(token.expires_at > Utc::now(), "token must not be expired");
    
    // ファイル保存
    self.save_token(&token).await?;
    
    Ok(token)
}
```

#### 簡潔な関数の場合
```rust
/// ファイル名から無効な文字を除去して安全な名前に変換する
/// 
/// # 事前条件
/// - filename が空でない文字列である
/// 
/// # 事後条件
/// - Windowsで有効なファイル名が返される
/// - 元の文字の意味が可能な限り保持される
/// - 長さがOSの制限内に収まる
/// 
/// # 不変条件
/// - 入力文字列は変更されない
/// - 出力は常に有効なファイル名である
pub fn sanitize_filename(filename: &str) -> String {
    assert!(!filename.is_empty(), "filename must not be empty");
    
    let invalid_chars = ['<', '>', ':', '"', '|', '?', '*', '/', '\\'];
    let mut result = filename.to_string();
    
    for ch in invalid_chars {
        result = result.replace(ch, "_");
    }
    
    // 長さ制限
    if result.len() > 255 {
        result.truncate(252);
        result.push_str("...");
    }
    
    debug_assert!(!result.is_empty(), "result must not be empty");
    debug_assert!(result.len() <= 255, "result must be within length limit");
    
    result
}
```

### Assertion実装規約

#### 事前条件のAssertion
```rust
// 引数検証
assert!(!input.is_empty(), "input must not be empty");
assert!(count > 0, "count must be positive");
assert!(start_date <= end_date, "start_date must be before or equal to end_date");

// 状態検証
assert!(self.is_authenticated(), "must be authenticated before API calls");
assert!(self.config.is_valid(), "configuration must be valid");

// リソース検証
assert!(path.exists(), "path must exist: {}", path.display());
assert!(file.metadata()?.len() > 0, "file must not be empty");
```

#### 事後条件のAssertion
```rust
// 戻り値検証
debug_assert!(!result.is_empty(), "result must not be empty");
debug_assert!(result.len() == expected_len, "result length must match expected");

// 状態変化検証
debug_assert!(self.state == ExpectedState::Completed, "state must be completed");
debug_assert!(self.download_count > old_count, "download count must increase");

// 不変条件検証
debug_assert_eq!(self.total_size, self.calculate_total_size(), "total size invariant violated");
```

#### カスタムAssertion マクロ
```rust
// プロジェクト固有のassertion
macro_rules! assert_valid_token {
    ($token:expr) => {
        assert!(!$token.access_token.is_empty(), "access token must not be empty");
        assert!($token.expires_at > chrono::Utc::now(), "token must not be expired");
        assert!(!$token.scope.is_empty(), "token scope must not be empty");
    };
}

macro_rules! assert_valid_recording {
    ($recording:expr) => {
        assert!(!$recording.meeting_id.as_str().is_empty(), "meeting_id must not be empty");
        assert!(!$recording.topic.is_empty(), "topic must not be empty");
        assert!(!$recording.recording_files.is_empty(), "must have at least one recording file");
    };
}

// 使用例
let token = authenticate().await?;
assert_valid_token!(token);

let recording = get_recording(meeting_id).await?;
assert_valid_recording!(recording);
```

## Windows対応実装

### Windows固有処理

#### 文字エンコーディング対応
```rust
/// Windows環境でのUTF-8コンソール設定
/// 
/// # 副作用
/// - Windows APIの呼び出し（SetConsoleOutputCP）
/// - コンソール設定の変更
/// 
/// # 事前条件
/// - Windows環境で実行されている
/// - 適切な権限でプロセスが実行されている
/// 
/// # 事後条件
/// - コンソール出力がUTF-8で処理される
/// - 日本語文字が正常に表示される
/// 
/// # 不変条件
/// - 他のコンソール設定は変更されない
#[cfg(target_os = "windows")]
pub fn setup_console_encoding() -> Result<()> {
    use windows::Win32::System::Console::SetConsoleOutputCP;
    
    // UTF-8コードページ (65001) を設定
    unsafe {
        SetConsoleOutputCP(65001)
            .map_err(|e| ZoomVideoMoverError::Platform {
                operation: "SetConsoleOutputCP".to_string(),
                error: e.to_string(),
            })?;
    }
    
    log::info!("Console encoding set to UTF-8");
    Ok(())
}

/// Windows環境でのパス正規化
/// 
/// # 事前条件
/// - path が有効なパス文字列である
/// 
/// # 事後条件
/// - Windows APIで処理可能なパス形式が返される
/// - 日本語文字が適切に処理される
/// - パス区切り文字が統一される
#[cfg(target_os = "windows")]
pub fn normalize_windows_path(path: &str) -> Result<PathBuf> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    
    // UTF-16変換による正規化
    let wide: Vec<u16> = path.encode_utf16().collect();
    let os_string = OsString::from_wide(&wide);
    let mut path_buf = PathBuf::from(os_string);
    
    // パス区切り文字の統一
    if let Some(path_str) = path_buf.to_str() {
        let normalized = path_str.replace('/', "\\");
        path_buf = PathBuf::from(normalized);
    }
    
    Ok(path_buf)
}
```

#### ファイルシステム対応
```rust
/// Windows対応のファイル名サニタイズ
/// 
/// # 事前条件
/// - filename が空でない
/// 
/// # 事後条件
/// - Windowsで有効なファイル名が返される
/// - 日本語文字が保持される
/// - 予約語が回避される
/// 
/// # 不変条件
/// - 元のファイル名の意味が可能な限り保持される
pub fn sanitize_windows_filename(filename: &str) -> String {
    assert!(!filename.is_empty(), "filename must not be empty");
    
    // Windows無効文字の置換
    let invalid_chars = ['<', '>', ':', '"', '|', '?', '*', '/', '\\'];
    let mut sanitized = filename.to_string();
    
    for ch in invalid_chars {
        sanitized = sanitized.replace(ch, "_");
    }
    
    // 制御文字の除去
    sanitized = sanitized.chars()
        .filter(|c| !c.is_control())
        .collect();
    
    // Windows予約語の回避
    let reserved_names = [
        "CON", "PRN", "AUX", "NUL",
        "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
        "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9"
    ];
    
    let name_upper = sanitized.to_uppercase();
    let base_name = name_upper.split('.').next().unwrap_or(&name_upper);
    
    if reserved_names.contains(&base_name) {
        sanitized = format!("_{}", sanitized);
    }
    
    // 長さ制限（255文字）
    if sanitized.len() > 255 {
        let extension = Path::new(&sanitized).extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| format!(".{}", ext))
            .unwrap_or_default();
        
        let max_base_len = 255 - extension.len();
        let base = &sanitized[..max_base_len.min(sanitized.len())];
        sanitized = format!("{}{}", base, extension);
    }
    
    // 末尾の空白・ピリオド除去
    sanitized = sanitized.trim_end_matches([' ', '.']).to_string();
    
    // 空文字の場合のフォールバック
    if sanitized.is_empty() {
        sanitized = "unnamed_file".to_string();
    }
    
    debug_assert!(!sanitized.is_empty(), "sanitized filename must not be empty");
    debug_assert!(sanitized.len() <= 255, "filename must be within Windows limit");
    
    sanitized
}

/// Windows環境でのディスク容量チェック
#[cfg(target_os = "windows")]
pub fn check_disk_space_windows(path: &Path, required_bytes: u64) -> Result<bool> {
    use std::ffi::CString;
    use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExA;
    use windows::Win32::Foundation::PSTR;
    
    let path_str = path.to_str()
        .ok_or_else(|| ZoomVideoMoverError::FileSystem {
            operation: "path_conversion".to_string(),
            path: path.to_path_buf(),
            source: std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid path"),
        })?;
    
    let path_cstring = CString::new(path_str)
        .map_err(|e| ZoomVideoMoverError::FileSystem {
            operation: "cstring_conversion".to_string(),
            path: path.to_path_buf(),
            source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
        })?;
    
    let mut free_bytes: u64 = 0;
    let mut total_bytes: u64 = 0;
    
    unsafe {
        GetDiskFreeSpaceExA(
            PSTR(path_cstring.as_ptr() as *mut u8),
            Some(&mut free_bytes),
            Some(&mut total_bytes),
            None
        ).map_err(|e| ZoomVideoMoverError::Platform {
            operation: "GetDiskFreeSpaceExA".to_string(),
            error: e.to_string(),
        })?;
    }
    
    Ok(free_bytes >= required_bytes)
}
```

## 性能最適化実装

### メモリ効率化

#### ストリーミング処理
```rust
/// 大容量ファイルのストリーミングダウンロード
/// 
/// # 副作用
/// - HTTPリクエストの送信
/// - ファイルシステムへの書き込み
/// - 進捗通知の送信
/// 
/// # 事前条件
/// - url が有効なHTTP/HTTPS URLである
/// - output_path の親ディレクトリが存在する
/// - 書き込み権限がある
/// 
/// # 事後条件
/// - ファイルが指定パスに保存される
/// - ファイルサイズが期待値と一致する
/// - 進捗が100%で完了する
/// 
/// # 不変条件
/// - メモリ使用量が一定範囲内に保たれる
/// - ダウンロード中断時に一時ファイルが適切に削除される
pub async fn download_streaming(
    &self,
    url: &str,
    output_path: &Path,
    progress_sender: Option<mpsc::Sender<ProgressUpdate>>,
) -> Result<u64> {
    assert!(!url.is_empty(), "url must not be empty");
    assert!(output_path.parent().map_or(false, |p| p.exists()), "parent directory must exist");
    
    // 一時ファイルパス生成
    let temp_path = output_path.with_extension("tmp");
    
    // HTTPストリーム開始
    let response = self.http_client.get(url).send().await?;
    let total_size = response.content_length().unwrap_or(0);
    
    if !response.status().is_success() {
        return Err(ZoomVideoMoverError::Http {
            status: response.status().as_u16(),
            message: format!("Failed to download from {}", url),
        });
    }
    
    // ファイル作成
    let mut file = tokio::fs::File::create(&temp_path).await
        .map_err(|e| ZoomVideoMoverError::FileSystem {
            operation: "create".to_string(),
            path: temp_path.clone(),
            source: e,
        })?;
    
    // ストリーミング書き込み
    let mut stream = response.bytes_stream();
    let mut downloaded = 0u64;
    let mut last_progress_report = Instant::now();
    
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result
            .map_err(|e| ZoomVideoMoverError::Network { source: e })?;
        
        // チャンク書き込み
        file.write_all(&chunk).await
            .map_err(|e| ZoomVideoMoverError::FileSystem {
                operation: "write".to_string(),
                path: temp_path.clone(),
                source: e,
            })?;
        
        downloaded += chunk.len() as u64;
        
        // 進捗報告（1秒間隔）
        if let Some(ref sender) = progress_sender {
            if last_progress_report.elapsed() >= Duration::from_secs(1) {
                let progress = ProgressUpdate {
                    url: url.to_string(),
                    bytes_downloaded: downloaded,
                    bytes_total: total_size,
                    timestamp: Utc::now(),
                };
                
                let _ = sender.try_send(progress);
                last_progress_report = Instant::now();
            }
        }
        
        // 定期的なフラッシュ（1MBごと）
        if downloaded % (1024 * 1024) == 0 {
            file.flush().await.map_err(|e| ZoomVideoMoverError::FileSystem {
                operation: "flush".to_string(),
                path: temp_path.clone(),
                source: e,
            })?;
        }
    }
    
    // ファイル完了処理
    file.sync_all().await.map_err(|e| ZoomVideoMoverError::FileSystem {
        operation: "sync".to_string(),
        path: temp_path.clone(),
        source: e,
    })?;
    
    drop(file); // 明示的にファイルハンドルを閉じる
    
    // 一時ファイルを最終ファイル名にリネーム
    tokio::fs::rename(&temp_path, output_path).await
        .map_err(|e| ZoomVideoMoverError::FileSystem {
            operation: "rename".to_string(),
            path: output_path.to_path_buf(),
            source: e,
        })?;
    
    // 事後条件検証
    debug_assert_eq!(
        tokio::fs::metadata(output_path).await.unwrap().len(),
        downloaded,
        "file size must match downloaded bytes"
    );
    
    Ok(downloaded)
}
```

#### リソースプール
```rust
/// 効率的なHTTPクライアントプール
pub struct HttpClientPool {
    clients: Vec<Arc<reqwest::Client>>,
    current_index: AtomicUsize,
}

impl HttpClientPool {
    pub fn new(pool_size: usize) -> Self {
        let clients = (0..pool_size)
            .map(|_| Arc::new(Self::create_client()))
            .collect();
        
        Self {
            clients,
            current_index: AtomicUsize::new(0),
        }
    }
    
    pub fn get_client(&self) -> Arc<reqwest::Client> {
        let index = self.current_index.fetch_add(1, Ordering::Relaxed) % self.clients.len();
        self.clients[index].clone()
    }
    
    fn create_client() -> reqwest::Client {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(90))
            .https_only(true)
            .build()
            .expect("Failed to create HTTP client")
    }
}

/// バッファプールによるメモリ効率化
pub struct BufferPool {
    buffers: Arc<Mutex<Vec<Vec<u8>>>>,
    buffer_size: usize,
    max_buffers: usize,
}

impl BufferPool {
    pub fn new(buffer_size: usize, max_buffers: usize) -> Self {
        Self {
            buffers: Arc::new(Mutex::new(Vec::new())),
            buffer_size,
            max_buffers,
        }
    }
    
    pub async fn get_buffer(&self) -> Vec<u8> {
        let mut buffers = self.buffers.lock().await;
        buffers.pop().unwrap_or_else(|| {
            Vec::with_capacity(self.buffer_size)
        })
    }
    
    pub async fn return_buffer(&self, mut buffer: Vec<u8>) {
        buffer.clear();
        
        let mut buffers = self.buffers.lock().await;
        if buffers.len() < self.max_buffers {
            buffers.push(buffer);
        }
        // max_buffers を超える場合は破棄（GCに任せる）
    }
}
```

### 並行処理最適化

#### セマフォによる制御
```rust
/// 制限付き並行実行マネージャー
pub struct ConcurrencyManager {
    // ダウンロード用セマフォ
    download_semaphore: Arc<Semaphore>,
    
    // API呼び出し用セマフォ
    api_semaphore: Arc<Semaphore>,
    
    // タスク監視
    active_tasks: Arc<AtomicUsize>,
    max_tasks: usize,
}

impl ConcurrencyManager {
    pub fn new(max_downloads: usize, max_api_calls: usize, max_total_tasks: usize) -> Self {
        Self {
            download_semaphore: Arc::new(Semaphore::new(max_downloads)),
            api_semaphore: Arc::new(Semaphore::new(max_api_calls)),
            active_tasks: Arc::new(AtomicUsize::new(0)),
            max_tasks: max_total_tasks,
        }
    }
    
    pub async fn execute_download<F, T>(&self, task: F) -> Result<T>
    where
        F: Future<Output = Result<T>> + Send + 'static,
        T: Send + 'static,
    {
        // 全体タスク数制限チェック
        if self.active_tasks.load(Ordering::Relaxed) >= self.max_tasks {
            return Err(ZoomVideoMoverError::ResourceLimit {
                resource: "total_tasks".to_string(),
                current: self.active_tasks.load(Ordering::Relaxed),
                limit: self.max_tasks,
            });
        }
        
        // ダウンロード用permit取得
        let _permit = self.download_semaphore.acquire().await
            .map_err(|_| ZoomVideoMoverError::Cancelled)?;
        
        self.active_tasks.fetch_add(1, Ordering::Relaxed);
        
        let result = task.await;
        
        self.active_tasks.fetch_sub(1, Ordering::Relaxed);
        
        result
    }
    
    pub async fn execute_api_call<F, T>(&self, task: F) -> Result<T>
    where
        F: Future<Output = Result<T>> + Send + 'static,
        T: Send + 'static,
    {
        let _permit = self.api_semaphore.acquire().await
            .map_err(|_| ZoomVideoMoverError::Cancelled)?;
        
        self.active_tasks.fetch_add(1, Ordering::Relaxed);
        
        let result = task.await;
        
        self.active_tasks.fetch_sub(1, Ordering::Relaxed);
        
        result
    }
}
```

## セキュリティ実装

### 認証情報保護

#### 安全な文字列型
```rust
use zeroize::{Zeroize, ZeroizeOnDrop};

/// 自動メモリクリア機能付きの機密文字列
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecretString {
    #[zeroize(skip)] // lengthは機密ではない
    length: usize,
    data: Vec<u8>,
}

impl SecretString {
    pub fn new(value: String) -> Self {
        let data = value.into_bytes();
        let length = data.len();
        Self { length, data }
    }
    
    pub fn expose_secret(&self) -> String {
        String::from_utf8_lossy(&self.data).to_string()
    }
    
    pub fn len(&self) -> usize {
        self.length
    }
    
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
}

impl std::fmt::Debug for SecretString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SecretString([REDACTED {} bytes])", self.length)
    }
}

/// 機密情報を含む設定
#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecureConfig {
    pub client_id: String, // 公開情報
    #[zeroize(skip)]
    pub redirect_uri: Option<String>, // 公開情報
    
    client_secret: SecretString, // 機密情報
}

impl SecureConfig {
    pub fn new(client_id: String, client_secret: String, redirect_uri: Option<String>) -> Self {
        Self {
            client_id,
            client_secret: SecretString::new(client_secret),
            redirect_uri,
        }
    }
    
    pub fn client_secret(&self) -> String {
        self.client_secret.expose_secret()
    }
}
```

#### 設定ファイル暗号化
```rust
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};
use rand::RngCore;

/// 設定ファイルの暗号化保存
pub struct EncryptedConfigStorage {
    file_path: PathBuf,
    encryption_key: Key<Aes256Gcm>,
}

impl EncryptedConfigStorage {
    pub fn new(file_path: PathBuf) -> Result<Self> {
        let key = Self::derive_key()?;
        Ok(Self {
            file_path,
            encryption_key: *Key::from_slice(&key),
        })
    }
    
    pub async fn save_config(&self, config: &SecureConfig) -> Result<()> {
        // 1. 設定をJSONシリアライズ
        let json_data = serde_json::to_vec(config)?;
        
        // 2. AES-256-GCM で暗号化
        let cipher = Aes256Gcm::new(&self.encryption_key);
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let encrypted_data = cipher.encrypt(nonce, json_data.as_ref())
            .map_err(|e| ZoomVideoMoverError::Security {
                operation: "encryption".to_string(),
                error: e.to_string(),
            })?;
        
        // 3. nonce + encrypted_data のフォーマットで保存
        let mut file_data = nonce_bytes.to_vec();
        file_data.extend_from_slice(&encrypted_data);
        
        // 4. ファイル権限設定（所有者のみ読み書き）
        tokio::fs::write(&self.file_path, file_data).await?;
        self.set_secure_file_permissions().await?;
        
        Ok(())
    }
    
    pub async fn load_config(&self) -> Result<SecureConfig> {
        // 1. ファイル読み込み
        let file_data = tokio::fs::read(&self.file_path).await?;
        
        if file_data.len() < 12 {
            return Err(ZoomVideoMoverError::Security {
                operation: "decryption".to_string(),
                error: "File too short for encrypted data".to_string(),
            });
        }
        
        // 2. nonce と暗号化データを分離
        let (nonce_bytes, encrypted_data) = file_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        // 3. 復号化
        let cipher = Aes256Gcm::new(&self.encryption_key);
        let decrypted_data = cipher.decrypt(nonce, encrypted_data)
            .map_err(|e| ZoomVideoMoverError::Security {
                operation: "decryption".to_string(),
                error: e.to_string(),
            })?;
        
        // 4. JSON デシリアライズ
        let config: SecureConfig = serde_json::from_slice(&decrypted_data)?;
        
        Ok(config)
    }
    
    fn derive_key() -> Result<[u8; 32]> {
        // プラットフォーム固有のキー派生
        #[cfg(target_os = "windows")]
        {
            Self::derive_key_windows()
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            Self::derive_key_generic()
        }
    }
    
    #[cfg(target_os = "windows")]
    fn derive_key_windows() -> Result<[u8; 32]> {
        // Windows Data Protection API (DPAPI) 使用
        use windows::Win32::Security::Cryptography::{CryptProtectData, DATA_BLOB};
        
        let entropy = b"ZoomVideoMover";
        let mut key = [0u8; 32];
        
        // マシン固有のエントロピーからキー生成
        let mut entropy_blob = DATA_BLOB {
            cbData: entropy.len() as u32,
            pbData: entropy.as_ptr() as *mut u8,
        };
        
        // 実装省略（DPAPIを使用したキー派生）
        
        Ok(key)
    }
    
    async fn set_secure_file_permissions(&self) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            // Windows ACL設定（所有者のみ読み書き可能）
            // 実装省略
        }
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = tokio::fs::metadata(&self.file_path).await?.permissions();
            perms.set_mode(0o600); // 所有者のみ読み書き
            tokio::fs::set_permissions(&self.file_path, perms).await?;
        }
        
        Ok(())
    }
}
```

## デバッグ・ログ実装

### 構造化ログ

#### ログシステム設計
```rust
use serde_json::json;

/// 構造化ログエントリ
#[derive(Debug, Serialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub target: String,
    pub message: String,
    pub module: Option<String>,
    pub line: Option<u32>,
    pub fields: serde_json::Map<String, serde_json::Value>,
}

/// ログコンテキスト
#[derive(Debug, Clone, Serialize)]
pub struct LogContext {
    pub operation_id: String,
    pub user_session: Option<String>,
    pub correlation_id: Option<String>,
}

/// アプリケーション固有のログマクロ
macro_rules! log_operation {
    ($level:expr, $operation:expr, $message:expr, $($key:expr => $value:expr),*) => {
        let mut fields = serde_json::Map::new();
        $(
            fields.insert($key.to_string(), json!($value));
        )*
        
        let entry = LogEntry {
            timestamp: chrono::Utc::now(),
            level: $level.to_string(),
            target: module_path!().to_string(),
            message: $message.to_string(),
            module: Some(module_path!().to_string()),
            line: Some(line!()),
            fields,
        };
        
        log::log!($level, "{}", serde_json::to_string(&entry).unwrap_or_default());
    };
}

/// OAuth操作の詳細ログ
pub fn log_oauth_operation(operation: &str, result: &Result<()>, duration: Duration) {
    match result {
        Ok(_) => {
            log_operation!(
                log::Level::Info,
                "oauth",
                format!("OAuth {} completed successfully", operation),
                "operation" => operation,
                "duration_ms" => duration.as_millis(),
                "status" => "success"
            );
        }
        Err(error) => {
            log_operation!(
                log::Level::Error,
                "oauth", 
                format!("OAuth {} failed: {}", operation, error),
                "operation" => operation,
                "duration_ms" => duration.as_millis(),
                "status" => "error",
                "error_type" => error.to_string(),
                "error_debug" => format!("{:?}", error)
            );
        }
    }
}

/// API呼び出しの詳細ログ
pub fn log_api_call(method: &str, url: &str, status: Option<u16>, duration: Duration, response_size: Option<usize>) {
    let level = match status {
        Some(code) if code >= 200 && code < 300 => log::Level::Info,
        Some(code) if code >= 400 && code < 500 => log::Level::Warn,
        Some(_) => log::Level::Error,
        None => log::Level::Error,
    };
    
    log_operation!(
        level,
        "api_call",
        format!("{} {} -> {:?}", method, url, status),
        "http_method" => method,
        "url" => url,
        "status_code" => status,
        "duration_ms" => duration.as_millis(),
        "response_size_bytes" => response_size
    );
}
```

### パフォーマンス監視

#### 実行時メトリクス
```rust
/// パフォーマンス測定器
pub struct PerformanceProfiler {
    operation_metrics: Arc<Mutex<HashMap<String, OperationMetrics>>>,
    memory_tracker: Arc<Mutex<VecDeque<MemorySnapshot>>>,
}

#[derive(Debug, Clone)]
pub struct OperationMetrics {
    pub count: u64,
    pub total_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub last_execution: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct MemorySnapshot {
    pub timestamp: DateTime<Utc>,
    pub allocated_bytes: u64,
    pub peak_allocated: u64,
}

impl PerformanceProfiler {
    pub async fn profile_operation<T, F>(&self, operation_name: &str, operation: F) -> T
    where
        F: Future<Output = T>,
    {
        let start_time = Instant::now();
        let start_memory = self.capture_memory_snapshot();
        
        let result = operation.await;
        
        let duration = start_time.elapsed();
        let end_memory = self.capture_memory_snapshot();
        
        // メトリクス更新
        self.update_operation_metrics(operation_name, duration).await;
        self.record_memory_usage(start_memory, end_memory).await;
        
        // 閾値チェック
        if duration > Duration::from_secs(5) {
            log_operation!(
                log::Level::Warn,
                "performance",
                format!("Slow operation detected: {}", operation_name),
                "operation" => operation_name,
                "duration_ms" => duration.as_millis(),
                "threshold_ms" => 5000
            );
        }
        
        result
    }
    
    async fn update_operation_metrics(&self, operation_name: &str, duration: Duration) {
        let mut metrics = self.operation_metrics.lock().await;
        let entry = metrics.entry(operation_name.to_string()).or_insert(OperationMetrics {
            count: 0,
            total_duration: Duration::ZERO,
            min_duration: duration,
            max_duration: duration,
            last_execution: Utc::now(),
        });
        
        entry.count += 1;
        entry.total_duration += duration;
        entry.min_duration = entry.min_duration.min(duration);
        entry.max_duration = entry.max_duration.max(duration);
        entry.last_execution = Utc::now();
    }
    
    pub async fn generate_performance_report(&self) -> PerformanceReport {
        let metrics = self.operation_metrics.lock().await;
        let memory_snapshots = self.memory_tracker.lock().await;
        
        PerformanceReport {
            operation_summary: metrics.clone(),
            memory_trend: memory_snapshots.clone(),
            generated_at: Utc::now(),
        }
    }
}

/// 自動パフォーマンス測定マクロ
macro_rules! measure_performance {
    ($profiler:expr, $operation:expr, $code:block) => {{
        $profiler.profile_operation($operation, async move {
            $code
        }).await
    }};
}
```

## 品質保証実装

### 実行時検証

#### 契約プログラミング
```rust
/// 契約条件チェック用マクロ
macro_rules! require {
    ($condition:expr, $message:expr) => {
        if !$condition {
            return Err(ZoomVideoMoverError::Precondition {
                condition: stringify!($condition).to_string(),
                message: $message.to_string(),
            });
        }
    };
}

macro_rules! ensure {
    ($condition:expr, $message:expr) => {
        debug_assert!($condition, "Postcondition violated: {}", $message);
        if cfg!(debug_assertions) && !$condition {
            log::error!("Postcondition violated: {} - {}", stringify!($condition), $message);
        }
    };
}

/// 契約条件を含む関数実装例
pub async fn download_with_verification(
    &self,
    url: &str,
    output_path: &Path,
    expected_size: Option<u64>,
) -> Result<u64> {
    // 事前条件
    require!(!url.is_empty(), "URL must not be empty");
    require!(url.starts_with("https://"), "URL must use HTTPS");
    require!(
        output_path.parent().map_or(false, |p| p.exists()),
        "Output directory must exist"
    );
    
    let initial_time = Instant::now();
    
    // ダウンロード実行
    let downloaded_size = self.download_file_impl(url, output_path).await?;
    
    // 事後条件
    ensure!(output_path.exists(), "Output file must exist after download");
    ensure!(downloaded_size > 0, "Downloaded size must be positive");
    
    if let Some(expected) = expected_size {
        ensure!(
            downloaded_size == expected,
            "Downloaded size must match expected size"
        );
    }
    
    // 実行時間チェック
    let elapsed = initial_time.elapsed();
    if elapsed > Duration::from_secs(300) { // 5分
        log::warn!(
            "Download took longer than expected: {} seconds for {} bytes",
            elapsed.as_secs(),
            downloaded_size
        );
    }
    
    Ok(downloaded_size)
}
```

### 実行時品質監視

#### 自動品質チェック
```rust
/// 実行時品質監視システム
pub struct QualityMonitor {
    error_counts: Arc<Mutex<HashMap<String, u32>>>,
    performance_degradation: Arc<AtomicBool>,
    last_health_check: Arc<Mutex<DateTime<Utc>>>,
}

impl QualityMonitor {
    pub async fn record_error(&self, error_type: &str) {
        let mut counts = self.error_counts.lock().await;
        let count = counts.entry(error_type.to_string()).or_insert(0);
        *count += 1;
        
        // エラー率チェック
        if *count > 10 {
            log::error!(
                "High error rate detected for {}: {} errors",
                error_type,
                count
            );
            
            // 自動フォールバック処理
            self.trigger_fallback_mode(error_type).await;
        }
    }
    
    pub async fn check_system_health(&self) -> HealthStatus {
        let mut health = HealthStatus::new();
        
        // エラー率チェック
        let error_counts = self.error_counts.lock().await;
        for (error_type, count) in error_counts.iter() {
            if *count > 5 {
                health.add_issue(HealthIssue::HighErrorRate {
                    error_type: error_type.clone(),
                    count: *count,
                });
            }
        }
        
        // メモリ使用量チェック
        if let Ok(memory_usage) = self.get_memory_usage() {
            if memory_usage > 1024 * 1024 * 1024 { // 1GB
                health.add_issue(HealthIssue::HighMemoryUsage {
                    current: memory_usage,
                    limit: 1024 * 1024 * 1024,
                });
            }
        }
        
        // パフォーマンス劣化チェック
        if self.performance_degradation.load(Ordering::Relaxed) {
            health.add_issue(HealthIssue::PerformanceDegradation);
        }
        
        health
    }
    
    async fn trigger_fallback_mode(&self, error_type: &str) {
        match error_type {
            "network" => {
                log::info!("Enabling network fallback mode");
                // ネットワーク関連のフォールバック
            }
            "auth" => {
                log::info!("Clearing authentication cache");
                // 認証キャッシュクリア
            }
            "download" => {
                log::info!("Reducing concurrent downloads");
                // 並行ダウンロード数削減
            }
            _ => {
                log::warn!("No fallback strategy for error type: {}", error_type);
            }
        }
    }
}

#[derive(Debug)]
pub struct HealthStatus {
    pub overall_status: OverallHealth,
    pub issues: Vec<HealthIssue>,
    pub checked_at: DateTime<Utc>,
}

#[derive(Debug)]
pub enum OverallHealth {
    Healthy,
    Degraded,
    Critical,
}

#[derive(Debug)]
pub enum HealthIssue {
    HighErrorRate { error_type: String, count: u32 },
    HighMemoryUsage { current: u64, limit: u64 },
    PerformanceDegradation,
}
```

## 結論

本実装方針は、**高品質・高性能・高安全性**を実現するRustアプリケーション開発のための包括的な実装ガイドラインを提供します。

### 実装方針の特徴
- **Rust最新機能の活用**: 型安全性・メモリ安全性・並行性の完全活用
- **防御的プログラミング**: 事前/事後条件・不変条件による堅牢性確保
- **パフォーマンス重視**: 非同期処理・ゼロコスト抽象化・リソース効率化
- **セキュリティファースト**: 認証情報保護・通信暗号化・実行時検証
- **品質保証**: 包括的エラーハンドリング・実行時監視・自動フォールバック
- **保守性**: 明確なコード構造・包括的ログ・テスト容易性

### 期待効果
- **開発効率**: 明確な実装指針による開発速度向上
- **品質向上**: 体系的な品質保証による信頼性確保
- **性能向上**: 最適化された非同期処理による高効率実現
- **セキュリティ**: 多層防御による堅牢なセキュリティ実現
- **保守性**: 構造化された実装による長期保守の容易化

この実装方針に従うことで、**ユーザーの期待を超え、技術的に優秀で、長期間にわたって安定稼働する**ソフトウェアシステムを構築できます。