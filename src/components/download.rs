//! ダウンロード実行コンポーネント
//!
//! # 責任
//! - ファイルダウンロード実行
//! - 並列処理管理
//! - 進捗監視
//! - エラー回復処理

use crate::errors::{AppError, AppResult};
use crate::components::{ComponentLifecycle, Configurable};
use async_trait::async_trait;
use reqwest::Client;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock, mpsc};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use futures::stream::StreamExt;
use std::time::{Duration, Instant};
use std::collections::{HashMap, VecDeque};
use log;

/// ダウンロード設定
#[derive(Debug, Clone)]
pub struct DownloadConfig {
    /// 同時ダウンロード数
    pub concurrent_downloads: usize,
    /// チャンクサイズ
    pub chunk_size: usize,
    /// タイムアウト
    pub timeout: Duration,
    /// リトライ回数
    pub max_retries: u32,
    /// 出力ディレクトリ
    pub output_directory: PathBuf,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            concurrent_downloads: 3,
            chunk_size: 8192 * 1024, // 8MB
            timeout: Duration::from_secs(300),
            max_retries: 3,
            output_directory: PathBuf::from("downloads"),
        }
    }
}

/// ダウンロードタスク
#[derive(Debug, Clone)]
pub struct DownloadTask {
    /// タスクID
    pub task_id: String,
    /// ダウンロードURL
    pub download_url: String,
    /// 出力ファイルパス
    pub output_path: PathBuf,
    /// 期待ファイルサイズ
    pub expected_size: Option<u64>,
    /// ファイル名
    pub file_name: String,
    /// タスク状態
    pub state: TaskState,
    /// 進捗情報
    pub progress: DownloadProgress,
    /// エラー情報
    pub error: Option<String>,
    /// リトライ数
    pub retry_count: u32,
}

/// タスク状態
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskState {
    /// 待機中
    Pending,
    /// 実行中
    InProgress,
    /// 完了
    Completed,
    /// 失敗
    Failed,
    /// キャンセル
    Cancelled,
}

/// ダウンロード進捗
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    /// ダウンロード済みバイト数
    pub downloaded_bytes: u64,
    /// 総バイト数
    pub total_bytes: Option<u64>,
    /// 進捗率（0.0-1.0）
    pub percentage: f64,
    /// 現在の転送速度（bytes/sec）
    pub current_speed: f64,
    /// 推定残り時間
    pub eta: Option<Duration>,
}

impl DownloadProgress {
    fn new() -> Self {
        Self {
            downloaded_bytes: 0,
            total_bytes: None,
            percentage: 0.0,
            current_speed: 0.0,
            eta: None,
        }
    }
    
    fn update(&mut self, downloaded: u64, total: Option<u64>, speed: f64) {
        self.downloaded_bytes = downloaded;
        self.total_bytes = total;
        self.current_speed = speed;
        
        if let Some(total) = total {
            if total > 0 {
                self.percentage = (downloaded as f64 / total as f64).min(1.0);
                
                if speed > 0.0 {
                    let remaining_bytes = total.saturating_sub(downloaded);
                    let remaining_seconds = remaining_bytes as f64 / speed;
                    self.eta = Some(Duration::from_secs_f64(remaining_seconds));
                }
            }
        }
    }
}

/// 全体進捗情報
#[derive(Debug, Clone)]
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
}

/// ダウンロードイベント
#[derive(Debug, Clone)]
pub enum DownloadEvent {
    /// タスク開始
    TaskStarted { task_id: String },
    /// 進捗更新
    ProgressUpdate { task_id: String, progress: DownloadProgress },
    /// タスク完了
    TaskCompleted { task_id: String, output_path: PathBuf },
    /// タスク失敗
    TaskFailed { task_id: String, error: String },
    /// 全体進捗更新
    OverallProgressUpdate(OverallProgress),
}

/// ダウンロード実行コンポーネント
pub struct DownloadComponent {
    /// 設定
    config: DownloadConfig,
    /// HTTPクライアント
    http_client: Client,
    /// タスクキュー
    task_queue: Arc<Mutex<VecDeque<DownloadTask>>>,
    /// アクティブタスク
    active_tasks: Arc<RwLock<HashMap<String, DownloadTask>>>,
    /// イベント送信チャネル
    event_sender: Option<mpsc::UnboundedSender<DownloadEvent>>,
    /// シャットダウンシグナル
    shutdown_signal: Arc<RwLock<bool>>,
}

