# Tokio非同期処理ポリシー - Zoom Video Mover

**技術要素**: tokio 1.0+, async/await  
**適用範囲**: 全非同期処理（HTTP通信、ファイルI/O、UI更新）

## Tokio非同期処理原則

### 基本原則
- **ノンブロッキング**: UIスレッドのブロッキング回避
- **効率的リソース使用**: 最小限のスレッド数で最大の並行性
- **エラー伝播**: 適切な非同期エラーハンドリング
- **キャンセル対応**: 非同期タスクの安全なキャンセル
- **バックプレッシャー**: 負荷制御による安定動作

### アーキテクチャ方針
- **Single-threaded UI**: GUI処理は単一スレッドで実行
- **Multi-threaded Workers**: バックグラウンド処理は並列実行
- **Channel通信**: スレッド間の安全なメッセージパッシング
- **Structured Concurrency**: 非同期タスクの構造化された管理

## Tokioランタイム設定

### メインランタイム設定
```rust
// main.rs
use tokio::runtime::Runtime;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // マルチスレッドランタイム設定
    let rt = Runtime::new()?;
    
    rt.block_on(async {
        run_application().await
    })
}

// 推奨設定
#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_application().await
}
```

### ランタイム設定詳細
```rust
use tokio::runtime::Builder;

/// カスタムランタイム構築
/// 
/// # 事前条件
/// - worker_threads > 0
/// - thread_name が有効な文字列
/// 
/// # 事後条件
/// - 設定されたスレッド数でランタイム作成
/// - スレッド名が適切に設定
pub fn build_custom_runtime(worker_threads: usize) -> Result<Runtime, std::io::Error> {
    assert!(worker_threads > 0, "worker_threads must be positive");
    assert!(worker_threads <= 16, "worker_threads should be reasonable");
    
    Builder::new_multi_thread()
        .worker_threads(worker_threads)
        .thread_name("zoom-downloader-worker")
        .thread_stack_size(3 * 1024 * 1024)  // 3MB stack
        .enable_all()
        .build()
}
```

## 非同期関数設計

### 基本パターン
```rust
/// 非同期HTTP通信の例
/// 
/// # 副作用
/// - HTTP通信の実行
/// - ネットワークリソースの使用
/// 
/// # 事前条件
/// - client が初期化済み
/// - url が有効なHTTPS URL
/// - timeout > 0
/// 
/// # 事後条件
/// - 成功時: 有効なレスポンスデータを返す
/// - 失敗時: 適切なエラーを返す
/// - リソースは確実に解放される
/// 
/// # 不変条件
/// - 実行中にclientの設定は変更されない
/// - timeoutは指定値を超えない
pub async fn fetch_with_retry(
    client: &reqwest::Client,
    url: &str,
    max_retries: u32,
    timeout: Duration,
) -> Result<String, NetworkError> {
    // 事前条件チェック
    assert!(!url.is_empty(), "URL must not be empty");
    assert!(url.starts_with("https://"), "URL must use HTTPS");
    assert!(timeout > Duration::ZERO, "timeout must be positive");
    debug_assert!(max_retries <= 5, "max_retries should be reasonable");
    
    for attempt in 0..=max_retries {
        match timeout_request(client, url, timeout).await {
            Ok(response) => {
                debug_assert!(!response.is_empty(), "response should not be empty");
                return Ok(response);
            }
            Err(e) if attempt < max_retries && is_retryable(&e) => {
                let delay = Duration::from_millis(100 * 2_u64.pow(attempt));
                tokio::time::sleep(delay).await;
                continue;
            }
            Err(e) => return Err(e),
        }
    }
    
    unreachable!("loop should always return");
}

/// タイムアウト付きリクエスト
async fn timeout_request(
    client: &reqwest::Client,
    url: &str,
    timeout: Duration,
) -> Result<String, NetworkError> {
    tokio::time::timeout(timeout, async {
        let response = client.get(url).send().await?;
        let text = response.text().await?;
        Ok(text)
    })
    .await
    .map_err(|_| NetworkError::Timeout)?
}
```

### エラーハンドリング
```rust
/// 非同期エラーの適切なハンドリング
pub async fn download_file_safely(
    url: &str,
    output_path: &Path,
) -> Result<(), DownloadError> {
    // リソース管理をasyncブロックで確実に行う
    let _guard = DownloadGuard::new(output_path)?;
    
    // 並列処理でのエラー伝播
    let result = tokio::try_join!(
        fetch_file_metadata(url),
        create_output_directory(output_path.parent().unwrap()),
        verify_permissions(output_path)
    );
    
    match result {
        Ok((metadata, (), ())) => {
            execute_download(url, output_path, metadata).await
        }
        Err(e) => {
            // 部分的成功でもクリーンアップ
            cleanup_partial_download(output_path).await?;
            Err(e)
        }
    }
}
```

