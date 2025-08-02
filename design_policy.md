# 設計方針 - Zoom Video Mover

## 設計の基本方針

### 設計原則・哲学
- **シンプルさ**: 複雑性を最小限に抑え、理解しやすい設計
- **モジュール性**: 疎結合・高凝集なコンポーネント設計
- **拡張性**: 将来の機能追加・変更に柔軟に対応
- **信頼性**: 堅牢なエラーハンドリングと自動回復機能
- **性能**: 非同期処理とリソース効率の最適化
- **保守性**: 明確な責任分離と文書化

### アーキテクチャスタイル
- **レイヤードアーキテクチャ**: UI・ビジネスロジック・インフラの分離
- **非同期メッセージング**: tokio Channel による疎結合な通信
- **イベント駆動**: 状態変更とUIの効率的な同期
- **依存性注入**: テスト容易性と柔軟性の向上

## システムアーキテクチャ設計

### 全体構成

#### レイヤー構造
```
┌─────────────────────────────────────────┐
│        Presentation Layer               │
│  ┌─────────────┐  ┌─────────────────┐   │
│  │   GUI       │  │   Windows       │   │
│  │ (gui.rs)    │  │ Console Support │   │
│  └─────────────┘  └─────────────────┘   │
├─────────────────────────────────────────┤
│        Application Layer                │
│  ┌─────────────────────────────────────┐ │
│  │     Core Business Logic             │ │
│  │        (lib.rs)                     │ │
│  └─────────────────────────────────────┘ │
├─────────────────────────────────────────┤
│        Infrastructure Layer            │
│  ┌─────────┐ ┌─────────┐ ┌─────────────┐ │
│  │  HTTP   │ │  OAuth  │ │ File System │ │
│  │ Client  │ │  Client │ │   Manager   │ │
│  └─────────┘ └─────────┘ └─────────────┘ │
└─────────────────────────────────────────┘
```

#### コンポーネント責任

**Presentation Layer**:
- `gui.rs`: eframe/eguiベースのGUI実装
- `windows_console.rs`: Windows固有のコンソール処理
- **責任**: ユーザー入力の受付・画面表示・プラットフォーム固有処理

**Application Layer**:
- `lib.rs`: コアビジネスロジック
- **責任**: ドメインルール・ワークフロー制御・状態管理

**Infrastructure Layer**:
- HTTP通信・OAuth認証・ファイルシステム操作
- **責任**: 外部リソースへのアクセス・技術的な詳細処理

### 設計パターン

#### 1. Repository Pattern
```rust
// 抽象インターフェース
trait RecordingRepository {
    async fn get_recordings(&self, from: &str, to: &str) -> Result<Vec<Recording>>;
    async fn get_ai_summary(&self, meeting_id: &str) -> Result<Option<AiSummary>>;
}

// 具象実装
struct ZoomApiRepository {
    client: reqwest::Client,
    access_token: String,
}
```

**目的**: データアクセス層の抽象化・テスト容易性向上

#### 2. Builder Pattern
```rust
// 設定オブジェクトの構築
pub struct ConfigBuilder {
    client_id: Option<String>,
    client_secret: Option<String>,
    redirect_uri: Option<String>,
}

impl ConfigBuilder {
    pub fn client_id(mut self, id: String) -> Self { /* */ }
    pub fn build(self) -> Result<Config, ValidationError> { /* */ }
}
```

**目的**: 複雑なオブジェクト構築の簡素化・検証の集約

#### 3. Strategy Pattern
```rust
// ダウンロード戦略の抽象化
trait DownloadStrategy {
    async fn download(&self, request: DownloadRequest) -> Result<PathBuf>;
}

struct ParallelDownloadStrategy { /* */ }
struct SequentialDownloadStrategy { /* */ }
```

**目的**: アルゴリズムの交換可能性・動作の柔軟性

#### 4. Observer Pattern (Message Passing)
```rust
// 非同期メッセージによる状態通知
enum AppMessage {
    AuthCompleted(AuthToken),
    DownloadProgress(ProgressUpdate),
    Error(AppError),
}

// メッセージチャネル
let (sender, receiver) = tokio::sync::mpsc::channel(100);
```

**目的**: 疎結合な状態通知・リアルタイム更新

### データ設計

#### ドメインモデル

