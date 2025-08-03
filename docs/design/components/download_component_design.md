# ダウンロード実行コンポーネント詳細設計書 - Zoom Video Mover

## 文書概要
**文書ID**: DES-DOWNLOAD-001  
**コンポーネント名**: ダウンロード実行コンポーネント（Download Execution Component）  
**作成日**: 2025-08-03  
**作成者**: エンジン設計者  
**レビューア**: 性能エンジニア  
**バージョン**: 1.0  

## コンポーネント概要

### 責任・役割
- **並列ダウンロード制御**: tokio非同期処理による効率的な並列ダウンロード
- **進捗監視・通知**: リアルタイム進捗追跡とユーザー通知システム
- **ファイル管理**: ダウンロードファイルの整合性検証・重複処理・クリーンアップ
- **エラー処理・回復**: ネットワークエラーの自動回復・レジューム機能

### アーキテクチャ位置
```
┌─────────────────────────────────────────────────────────────────┐
│                   Application Layer                             │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │              Download Execution Component                    │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │ │
│  │  │  Parallel   │  │  Progress   │  │     File            │ │ │
│  │  │  Download   │  │  Monitor    │  │     Manager         │ │ │
│  │  │  Engine     │  │             │  │                     │ │ │
│  │  └─────────────┘  └─────────────┘  └─────────────────────┘ │ │
│  │  ┌─────────────────────────────────────────────────────────┐ │ │
│  │  │           Error Recovery Engine                         │ │ │
│  │  └─────────────────────────────────────────────────────────┘ │ │
│  └─────────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                 Infrastructure Layer                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │   HTTP      │  │   File      │  │    Hash                 │  │
│  │   Stream    │  │ System API  │  │  Calculation            │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## モジュール構造設計

### 内部モジュール構成
```rust
pub mod download {
    /// 並列ダウンロードエンジン
    pub mod parallel_engine;
    
    /// 進捗監視システム
    pub mod progress_monitor;
    
    /// ファイル管理
    pub mod file_manager;
    
    /// エラー回復エンジン
    pub mod error_recovery;
    
    /// ダウンロードタスク管理
    pub mod task_manager;
    
    /// チャンク処理
    pub mod chunk_processor;
    
    /// 完全性検証
    pub mod integrity_verifier;
    
    /// レジューム機能
    pub mod resume_handler;
    
    /// エラー定義
    pub mod error;
    
    /// 設定・定数
    pub mod config;
}
```

### モジュール依存関係
```
parallel_engine
    ├── → task_manager
    ├── → progress_monitor
    ├── → error_recovery
    └── → error

task_manager
    ├── → chunk_processor
    ├── → resume_handler
    └── → error

progress_monitor
    ├── → task_manager
    └── → error

file_manager
    ├── → integrity_verifier
    ├── → chunk_processor
    └── → error

error_recovery
    ├── → resume_handler
    ├── → task_manager
    └── → error

chunk_processor
    └── → error

integrity_verifier
    └── → error

resume_handler
    └── → error
```

## データ構造設計

### コアデータ構造

#### 1. ダウンロードタスク
```rust
/// ダウンロードタスク（非同期実行単位）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadTask {
    /// タスクID（UUID）
    pub task_id: String,
    
    /// ダウンロードURL
    pub download_url: String,
    
    /// 出力ファイルパス
    pub output_path: PathBuf,
    
    /// 期待ファイルサイズ
    pub expected_size: Option<u64>,
    
    /// ファイル情報
    pub file_info: DownloadFileInfo,
    
    /// ダウンロード設定
    pub config: DownloadTaskConfig,
    
    /// タスク状態
    pub state: TaskState,
    
    /// 進捗情報
    pub progress: DownloadProgress,
    
    /// エラー履歴
    pub error_history: Vec<DownloadError>,
    
    /// 作成時刻
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// 開始時刻
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    
    /// 完了時刻
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    
    /// レジューム情報
    pub resume_info: Option<ResumeInfo>,
}

/// ダウンロードファイル情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadFileInfo {
    /// ファイル名
    pub file_name: String,
    
    /// ファイル種別
    pub file_type: crate::recording::RecordingFileType,
    
    /// MIME タイプ
    pub mime_type: Option<String>,
    
    /// ハッシュ値（検証用）
    pub expected_hash: Option<FileHash>,
    
    /// 録画メタデータ参照
    pub recording_metadata: RecordingReference,
}

/// ダウンロードタスク状態
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskState {
    /// 待機中
    Pending,
    
    /// 実行中
    InProgress {
        started_at: chrono::DateTime<chrono::Utc>,
        worker_id: String,
    },
    
    /// 一時停止中
    Paused {
        paused_at: chrono::DateTime<chrono::Utc>,
        reason: PauseReason,
    },
    
    /// 完了
    Completed {
        completed_at: chrono::DateTime<chrono::Utc>,
        verification_result: VerificationResult,
    },
    
    /// 失敗
    Failed {
        failed_at: chrono::DateTime<chrono::Utc>,
        error: DownloadError,
        retry_count: u32,
    },
    
    /// キャンセル
    Cancelled {
        cancelled_at: chrono::DateTime<chrono::Utc>,
        reason: CancellationReason,
    },
}
```

#### 2. 進捗情報
```rust
/// ダウンロード進捗情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    /// ダウンロード済みバイト数
    pub downloaded_bytes: u64,
    
    /// 総バイト数
    pub total_bytes: Option<u64>,
    
    /// 進捗率（0.0-1.0）
    pub percentage: f64,
    
    /// 現在の転送速度（bytes/sec）
    pub current_speed: f64,
    
    /// 平均転送速度（bytes/sec）
    pub average_speed: f64,
    
    /// 推定残り時間
    pub eta: Option<Duration>,
    
    /// 完了予定時刻
    pub estimated_completion: Option<chrono::DateTime<chrono::Utc>>,
    
    /// アクティブチャンク数
    pub active_chunks: u32,
    
    /// 完了チャンク数
    pub completed_chunks: u32,
    
    /// 総チャンク数
    pub total_chunks: u32,
    
    /// 最終更新時刻
    pub last_updated: chrono::DateTime<chrono::Utc>,
    
    /// 速度履歴（移動平均計算用）
    pub speed_history: VecDeque<SpeedSample>,
}

/// 速度サンプル
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedSample {
    /// サンプル時刻
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// バイト数
    pub bytes: u64,
    
    /// 時間間隔（秒）
    pub interval: f64,
}

/// 全体進捗情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverallProgress {
    /// 総タスク数
    pub total_tasks: u32,
    
    /// 完了タスク数
    pub completed_tasks: u32,
    
    /// 失敗タスク数
    pub failed_tasks: u32,
    
    /// アクティブタスク数
    pub active_tasks: u32,
    
    /// 全体進捗率
    pub overall_percentage: f64,
    
    /// 全体ダウンロード済みバイト数
    pub total_downloaded_bytes: u64,
    
    /// 全体予想バイト数
    pub total_expected_bytes: u64,
    
    /// 全体平均速度
    pub overall_average_speed: f64,
    
    /// 全体推定残り時間
    pub overall_eta: Option<Duration>,
    
    /// 開始時刻
    pub started_at: chrono::DateTime<chrono::Utc>,
    
    /// 推定完了時刻
    pub estimated_completion: Option<chrono::DateTime<chrono::Utc>>,
}
```

#### 3. チャンク処理
```rust
/// ダウンロードチャンク
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadChunk {
    /// チャンクID
    pub chunk_id: String,
    
    /// タスクID参照
    pub task_id: String,
    
    /// 開始オフセット
    pub start_offset: u64,
    
    /// 終了オフセット
    pub end_offset: u64,
    
    /// チャンクサイズ
    pub chunk_size: u64,
    
    /// チャンク状態
    pub state: ChunkState,
    
    /// ダウンロード済みバイト数
    pub downloaded_bytes: u64,
    
    /// リトライ回数
    pub retry_count: u32,
    
    /// 作成時刻
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// 開始時刻
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    
    /// 完了時刻
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    
    /// エラー情報
    pub last_error: Option<ChunkError>,
}