impl DownloadComponent {
    /// 新しいダウンロードコンポーネントを作成
    /// 
    /// # 事前条件
    /// - config は有効なダウンロード設定である
    /// 
    /// # 事後条件
    /// - DownloadComponentインスタンスが作成される
    /// - 内部状態が適切に初期化される
    pub fn new(config: DownloadConfig) -> Self {
        let http_client = Client::builder()
            .timeout(config.timeout)
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            config,
            http_client,
            task_queue: Arc::new(Mutex::new(VecDeque::new())),
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
            event_sender: None,
            shutdown_signal: Arc::new(RwLock::new(false)),
        }
    }
    
    /// イベントリスナーを設定
    /// 
    /// # 副作用
    /// - 内部の event_sender が更新される
    /// 
    /// # 事前条件
    /// - sender は有効な UnboundedSender である
    /// 
    /// # 事後条件
    /// - イベント通知が有効になる
    /// - 以降のダウンロード進捗がイベントとして送信される
    /// 
    /// # 不変条件
    /// - コンポーネントの他の設定は変更されない
    pub fn set_event_listener(&mut self, sender: mpsc::UnboundedSender<DownloadEvent>) {
        self.event_sender = Some(sender);
    }
    
    /// ダウンロードタスクを追加
    /// 
    /// # 副作用
    /// - タスクキューへの追加
    /// 
    /// # 事前条件
    /// - task_id は一意である
    /// - download_url は有効なURLである
    /// - file_name は空でない
    /// 
    /// # 事後条件
    /// - タスクがキューに追加される
    pub async fn add_download_task(
        &self,
        task_id: String,
        download_url: String,
        file_name: String,
        expected_size: Option<u64>,
    ) -> AppResult<()> {
        assert!(!task_id.is_empty(), "task_id must not be empty");
        assert!(!download_url.is_empty(), "download_url must not be empty");
        assert!(!file_name.is_empty(), "file_name must not be empty");
        
        // 出力パスの構築
        let output_path = self.config.output_directory.join(&file_name);
        
        let task = DownloadTask {
            task_id: task_id.clone(),
            download_url,
            output_path,
            expected_size,
            file_name,
            state: TaskState::Pending,
            progress: DownloadProgress::new(),
            error: None,
            retry_count: 0,
        };
        
        let mut queue = self.task_queue.lock().await;
        queue.push_back(task);
        
        log::info!("Download task added: {}", task_id);
        Ok(())
    }
    
    /// ダウンロード処理を開始
    /// 
    /// # 副作用
    /// - 非同期ワーカーの起動
    /// - HTTPリクエストの送信
    /// - ファイルの書き込み
    /// 
    /// # 事前条件
    /// - コンポーネントが初期化されている
    /// 
    /// # 事後条件
    /// - ダウンロードワーカーが起動される
    pub async fn start_downloads(&self) -> AppResult<()> {
        let concurrent_downloads = self.config.concurrent_downloads;
        
        for worker_id in 0..concurrent_downloads {
            let task_queue = self.task_queue.clone();
            let active_tasks = self.active_tasks.clone();
            let http_client = self.http_client.clone();
            let event_sender = self.event_sender.clone();
            let shutdown_signal = self.shutdown_signal.clone();
            let config = self.config.clone();
            
            tokio::spawn(async move {
                Self::download_worker(
                    worker_id,
                    task_queue,
                    active_tasks,
                    http_client,
                    event_sender,
                    shutdown_signal,
                    config,
                )
                .await;
            });
        }
        
        log::info!("Started {} download workers", concurrent_downloads);
        Ok(())
    }
    
    /// ダウンロードワーカー
    async fn download_worker(
        worker_id: usize,
        task_queue: Arc<Mutex<VecDeque<DownloadTask>>>,
        active_tasks: Arc<RwLock<HashMap<String, DownloadTask>>>,
        http_client: Client,
        event_sender: Option<mpsc::UnboundedSender<DownloadEvent>>,
        shutdown_signal: Arc<RwLock<bool>>,
        config: DownloadConfig,
    ) {
        log::info!("Download worker {} started", worker_id);
        
        loop {
            // シャットダウンチェック
            if *shutdown_signal.read().await {
                log::info!("Download worker {} shutting down", worker_id);
                break;
            }
            
            // タスク取得
            let task = {
                let mut queue = task_queue.lock().await;
                queue.pop_front()
            };
            
            match task {
                Some(mut task) => {
                    let task_id = task.task_id.clone();
                    
                    // タスク開始イベント
                    if let Some(sender) = &event_sender {
                        let _ = sender.send(DownloadEvent::TaskStarted { task_id: task_id.clone() });
                    }
                    
                    // アクティブタスクに追加
                    task.state = TaskState::InProgress;
                    active_tasks.write().await.insert(task_id.clone(), task.clone());
                    
                    // ダウンロード実行
                    let result = Self::download_file(
                        &http_client,
                        &mut task,
                        &event_sender,
                        &config,
                    )
                    .await;
                    
                    // 結果処理
                    match result {
                        Ok(_) => {
                            task.state = TaskState::Completed;
                            if let Some(sender) = &event_sender {
                                let _ = sender.send(DownloadEvent::TaskCompleted {
                                    task_id: task_id.clone(),
                                    output_path: task.output_path.clone(),
                                });
                            }
                            log::info!("Download completed: {}", task_id);
                        }
                        Err(e) => {
                            task.error = Some(e.to_string());
                            task.retry_count += 1;
                            
                            if task.retry_count < config.max_retries {
                                // リトライ
                                task.state = TaskState::Pending;
                                task_queue.lock().await.push_back(task.clone());
                                log::warn!("Download failed, retrying: {} (attempt {})", task_id, task.retry_count + 1);
                            } else {
                                // 最終的に失敗
                                task.state = TaskState::Failed;
                                if let Some(sender) = &event_sender {
                                    let _ = sender.send(DownloadEvent::TaskFailed {
                                        task_id: task_id.clone(),
                                        error: e.to_string(),
                                    });
                                }
                                log::error!("Download failed after {} retries: {}", config.max_retries, task_id);
                            }
                        }
                    }
                    
                    // アクティブタスクから削除
                    active_tasks.write().await.remove(&task_id);
                    
                    // 全体進捗更新
                    Self::update_overall_progress(&active_tasks, &event_sender).await;
                }
                None => {
                    // タスクがない場合は少し待機
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }
    
    /// ファイルをダウンロード
    async fn download_file(
        http_client: &Client,
        task: &mut DownloadTask,
        event_sender: &Option<mpsc::UnboundedSender<DownloadEvent>>,
        _config: &DownloadConfig,
    ) -> AppResult<()> {
        // 出力ディレクトリの作成
        if let Some(parent) = task.output_path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| AppError::io("Failed to create output directory", Some(e)))?;
        }
        
        // HTTPリクエスト
        let response = http_client
            .get(&task.download_url)
            .send()
            .await
            .map_err(|e| AppError::network("Failed to send download request", Some(e)))?;
        
        // ステータスチェック
        if !response.status().is_success() {
            return Err(AppError::external_service(format!(
                "Download failed with status: {}",
                response.status()
            )));
        }
        
        // ファイルサイズ取得
        let total_size = response
            .headers()
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|ct_len| ct_len.to_str().ok())
            .and_then(|ct_len| ct_len.parse::<u64>().ok());
        
        task.progress.total_bytes = total_size;
        
        // ファイル作成
        let mut file = File::create(&task.output_path).await
            .map_err(|e| AppError::io("Failed to create output file", Some(e)))?;
        
        // ストリームダウンロード
        let mut stream = response.bytes_stream();
        let mut downloaded = 0u64;
        let start_time = Instant::now();
        let mut last_update_time = start_time;
        
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result
                .map_err(|e| AppError::network("Failed to download chunk", Some(e)))?;
            
            file.write_all(&chunk).await
                .map_err(|e| AppError::io("Failed to write chunk", Some(e)))?;
            
            downloaded += chunk.len() as u64;
            
            // 進捗更新（100msごと）
            let now = Instant::now();
            if now.duration_since(last_update_time) > Duration::from_millis(100) {
                let elapsed = now.duration_since(start_time).as_secs_f64();
                let speed = if elapsed > 0.0 {
                    downloaded as f64 / elapsed
                } else {
                    0.0
                };
                
                task.progress.update(downloaded, total_size, speed);
                
                if let Some(sender) = event_sender {
                    let _ = sender.send(DownloadEvent::ProgressUpdate {
                        task_id: task.task_id.clone(),
                        progress: task.progress.clone(),
                    });
                }
                
                last_update_time = now;
            }
        }
        
        // ファイルをフラッシュ
        file.flush().await
            .map_err(|e| AppError::io("Failed to flush file", Some(e)))?;
        
        // 最終進捗更新
        task.progress.downloaded_bytes = downloaded;
        task.progress.percentage = 1.0;
        
        // サイズ検証
        if let Some(expected_size) = task.expected_size {
            if downloaded != expected_size {
                return Err(AppError::data_integrity(format!(
                    "File size mismatch: expected {}, got {}",
                    expected_size, downloaded
                )));
            }
        }
        
        Ok(())
    }
    
    /// 全体進捗を更新
    async fn update_overall_progress(
        _active_tasks: &Arc<RwLock<HashMap<String, DownloadTask>>>,
        _event_sender: &Option<mpsc::UnboundedSender<DownloadEvent>>,
    ) {
        // TODO: 全体進捗の計算・通知
    }
    
    /// ダウンロードを停止
    /// 
    /// # 副作用
    /// - shutdown_signal が true に設定される
    /// - ログ出力が実行される
    /// - 実行中のダウンロードワーカーが停止される
    /// 
    /// # 事前条件
    /// - コンポーネントが初期化されている
    /// 
    /// # 事後条件
    /// - 成功時: ダウンロードが停止される
    /// - 新しいダウンロードが開始されなくなる
    /// - 実行中のダウンロードは完了まで継続される
    /// 
    /// # 不変条件
    /// - 既存のタスクキューは保持される
    pub async fn stop_downloads(&self) -> AppResult<()> {
        *self.shutdown_signal.write().await = true;
        log::info!("Download component shutdown initiated");
        Ok(())
    }
}