**主要エンティティ**:
```rust
// 設定情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: Option<String>,
    pub output_directory: PathBuf,
}

// 認証トークン
#[derive(Debug, Clone)]
pub struct AuthToken {
    pub access_token: String,
    pub token_type: String,
    pub expires_at: DateTime<Utc>,
    pub refresh_token: Option<String>,
    pub scope: String,
}

// 録画情報
#[derive(Debug, Clone)]
pub struct Recording {
    pub meeting_id: String,
    pub meeting_uuid: String,
    pub topic: String,
    pub start_time: DateTime<Utc>,
    pub duration: u32,
    pub recording_files: Vec<RecordingFile>,
    pub ai_summary_available: bool,
}

// 録画ファイル
#[derive(Debug, Clone)]
pub struct RecordingFile {
    pub id: String,
    pub file_type: String,
    pub file_size: u64,
    pub download_url: String,
    pub recording_start: DateTime<Utc>,
    pub recording_end: DateTime<Utc>,
}

// AI要約
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSummaryResponse {
    pub meeting_id: String,
    pub summary_overview: String,
    pub key_points: Vec<String>,
    pub next_steps: Vec<String>,
    pub detailed_sections: Vec<DetailedSection>,
}
```

#### データフロー設計

**一方向データフロー**:
```
User Input → State Update → UI Render
     ↑                          ↓
External Events ← Background Tasks
```

**状態管理**:
- **Single Source of Truth**: 中央集権的な状態管理
- **Immutable Updates**: 状態変更時の不変性保証
- **Event Sourcing**: 状態変更の履歴保持

#### 永続化設計

**設定ファイル (TOML)**:
```toml
client_id = "zoom_client_id"
client_secret = "zoom_client_secret" 
redirect_uri = "http://localhost:8080/callback"
output_directory = "C:\\Downloads\\Zoom"

[advanced]
max_concurrent_downloads = 5
timeout_seconds = 300
```

**デバッグデータ (JSON)**:
```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "request_type": "ai_summary",
  "meeting_id": "123456789",
  "endpoint": "v2/meetings/123456789/ai_companion",
  "response_status": 200,
  "response_body": { /* API response */ }
}
```

## 非同期アーキテクチャ設計

### tokio アーキテクチャ

#### 非同期タスク設計
```rust
// メインアプリケーション実行時
#[tokio::main]
async fn main() -> Result<()> {
    // 1. ランタイム初期化
    let runtime = tokio::runtime::Runtime::new()?;
    
    // 2. GUI タスク起動
    let gui_handle = tokio::spawn(async {
        run_gui_application().await
    });
    
    // 3. バックグラウンドタスク起動
    let background_handle = tokio::spawn(async {
        run_background_services().await
    });
    
    // 4. 並行実行・結果待機
    tokio::try_join!(gui_handle, background_handle)?;
    Ok(())
}
```

#### 並行処理設計
```rust
// 並列ダウンロード制御
use tokio::sync::Semaphore;

pub struct DownloadManager {
    semaphore: Arc<Semaphore>,
    progress_sender: mpsc::Sender<ProgressEvent>,
}

impl DownloadManager {
    pub async fn download_files(&self, requests: Vec<DownloadRequest>) -> Result<Vec<PathBuf>> {
        let mut tasks = Vec::new();
        
        for request in requests {
            let permit = self.semaphore.clone().acquire_owned().await?;
            let progress_sender = self.progress_sender.clone();
            
            let task = tokio::spawn(async move {
                let _permit = permit; // Keep permit until completion
                download_single_file(request, progress_sender).await
            });
            
            tasks.push(task);
        }
        
        // すべてのタスクの完了を待機
        let results = futures::future::try_join_all(tasks).await?;
        Ok(results.into_iter().collect::<Result<Vec<_>, _>>()?)
    }
}
```

#### エラー伝播設計
```rust
// 階層的エラーハンドリング
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Authentication failed: {0}")]
    Auth(#[from] AuthError),
    
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),
    
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
}

// エラー回復戦略
async fn with_retry<F, T, E>(operation: F, max_attempts: u32) -> Result<T, E>
where
    F: Fn() -> Pin<Box<dyn Future<Output = Result<T, E>> + Send>>,
    E: std::error::Error + Send + Sync + 'static,
{
    let mut attempt = 0;
    loop {
        attempt += 1;
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) if attempt >= max_attempts => return Err(error),
            Err(_) => {
                let delay = Duration::from_secs(2u64.pow(attempt.min(6)));
                tokio::time::sleep(delay).await;
            }
        }
    }
}
```

### メッセージング設計