/// チャンク状態
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChunkState {
    /// 待機中
    Pending,
    
    /// ダウンロード中
    Downloading {
        worker_id: String,
        started_at: chrono::DateTime<chrono::Utc>,
    },
    
    /// 完了
    Completed {
        completed_at: chrono::DateTime<chrono::Utc>,
        hash: Option<String>,
    },
    
    /// 失敗
    Failed {
        failed_at: chrono::DateTime<chrono::Utc>,
        error: ChunkError,
    },
    
    /// リトライ待機中
    AwaitingRetry {
        retry_at: chrono::DateTime<chrono::Utc>,
        retry_count: u32,
    },
}

/// レジューム情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeInfo {
    /// 部分ファイルパス
    pub partial_file_path: PathBuf,
    
    /// 既存ダウンロード済みサイズ
    pub existing_size: u64,
    
    /// チャンク完了状況
    pub chunk_completion: HashMap<String, ChunkState>,
    
    /// 最終レジューム時刻
    pub last_resume_at: chrono::DateTime<chrono::Utc>,
    
    /// レジューム回数
    pub resume_count: u32,
    
    /// ファイル整合性情報
    pub integrity_info: Option<PartialIntegrityInfo>,
}
```

## インターフェース設計

### 公開API

#### 1. ダウンロードエンジン
```rust
/// ダウンロードエンジン - コンポーネントのメインインターフェース
#[async_trait]
pub trait DownloadEngine: Send + Sync {
    /// 単一ファイルダウンロード
    async fn download_file(&self, task: DownloadTask) -> Result<DownloadResult, DownloadError>;
    
    /// 複数ファイル並列ダウンロード
    async fn download_multiple(&self, tasks: Vec<DownloadTask>) -> Result<Vec<DownloadResult>, DownloadError>;
    
    /// ダウンロード進捗監視
    fn subscribe_progress(&self) -> broadcast::Receiver<ProgressUpdate>;
    
    /// アクティブダウンロード一覧取得
    fn get_active_downloads(&self) -> Vec<DownloadTask>;
    
    /// ダウンロード一時停止
    async fn pause_download(&self, task_id: &str) -> Result<(), DownloadError>;
    
    /// ダウンロード再開
    async fn resume_download(&self, task_id: &str) -> Result<(), DownloadError>;
    
    /// ダウンロードキャンセル
    async fn cancel_download(&self, task_id: &str) -> Result<(), DownloadError>;
    
    /// 全ダウンロード停止
    async fn cancel_all_downloads(&self) -> Result<(), DownloadError>;
    
    /// ダウンロード統計取得
    fn get_download_statistics(&self) -> DownloadStatistics;
    
    /// エラー回復試行
    async fn retry_failed_download(&self, task_id: &str) -> Result<(), DownloadError>;
}
```

#### 2. 実装クラス
```rust
/// ダウンロードエンジン実装
pub struct TokioDownloadEngine {
    /// 並列制御セマフォ
    concurrency_semaphore: Arc<Semaphore>,
    
    /// タスクマネージャー
    task_manager: Arc<DownloadTaskManager>,
    
    /// 進捗監視システム
    progress_monitor: Arc<ProgressMonitor>,
    
    /// ファイルマネージャー
    file_manager: Arc<FileManager>,
    
    /// エラー回復エンジン
    error_recovery: Arc<ErrorRecoveryEngine>,
    
    /// HTTP クライアント
    http_client: Arc<HttpClient>,
    
    /// 設定情報
    config: DownloadConfig,
    
    /// キャンセレーショントークン
    cancellation_token: CancellationToken,
    
    /// 進捗通知チャンネル
    progress_tx: broadcast::Sender<ProgressUpdate>,
    
    /// 統計収集器
    statistics_collector: Arc<StatisticsCollector>,
}

impl TokioDownloadEngine {
    /// 新しいダウンロードエンジンを作成
    pub fn new(config: DownloadConfig) -> Result<Self, DownloadError> {
        let concurrency_semaphore = Arc::new(Semaphore::new(config.max_concurrent_downloads));
        let task_manager = Arc::new(DownloadTaskManager::new(&config)?);
        let progress_monitor = Arc::new(ProgressMonitor::new(&config)?);
        let file_manager = Arc::new(FileManager::new(&config)?);
        let error_recovery = Arc::new(ErrorRecoveryEngine::new(&config)?);
        let http_client = Arc::new(HttpClient::new(&config)?);
        let cancellation_token = CancellationToken::new();
        let (progress_tx, _) = broadcast::channel(1000);
        let statistics_collector = Arc::new(StatisticsCollector::new());
        
        Ok(Self {
            concurrency_semaphore,
            task_manager,
            progress_monitor,
            file_manager,
            error_recovery,
            http_client,
            config,
            cancellation_token,
            progress_tx,
            statistics_collector,
        })
    }
}

#[async_trait]
impl DownloadEngine for TokioDownloadEngine {
    async fn download_file(&self, mut task: DownloadTask) -> Result<DownloadResult, DownloadError> {
        // 1. タスク登録・状態更新
        task.state = TaskState::InProgress {
            started_at: chrono::Utc::now(),
            worker_id: self.generate_worker_id(),
        };
        self.task_manager.register_task(task.clone()).await?;
        
        // 2. 並列制御
        let _permit = self.concurrency_semaphore.acquire().await
            .map_err(|e| DownloadError::ConcurrencyError(e.to_string()))?;
        
        // 3. レジューム情報確認
        if let Some(resume_info) = &task.resume_info {
            return self.resume_download_internal(&task, resume_info).await;
        }
        
        // 4. 新規ダウンロード実行
        self.execute_download_task(task).await
    }
    
    async fn download_multiple(&self, tasks: Vec<DownloadTask>) -> Result<Vec<DownloadResult>, DownloadError> {
        let mut handles = Vec::new();
        let mut results = Vec::new();
        
        // 1. 全体進捗監視開始
        let overall_progress = OverallProgress::new(tasks.len() as u32);
        self.progress_monitor.start_overall_monitoring(overall_progress).await;
        
        // 2. 各タスクを並列実行
        for task in tasks {
            let engine_clone = self.clone_for_task();
            let handle = tokio::spawn(async move {
                engine_clone.download_file(task).await
            });
            handles.push(handle);
        }
        
        // 3. 全タスク完了待機
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(e) => {
                    // JoinError を DownloadError に変換
                    results.push(Err(DownloadError::TaskJoinError(e.to_string())));
                }
            }
        }
        
        // 4. 統計情報更新
        self.statistics_collector.record_batch_completion(&results).await;
        
        // 5. 成功・失敗の結果分離
        let (successes, failures): (Vec<_>, Vec<_>) = results.into_iter().partition(|r| r.is_ok());
        
        if failures.is_empty() {
            Ok(successes.into_iter().map(|r| r.unwrap()).collect())
        } else {
            Err(DownloadError::BatchDownloadError {
                successful_count: successes.len(),
                failed_count: failures.len(),
                sample_errors: failures.into_iter().take(5).map(|r| r.unwrap_err()).collect(),
            })
        }
    }
    
    async fn pause_download(&self, task_id: &str) -> Result<(), DownloadError> {
        // 1. タスク取得・状態確認
        let mut task = self.task_manager.get_task(task_id).await?;
        
        if !matches!(task.state, TaskState::InProgress { .. }) {
            return Err(DownloadError::InvalidTaskState {
                task_id: task_id.to_string(),
                current_state: format!("{:?}", task.state),
                expected_states: vec!["InProgress".to_string()],
            });
        }
        
        // 2. タスク一時停止
        task.state = TaskState::Paused {
            paused_at: chrono::Utc::now(),
            reason: PauseReason::UserRequested,
        };
        
        // 3. アクティブチャンクの停止
        self.task_manager.pause_active_chunks(task_id).await?;
        
        // 4. レジューム情報生成・保存
        let resume_info = self.create_resume_info(&task).await?;
        task.resume_info = Some(resume_info);
        
        // 5. タスク状態更新
        self.task_manager.update_task(task).await?;
        
        // 6. 進捗通知
        self.progress_tx.send(ProgressUpdate::TaskPaused {
            task_id: task_id.to_string(),
        }).ok();
        
        Ok(())
    }
}
```

### 内部インターフェース

#### 1. タスクマネージャー
```rust
/// ダウンロードタスク管理インターフェース
#[async_trait]
pub trait DownloadTaskManager: Send + Sync {
    /// タスク登録
    async fn register_task(&self, task: DownloadTask) -> Result<(), TaskManagerError>;
    
