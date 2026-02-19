# Rustコーディングルール（技術固有）

**適用範囲**: Rust言語を使用するプロジェクト  
**依存性レベル**: Level 2 (Technology-Specific) - Rust技術依存  
**参照ポリシー**: [rust_coding_policy.md](../../policies/technology-specific/rust/rust_coding_policy.md), [rust_coding_standards.md](../../policies/technology-specific/rust/rust_coding_standards.md)

## 1. 命名規則

### 1.1 基本命名規則
```rust
// ✅ 良い例
// モジュール・パッケージ: snake_case
mod zoom_api;
mod download_manager;

// 構造体・列挙型・トレイト: PascalCase  
struct ZoomRecording;
enum DownloadStatus;
trait ApiClient;

// 関数・変数・フィールド: snake_case
fn download_recording(file_url: &str) -> Result<(), Error>;
let recording_list = vec![];

// 定数・静的変数: SCREAMING_SNAKE_CASE
const MAX_CONCURRENT_DOWNLOADS: usize = 5;
static API_BASE_URL: &str = "https://api.zoom.us/v2/";

// 型パラメータ: 単一大文字（T, U, V）または説明的PascalCase
fn process_data<T, ErrorType>(data: T) -> Result<T, ErrorType>;
```

### 1.2 意味のある命名
```rust
// ❌ 避けるべき命名
let d = download();
let mgr = manager;
fn proc(x: i32) -> bool;

// ✅ 推奨命名
let download_result = download_recording();
let download_manager = DownloadManager::new();
fn is_download_complete(download_id: i32) -> bool;

// 略語・頭字語の扱い
struct HttpClient;  // ✅ Http (最初のみ大文字)
struct XMLParser;   // ❌ 全て大文字は避ける
struct XmlParser;   // ✅ Xml (最初のみ大文字)
```

## 2. 構造・設計ルール

### 2.1 モジュール構成
```rust
// ✅ 推奨モジュール構成
pub mod api {
    pub mod client;
    pub mod types;
    pub mod error;
}

pub mod download {
    pub mod manager;
    pub mod task;
    pub mod progress;
}

pub mod config {
    pub mod settings;
    pub mod validation;
}

// re-export で使いやすいAPIを提供
pub use api::client::ZoomClient;
pub use download::manager::DownloadManager;
```

### 2.2 エラーハンドリング
```rust
// ✅ thiserrorを活用したエラー定義
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ZoomApiError {
    #[error("Authentication failed: {message}")]
    AuthenticationFailed { message: String },
    
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    
    #[error("Invalid response format")]
    InvalidResponse,
    
    #[error("Rate limit exceeded. Retry after {seconds} seconds")]
    RateLimited { seconds: u32 },
}

// Result型の使用
type ZoomResult<T> = Result<T, ZoomApiError>;

// ✅ エラーの適切な伝播
fn fetch_recordings() -> ZoomResult<Vec<Recording>> {
    let client = create_client()?;  // ?演算子でエラー伝播
    let response = client.get_recordings().await?;
    
    response.recordings
        .ok_or(ZoomApiError::InvalidResponse)
}
```

### 2.3 非同期処理
```rust
// ✅ tokioを活用した非同期処理
use tokio::time::{sleep, Duration};
use tokio::sync::Semaphore;

pub struct DownloadManager {
    semaphore: Arc<Semaphore>,
    client: Arc<HttpClient>,
}

impl DownloadManager {
    // 並行ダウンロード数制限
    pub async fn download_files(&self, urls: Vec<String>) -> Vec<Result<Vec<u8>, Error>> {
        let tasks = urls.into_iter().map(|url| {
            let semaphore = self.semaphore.clone();
            let client = self.client.clone();
            
            tokio::spawn(async move {
                let _permit = semaphore.acquire().await?;
                client.download(url).await
            })
        });
        
        // 全タスクの完了を待機
        futures::future::join_all(tasks).await
            .into_iter()
            .map(|result| result.unwrap_or_else(|e| Err(e.into())))
            .collect()
    }
}
```