## Channel通信パターン

### Producer-Consumerパターン
```rust
use tokio::sync::mpsc;

/// ダウンロード進捗通知システム
/// 
/// # 事前条件
/// - buffer_size > 0
/// 
/// # 事後条件
/// - 送信者・受信者チャンネルを返す
/// - バックグラウンドタスクが起動される
pub fn setup_progress_channels(buffer_size: usize) -> (
    mpsc::Sender<DownloadProgress>,
    mpsc::Receiver<ProgressEvent>
) {
    assert!(buffer_size > 0, "buffer_size must be positive");
    debug_assert!(buffer_size <= 10000, "buffer_size should be reasonable");
    
    let (progress_tx, progress_rx) = mpsc::channel(buffer_size);
    let (event_tx, event_rx) = mpsc::channel(buffer_size);
    
    // 進捗処理タスク
    tokio::spawn(async move {
        process_progress_updates(progress_rx, event_tx).await;
    });
    
    (progress_tx, event_rx)
}

/// 進捗更新処理
async fn process_progress_updates(
    mut progress_rx: mpsc::Receiver<DownloadProgress>,
    event_tx: mpsc::Sender<ProgressEvent>,
) {
    while let Some(progress) = progress_rx.recv().await {
        let event = ProgressEvent {
            percentage: progress.percentage(),
            estimated_remaining: progress.estimated_remaining(),
            current_speed: progress.current_speed(),
            timestamp: chrono::Utc::now(),
        };
        
        // 送信失敗は受信者終了を意味するので正常終了
        if event_tx.send(event).await.is_err() {
            break;
        }
    }
}
```

### Broadcast通知パターン
```rust
use tokio::sync::broadcast;

/// システム全体の状態変更通知
pub struct StateNotifier {
    sender: broadcast::Sender<StateChange>,
}

impl StateNotifier {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }
    
    /// 状態変更の通知
    /// 
    /// # 事前条件
    /// - change は有効な状態変更
    /// 
    /// # 事後条件
    /// - 全購読者に通知送信
    /// - 購読者なしでもエラーにならない
    pub async fn notify_state_change(&self, change: StateChange) -> Result<(), NotificationError> {
        // 購読者がいない場合はOK（RecvError::Laggedは送信成功）
        match self.sender.send(change) {
            Ok(_) => Ok(()),
            Err(broadcast::error::SendError(_)) => Ok(()), // 購読者なし
        }
    }
    
    /// 新しい購読者作成
    pub fn subscribe(&self) -> broadcast::Receiver<StateChange> {
        self.sender.subscribe()
    }
}
```

## 並行処理パターン

### 並列タスク実行
```rust
/// 並列ダウンロード処理
/// 
/// # 副作用
/// - 複数ファイルの同時ダウンロード
/// - ネットワーク・ディスクリソースの使用
/// 
/// # 事前条件
/// - tasks が空でない
/// - 全taskのURLが有効
/// - concurrent_limit > 0 かつ <= tasks.len()
/// 
/// # 事後条件
/// - 成功時: 全ファイルがダウンロード完了
/// - 部分失敗時: 成功分は保持、失敗分はエラー報告
/// 
/// # 不変条件
/// - 同時実行数は concurrent_limit を超えない
/// - 各タスクは独立して実行される
pub async fn execute_parallel_downloads(
    tasks: Vec<DownloadTask>,
    concurrent_limit: usize,
) -> Result<Vec<DownloadResult>, DownloadError> {
    // 事前条件チェック
    assert!(!tasks.is_empty(), "tasks must not be empty");
    assert!(concurrent_limit > 0, "concurrent_limit must be positive");
    assert!(concurrent_limit <= tasks.len(), "concurrent_limit should not exceed task count");
    
    use tokio::sync::Semaphore;
    
    // 並行数制御用セマフォ
    let semaphore = Arc::new(Semaphore::new(concurrent_limit));
    let mut handles = Vec::with_capacity(tasks.len());
    
    for task in tasks {
        let semaphore = semaphore.clone();
        let handle = tokio::spawn(async move {
            // セマフォ取得（並行数制御）
            let _permit = semaphore.acquire().await.unwrap();
            execute_single_download(task).await
        });
        handles.push(handle);
    }
    
    // 全タスクの完了を待機
    let results = futures::future::try_join_all(handles).await
        .map_err(|e| DownloadError::TaskJoinError(e))?;
    
    // 事後条件チェック
    debug_assert_eq!(results.len(), handles.len(), "result count should match task count");
    
    Ok(results)
}
```