#### チャネル戦略
```rust
// アプリケーション全体のメッセージバス
pub struct MessageBus {
    // GUI → ビジネスロジック
    command_sender: mpsc::Sender<Command>,
    command_receiver: mpsc::Receiver<Command>,
    
    // ビジネスロジック → GUI
    event_sender: broadcast::Sender<Event>,
    
    // 進捗通知専用チャネル
    progress_sender: mpsc::Sender<ProgressEvent>,
}

#[derive(Debug, Clone)]
pub enum Command {
    StartAuthentication,
    SearchRecordings { from: String, to: String },
    DownloadFiles(Vec<DownloadRequest>),
    CancelDownload,
}

#[derive(Debug, Clone)]
pub enum Event {
    AuthenticationCompleted(AuthToken),
    RecordingsLoaded(Vec<Recording>),
    DownloadCompleted(PathBuf),
    ErrorOccurred(AppError),
}
```

#### バックプレッシャー制御
```rust
// チャネル容量による流量制御
let (sender, receiver) = mpsc::channel(1000); // バッファサイズ

// 送信側での背圧処理
async fn send_with_backpressure<T>(sender: &mpsc::Sender<T>, item: T) -> Result<()> {
    match sender.try_send(item) {
        Ok(()) => Ok(()),
        Err(mpsc::error::TrySendError::Full(item)) => {
            // チャネルが満杯の場合は待機
            sender.send(item).await.map_err(|_| AppError::ChannelClosed)
        }
        Err(mpsc::error::TrySendError::Closed(_)) => {
            Err(AppError::ChannelClosed)
        }
    }
}
```

## GUI アーキテクチャ設計

### egui/eframe 設計パターン

#### アプリケーション状態管理
```rust
// メインアプリケーション状態
pub struct ZoomDownloaderApp {
    // 設定状態
    config: Option<Config>,
    config_input: ConfigInput,
    
    // 認証状態
    auth_state: AuthState,
    access_token: Option<AuthToken>,
    
    // 録画データ状態
    recordings: Vec<Recording>,
    selected_files: HashSet<String>,
    
    // ダウンロード状態
    download_progress: DownloadProgressState,
    is_downloading: bool,
    
    // UI状態
    current_tab: TabType,
    error_message: Option<String>,
    
    // 非同期通信
    message_sender: mpsc::Sender<Command>,
    event_receiver: mpsc::Receiver<Event>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AuthState {
    NotConfigured,
    ReadyToAuth,
    AuthUrlGenerated(String),
    WaitingForCode,
    Authenticating,
    Authenticated,
    AuthFailed(String),
}
```

#### 状態遷移設計
```rust
impl ZoomDownloaderApp {
    // 状態遷移関数
    fn handle_event(&mut self, event: Event) {
        match event {
            Event::AuthenticationCompleted(token) => {
                self.access_token = Some(token);
                self.auth_state = AuthState::Authenticated;
                self.current_tab = TabType::Recordings;
            }
            Event::RecordingsLoaded(recordings) => {
                self.recordings = recordings;
            }
            Event::DownloadCompleted(path) => {
                self.download_progress.mark_completed(path);
                if self.download_progress.is_all_completed() {
                    self.is_downloading = false;
                }
            }
            Event::ErrorOccurred(error) => {
                self.error_message = Some(error.to_string());
            }
        }
    }
    
    // UI更新処理
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // 1. 非同期イベント処理
        while let Ok(event) = self.event_receiver.try_recv() {
            self.handle_event(event);
        }
        
        // 2. UI描画
        self.render_ui(ctx);
        
        // 3. 定期的な再描画要求
        ctx.request_repaint_after(Duration::from_millis(100));
    }
}
```

#### コンポーネント設計
```rust
// 再利用可能なUIコンポーネント
trait UIComponent {
    type State;
    type Message;
    
    fn render(&mut self, ui: &mut egui::Ui, state: &Self::State) -> Option<Self::Message>;
}

// 設定画面コンポーネント
struct ConfigPanel {
    client_id_input: String,
    client_secret_input: String,
    output_dir_input: String,
}

impl UIComponent for ConfigPanel {
    type State = ConfigInput;
    type Message = ConfigMessage;
    
    fn render(&mut self, ui: &mut egui::Ui, state: &Self::State) -> Option<Self::Message> {
        ui.vertical(|ui| {
            ui.heading("OAuth Configuration");
            
            ui.horizontal(|ui| {
                ui.label("Client ID:");
                if ui.text_edit_singleline(&mut self.client_id_input).changed() {
                    return Some(ConfigMessage::ClientIdChanged(self.client_id_input.clone()));
                }
            });
            
            // ... その他のフィールド
            
            if ui.button("Save Configuration").clicked() {
                Some(ConfigMessage::SaveRequested)
            } else {
                None
            }
        }).inner
    }
}
```

### レスポンシブデザイン