## 3. メモリ・所有権管理

### 3.1 所有権・借用ルール
```rust
// ✅ 所有権の明確な管理
pub struct RecordingFile {
    filename: String,      // 所有
    content: Vec<u8>,     // 所有
}

impl RecordingFile {
    // 不変借用でアクセス
    pub fn filename(&self) -> &str {
        &self.filename
    }
    
    // 可変借用で変更
    pub fn append_content(&mut self, data: &[u8]) {
        self.content.extend_from_slice(data);
    }
    
    // 消費してムーブ
    pub fn into_content(self) -> Vec<u8> {
        self.content
    }
}

// ✅ Clone vs 借用の適切な選択
fn process_filename(filename: &str) -> String {  // 借用で十分
    format!("processed_{}", filename)
}

fn store_recording(recording: RecordingFile) {   // 所有権移転が必要
    database::save(recording);
}
```

### 3.2 ライフタイム管理
```rust
// ✅ 明示的ライフタイム注釈
pub struct RecordingRef<'a> {
    title: &'a str,
    content: &'a [u8],
}

impl<'a> RecordingRef<'a> {
    pub fn new(title: &'a str, content: &'a [u8]) -> Self {
        Self { title, content }
    }
    
    // 複数ライフタイムパラメータ
    pub fn compare_with<'b>(&self, other: &'b RecordingRef<'b>) -> bool
    where
        'a: 'b,  // 'a が 'b より長生きすることを保証
    {
        self.title == other.title
    }
}
```

## 4. トレイト・ジェネリクス

### 4.1 トレイト定義・実装
```rust
// ✅ トレイト設計
pub trait Downloadable {
    type Error;
    
    async fn download(&self) -> Result<Vec<u8>, Self::Error>;
    fn download_url(&self) -> &str;
    fn file_size(&self) -> Option<u64> { None }  // デフォルト実装
}

// 具体的な実装
impl Downloadable for ZoomRecording {
    type Error = ZoomApiError;
    
    async fn download(&self) -> Result<Vec<u8>, Self::Error> {
        let client = reqwest::Client::new();
        let response = client.get(self.download_url()).send().await?;
        Ok(response.bytes().await?.to_vec())
    }
    
    fn download_url(&self) -> &str {
        &self.download_url
    }
    
    fn file_size(&self) -> Option<u64> {
        self.size
    }
}
```

### 4.2 ジェネリクス・制約
```rust
// ✅ 適切な制約付きジェネリクス
use std::fmt::Debug;
use serde::{Serialize, Deserialize};

pub struct ApiResponse<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Debug,
{
    data: T,
    status: ResponseStatus,
}

impl<T> ApiResponse<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Debug,
{
    pub fn new(data: T, status: ResponseStatus) -> Self {
        tracing::debug!("Creating response with data: {:?}", data);
        Self { data, status }
    }
}

// 関連型を使用した設計
pub trait Repository {
    type Item;
    type Error;
    
    async fn save(&self, item: Self::Item) -> Result<(), Self::Error>;
    async fn find_by_id(&self, id: u64) -> Result<Option<Self::Item>, Self::Error>;
}
```

## 5. パフォーマンス・効率性

### 5.1 メモリ効率化
```rust
// ✅ 効率的なデータ構造選択
use std::collections::HashMap;
use indexmap::IndexMap;

pub struct RecordingCache {
    // 順序保持が必要な場合
    recordings: IndexMap<String, Recording>,
    // 高速ルックアップのみ必要
    metadata: HashMap<String, RecordingMetadata>,
}

// ✅ 不要なクローンの回避
impl RecordingCache {
    pub fn get_recording(&self, id: &str) -> Option<&Recording> {
        self.recordings.get(id)  // 借用で返す
    }
    
    pub fn get_recording_clone(&self, id: &str) -> Option<Recording> {
        self.recordings.get(id).cloned()  // 必要時のみクローン
    }
}
```