    /// タスク取得
    async fn get_task(&self, task_id: &str) -> Result<DownloadTask, TaskManagerError>;
    
    /// タスク状態更新
    async fn update_task(&self, task: DownloadTask) -> Result<(), TaskManagerError>;
    
    /// アクティブタスク一覧
    async fn get_active_tasks(&self) -> Vec<DownloadTask>;
    
    /// チャンク生成・管理
    async fn create_chunks(&self, task: &DownloadTask) -> Result<Vec<DownloadChunk>, TaskManagerError>;
    
    /// チャンク状態更新
    async fn update_chunk(&self, chunk: DownloadChunk) -> Result<(), TaskManagerError>;
    
    /// タスク削除
    async fn remove_task(&self, task_id: &str) -> Result<(), TaskManagerError>;
}

/// ダウンロードタスクマネージャー実装
pub struct MemoryTaskManager {
    /// タスクストレージ
    tasks: Arc<RwLock<HashMap<String, DownloadTask>>>,
    
    /// チャンクストレージ
    chunks: Arc<RwLock<HashMap<String, DownloadChunk>>>,
    
    /// タスクインデックス（状態別）
    task_by_state: Arc<RwLock<HashMap<TaskState, HashSet<String>>>>,
    
    /// 設定情報
    config: TaskManagerConfig,
}

impl MemoryTaskManager {
    pub fn new(config: &DownloadConfig) -> Result<Self, TaskManagerError> {
        let tasks = Arc::new(RwLock::new(HashMap::new()));
        let chunks = Arc::new(RwLock::new(HashMap::new()));
        let task_by_state = Arc::new(RwLock::new(HashMap::new()));
        let config = TaskManagerConfig::from_download_config(config);
        
        Ok(Self {
            tasks,
            chunks,
            task_by_state,
            config,
        })
    }
}

#[async_trait]
impl DownloadTaskManager for MemoryTaskManager {
    async fn create_chunks(&self, task: &DownloadTask) -> Result<Vec<DownloadChunk>, TaskManagerError> {
        // 1. ファイルサイズ取得
        let total_size = if let Some(size) = task.expected_size {
            size
        } else {
            self.get_remote_file_size(&task.download_url).await?
        };
        
        // 2. チャンクサイズ計算
        let chunk_size = self.calculate_optimal_chunk_size(total_size);
        
        // 3. チャンク分割
        let mut chunks = Vec::new();
        let mut start_offset = 0;
        let mut chunk_index = 0;
        
        while start_offset < total_size {
            let end_offset = std::cmp::min(start_offset + chunk_size - 1, total_size - 1);
            
            let chunk = DownloadChunk {
                chunk_id: format!("{}_{:04}", task.task_id, chunk_index),
                task_id: task.task_id.clone(),
                start_offset,
                end_offset,
                chunk_size: end_offset - start_offset + 1,
                state: ChunkState::Pending,
                downloaded_bytes: 0,
                retry_count: 0,
                created_at: chrono::Utc::now(),
                started_at: None,
                completed_at: None,
                last_error: None,
            };
            
            chunks.push(chunk);
            start_offset = end_offset + 1;
            chunk_index += 1;
        }
        
        // 4. チャンク登録
        let mut chunks_store = self.chunks.write().await;
        for chunk in &chunks {
            chunks_store.insert(chunk.chunk_id.clone(), chunk.clone());
        }
        
        Ok(chunks)
    }
    
    /// 最適チャンクサイズ計算
    fn calculate_optimal_chunk_size(&self, total_size: u64) -> u64 {
        // ファイルサイズに基づく動的チャンクサイズ決定
        match total_size {
            0..=1_048_576 => total_size, // 1MB以下は分割しない
            1_048_577..=10_485_760 => 1_048_576, // 10MB以下は1MBチャンク
            10_485_761..=104_857_600 => 2_097_152, // 100MB以下は2MBチャンク
            104_857_601..=1_073_741_824 => 5_242_880, // 1GB以下は5MBチャンク
            _ => 10_485_760, // 1GB超は10MBチャンク
        }
    }
}
```

#### 2. 進捗監視システム
```rust
/// 進捗監視システムインターフェース
#[async_trait]
pub trait ProgressMonitor: Send + Sync {
    /// 進捗更新
    async fn update_progress(&self, task_id: &str, progress: DownloadProgress) -> Result<(), ProgressError>;
    
    /// 全体進捗計算
    async fn calculate_overall_progress(&self) -> OverallProgress;
    
    /// 進捗統計取得
    fn get_progress_statistics(&self, task_id: &str) -> Option<ProgressStatistics>;
    
    /// 進捗監視開始
    async fn start_monitoring(&self, task_id: &str) -> Result<(), ProgressError>;
    
    /// 進捗監視停止
    async fn stop_monitoring(&self, task_id: &str) -> Result<(), ProgressError>;
}

/// 進捗監視システム実装
pub struct RealTimeProgressMonitor {
    /// 進捗データストレージ
    progress_data: Arc<RwLock<HashMap<String, DownloadProgress>>>,
    
    /// 進捗更新チャンネル
    progress_tx: broadcast::Sender<ProgressUpdate>,
    
    /// 監視タスクハンドル
    monitoring_handles: Arc<RwLock<HashMap<String, AbortHandle>>>,
    
    /// 設定情報
    config: ProgressConfig,
    
    /// 統計収集器
    statistics: Arc<ProgressStatisticsCollector>,
}

impl RealTimeProgressMonitor {
    pub fn new(config: &DownloadConfig) -> Result<Self, ProgressError> {
        let progress_data = Arc::new(RwLock::new(HashMap::new()));
        let (progress_tx, _) = broadcast::channel(1000);
        let monitoring_handles = Arc::new(RwLock::new(HashMap::new()));
        let config = ProgressConfig::from_download_config(config);
        let statistics = Arc::new(ProgressStatisticsCollector::new());
        
        Ok(Self {
            progress_data,
            progress_tx,
            monitoring_handles,
            config,
            statistics,
        })
    }
}

#[async_trait]
impl ProgressMonitor for RealTimeProgressMonitor {
    async fn update_progress(&self, task_id: &str, mut progress: DownloadProgress) -> Result<(), ProgressError> {
        // 1. 進捗計算・更新
        progress.percentage = if let Some(total) = progress.total_bytes {
            if total > 0 {
                (progress.downloaded_bytes as f64 / total as f64).min(1.0)
            } else {
                0.0
            }
        } else {
            0.0
        };
        
        // 2. 速度計算
        self.calculate_and_update_speed(&mut progress).await;
        
        // 3. ETA計算
        self.calculate_eta(&mut progress).await;
        
        // 4. 進捗データ保存
        {
            let mut data = self.progress_data.write().await;
            data.insert(task_id.to_string(), progress.clone());
        }
        
        // 5. 進捗通知
        self.progress_tx.send(ProgressUpdate::TaskProgress {
            task_id: task_id.to_string(),
            progress: progress.clone(),
        }).ok();
        
        // 6. 統計記録
        self.statistics.record_progress_update(task_id, &progress).await;
        
        Ok(())
    }
    