#### 画面サイズ対応
```rust
fn render_adaptive_layout(&mut self, ui: &mut egui::Ui) {
    let available_width = ui.available_width();
    
    if available_width > 1200.0 {
        // 大画面: 3列レイアウト
        ui.columns(3, |columns| {
            self.render_config_panel(&mut columns[0]);
            self.render_recording_list(&mut columns[1]);
            self.render_progress_panel(&mut columns[2]);
        });
    } else if available_width > 800.0 {
        // 中画面: 2列レイアウト
        ui.columns(2, |columns| {
            self.render_main_content(&mut columns[0]);
            self.render_side_panel(&mut columns[1]);
        });
    } else {
        // 小画面: 1列レイアウト（タブ切り替え）
        self.render_tabbed_layout(ui);
    }
}
```

#### DPI対応
```rust
fn setup_ui_scaling(ctx: &egui::Context) {
    let native_pixels_per_point = ctx.native_pixels_per_point();
    let scale_factor = if native_pixels_per_point > 1.5 {
        1.5 // 高DPI環境での最適化
    } else {
        1.0
    };
    
    ctx.set_pixels_per_point(scale_factor);
    
    // フォントサイズの調整
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "japanese".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/NotoSansJP-Regular.ttf")),
    );
    ctx.set_fonts(fonts);
}
```

## エラー処理アーキテクチャ

### エラー分類・階層化

#### エラータイプ階層
```rust
// 最上位エラータイプ
#[derive(Debug, thiserror::Error)]
pub enum ZoomVideoMoverError {
    #[error("Authentication error: {0}")]
    Auth(#[from] AuthError),
    
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),
    
    #[error("File system error: {0}")]
    FileSystem(#[from] FileSystemError),
    
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    
    #[error("API error: {0}")]
    Api(#[from] ApiError),
}

// 認証関連エラー
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid client credentials")]
    InvalidCredentials,
    
    #[error("Token expired")]
    TokenExpired,
    
    #[error("OAuth flow failed: {reason}")]
    OAuthFailed { reason: String },
    
    #[error("Insufficient permissions: missing scope {scope}")]
    InsufficientPermissions { scope: String },
}

// ネットワーク関連エラー
#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("Connection timeout after {seconds} seconds")]
    Timeout { seconds: u64 },
    
    #[error("DNS resolution failed for {host}")]
    DnsResolution { host: String },
    
    #[error("TLS handshake failed")]
    TlsHandshake,
    
    #[error("HTTP error: {status} - {message}")]
    Http { status: u16, message: String },
}
```

#### エラー回復戦略
```rust
// 回復可能性による分類
#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryStrategy {
    Retry(RetryConfig),
    UserAction(UserActionRequired),
    Graceful(GracefulDegradation),
    Fatal,
}

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub backoff_factor: f64,
    pub max_delay: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UserActionRequired {
    ReconfigureAuth,
    CheckNetworkConnection,
    FreeUpDiskSpace,
    UpdateApplication,
}

// エラー処理ファサード
pub struct ErrorHandler {
    retry_policies: HashMap<String, RetryConfig>,
    user_notifier: mpsc::Sender<UserNotification>,
}

impl ErrorHandler {
    pub async fn handle_error(&self, error: ZoomVideoMoverError, context: &str) -> RecoveryAction {
        let strategy = self.determine_recovery_strategy(&error);
        
        match strategy {
            RecoveryStrategy::Retry(config) => {
                self.execute_retry(error, config, context).await
            }
            RecoveryStrategy::UserAction(action) => {
                self.notify_user_action_required(action, context).await;
                RecoveryAction::RequireUserIntervention
            }
            RecoveryStrategy::Graceful(degradation) => {
                self.apply_graceful_degradation(degradation).await
            }
            RecoveryStrategy::Fatal => {
                self.handle_fatal_error(error, context).await;
                RecoveryAction::Terminate
            }
        }
    }
}
```

### ログ・モニタリング設計

#### 構造化ログ
```rust
// ログイベント構造
#[derive(Debug, Serialize)]
pub struct LogEvent {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub target: String,
    pub message: String,
    pub context: LogContext,
    pub error_details: Option<ErrorDetails>,
}

#[derive(Debug, Serialize)]
pub struct LogContext {
    pub user_id: Option<String>,
    pub session_id: String,
    pub operation: String,
    pub request_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetails {
    pub error_type: String,
    pub error_code: Option<String>,
    pub stack_trace: Option<String>,
    pub related_data: serde_json::Value,
}

// ログ出力マクロ
macro_rules! log_operation {
    ($level:expr, $target:expr, $operation:expr, $message:expr, $context:expr) => {
        log::log!(
            target: $target,
            $level,
            "{}",
            serde_json::to_string(&LogEvent {
                timestamp: chrono::Utc::now(),
                level: $level,
                target: $target.to_string(),
                message: $message.to_string(),
                context: $context,
                error_details: None,
            }).unwrap_or_default()
        );
    };
}
```