### 5.2 並行処理最適化
```rust
// ✅ 効率的な並行処理
use tokio::sync::{mpsc, RwLock};
use std::sync::Arc;

pub struct ConcurrentDownloader {
    max_concurrent: usize,
    progress_tx: mpsc::UnboundedSender<DownloadProgress>,
}

impl ConcurrentDownloader {
    pub async fn download_batch(&self, urls: Vec<String>) -> Vec<DownloadResult> {
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent));
        let results = Arc::new(RwLock::new(Vec::new()));
        
        let tasks: Vec<_> = urls.into_iter().enumerate().map(|(index, url)| {
            let semaphore = semaphore.clone();
            let results = results.clone();
            let progress_tx = self.progress_tx.clone();
            
            tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                
                match download_file(&url).await {
                    Ok(data) => {
                        progress_tx.send(DownloadProgress::Completed { index }).ok();
                        results.write().await.push(DownloadResult::Success(data));
                    }
                    Err(e) => {
                        progress_tx.send(DownloadProgress::Failed { index, error: e.clone() }).ok();
                        results.write().await.push(DownloadResult::Error(e));
                    }
                }
            })
        }).collect();
        
        futures::future::join_all(tasks).await;
        Arc::try_unwrap(results).unwrap().into_inner()
    }
}
```

## 6. テスト・デバッグ

### 6.1 単体テスト
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;
    
    #[tokio::test]
    async fn test_download_success() {
        let manager = DownloadManager::new(5);
        let url = "https://example.com/test.mp4";
        
        let result = manager.download(url).await;
        
        assert!(result.is_ok());
        let data = result.unwrap();
        assert!(!data.is_empty());
    }
    
    #[test]
    fn test_recording_file_creation() {
        let file = RecordingFile::new("test.mp4", vec![1, 2, 3, 4]);
        
        assert_eq!(file.filename(), "test.mp4");
        assert_eq!(file.size(), 4);
    }
    
    // proptest を活用した Property-based テスト
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_filename_validation(s in "\\PC*") {
            let result = validate_filename(&s);
            // ファイル名として無効な文字が含まれていないことを確認
            if s.chars().any(|c| ['<', '>', ':', '"', '|', '?', '*'].contains(&c)) {
                assert!(result.is_err());
            }
        }
    }
}
```

### 6.2 統合テスト・モック
```rust
// tests/integration_test.rs
use zoom_video_mover::*;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

#[tokio::test]
async fn test_api_integration() {
    // モックサーバーの起動
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/recordings"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(serde_json::json!({
                "recordings": [
                    {
                        "id": "test123",
                        "title": "Test Recording",
                        "download_url": "https://example.com/test.mp4"
                    }
                ]
            })))
        .mount(&mock_server)
        .await;
    
    // テスト実行
    let client = ZoomClient::new(&mock_server.uri());
    let recordings = client.get_recordings().await.unwrap();
    
    assert_eq!(recordings.len(), 1);
    assert_eq!(recordings[0].title, "Test Recording");
}
```

## 7. ドキュメント・コメント

### 7.1 docコメント
```rust
/// Zoomクラウドレコーディングをダウンロード・管理するメインクラス
/// 
/// # Examples
/// 
/// ```rust
/// use zoom_video_mover::DownloadManager;
/// 
/// #[tokio::main]
/// async fn main() {
///     let manager = DownloadManager::new(5);
///     let result = manager.download("https://example.com/recording.mp4").await;
///     println!("Download result: {:?}", result);
/// }
/// ```
/// 
/// # Panics
/// 
/// この関数は、無効なURLが渡された場合にパニックすることがあります。
/// 
/// # Errors
/// 
/// 以下の場合に`ZoomApiError`を返します：
/// - ネットワーク接続エラー
/// - 認証エラー
/// - ファイルアクセスエラー
pub struct DownloadManager {
    /// 同時ダウンロード数の上限
    max_concurrent: usize,
    /// HTTPクライアント
    client: reqwest::Client,
}