    /// 転送速度計算・更新
    async fn calculate_and_update_speed(&self, progress: &mut DownloadProgress) {
        let now = chrono::Utc::now();
        
        // 新しい速度サンプル追加
        if let Some(last_sample) = progress.speed_history.back() {
            let interval = (now - last_sample.timestamp).num_seconds() as f64;
            if interval > 0.0 {
                let bytes_diff = progress.downloaded_bytes.saturating_sub(last_sample.bytes);
                let current_speed = bytes_diff as f64 / interval;
                
                progress.current_speed = current_speed;
                progress.speed_history.push_back(SpeedSample {
                    timestamp: now,
                    bytes: progress.downloaded_bytes,
                    interval,
                });
            }
        } else {
            // 初回サンプル
            progress.speed_history.push_back(SpeedSample {
                timestamp: now,
                bytes: progress.downloaded_bytes,
                interval: 0.0,
            });
        }
        
        // 古いサンプル削除（移動平均用に直近N個保持）
        while progress.speed_history.len() > self.config.speed_sample_count {
            progress.speed_history.pop_front();
        }
        
        // 平均速度計算
        if progress.speed_history.len() > 1 {
            let total_bytes: u64 = progress.speed_history.iter().map(|s| s.bytes).sum();
            let total_time: f64 = progress.speed_history.iter().map(|s| s.interval).sum();
            
            if total_time > 0.0 {
                progress.average_speed = total_bytes as f64 / total_time;
            }
        }
        
        progress.last_updated = now;
    }
    
    /// ETA（残り時間）計算
    async fn calculate_eta(&self, progress: &mut DownloadProgress) {
        if let Some(total_bytes) = progress.total_bytes {
            let remaining_bytes = total_bytes.saturating_sub(progress.downloaded_bytes);
            
            if remaining_bytes > 0 && progress.average_speed > 0.0 {
                let eta_seconds = remaining_bytes as f64 / progress.average_speed;
                progress.eta = Some(Duration::from_secs(eta_seconds as u64));
                
                let completion_time = chrono::Utc::now() + chrono::Duration::seconds(eta_seconds as i64);
                progress.estimated_completion = Some(completion_time);
            } else {
                progress.eta = None;
                progress.estimated_completion = None;
            }
        }
    }
}
```

## アルゴリズム設計

### 並列ダウンロードアルゴリズム

#### チャンク並列処理
```rust
impl ParallelDownloadEngine {
    /// 並列チャンクダウンロード実行
    pub async fn execute_parallel_download(&self, task: DownloadTask) -> Result<DownloadResult, DownloadError> {
        // 1. チャンク生成
        let chunks = self.task_manager.create_chunks(&task).await?;
        
        // 2. 並列度制御セマフォ取得
        let concurrency = std::cmp::min(chunks.len(), self.config.max_chunk_concurrency);
        let semaphore = Arc::new(Semaphore::new(concurrency));
        
        // 3. 並列ダウンロードタスク生成
        let download_handles: Vec<_> = chunks.into_iter().map(|chunk| {
            let semaphore = semaphore.clone();
            let http_client = self.http_client.clone();
            let progress_monitor = self.progress_monitor.clone();
            let task_url = task.download_url.clone();
            let output_path = task.output_path.clone();
            
            tokio::spawn(async move {
                let _permit = semaphore.acquire().await?;
                Self::download_chunk(
                    chunk,
                    task_url,
                    output_path,
                    http_client,
                    progress_monitor,
                ).await
            })
        }).collect();
        
        // 4. 全チャンク完了待機
        let mut chunk_results = Vec::new();
        for handle in download_handles {
            match handle.await {
                Ok(result) => chunk_results.push(result),
                Err(e) => return Err(DownloadError::ChunkJoinError(e.to_string())),
            }
        }
        
        // 5. チャンク結果検証
        let failed_chunks: Vec<_> = chunk_results.iter()
            .filter(|r| r.is_err())
            .collect();
        
        if !failed_chunks.is_empty() {
            return Err(DownloadError::ChunkDownloadFailed {
                total_chunks: chunk_results.len(),
                failed_chunks: failed_chunks.len(),
                sample_errors: failed_chunks.into_iter().take(3).cloned().collect(),
            });
        }
        
        // 6. チャンク結合
        self.merge_chunks(&task, &chunk_results?).await
    }
    
    /// 単一チャンクダウンロード
    async fn download_chunk(
        mut chunk: DownloadChunk,
        base_url: String,
        output_path: PathBuf,
        http_client: Arc<HttpClient>,
        progress_monitor: Arc<dyn ProgressMonitor>,
    ) -> Result<DownloadChunk, DownloadError> {
        let start_time = Instant::now();
        
        // 1. チャンク状態更新
        chunk.state = ChunkState::Downloading {
            worker_id: format!("worker_{}", thread_id::get()),
            started_at: chrono::Utc::now(),
        };
        chunk.started_at = Some(chrono::Utc::now());
        
        // 2. HTTP Range リクエスト作成
        let range_header = format!("bytes={}-{}", chunk.start_offset, chunk.end_offset);
        let request = http_client
            .get(&base_url)
            .header("Range", range_header)
            .build()
            .map_err(|e| DownloadError::HttpRequestError(e.to_string()))?;
        
        // 3. レスポンス取得・ストリーミング処理
        let response = http_client.execute(request).await
            .map_err(|e| DownloadError::HttpError(e))?;
        
        if !response.status().is_success() {
            return Err(DownloadError::HttpStatusError {
                status_code: response.status().as_u16(),
                status_text: response.status().to_string(),
            });
        }
        
        // 4. チャンクファイル書き込み
        let chunk_file_path = Self::get_chunk_file_path(&output_path, &chunk.chunk_id);
        let mut chunk_file = tokio::fs::File::create(&chunk_file_path).await
            .map_err(|e| DownloadError::FileCreateError {
                path: chunk_file_path.clone(),
                source: e,
            })?;
        
        let mut stream = response.bytes_stream();
        let mut downloaded_bytes = 0u64;
        let mut hasher = sha2::Sha256::new();
        
        // 5. ストリームデータ処理
        while let Some(bytes_result) = stream.next().await {
            let bytes = bytes_result.map_err(|e| DownloadError::StreamError(e.to_string()))?;
            
            // ファイル書き込み
            chunk_file.write_all(&bytes).await
                .map_err(|e| DownloadError::FileWriteError {
                    path: chunk_file_path.clone(),
                    source: e,
                })?;
            
            // ハッシュ更新
            hasher.update(&bytes);
            
            // 進捗更新
            downloaded_bytes += bytes.len() as u64;
            chunk.downloaded_bytes = downloaded_bytes;
            
            // 進捗通知（間引き）
            if downloaded_bytes % 65536 == 0 { // 64KB ごと
                let progress = DownloadProgress {
                    downloaded_bytes,
                    total_bytes: Some(chunk.chunk_size),
                    percentage: downloaded_bytes as f64 / chunk.chunk_size as f64,
                    current_speed: Self::calculate_instantaneous_speed(downloaded_bytes, start_time.elapsed()),
                    ..Default::default()
                };
                
                progress_monitor.update_progress(&chunk.task_id, progress).await.ok();
            }
        }
        
        // 6. ファイル同期・完了処理
        chunk_file.sync_all().await
            .map_err(|e| DownloadError::FileSyncError {
                path: chunk_file_path.clone(),
                source: e,
            })?;
        
        // 7. チャンク完了状態設定
        let hash = format!("{:x}", hasher.finalize());
        chunk.state = ChunkState::Completed {
            completed_at: chrono::Utc::now(),
            hash: Some(hash),
        };
        chunk.completed_at = Some(chrono::Utc::now());
        
        Ok(chunk)
    }
    