#[async_trait]
impl ComponentLifecycle for DownloadComponent {
    async fn initialize(&mut self) -> AppResult<()> {
        log::info!("Initializing DownloadComponent");
        
        // 出力ディレクトリの作成
        tokio::fs::create_dir_all(&self.config.output_directory).await
            .map_err(|e| AppError::io("Failed to create output directory", Some(e)))?;
        
        log::info!("DownloadComponent initialized successfully");
        Ok(())
    }
    
    async fn shutdown(&mut self) -> AppResult<()> {
        log::info!("Shutting down DownloadComponent");
        
        // ダウンロード停止
        self.stop_downloads().await?;
        
        // アクティブタスクのログ
        let active_tasks = self.active_tasks.read().await;
        if !active_tasks.is_empty() {
            log::warn!("Shutting down with {} active downloads", active_tasks.len());
        }
        
        log::info!("DownloadComponent shut down successfully");
        Ok(())
    }
    
    async fn health_check(&self) -> bool {
        // シャットダウン中でないことを確認
        !*self.shutdown_signal.read().await
    }
}

impl Configurable<DownloadConfig> for DownloadComponent {
    fn update_config(&mut self, config: DownloadConfig) -> AppResult<()> {
        self.config = config;
        
        // HTTPクライアントの再構築
        self.http_client = Client::builder()
            .timeout(self.config.timeout)
            .build()
            .map_err(|e| AppError::configuration("Failed to create HTTP client", Some(e)))?;
        
        log::info!("DownloadComponent configuration updated");
        Ok(())
    }
    