#### メトリクス収集
```rust
// パフォーマンスメトリクス
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub operation_duration: Duration,
    pub memory_usage: u64,
    pub network_bytes_transferred: u64,
    pub api_calls_count: u32,
    pub error_count: u32,
}

// メトリクス収集器
pub struct MetricsCollector {
    operation_timings: HashMap<String, Vec<Duration>>,
    error_counts: HashMap<String, u32>,
    resource_usage: VecDeque<ResourceSnapshot>,
}

impl MetricsCollector {
    pub fn record_operation<T, F>(&mut self, operation_name: &str, operation: F) -> T
    where
        F: FnOnce() -> T,
    {
        let start = Instant::now();
        let result = operation();
        let duration = start.elapsed();
        
        self.operation_timings
            .entry(operation_name.to_string())
            .or_insert_with(Vec::new)
            .push(duration);
        
        result
    }
    
    pub fn get_performance_summary(&self) -> PerformanceSummary {
        PerformanceSummary {
            average_operation_time: self.calculate_average_timing(),
            total_errors: self.error_counts.values().sum(),
            peak_memory_usage: self.get_peak_memory_usage(),
        }
    }
}
```

## セキュリティ設計

### 認証情報保護

#### 設定ファイルセキュリティ
```rust
// 設定ファイルの暗号化保存
pub struct SecureConfigStorage {
    encryption_key: [u8; 32],
    file_path: PathBuf,
}

impl SecureConfigStorage {
    pub fn save_config(&self, config: &Config) -> Result<(), SecurityError> {
        // 1. 設定をJSON化
        let json_data = serde_json::to_vec(config)?;
        
        // 2. AES-256-GCM で暗号化
        let encrypted_data = self.encrypt_data(&json_data)?;
        
        // 3. ファイル権限設定（所有者のみ読み書き可能）
        let mut file = File::create(&self.file_path)?;
        self.set_file_permissions(&file)?;
        
        // 4. 暗号化データを書き込み
        file.write_all(&encrypted_data)?;
        file.sync_all()?;
        
        Ok(())
    }
    
    #[cfg(target_os = "windows")]
    fn set_file_permissions(&self, file: &File) -> Result<(), SecurityError> {
        use std::os::windows::fs::MetadataExt;
        // Windows ACL設定により、所有者のみアクセス許可
        // 実装省略
        Ok(())
    }
}
```

#### メモリセキュリティ
```rust
// 機密情報のメモリ管理
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecretString {
    inner: String,
}

impl SecretString {
    pub fn new(value: String) -> Self {
        Self { inner: value }
    }
    
    pub fn expose_secret(&self) -> &str {
        &self.inner
    }
}

// 自動メモリクリア機能
impl Drop for AuthToken {
    fn drop(&mut self) {
        // 機密情報をメモリから完全削除
        self.access_token.zeroize();
        if let Some(ref mut refresh_token) = self.refresh_token {
            refresh_token.zeroize();
        }
    }
}
```

### 通信セキュリティ

#### HTTPS/TLS設定
```rust
// セキュアHTTPクライアント設定
pub fn create_secure_client() -> Result<reqwest::Client, reqwest::Error> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .connection_verbose(true)
        .https_only(true)  // HTTPS強制
        .min_tls_version(reqwest::tls::Version::TLS_1_2)  // TLS 1.2以上
        .use_rustls_tls()  // Rust-native TLS実装
        .build()
}

// 証明書ピン留め（高セキュリティ要件時）
pub fn create_certificate_pinned_client() -> Result<reqwest::Client, reqwest::Error> {
    let mut root_cert_store = rustls::RootCertStore::empty();
    
    // Zoom APIの証明書チェーン追加
    let zoom_cert = include_bytes!("../certs/zoom_api_cert.pem");
    let zoom_cert = rustls::Certificate(zoom_cert.to_vec());
    root_cert_store.add(&zoom_cert).map_err(|_| {
        reqwest::Error::from(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid certificate"
        ))
    })?;
    
    let tls_config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();
    
    reqwest::Client::builder()
        .use_preconfigured_tls(tls_config)
        .build()
}
```