impl DownloadManager {
    /// 新しいDownloadManagerインスタンスを作成
    /// 
    /// # Arguments
    /// 
    /// * `max_concurrent` - 同時ダウンロード数の上限
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let manager = DownloadManager::new(10);
    /// ```
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            max_concurrent,
            client: reqwest::Client::new(),
        }
    }
}
```

### 7.2 実装コメント
```rust
impl ZoomClient {
    pub async fn authenticate(&mut self) -> ZoomResult<()> {
        // OAuth 2.0フローでアクセストークンを取得
        // RFC 6749に準拠した実装
        let token_request = self.build_token_request();
        
        // Rate limiting対策: 指数バックオフでリトライ
        let mut retry_count = 0;
        let max_retries = 3;
        
        while retry_count < max_retries {
            match self.send_token_request(&token_request).await {
                Ok(token) => {
                    self.access_token = Some(token);
                    return Ok(());
                }
                Err(ZoomApiError::RateLimited { seconds }) => {
                    // Rate limit時は指定秒数待機後リトライ
                    tokio::time::sleep(Duration::from_secs(seconds as u64)).await;
                    retry_count += 1;
                }
                Err(e) => return Err(e),
            }
        }
        
        Err(ZoomApiError::AuthenticationFailed {
            message: "Max retries exceeded".to_string(),
        })
    }
}
```

## 8. セキュリティ・ベストプラクティス

### 8.1 機密情報の取り扱い
```rust
use secrecy::{Secret, ExposeSecret};
use zeroize::Zeroize;

/// APIキーを安全に管理する構造体
#[derive(Zeroize)]
pub struct ApiCredentials {
    #[zeroize(skip)]  // Secret型が自動でzeroizeを行う
    api_key: Secret<String>,
    #[zeroize(skip)]
    api_secret: Secret<String>,
}

impl ApiCredentials {
    pub fn new(api_key: String, api_secret: String) -> Self {
        Self {
            api_key: Secret::new(api_key),
            api_secret: Secret::new(api_secret),
        }
    }
    
    // 安全にAPIキーを使用
    pub fn create_auth_header(&self) -> String {
        let key = self.api_key.expose_secret();
        let secret = self.api_secret.expose_secret();
        
        // base64エンコードで認証ヘッダー作成
        let credentials = format!("{}:{}", key, secret);
        let encoded = base64::encode(credentials);
        format!("Basic {}", encoded)
    }
}

// Dropトレイト実装でメモリ消去を保証
impl Drop for ApiCredentials {
    fn drop(&mut self) {
        self.zeroize();
    }
}
```

### 8.2 入力検証・サニタイゼーション
```rust
use regex::Regex;
use once_cell::sync::Lazy;

// コンパイル時に正規表現を初期化
static FILENAME_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9._\-\s]+$").unwrap()
});

/// ファイル名の検証・サニタイゼーション
pub fn validate_and_sanitize_filename(filename: &str) -> Result<String, ValidationError> {
    // 1. 長さチェック
    if filename.is_empty() || filename.len() > 255 {
        return Err(ValidationError::InvalidLength);
    }
    
    // 2. 不正文字チェック
    let invalid_chars = ['<', '>', ':', '"', '|', '?', '*', '\\', '/'];
    if filename.chars().any(|c| invalid_chars.contains(&c)) {
        return Err(ValidationError::InvalidCharacters);
    }
    
    // 3. 予約語チェック
    let reserved_names = ["CON", "PRN", "AUX", "NUL"];
    let name_upper = filename.to_uppercase();
    if reserved_names.contains(&name_upper.as_str()) {
        return Err(ValidationError::ReservedName);
    }
    
    // 4. パス操作攻撃防止
    if filename.contains("..") || filename.contains("~") {
        return Err(ValidationError::PathTraversal);
    }
    
    // 5. サニタイゼーション
    let sanitized = filename
        .chars()
        .map(|c| if c.is_control() { '_' } else { c })
        .collect::<String>()
        .trim()
        .to_string();
    
    Ok(sanitized)
}
```

---

**策定日**: 2025-08-09  
**適用範囲**: Rust言語使用プロジェクト  
**見直し頻度**: Rustバージョンアップ時または四半期毎