### Select操作パターン
```rust
/// 複数の非同期操作から最初の完了を待機
pub async fn wait_for_first_success(
    urls: Vec<String>,
    timeout: Duration,
) -> Result<String, MultiSourceError> {
    assert!(!urls.is_empty(), "urls must not be empty");
    
    let futures: Vec<_> = urls.into_iter()
        .map(|url| Box::pin(fetch_with_timeout(url, timeout)))
        .collect();
    
    // 最初の成功を待機
    match futures::future::select_ok(futures).await {
        Ok((result, _remaining)) => Ok(result),
        Err(errors) => Err(MultiSourceError::AllFailed(errors)),
    }
}

/// タイムアウト・キャンセル対応
pub async fn cancellable_operation(
    operation: impl Future<Output = Result<String, OperationError>>,
    cancel_token: CancellationToken,
) -> Result<String, OperationError> {
    tokio::select! {
        result = operation => result,
        _ = cancel_token.cancelled() => Err(OperationError::Cancelled),
    }
}
```

## タスク管理・キャンセル

### 構造化並行性
```rust
use tokio_util::sync::CancellationToken;

/// 構造化されたタスク管理
pub struct TaskManager {
    cancel_token: CancellationToken,
    tasks: Arc<Mutex<Vec<JoinHandle<()>>>>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            cancel_token: CancellationToken::new(),
            tasks: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// タスク起動と管理への追加
    pub fn spawn_managed<F, T>(&self, future: F) -> JoinHandle<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let cancel_token = self.cancel_token.clone();
        let handle = tokio::spawn(async move {
            tokio::select! {
                result = future => result,
                _ = cancel_token.cancelled() => {
                    // キャンセル時のクリーンアップ
                    return Default::default(); // または適切なキャンセル値
                }
            }
        });
        
        self.tasks.lock().unwrap().push(handle.clone());
        handle
    }
    
    /// 全タスクのキャンセル
    pub async fn cancel_all(&self) {
        self.cancel_token.cancel();
        
        let handles = {
            let mut tasks = self.tasks.lock().unwrap();
            std::mem::take(&mut *tasks)
        };
        
        // 全タスクの完了を待機
        for handle in handles {
            let _ = handle.await;
        }
    }
}

impl Drop for TaskManager {
    fn drop(&mut self) {
        self.cancel_token.cancel();
    }
}
```

## パフォーマンス最適化

### 非同期ストリーミング
```rust
use tokio::io::{AsyncRead, AsyncWrite};
use futures::stream::StreamExt;

/// 大容量ファイルのストリーミングダウンロード
/// 
/// # 副作用
/// - ファイルI/O操作
/// - ネットワーク通信
/// 
/// # 事前条件
/// - reader が有効なストリーム
/// - writer が書き込み可能
/// - buffer_size > 0
/// 
/// # 事後条件
/// - 全データが writer に書き込まれる
/// - 進捗が継続的に報告される
pub async fn stream_large_file<R, W>(
    mut reader: R,
    mut writer: W,
    buffer_size: usize,
    progress_tx: Option<mpsc::Sender<u64>>,
) -> Result<u64, StreamError>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    assert!(buffer_size > 0, "buffer_size must be positive");
    debug_assert!(buffer_size <= 1024 * 1024, "buffer_size should be reasonable (<=1MB)");
    
    use tokio::io::{copy_buf, BufReader, BufWriter};
    
    let mut buf_reader = BufReader::with_capacity(buffer_size, reader);
    let mut buf_writer = BufWriter::with_capacity(buffer_size, writer);
    
    let mut total_copied = 0u64;
    
    // 進捗報告付きコピー
    loop {
        let bytes_copied = copy_buf(&mut buf_reader, &mut buf_writer).await?;
        if bytes_copied == 0 {
            break;
        }
        
        total_copied += bytes_copied;
        
        // 進捗報告（ノンブロッキング）
        if let Some(ref tx) = progress_tx {
            let _ = tx.try_send(total_copied);
        }
    }
    
    buf_writer.flush().await?;
    
    debug_assert!(total_copied > 0, "should have copied some data");
    Ok(total_copied)
}
```