    /// チャンク結合処理
    async fn merge_chunks(
        &self,
        task: &DownloadTask,
        chunk_results: &[Result<DownloadChunk, DownloadError>],
    ) -> Result<DownloadResult, DownloadError> {
        // 1. 結果チャンクソート（オフセット順）
        let mut completed_chunks: Vec<_> = chunk_results.iter()
            .filter_map(|r| r.as_ref().ok())
            .collect();
        completed_chunks.sort_by_key(|chunk| chunk.start_offset);
        
        // 2. 最終ファイル作成
        let mut output_file = tokio::fs::File::create(&task.output_path).await
            .map_err(|e| DownloadError::FileCreateError {
                path: task.output_path.clone(),
                source: e,
            })?;
        
        let mut total_bytes_written = 0u64;
        let mut overall_hasher = sha2::Sha256::new();
        
        // 3. チャンクファイル順次結合
        for chunk in completed_chunks {
            let chunk_file_path = Self::get_chunk_file_path(&task.output_path, &chunk.chunk_id);
            let mut chunk_file = tokio::fs::File::open(&chunk_file_path).await
                .map_err(|e| DownloadError::FileOpenError {
                    path: chunk_file_path.clone(),
                    source: e,
                })?;
            
            // チャンクデータコピー
            let bytes_copied = tokio::io::copy(&mut chunk_file, &mut output_file).await
                .map_err(|e| DownloadError::FileCopyError {
                    source_path: chunk_file_path.clone(),
                    dest_path: task.output_path.clone(),
                    source: e,
                })?;
            
            total_bytes_written += bytes_copied;
            
            // 全体ハッシュ更新（チャンクファイル再読み込み）
            let chunk_data = tokio::fs::read(&chunk_file_path).await
                .map_err(|e| DownloadError::FileReadError {
                    path: chunk_file_path.clone(),
                    source: e,
                })?;
            overall_hasher.update(&chunk_data);
            
            // チャンク一時ファイル削除
            tokio::fs::remove_file(&chunk_file_path).await.ok();
        }
        
        // 4. ファイル整合性検証
        output_file.sync_all().await
            .map_err(|e| DownloadError::FileSyncError {
                path: task.output_path.clone(),
                source: e,
            })?;
        
        let final_hash = format!("{:x}", overall_hasher.finalize());
        
        // 5. 期待ハッシュとの比較（利用可能な場合）
        if let Some(expected_hash) = &task.file_info.expected_hash {
            if expected_hash.value != final_hash {
                return Err(DownloadError::IntegrityVerificationFailed {
                    expected_hash: expected_hash.value.clone(),
                    actual_hash: final_hash,
                    file_path: task.output_path.clone(),
                });
            }
        }
        
        // 6. ダウンロード結果作成
        Ok(DownloadResult {
            task_id: task.task_id.clone(),
            output_path: task.output_path.clone(),
            downloaded_bytes: total_bytes_written,
            file_hash: final_hash,
            download_duration: task.started_at.map(|start| chrono::Utc::now() - start),
            chunk_count: completed_chunks.len() as u32,
            verification_result: VerificationResult::Passed,
        })
    }
}
```

### レジューム機能アルゴリズム

#### 中断からの再開処理
```rust
impl ResumeHandler {
    /// ダウンロード再開処理
    pub async fn resume_download(&self, task: &DownloadTask, resume_info: &ResumeInfo) -> Result<DownloadResult, DownloadError> {
        // 1. 部分ファイル整合性確認
        self.verify_partial_file_integrity(task, resume_info).await?;
        
        // 2. 未完了チャンク特定
        let incomplete_chunks = self.identify_incomplete_chunks(task, resume_info).await?;
        
        // 3. レジューム可能性判定
        if incomplete_chunks.is_empty() {
            return self.finalize_completed_download(task, resume_info).await;
        }
        
        // 4. 未完了チャンクの並列ダウンロード
        let resume_result = self.download_incomplete_chunks(task, incomplete_chunks).await?;
        
        // 5. ファイル再構築・検証
        self.rebuild_and_verify_file(task, resume_result).await
    }
    
    /// 部分ファイル整合性確認
    async fn verify_partial_file_integrity(&self, task: &DownloadTask, resume_info: &ResumeInfo) -> Result<(), DownloadError> {
        let partial_file_path = &resume_info.partial_file_path;
        
        // 1. 部分ファイル存在確認
        if !partial_file_path.exists() {
            return Err(DownloadError::PartialFileNotFound {
                path: partial_file_path.clone(),
            });
        }
        
        // 2. ファイルサイズ確認
        let actual_size = tokio::fs::metadata(partial_file_path).await
            .map_err(|e| DownloadError::FileMetadataError {
                path: partial_file_path.clone(),
                source: e,
            })?.len();
        
        if actual_size != resume_info.existing_size {
            return Err(DownloadError::PartialFileSizeMismatch {
                expected_size: resume_info.existing_size,
                actual_size,
                path: partial_file_path.clone(),
            });
        }
        
        // 3. 整合性情報による検証（利用可能な場合）
        if let Some(integrity_info) = &resume_info.integrity_info {
            self.verify_partial_integrity(partial_file_path, integrity_info).await?;
        }
        
        Ok(())
    }
    
    /// 未完了チャンク特定
    async fn identify_incomplete_chunks(&self, task: &DownloadTask, resume_info: &ResumeInfo) -> Result<Vec<DownloadChunk>, DownloadError> {
        let mut incomplete_chunks = Vec::new();
        
        // 1. 全チャンク再生成
        let all_chunks = self.task_manager.create_chunks(task).await?;
        
        // 2. 完了済みチャンクとの比較
        for chunk in all_chunks {
            match resume_info.chunk_completion.get(&chunk.chunk_id) {
                Some(ChunkState::Completed { .. }) => {
                    // 完了済みチャンクはスキップ
                    continue;
                },
                Some(ChunkState::Failed { .. }) |
                Some(ChunkState::AwaitingRetry { .. }) |
                None => {
                    // 未完了・失敗・未処理チャンクを対象に追加
                    incomplete_chunks.push(chunk);
                },
                _ => {
                    // その他の状態（進行中など）も未完了として扱う
                    incomplete_chunks.push(chunk);
                }
            }
        }
        
        Ok(incomplete_chunks)
    }
    
    /// レジューム情報生成
    pub async fn create_resume_info(&self, task: &DownloadTask) -> Result<ResumeInfo, DownloadError> {
        // 1. 部分ファイルパス決定
        let partial_file_path = Self::get_partial_file_path(&task.output_path);
        
        // 2. 現在のファイルサイズ取得
        let existing_size = if partial_file_path.exists() {
            tokio::fs::metadata(&partial_file_path).await
                .map_err(|e| DownloadError::FileMetadataError {
                    path: partial_file_path.clone(),
                    source: e,
                })?.len()
        } else {
            0
        };
        
        // 3. チャンク完了状況収集
        let chunk_completion = self.collect_chunk_completion(task).await?;
        
        // 4. 部分ファイル整合性情報生成
        let integrity_info = if existing_size > 0 {
            Some(self.generate_partial_integrity_info(&partial_file_path, existing_size).await?)
        } else {
            None
        };
        
        Ok(ResumeInfo {
            partial_file_path,
            existing_size,
            chunk_completion,
            last_resume_at: chrono::Utc::now(),
            resume_count: task.resume_info.as_ref().map(|r| r.resume_count + 1).unwrap_or(1),
            integrity_info,
        })
    }
    