#### リクエスト検証
```rust
// API リクエストの整合性検証
pub struct RequestValidator {
    expected_hosts: HashSet<String>,
    rate_limiter: RateLimiter,
}

impl RequestValidator {
    pub fn validate_request(&self, request: &reqwest::Request) -> Result<(), SecurityError> {
        // 1. ホスト名検証
        let host = request.url().host_str()
            .ok_or(SecurityError::InvalidHost)?;
        
        if !self.expected_hosts.contains(host) {
            return Err(SecurityError::UnauthorizedHost(host.to_string()));
        }
        
        // 2. スキーム検証（HTTPS必須）
        if request.url().scheme() != "https" {
            return Err(SecurityError::InsecureScheme);
        }
        
        // 3. レート制限チェック
        if !self.rate_limiter.check_rate_limit() {
            return Err(SecurityError::RateLimitExceeded);
        }
        
        Ok(())
    }
}
```

## 性能設計

### 非同期処理最適化

#### タスク管理戦略
```rust
// 効率的なタスクプール管理
pub struct TaskManager {
    // 軽量タスク用（UI更新など）
    light_pool: tokio::task::LocalSet,
    
    // 重量タスク用（ファイルダウンロードなど）
    heavy_executor: Arc<tokio::runtime::Handle>,
    
    // 制限付き並行実行
    download_semaphore: Arc<Semaphore>,
    api_semaphore: Arc<Semaphore>,
}

impl TaskManager {
    pub async fn spawn_download_task<F>(&self, task: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let permit = self.download_semaphore.clone().acquire_owned().await.unwrap();
        
        self.heavy_executor.spawn(async move {
            let _permit = permit; // Hold permit for task duration
            task.await
        })
    }
    
    pub fn spawn_ui_task<F>(&self, task: F)
    where
        F: Future<Output = ()> + 'static,
    {
        self.light_pool.spawn_local(task);
    }
}
```

#### メモリ効率化
```rust
// ストリーミング処理によるメモリ効率化
pub struct StreamingDownloader {
    client: reqwest::Client,
    chunk_size: usize,
    buffer_pool: Arc<Mutex<Vec<Vec<u8>>>>, // バッファ再利用
}

impl StreamingDownloader {
    pub async fn download_stream(
        &self,
        url: &str,
        output_path: &Path,
        progress_sender: mpsc::Sender<u64>,
    ) -> Result<(), DownloadError> {
        let response = self.client.get(url).send().await?;
        let total_size = response.content_length().unwrap_or(0);
        
        let mut file = tokio::fs::File::create(output_path).await?;
        let mut stream = response.bytes_stream();
        let mut downloaded = 0u64;
        
        // バッファプールからバッファを取得
        let mut buffer = self.get_or_create_buffer();
        
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            
            // チャンク書き込み
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            
            // 進捗通知（非ブロッキング）
            let _ = progress_sender.try_send(downloaded);
            
            // 定期的なフラッシュ（パフォーマンスバランス）
            if downloaded % (1024 * 1024) == 0 { // 1MBごと
                file.flush().await?;
            }
        }
        
        // バッファをプールに戻す
        self.return_buffer(buffer);
        
        file.sync_all().await?;
        Ok(())
    }
}
```

### キャッシュ戦略

#### APIレスポンスキャッシュ
```rust
// LRUキャッシュによるAPI効率化
use lru::LruCache;

pub struct ApiCache {
    recording_cache: Arc<Mutex<LruCache<String, Vec<Recording>>>>,
    ai_summary_cache: Arc<Mutex<LruCache<String, AiSummaryResponse>>>,
    cache_ttl: Duration,
}

impl ApiCache {
    pub async fn get_recordings_cached(
        &self,
        date_range: &str,
        fetcher: impl Fn(&str) -> BoxFuture<'_, Result<Vec<Recording>, ApiError>>,
    ) -> Result<Vec<Recording>, ApiError> {
        let cache_key = format!("recordings_{}", date_range);
        
        // キャッシュ確認
        if let Some(cached) = self.recording_cache.lock().await.get(&cache_key) {
            return Ok(cached.clone());
        }
        
        // キャッシュミス時はAPI呼び出し
        let recordings = fetcher(date_range).await?;
        
        // 結果をキャッシュに保存
        self.recording_cache.lock().await.put(cache_key, recordings.clone());
        
        Ok(recordings)
    }
}
```