### 並行数制御
```rust
/// 適応的並行数制御
pub struct AdaptiveConcurrencyControl {
    current_limit: Arc<AtomicUsize>,
    success_rate: Arc<AtomicUsize>, // パーセンテージ（0-100）
    adjustment_interval: Duration,
}

impl AdaptiveConcurrencyControl {
    pub fn new(initial_limit: usize) -> Self {
        Self {
            current_limit: Arc::new(AtomicUsize::new(initial_limit)),
            success_rate: Arc::new(AtomicUsize::new(100)),
            adjustment_interval: Duration::from_secs(30),
        }
    }
    
    /// 現在の並行数制限取得
    pub fn current_limit(&self) -> usize {
        self.current_limit.load(Ordering::Acquire)
    }
    
    /// 成功率に基づく並行数調整
    pub async fn adjust_based_on_performance(&self) {
        let mut interval = tokio::time::interval(self.adjustment_interval);
        
        loop {
            interval.tick().await;
            
            let success_rate = self.success_rate.load(Ordering::Acquire);
            let current = self.current_limit.load(Ordering::Acquire);
            
            let new_limit = if success_rate < 80 {
                // 成功率低下時は並行数を減らす
                (current * 80 / 100).max(1)
            } else if success_rate > 95 {
                // 高成功率時は並行数を増やす
                (current * 110 / 100).min(20)
            } else {
                current // 現状維持
            };
            
            if new_limit != current {
                self.current_limit.store(new_limit, Ordering::Release);
                log::info!("Adjusted concurrency limit: {} -> {} (success rate: {}%)", 
                          current, new_limit, success_rate);
            }
        }
    }
}
```

## 監視・デバッグ

### 非同期タスク監視
```rust
/// タスクメトリクス収集
#[derive(Debug, Clone)]
pub struct TaskMetrics {
    pub total_spawned: Arc<AtomicU64>,
    pub currently_running: Arc<AtomicU64>,
    pub completed: Arc<AtomicU64>,
    pub failed: Arc<AtomicU64>,
}

impl TaskMetrics {
    pub fn new() -> Self {
        Self {
            total_spawned: Arc::new(AtomicU64::new(0)),
            currently_running: Arc::new(AtomicU64::new(0)),
            completed: Arc::new(AtomicU64::new(0)),
            failed: Arc::new(AtomicU64::new(0)),
        }
    }
    
    /// 監視対象タスクとして起動
    pub fn spawn_monitored<F, T>(&self, future: F) -> JoinHandle<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let metrics = self.clone();
        
        metrics.total_spawned.fetch_add(1, Ordering::Relaxed);
        metrics.currently_running.fetch_add(1, Ordering::Relaxed);
        
        tokio::spawn(async move {
            let result = future.await;
            
            metrics.currently_running.fetch_sub(1, Ordering::Relaxed);
            metrics.completed.fetch_add(1, Ordering::Relaxed);
            
            result
        })
    }
    
    /// メトリクス情報取得
    pub fn get_summary(&self) -> TaskSummary {
        TaskSummary {
            total_spawned: self.total_spawned.load(Ordering::Relaxed),
            currently_running: self.currently_running.load(Ordering::Relaxed),
            completed: self.completed.load(Ordering::Relaxed),
            failed: self.failed.load(Ordering::Relaxed),
        }
    }
}
```

## エラー処理ベストプラクティス

### 非同期エラーの伝播
```rust
/// 非同期処理での適切なエラー伝播
#[derive(Debug, thiserror::Error)]
pub enum AsyncOperationError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Timeout error: operation took longer than {timeout:?}")]
    Timeout { timeout: Duration },
    
    #[error("Cancellation error: operation was cancelled")]
    Cancelled,
    
    #[error("Task join error: {0}")]
    TaskJoin(#[from] tokio::task::JoinError),
}

impl AsyncOperationError {
    /// エラーがリトライ可能かどうか判定
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Network(e) => e.is_timeout() || e.is_connect(),
            Self::Io(e) => matches!(e.kind(), std::io::ErrorKind::TimedOut | std::io::ErrorKind::Interrupted),
            Self::Timeout { .. } => true,
            Self::Cancelled => false,
            Self::TaskJoin(_) => false,
        }
    }
}
```

## 品質目標

- **並行処理バグ**: 0件
- **デッドロック**: 0件  
- **リソースリーク**: 0件
- **非同期タスク完了率**: 99%以上
- **レスポンス時間**: UI操作16ms以内
- **メモリ使用量**: 1GB以内

効率的で安全な非同期処理により、responsive なユーザー体験を実現します。