    /// 部分ファイル整合性情報生成
    async fn generate_partial_integrity_info(&self, file_path: &PathBuf, size: u64) -> Result<PartialIntegrityInfo, DownloadError> {
        // 1. ファイルの部分ハッシュ計算
        let file = tokio::fs::File::open(file_path).await
            .map_err(|e| DownloadError::FileOpenError {
                path: file_path.clone(),
                source: e,
            })?;
        
        let mut reader = tokio::io::BufReader::new(file);
        let mut hasher = sha2::Sha256::new();
        let mut buffer = vec![0u8; 65536]; // 64KB バッファ
        let mut total_read = 0u64;
        
        // 2. チャンク単位ハッシュ計算
        let mut chunk_hashes = Vec::new();
        let chunk_size = 1_048_576; // 1MB チャンク
        let mut chunk_buffer = Vec::new();
        
        while total_read < size {
            let bytes_read = reader.read(&mut buffer).await
                .map_err(|e| DownloadError::FileReadError {
                    path: file_path.clone(),
                    source: e,
                })?;
            
            if bytes_read == 0 {
                break;
            }
            
            let data = &buffer[..bytes_read];
            hasher.update(data);
            chunk_buffer.extend_from_slice(data);
            total_read += bytes_read as u64;
            
            // チャンクサイズに達したらハッシュ記録
            if chunk_buffer.len() >= chunk_size {
                let chunk_hash = sha2::Sha256::digest(&chunk_buffer);
                chunk_hashes.push(format!("{:x}", chunk_hash));
                chunk_buffer.clear();
            }
        }
        
        // 3. 残りデータのハッシュ
        if !chunk_buffer.is_empty() {
            let chunk_hash = sha2::Sha256::digest(&chunk_buffer);
            chunk_hashes.push(format!("{:x}", chunk_hash));
        }
        
        let overall_hash = format!("{:x}", hasher.finalize());
        
        Ok(PartialIntegrityInfo {
            overall_hash,
            chunk_hashes,
            file_size: total_read,
            chunk_size,
            generated_at: chrono::Utc::now(),
        })
    }
}
```

## エラー処理設計

### エラー階層構造
```rust
/// ダウンロード実行エラー定義
#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    /// ネットワークエラー
    #[error("Network error during download: {source}")]
    NetworkError {
        #[from]
        source: reqwest::Error,
        retry_count: u32,
        max_retries: u32,
    },
    
    /// HTTP ステータスエラー
    #[error("HTTP error: {status_code} - {status_text}")]
    HttpStatusError {
        status_code: u16,
        status_text: String,
    },
    
    /// ファイル操作エラー
    #[error("File operation error: {operation} on {path}")]
    FileOperationError {
        operation: String,
        path: PathBuf,
        source: std::io::Error,
    },
    
    /// チャンクダウンロードエラー
    #[error("Chunk download failed: {failed_chunks}/{total_chunks} chunks failed")]
    ChunkDownloadFailed {
        total_chunks: usize,
        failed_chunks: usize,
        sample_errors: Vec<Result<DownloadChunk, DownloadError>>,
    },
    
    /// 整合性検証エラー
    #[error("File integrity verification failed: expected {expected_hash}, got {actual_hash}")]
    IntegrityVerificationFailed {
        expected_hash: String,
        actual_hash: String,
        file_path: PathBuf,
    },
    
    /// レジューム機能エラー
    #[error("Resume operation failed: {reason}")]
    ResumeError {
        reason: String,
        task_id: String,
        resume_info: Option<ResumeInfo>,
    },
    
    /// 並行処理エラー
    #[error("Concurrency error: {message}")]
    ConcurrencyError {
        message: String,
    },
    
    /// タスク管理エラー
    #[error("Task management error: {source}")]
    TaskManagementError {
        #[from]
        source: TaskManagerError,
        task_id: Option<String>,
    },
    
    /// 設定エラー
    #[error("Download configuration error: {parameter} - {message}")]
    ConfigurationError {
        parameter: String,
        message: String,
    },
    
    /// リソース不足エラー
    #[error("Insufficient resources: {resource_type} - {details}")]
    InsufficientResourcesError {
        resource_type: String,
        details: String,
        required_amount: Option<u64>,
        available_amount: Option<u64>,
    },
}

/// エラー回復戦略実装
pub struct DownloadErrorRecoveryStrategy {
    /// リトライ設定
    retry_config: RetryConfig,
    
    /// バックオフ戦略
    backoff_strategy: ExponentialBackoff,
    
    /// 回復アクション定義
    recovery_actions: HashMap<DownloadErrorType, RecoveryAction>,
    
    /// エラー統計
    error_statistics: Arc<ErrorStatisticsCollector>,
}