    fn get_config(&self) -> &DownloadConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_download_progress_calculation() {
        let mut progress = DownloadProgress::new();
        
        // 初期状態テスト
        assert_eq!(progress.downloaded_bytes, 0);
        assert_eq!(progress.percentage, 0.0);
        assert!(progress.eta.is_none());
        
        // 進捗更新テスト（50%完了）
        progress.update(50, Some(100), 1000.0);
        assert_eq!(progress.percentage, 0.5);
        assert_eq!(progress.downloaded_bytes, 50);
        assert_eq!(progress.current_speed, 1000.0);
        
        // ETA計算テスト（残り50バイト、速度1000.0 = 0.05秒）
        if let Some(eta) = progress.eta {
            // 残り時間は0.05秒なので、ミリ秒単位で確認
            assert!(eta.as_millis() >= 50); // 少なくとも50ms
        }
        
        // 完了状態テスト
        progress.update(100, Some(100), 1000.0);
        assert_eq!(progress.percentage, 1.0);
        assert_eq!(progress.downloaded_bytes, 100);
        
        // 完了時はETAが0になる
        if let Some(eta) = progress.eta {
            assert_eq!(eta.as_secs(), 0);
        }
    }
    
    #[tokio::test]
    async fn test_download_component_lifecycle() {
        let config = DownloadConfig::default();
        let mut component = DownloadComponent::new(config);
        
        // 初期化テスト
        assert!(component.initialize().await.is_ok());
        assert!(component.health_check().await);
        
        // 終了処理テスト
        assert!(component.shutdown().await.is_ok());
        assert!(!component.health_check().await);
    }
}