#### ファイルシステムキャッシュ
```rust
// メタデータキャッシュ
pub struct FileSystemCache {
    metadata_cache: DashMap<PathBuf, FileMetadata>,
    max_cache_size: usize,
}

#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub size: u64,
    pub modified: SystemTime,
    pub permissions: std::fs::Permissions,
    pub cached_at: Instant,
}

impl FileSystemCache {
    pub fn get_file_size(&self, path: &Path) -> std::io::Result<u64> {
        if let Some(metadata) = self.metadata_cache.get(path) {
            // キャッシュの有効性確認（5分間有効）
            if metadata.cached_at.elapsed() < Duration::from_secs(300) {
                return Ok(metadata.size);
            }
        }
        
        // キャッシュミス時は実際にファイルアクセス
        let metadata = std::fs::metadata(path)?;
        let file_metadata = FileMetadata {
            size: metadata.len(),
            modified: metadata.modified()?,
            permissions: metadata.permissions(),
            cached_at: Instant::now(),
        };
        
        self.metadata_cache.insert(path.to_path_buf(), file_metadata.clone());
        Ok(file_metadata.size)
    }
}
```

## テスト設計支援

### テスト容易性設計

#### 依存性注入
```rust
// テスト可能な設計のための抽象化
#[async_trait::async_trait]
pub trait ZoomApiClient: Send + Sync {
    async fn get_recordings(&self, from: &str, to: &str) -> Result<Vec<Recording>>;
    async fn download_file(&self, url: &str, path: &Path) -> Result<()>;
}

// 本番実装
pub struct ProductionZoomApiClient {
    client: reqwest::Client,
    access_token: String,
}

// テスト用モック実装
pub struct MockZoomApiClient {
    recordings_response: Vec<Recording>,
    download_behavior: DownloadBehavior,
}

#[async_trait::async_trait]
impl ZoomApiClient for MockZoomApiClient {
    async fn get_recordings(&self, _from: &str, _to: &str) -> Result<Vec<Recording>> {
        Ok(self.recordings_response.clone())
    }
    
    async fn download_file(&self, _url: &str, _path: &Path) -> Result<()> {
        match &self.download_behavior {
            DownloadBehavior::Success => Ok(()),
            DownloadBehavior::NetworkError => Err(ApiError::Network("Simulated error".to_string())),
            DownloadBehavior::Slow(delay) => {
                tokio::time::sleep(*delay).await;
                Ok(())
            }
        }
    }
}
```

#### テストユーティリティ
```rust
// テスト用のファクトリー関数
pub mod test_utils {
    use super::*;
    
    pub fn create_test_config() -> Config {
        Config {
            client_id: "test_client_id".to_string(),
            client_secret: "test_client_secret".to_string(),
            redirect_uri: Some("http://localhost:8080/callback".to_string()),
            output_directory: PathBuf::from("./test_downloads"),
        }
    }
    
    pub fn create_test_recording() -> Recording {
        Recording {
            meeting_id: "123456789".to_string(),
            meeting_uuid: "test-uuid-123".to_string(),
            topic: "Test Meeting".to_string(),
            start_time: Utc.ymd(2024, 1, 15).and_hms(10, 0, 0),
            duration: 60,
            recording_files: vec![
                RecordingFile {
                    id: "file1".to_string(),
                    file_type: "MP4".to_string(),
                    file_size: 1073741824, // 1GB
                    download_url: "https://example.com/video.mp4".to_string(),
                    recording_start: Utc.ymd(2024, 1, 15).and_hms(10, 0, 0),
                    recording_end: Utc.ymd(2024, 1, 15).and_hms(11, 0, 0),
                }
            ],
            ai_summary_available: true,
        }
    }
    
    pub async fn create_test_app_with_mocks() -> ZoomDownloaderApp {
        let (command_sender, command_receiver) = mpsc::channel(100);
        let (event_sender, event_receiver) = mpsc::channel(100);
        
        ZoomDownloaderApp::new_with_dependencies(
            Box::new(MockZoomApiClient::default()),
            Box::new(MockFileStorage::default()),
            command_sender,
            event_receiver,
        )
    }
}
```

## 拡張性・保守性設計

### プラグインアーキテクチャ

#### 拡張ポイント設計
```rust
// プラグインインターフェース
#[async_trait::async_trait]
pub trait DownloadPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    
    async fn process_file(
        &self,
        file_info: &RecordingFile,
        downloaded_path: &Path,
    ) -> Result<ProcessedFile, PluginError>;
}

// プラグイン管理システム
pub struct PluginManager {
    plugins: Vec<Box<dyn DownloadPlugin>>,
    enabled_plugins: HashSet<String>,
}

impl PluginManager {
    pub fn register_plugin(&mut self, plugin: Box<dyn DownloadPlugin>) {
        let name = plugin.name().to_string();
        self.plugins.push(plugin);
        self.enabled_plugins.insert(name);
    }
    
    pub async fn process_with_plugins(
        &self,
        file_info: &RecordingFile,
        path: &Path,
    ) -> Result<Vec<ProcessedFile>, Vec<PluginError>> {
        let mut results = Vec::new();
        let mut errors = Vec::new();
        
        for plugin in &self.plugins {
            if self.enabled_plugins.contains(plugin.name()) {
                match plugin.process_file(file_info, path).await {
                    Ok(processed) => results.push(processed),
                    Err(error) => errors.push(error),
                }
            }
        }
        
        if errors.is_empty() {
            Ok(results)
        } else {
            Err(errors)
        }
    }
}
```