impl DownloadErrorRecoveryStrategy {
    /// エラー種別に基づく自動回復
    pub async fn attempt_recovery(&self, error: &DownloadError, context: &DownloadContext) -> RecoveryResult {
        // エラー統計記録
        self.error_statistics.record_error(error, context).await;
        
        match error {
            DownloadError::NetworkError { retry_count, max_retries, .. } => {
                if *retry_count < *max_retries {
                    let backoff_delay = self.backoff_strategy.calculate_delay(*retry_count);
                    RecoveryResult::RetryAfter {
                        delay: backoff_delay,
                        retry_count: *retry_count + 1,
                        modification: None,
                    }
                } else {
                    RecoveryResult::RequiresUserIntervention
                }
            },
            
            DownloadError::HttpStatusError { status_code, .. } => {
                match *status_code {
                    429 => {
                        // Rate Limit: 長めの待機後リトライ
                        RecoveryResult::RetryAfter {
                            delay: Duration::from_secs(60),
                            retry_count: context.retry_count + 1,
                            modification: Some("Reduced concurrency for rate limiting".to_string()),
                        }
                    },
                    500..=599 => {
                        // Server Error: 短時間後リトライ
                        RecoveryResult::RetryAfter {
                            delay: Duration::from_secs(5),
                            retry_count: context.retry_count + 1,
                            modification: None,
                        }
                    },
                    404 => {
                        // Not Found: 回復不可能
                        RecoveryResult::Unrecoverable
                    },
                    _ => RecoveryResult::RequiresUserIntervention,
                }
            },
            
            DownloadError::ChunkDownloadFailed { failed_chunks, total_chunks, .. } => {
                let failure_rate = *failed_chunks as f64 / *total_chunks as f64;
                
                if failure_rate < 0.3 {
                    // 失敗率30%未満: 失敗チャンクのみリトライ
                    RecoveryResult::PartialRecovery {
                        recovered_items: total_chunks - failed_chunks,
                        failed_items: *failed_chunks,
                        action: "Retry failed chunks only".to_string(),
                    }
                } else {
                    // 失敗率高: 全体リトライ
                    RecoveryResult::RetryWithModification {
                        modification: "Reduced chunk size and concurrency".to_string(),
                        retry_count: context.retry_count + 1,
                    }
                }
            },
            
            DownloadError::IntegrityVerificationFailed { .. } => {
                // 整合性エラー: 再ダウンロード
                RecoveryResult::RetryWithModification {
                    modification: "Full re-download due to integrity failure".to_string(),
                    retry_count: context.retry_count + 1,
                }
            },
            
            DownloadError::InsufficientResourcesError { resource_type, .. } => {
                match resource_type.as_str() {
                    "disk_space" => {
                        RecoveryResult::RequiresUserIntervention
                    },
                    "memory" => {
                        RecoveryResult::RetryWithModification {
                            modification: "Reduced memory usage (smaller buffers)".to_string(),
                            retry_count: context.retry_count + 1,
                        }
                    },
                    "network_bandwidth" => {
                        RecoveryResult::RetryWithModification {
                            modification: "Reduced concurrent downloads".to_string(),
                            retry_count: context.retry_count + 1,
                        }
                    },
                    _ => RecoveryResult::RequiresUserIntervention,
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
    use tempfile::TempDir;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    
    // HTTPサーバーモック
    async fn setup_mock_download_server() -> MockServer {
        let mock_server = MockServer::start().await;
        
        // 正常ダウンロードレスポンス
        Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::path("/test_file.mp4"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_bytes(create_test_file_content(1024 * 1024))) // 1MB
            .mount(&mock_server)
            .await;
        
        // Range リクエスト対応
        Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::header("Range", wiremock::matchers::any()))
            .respond_with(ResponseTemplate::new(206)
                .insert_header("Content-Range", "bytes 0-1023/1024")
                .set_body_bytes(create_test_chunk_content(1024)))
            .mount(&mock_server)
            .await;
        
        mock_server
    }
    
    #[tokio::test]
    async fn test_successful_single_file_download() {
        // Arrange
        let mock_server = setup_mock_download_server().await;
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_download_config();
        
        let download_engine = TokioDownloadEngine::new(config).unwrap();
        
        let task = DownloadTask {
            task_id: "test_task_001".to_string(),
            download_url: format!("{}/test_file.mp4", mock_server.uri()),
            output_path: temp_dir.path().join("downloaded_file.mp4"),
            expected_size: Some(1024 * 1024),
            file_info: create_test_file_info(),
            config: DownloadTaskConfig::default(),
            state: TaskState::Pending,
            progress: DownloadProgress::default(),
            error_history: Vec::new(),
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
            resume_info: None,
        };
        
        // Act
        let result = download_engine.download_file(task.clone()).await;
        
        // Assert
        assert!(result.is_ok());
        let download_result = result.unwrap();
        assert_eq!(download_result.task_id, task.task_id);
        assert_eq!(download_result.downloaded_bytes, 1024 * 1024);
        assert!(download_result.output_path.exists());
        assert_eq!(download_result.verification_result, VerificationResult::Passed);
    }
    
    #[tokio::test]
    async fn test_parallel_chunk_download() {
        // Arrange
        let mock_server = setup_mock_download_server().await;
        let temp_dir = TempDir::new().unwrap();
        let mut config = create_test_download_config();
        config.max_chunk_concurrency = 4;
        
        let download_engine = TokioDownloadEngine::new(config).unwrap();
        
        let large_file_size = 10 * 1024 * 1024; // 10MB
        let task = create_large_download_task(&mock_server, &temp_dir, large_file_size);
        
        // Act
        let result = download_engine.download_file(task).await;
        
        // Assert
        assert!(result.is_ok());
        let download_result = result.unwrap();
        assert_eq!(download_result.downloaded_bytes, large_file_size);
        assert!(download_result.chunk_count > 1);
        
        // ファイル整合性確認
        let downloaded_data = std::fs::read(&download_result.output_path).unwrap();
        assert_eq!(downloaded_data.len(), large_file_size as usize);
    }
    
    #[tokio::test]
    async fn test_download_resume_functionality() {
        // Arrange
        let mock_server = setup_mock_download_server().await;
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_download_config();
        
        let download_engine = TokioDownloadEngine::new(config).unwrap();
        
        let file_size = 5 * 1024 * 1024; // 5MB
        let mut task = create_large_download_task(&mock_server, &temp_dir, file_size);
        
        // 部分ダウンロード情報をシミュレート
        let partial_size = 2 * 1024 * 1024; // 2MB
        let resume_info = create_test_resume_info(&task, partial_size);
        task.resume_info = Some(resume_info);
        
        // 部分ファイル作成
        create_partial_file(&task.output_path, partial_size).await;
        
        // Act
        let result = download_engine.download_file(task).await;
        
        // Assert
        assert!(result.is_ok());
        let download_result = result.unwrap();
        assert_eq!(download_result.downloaded_bytes, file_size);
        
        // レジュームが正しく動作したか確認
        let final_file_size = std::fs::metadata(&download_result.output_path).unwrap().len();
        assert_eq!(final_file_size, file_size);
    }
    
    #[tokio::test]
    async fn test_error_recovery_mechanism() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_download_config();
        
        let download_engine = TokioDownloadEngine::new(config).unwrap();
        
        // 失敗する URL でタスク作成
        let task = DownloadTask {
            task_id: "test_error_task".to_string(),
            download_url: "http://nonexistent.example.com/file.mp4".to_string(),
            output_path: temp_dir.path().join("failed_file.mp4"),
            expected_size: Some(1024),
            file_info: create_test_file_info(),
            config: DownloadTaskConfig {
                max_retries: 3,
                ..Default::default()
            },
            state: TaskState::Pending,
            progress: DownloadProgress::default(),
            error_history: Vec::new(),
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
            resume_info: None,
        };
        
        // Act
        let result = download_engine.download_file(task).await;
        
        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err();
        
        // エラーの種類を確認
        assert!(matches!(error, DownloadError::NetworkError { .. }));
        
        // リトライが実行されたことを確認
        if let DownloadError::NetworkError { retry_count, .. } = error {
            assert!(retry_count > 0);
        }
    }
    
    #[tokio::test]
    async fn test_progress_monitoring_accuracy() {
        // Arrange
        let mock_server = setup_mock_download_server().await;
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_download_config();
        
        let download_engine = TokioDownloadEngine::new(config).unwrap();
        let mut progress_receiver = download_engine.subscribe_progress();
        
        let task = create_test_download_task(&mock_server, &temp_dir, 1024 * 1024);
        
        // Act & Assert (同時実行)
        let download_handle = tokio::spawn(async move {
            download_engine.download_file(task).await
        });
        
        let progress_handle = tokio::spawn(async move {
            let mut progress_updates = Vec::new();
            let mut last_percentage = 0.0;
            
            while let Ok(update) = progress_receiver.recv().await {
                if let ProgressUpdate::TaskProgress { progress, .. } = update {
                    progress_updates.push(progress.clone());
                    
                    // 進捗が単調増加することを確認
                    assert!(progress.percentage >= last_percentage);
                    last_percentage = progress.percentage;
                    
                    // 最後の更新で100%になることを確認
                    if progress.percentage >= 1.0 {
                        break;
                    }
                }
            }
            
            progress_updates
        });
        
        let download_result = download_handle.await.unwrap();
        let progress_updates = progress_handle.await.unwrap();
        
        // ダウンロード成功確認
        assert!(download_result.is_ok());
        
        // 進捗更新が適切に行われたことを確認
        assert!(!progress_updates.is_empty());
        assert_eq!(progress_updates.last().unwrap().percentage, 1.0);
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
        /// チャンク分割アルゴリズムの完全性テスト
        #[test]
        fn test_chunk_division_completeness(
            file_size in 1u64..10_737_418_240u64, // 1B to 10GB
            chunk_size in 1_048_576u64..10_485_760u64, // 1MB to 10MB
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let task_manager = create_test_task_manager().await;
                
                let task = DownloadTask {
                    task_id: "prop_test_task".to_string(),
                    download_url: "http://example.com/file".to_string(),
                    output_path: PathBuf::from("/tmp/test_file"),
                    expected_size: Some(file_size),
                    file_info: create_test_file_info(),
                    config: DownloadTaskConfig {
                        chunk_size: Some(chunk_size),
                        ..Default::default()
                    },
                    state: TaskState::Pending,
                    progress: DownloadProgress::default(),
                    error_history: Vec::new(),
                    created_at: chrono::Utc::now(),
                    started_at: None,
                    completed_at: None,
                    resume_info: None,
                };
                
                let chunks = task_manager.create_chunks(&task).await.unwrap();
                
                // Property: チャンクの総サイズが元ファイルサイズと一致
                let total_chunk_size: u64 = chunks.iter().map(|c| c.chunk_size).sum();
                prop_assert_eq!(total_chunk_size, file_size);
                
                // Property: チャンクが連続している（隙間がない）
                let mut sorted_chunks = chunks.clone();
                sorted_chunks.sort_by_key(|c| c.start_offset);
                
                for i in 1..sorted_chunks.len() {
                    prop_assert_eq!(
                        sorted_chunks[i-1].end_offset + 1,
                        sorted_chunks[i].start_offset
                    );
                }
                
                // Property: 最初のチャンクは0から開始
                if !chunks.is_empty() {
                    prop_assert_eq!(sorted_chunks[0].start_offset, 0);
                    prop_assert_eq!(sorted_chunks.last().unwrap().end_offset, file_size - 1);
                }
                
                // Property: 各チャンクサイズが指定範囲内（最後以外）
                for (i, chunk) in sorted_chunks.iter().enumerate() {
                    if i < sorted_chunks.len() - 1 {
                        prop_assert_eq!(chunk.chunk_size, chunk_size);
                    } else {
                        // 最後のチャンクは chunk_size 以下
                        prop_assert!(chunk.chunk_size <= chunk_size);
                    }
                }
            });
        }
        
        /// 進捗計算の一貫性テスト
        #[test]
        fn test_progress_calculation_consistency(
            downloaded_bytes in 0u64..1_073_741_824u64, // 0B to 1GB
            total_bytes in 1u64..1_073_741_824u64, // 1B to 1GB
        ) {
            prop_assume!(downloaded_bytes <= total_bytes);
            
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let progress_monitor = create_test_progress_monitor().await;
                
                let mut progress = DownloadProgress {
                    downloaded_bytes,
                    total_bytes: Some(total_bytes),
                    percentage: 0.0,
                    current_speed: 0.0,
                    average_speed: 0.0,
                    eta: None,
                    estimated_completion: None,
                    active_chunks: 1,
                    completed_chunks: 0,
                    total_chunks: 1,
                    last_updated: chrono::Utc::now(),
                    speed_history: VecDeque::new(),
                };
                
                progress_monitor.update_progress("test_task", &mut progress).await.unwrap();
                
                // Property: 進捗率は0.0-1.0の範囲内
                prop_assert!(progress.percentage >= 0.0 && progress.percentage <= 1.0);
                
                // Property: 進捗率の計算が正確
                let expected_percentage = downloaded_bytes as f64 / total_bytes as f64;
                prop_assert!((progress.percentage - expected_percentage).abs() < f64::EPSILON);
                
                // Property: 完了時は進捗率100%
                if downloaded_bytes == total_bytes {
                    prop_assert_eq!(progress.percentage, 1.0);
                }
                
                // Property: 未完了時は進捗率100%未満
                if downloaded_bytes < total_bytes {
                    prop_assert!(progress.percentage < 1.0);
                }
            });
        }
        
        /// ファイル整合性検証の信頼性テスト
        #[test]
        fn test_file_integrity_verification_reliability(
            file_content in prop::collection::vec(any::<u8>(), 1..1_048_576), // 1B to 1MB
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let temp_dir = tempfile::TempDir::new().unwrap();
                let file_path = temp_dir.path().join("test_file.bin");
                
                // ファイル書き込み
                tokio::fs::write(&file_path, &file_content).await.unwrap();
                
                // ハッシュ計算
                let expected_hash = sha2::Sha256::digest(&file_content);
                let expected_hash_str = format!("{:x}", expected_hash);
                
                // 整合性検証
                let verifier = create_test_integrity_verifier().await;
                let result = verifier.verify_file_integrity(&file_path, &expected_hash_str).await;
                
                // Property: 正しいハッシュで検証成功
                prop_assert!(result.is_ok());
                prop_assert_eq!(result.unwrap(), VerificationResult::Passed);
                
                // Property: 間違ったハッシュで検証失敗
                let wrong_hash = "0000000000000000000000000000000000000000000000000000000000000000";
                let wrong_result = verifier.verify_file_integrity(&file_path, wrong_hash).await;
                prop_assert!(wrong_result.is_err());
            });
        }
        
        /// レジューム機能の完全性テスト
        #[test]
        fn test_resume_functionality_completeness(
            total_size in 1_048_576u64..10_485_760u64, // 1MB to 10MB
            completed_percentage in 0.1f64..0.9f64, // 10% to 90% completed
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let temp_dir = tempfile::TempDir::new().unwrap();
                let completed_size = (total_size as f64 * completed_percentage) as u64;
                
                let task = DownloadTask {
                    task_id: "resume_test_task".to_string(),
                    download_url: "http://example.com/file".to_string(),
                    output_path: temp_dir.path().join("resume_test_file"),
                    expected_size: Some(total_size),
                    file_info: create_test_file_info(),
                    config: DownloadTaskConfig::default(),
                    state: TaskState::Pending,
                    progress: DownloadProgress::default(),
                    error_history: Vec::new(),
                    created_at: chrono::Utc::now(),
                    started_at: None,
                    completed_at: None,
                    resume_info: None,
                };
                
                let resume_handler = create_test_resume_handler().await;
                
                // 部分的にダウンロード済みのファイルを作成
                let partial_content = vec![0u8; completed_size as usize];
                let partial_file_path = ResumeHandler::get_partial_file_path(&task.output_path);
                tokio::fs::write(&partial_file_path, &partial_content).await.unwrap();
                
                // レジューム情報生成
                let resume_info = resume_handler.create_resume_info(&task).await.unwrap();
                
                // Property: レジューム情報の整合性
                prop_assert_eq!(resume_info.existing_size, completed_size);
                prop_assert_eq!(resume_info.partial_file_path, partial_file_path);
                
                // Property: 未完了チャンクの特定が正確
                let incomplete_chunks = resume_handler.identify_incomplete_chunks(&task, &resume_info).await.unwrap();
                let total_incomplete_size: u64 = incomplete_chunks.iter().map(|c| c.chunk_size).sum();
                let remaining_size = total_size - completed_size;
                
                // 許容誤差範囲内での一致確認（チャンク境界による微調整考慮）
                let size_diff = if total_incomplete_size > remaining_size {
                    total_incomplete_size - remaining_size
                } else {
                    remaining_size - total_incomplete_size
                };
                prop_assert!(size_diff < 1_048_576); // 1MB以内の誤差
            });
        }
    }
    
    /// 任意のダウンロードタスク生成
    fn arb_download_task() -> impl Strategy<Value = DownloadTask> {
        (
            "[a-zA-Z0-9]{10,20}",  // task_id
            "http://[a-zA-Z0-9\\.]+/[a-zA-Z0-9_\\.]+", // download_url
            1u64..1_073_741_824u64, // expected_size (1B to 1GB)
        ).prop_map(|(task_id, download_url, expected_size)| {
            DownloadTask {
                task_id,
                download_url,
                output_path: PathBuf::from(format!("/tmp/{}.tmp", uuid::Uuid::new_v4())),
                expected_size: Some(expected_size),
                file_info: create_test_file_info(),
                config: DownloadTaskConfig::default(),
                state: TaskState::Pending,
                progress: DownloadProgress::default(),
                error_history: Vec::new(),
                created_at: chrono::Utc::now(),
                started_at: None,
                completed_at: None,
                resume_info: None,
            }
        })
    }
}
```

## 性能・セキュリティ考慮事項

### 性能最適化
1. **並列ダウンロード**: CPU・ネットワーク資源を最大限活用する並列度制御
2. **チャンク最適化**: ファイルサイズに応じた動的チャンクサイズ調整
3. **メモリ効率**: ストリーミング処理によるメモリ使用量最小化
4. **進捗バッチング**: 進捗通知の間引きによるUI応答性向上

### セキュリティ強化
1. **ファイル整合性**: SHA-256 ハッシュによる改ざん検出
2. **パス検証**: ディレクトリトラバーサル攻撃の防止
3. **リソース制限**: メモリ・ディスク使用量の制限
4. **エラー情報**: セキュリティに影響する情報の適切な除去

---

**承認**:  
エンジン設計者: [ ] 承認  
性能エンジニア: [ ] 承認  
**承認日**: ___________