### 設定システム拡張

#### 階層設定管理
```rust
// 拡張可能な設定システム
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationConfig {
    // 基本設定
    pub oauth: OAuthConfig,
    pub download: DownloadConfig,
    pub ui: UiConfig,
    
    // 拡張設定
    pub plugins: HashMap<String, serde_json::Value>,
    pub experimental: ExperimentalConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentalConfig {
    pub enable_ai_features: bool,
    pub use_experimental_ui: bool,
    pub debug_mode: bool,
    
    #[serde(flatten)]
    pub additional_settings: HashMap<String, serde_json::Value>,
}

// 設定マイグレーション
pub struct ConfigMigrator {
    migrations: Vec<Box<dyn ConfigMigration>>,
}

pub trait ConfigMigration {
    fn version(&self) -> u32;
    fn migrate(&self, old_config: serde_json::Value) -> Result<serde_json::Value, MigrationError>;
}
```

## 品質保証設計

### 品質メトリクス

#### コード品質指標
```rust
// 自動品質チェック
pub struct QualityMetrics {
    pub cyclomatic_complexity: HashMap<String, u32>,
    pub test_coverage: f64,
    pub documentation_coverage: f64,
    pub dependency_freshness: Vec<OutdatedDependency>,
}

impl QualityMetrics {
    pub fn check_quality_gates(&self) -> QualityReport {
        let mut issues = Vec::new();
        
        // 複雑度チェック
        for (function, complexity) in &self.cyclomatic_complexity {
            if *complexity > 10 {
                issues.push(QualityIssue::HighComplexity {
                    function: function.clone(),
                    complexity: *complexity,
                });
            }
        }
        
        // カバレッジチェック
        if self.test_coverage < 0.90 {
            issues.push(QualityIssue::LowTestCoverage {
                current: self.test_coverage,
                required: 0.90,
            });
        }
        
        QualityReport {
            issues,
            overall_score: self.calculate_overall_score(),
        }
    }
}
```

### 性能監視設計

#### 自動性能測定
```rust
// 性能プロファイリング
pub struct PerformanceProfiler {
    operation_timers: HashMap<String, Vec<Duration>>,
    memory_snapshots: VecDeque<MemorySnapshot>,
    network_stats: NetworkStatistics,
}

impl PerformanceProfiler {
    pub async fn profile_operation<T, F>(&mut self, name: &str, operation: F) -> T
    where
        F: Future<Output = T>,
    {
        let start_memory = self.capture_memory_snapshot();
        let start_time = Instant::now();
        
        let result = operation.await;
        
        let duration = start_time.elapsed();
        let end_memory = self.capture_memory_snapshot();
        
        self.record_operation_timing(name, duration);
        self.record_memory_usage(name, start_memory, end_memory);
        
        result
    }
    
    pub fn generate_performance_report(&self) -> PerformanceReport {
        PerformanceReport {
            slow_operations: self.identify_slow_operations(),
            memory_leaks: self.detect_memory_leaks(),
            network_efficiency: self.calculate_network_efficiency(),
            recommendations: self.generate_recommendations(),
        }
    }
}
```

## 結論

本設計方針は、**高品質・高性能・高保守性**を実現するための包括的な設計ガイドラインを提供します。

### 設計の特徴
- **モジュラー設計**: 疎結合・高凝集なコンポーネント構成
- **非同期アーキテクチャ**: tokio ベースの効率的な並行処理
- **堅牢なエラーハンドリング**: 階層化された回復可能なエラー処理
- **セキュリティ重視**: 認証情報保護・通信暗号化の徹底
- **テスト容易性**: 依存性注入による高いテスト可能性
- **拡張性**: プラグイン対応・設定システムの柔軟性

### 期待効果
- **開発効率**: 明確な設計指針による開発速度向上
- **品質向上**: 体系的な品質保証による信頼性確保
- **保守性**: 構造化された設計による長期保守の容易化
- **拡張性**: 将来要件への柔軟な対応力

この設計方針に従うことで、**ユーザーのニーズを満たし、技術的に優秀で、長期間にわたって成長可能**なソフトウェアシステムを構築できます。