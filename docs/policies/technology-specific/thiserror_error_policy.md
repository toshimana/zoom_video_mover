# thiserror エラーハンドリングポリシー - Zoom Video Mover

**技術要素**: thiserror 1.0+, anyhow  
**適用範囲**: 全エラー型定義・エラーハンドリング実装

## thiserror エラー設計原則

### 基本原則
- **型安全性**: エラー種別の明確な型分離
- **情報保持**: エラー原因・文脈の完全保持
- **ユーザビリティ**: 理解しやすいエラーメッセージ
- **トレーサビリティ**: エラー伝播パスの追跡可能性
- **回復可能性**: エラー種別による適切な回復戦略

### 設計方針
- **階層化**: アプリケーション層・ドメイン層・インフラ層での分離
- **変換可能性**: 下位層エラーから上位層エラーへの適切な変換
- **国際化対応**: 多言語エラーメッセージサポート
- **ログ統合**: 構造化ログとの連携

## エラー型階層設計

### アプリケーションレベルエラー
```rust
/// Zoom Video Mover アプリケーション全体エラー
/// 
/// # 事前条件
/// - 各バリアントは適切な context を持つ
/// 
/// # 事後条件
/// - エラー情報の完全保持
/// - 適切なユーザーメッセージ生成
/// 
/// # 不変条件
/// - エラーチェーンの整合性維持
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("認証エラー: {message}")]
    Authentication { 
        message: String,
        #[source]
        source: Option<AuthError>,
    },
    
    #[error("ダウンロードエラー: {operation}が失敗しました")]
    Download { 
        operation: String,
        #[source]
        source: DownloadError,
    },
    
    #[error("設定エラー: {config_type}の読み込みに失敗")]
    Configuration { 
        config_type: String,
        #[source]
        source: ConfigError,
    },
    
    #[error("システムエラー: {context}")]
    System { 
        context: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    #[error("GUI エラー: {component}で問題が発生")]
    Gui { 
        component: String,
        details: String,
    },
}

impl AppError {
    /// ユーザーフレンドリーなメッセージ生成
    /// 
    /// # 事前条件
    /// - self が有効なエラー
    /// 
    /// # 事後条件
    /// - 技術詳細を隠したユーザー向けメッセージを返す
    pub fn user_message(&self) -> String {
        match self {
            Self::Authentication { message, .. } => {
                format!("認証に失敗しました: {}", message)
            }
            Self::Download { operation, .. } => {
                format!("{}の処理中にエラーが発生しました。\n再試行してください。", operation)
            }
            Self::Configuration { config_type, .. } => {
                format!("{}の設定に問題があります。\n設定を確認してください。", config_type)
            }
            Self::System { context, .. } => {
                format!("システムエラーが発生しました: {}\nサポートにお問い合わせください。", context)
            }
            Self::Gui { component, details } => {
                format!("画面操作でエラーが発生しました: {} ({})", component, details)
            }
        }
    }
    
    /// エラーの回復可能性判定
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::Authentication { .. } => true,  // 再認証可能
            Self::Download { .. } => true,        // 再ダウンロード可能
            Self::Configuration { .. } => true,   // 設定修正可能
            Self::System { .. } => false,         // システムレベルエラー
            Self::Gui { .. } => true,            // UI再描画可能
        }
    }
}
```

### ドメインレベルエラー
```rust
/// OAuth認証エラー
/// 
/// # 不変条件
/// - エラー種別と詳細情報の一貫性
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("OAuth設定エラー: client_id または client_secret が無効")]
    InvalidCredentials,
    
    #[error("認証フローエラー: {step}で失敗")]
    FlowError { step: String },
    
    #[error("トークンエラー: {reason}")]
    TokenError { reason: String },
    
    #[error("OAuth プロバイダーエラー: {provider_error}")]
    ProviderError { provider_error: String },
    
    #[error("ネットワークエラー: {source}")]
    Network {
        #[from]
        source: reqwest::Error,
    },
    
    #[error("タイムアウト: {operation}が{timeout_seconds}秒でタイムアウト")]
    Timeout { 
        operation: String, 
        timeout_seconds: u64,
    },
}

/// ダウンロード処理エラー
#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error("ファイル作成エラー: {path}への書き込みができません")]
    FileCreation { 
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    
    #[error("容量不足エラー: {required_bytes}バイト必要、{available_bytes}バイト利用可能")]
    InsufficientSpace { 
        required_bytes: u64, 
        available_bytes: u64,
    },
    
    #[error("ダウンロード中断: {progress:.1}%完了時点で中断")]
    Interrupted { progress: f32 },
    
    #[error("チェックサム不一致: 期待値={expected}, 実際値={actual}")]
    ChecksumMismatch { 
        expected: String, 
        actual: String,
    },
    
    #[error("ネットワークエラー: {source}")]
    Network {
        #[from]
        source: reqwest::Error,
    },
}

impl DownloadError {
    /// エラーに応じた推奨アクション
    /// 
    /// # 事後条件
    /// - エラー種別に適した対処法を返す
    pub fn recommended_action(&self) -> &'static str {
        match self {
            Self::FileCreation { .. } => "書き込み権限を確認するか、別の保存場所を選択してください",
            Self::InsufficientSpace { .. } => "ディスク容量を確保してから再試行してください",
            Self::Interrupted { .. } => "ネットワーク接続を確認して再試行してください",
            Self::ChecksumMismatch { .. } => "ファイルが破損している可能性があります。再ダウンロードしてください",
            Self::Network { .. } => "インターネット接続を確認してください",
        }
    }
}
```

### インフラレベルエラー
```rust
/// ファイルシステム操作エラー
/// 
/// # 事前条件
/// - path は有効なパス文字列
/// - operation は実行された操作名
#[derive(Debug, thiserror::Error)]
pub enum FileSystemError {
    #[error("ファイル読み込みエラー: {path}")]
    ReadError { 
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    
    #[error("ファイル書き込みエラー: {path}")]
    WriteError { 
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    
    #[error("ディレクトリ作成エラー: {path}")]
    DirectoryCreation { 
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    
    #[error("権限エラー: {path}への{operation}権限がありません")]
    Permission { 
        path: std::path::PathBuf, 
        operation: String,
    },
    
    #[error("パスエラー: {path}は無効なパスです")]
    InvalidPath { path: std::path::PathBuf },
}

/// ネットワーク通信エラー
#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("HTTP エラー: {status_code} - {message}")]
    HttpStatus { 
        status_code: u16, 
        message: String,
    },
    
    #[error("接続エラー: {host}への接続に失敗")]
    Connection { host: String },
    
    #[error("タイムアウト: {operation}")]
    Timeout { operation: String },
    
    #[error("SSL/TLS エラー: {details}")]
    Tls { details: String },
    
    #[error("DNS 解決エラー: {hostname}")]
    DnsResolution { hostname: String },
    
    #[error("reqwest エラー: {source}")]
    Reqwest {
        #[from]
        source: reqwest::Error,
    },
}

impl NetworkError {
    /// エラーがリトライ可能かどうか判定
    /// 
    /// # 事後条件
    /// - 一時的エラーの場合true、永続的エラーの場合false
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::HttpStatus { status_code, .. } => {
                // 5xx系エラーや429（Rate Limit）はリトライ可能
                *status_code >= 500 || *status_code == 429
            }
            Self::Connection { .. } => true,
            Self::Timeout { .. } => true,
            Self::Tls { .. } => false,
            Self::DnsResolution { .. } => false,
            Self::Reqwest { source } => {
                source.is_timeout() || source.is_connect()
            }
        }
    }
}
```

## エラーハンドリングパターン

### Result チェーン処理
```rust
/// エラー変換とコンテキスト追加
/// 
/// # 副作用
/// - HTTP通信の実行
/// - ファイルシステムアクセス
/// 
/// # 事前条件
/// - url が有効なHTTPS URL
/// - save_path が書き込み可能なパス
/// 
/// # 事後条件
/// - 成功時: ファイルがダウンロード完了
/// - 失敗時: 適切なエラーコンテキストを返す
pub async fn download_and_save(
    url: &str, 
    save_path: &std::path::Path
) -> Result<(), AppError> {
    // ネットワークエラーをダウンロードエラーに変換
    let response = fetch_file(url)
        .await
        .map_err(|e| AppError::Download {
            operation: "ファイル取得".to_string(),
            source: DownloadError::Network { source: e },
        })?;
    
    // ファイルシステムエラーをダウンロードエラーに変換
    save_file(response, save_path)
        .await
        .map_err(|e| AppError::Download {
            operation: "ファイル保存".to_string(),
            source: DownloadError::FileCreation {
                path: save_path.to_path_buf(),
                source: e,
            },
        })?;
    
    Ok(())
}

/// リトライ機能付きエラーハンドリング
pub async fn download_with_retry<F, Fut>(
    operation: F,
    max_retries: u32,
) -> Result<String, NetworkError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<String, NetworkError>>,
{
    assert!(max_retries > 0, "max_retries must be positive");
    
    for attempt in 1..=max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if e.is_retryable() && attempt < max_retries => {
                let delay = std::time::Duration::from_millis(100 * 2_u64.pow(attempt - 1));
                tokio::time::sleep(delay).await;
                log::warn!("Retry attempt {} after error: {}", attempt, e);
                continue;
            }
            Err(e) => return Err(e),
        }
    }
    
    unreachable!("loop should always return");
}
```

### エラー変換ヘルパー
```rust
/// エラー変換トレイト
pub trait IntoAppError {
    fn into_app_error(self, context: String) -> AppError;
}

impl IntoAppError for std::io::Error {
    fn into_app_error(self, context: String) -> AppError {
        AppError::System {
            context,
            source: Box::new(self),
        }
    }
}

impl IntoAppError for reqwest::Error {
    fn into_app_error(self, context: String) -> AppError {
        AppError::Download {
            operation: context,
            source: DownloadError::Network { source: self },
        }
    }
}

/// コンテキスト付きエラー変換マクロ
macro_rules! app_error {
    ($expr:expr, $context:expr) => {
        $expr.map_err(|e| e.into_app_error($context.to_string()))
    };
}

// 使用例
pub async fn read_config_file(path: &std::path::Path) -> Result<String, AppError> {
    app_error!(
        tokio::fs::read_to_string(path).await,
        format!("設定ファイル読み込み: {}", path.display())
    )
}
```

## エラーログ統合

### 構造化ログ対応
```rust
use serde_json::json;

/// エラーの構造化ログ出力
/// 
/// # 副作用
/// - ログシステムへの出力
/// 
/// # 事前条件
/// - error は有効なエラー
/// - context は空でない文字列
pub fn log_error(error: &AppError, context: &str) {
    assert!(!context.is_empty(), "context must not be empty");
    
    let error_data = json!({
        "error_type": error_type_name(error),
        "message": error.to_string(),
        "context": context,
        "user_message": error.user_message(),
        "recoverable": error.is_recoverable(),
        "source_chain": collect_error_chain(error),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });
    
    match error {
        AppError::System { .. } => {
            log::error!("System error: {}", error_data);
        }
        AppError::Authentication { .. } => {
            log::warn!("Authentication error: {}", error_data);
        }
        _ => {
            log::info!("Application error: {}", error_data);
        }
    }
}

/// エラーチェーンの収集
fn collect_error_chain(error: &dyn std::error::Error) -> Vec<String> {
    let mut chain = Vec::new();
    let mut current = Some(error);
    
    while let Some(err) = current {
        chain.push(err.to_string());
        current = err.source();
    }
    
    chain
}

/// エラー型名の取得
fn error_type_name(error: &AppError) -> &'static str {
    match error {
        AppError::Authentication { .. } => "Authentication",
        AppError::Download { .. } => "Download",
        AppError::Configuration { .. } => "Configuration",
        AppError::System { .. } => "System",
        AppError::Gui { .. } => "Gui",
    }
}
```

## エラーメトリクス

### エラー統計収集
```rust
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// エラーメトリクス収集
#[derive(Debug, Clone)]
pub struct ErrorMetrics {
    counters: Arc<Mutex<HashMap<String, u64>>>,
}

impl ErrorMetrics {
    pub fn new() -> Self {
        Self {
            counters: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// エラー発生の記録
    /// 
    /// # 副作用
    /// - カウンターの更新
    /// 
    /// # 事前条件
    /// - error が有効なエラー
    pub fn record_error(&self, error: &AppError) {
        let error_type = error_type_name(error);
        
        if let Ok(mut counters) = self.counters.lock() {
            *counters.entry(error_type.to_string()).or_insert(0) += 1;
            
            // 総エラー数も記録
            *counters.entry("total".to_string()).or_insert(0) += 1;
        }
    }
    
    /// エラー統計の取得
    pub fn get_statistics(&self) -> Result<HashMap<String, u64>, String> {
        self.counters
            .lock()
            .map(|counters| counters.clone())
            .map_err(|_| "Failed to acquire lock".to_string())
    }
    
    /// エラー率の計算
    pub fn error_rate(&self, error_type: &str, total_operations: u64) -> Option<f64> {
        if total_operations == 0 {
            return None;
        }
        
        let counters = self.counters.lock().ok()?;
        let error_count = counters.get(error_type).copied().unwrap_or(0);
        
        Some(error_count as f64 / total_operations as f64 * 100.0)
    }
}
```

## テスト戦略

### Property-basedエラーテスト
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        /// エラーメッセージの整合性テスト
        #[test]
        fn error_message_consistency(
            message in "\\PC{1,100}",
            path in "\\PC{1,50}"
        ) {
            let error = AppError::Download {
                operation: message.clone(),
                source: DownloadError::FileCreation {
                    path: std::path::PathBuf::from(path),
                    source: std::io::Error::new(std::io::ErrorKind::Other, "test"),
                },
            };
            
            // エラーメッセージが空でないこと
            prop_assert!(!error.to_string().is_empty());
            prop_assert!(!error.user_message().is_empty());
            
            // 元の情報が含まれること
            prop_assert!(error.to_string().contains(&message));
        }
        
        /// エラー変換の可逆性テスト
        #[test]
        fn error_conversion_preserves_information(
            error_msg in "\\PC{1,100}"
        ) {
            let io_error = std::io::Error::new(std::io::ErrorKind::Other, error_msg.clone());
            let app_error = io_error.into_app_error("test context".to_string());
            
            // 元のエラー情報が保持されること
            prop_assert!(app_error.to_string().contains(&error_msg));
            prop_assert!(app_error.to_string().contains("test context"));
        }
    }
    
    #[test]
    fn test_error_recovery_classification() {
        // 回復可能エラー
        let auth_error = AppError::Authentication { 
            message: "Invalid token".to_string(),
            source: None,
        };
        assert!(auth_error.is_recoverable());
        
        // 回復不可能エラー
        let system_error = AppError::System { 
            context: "Out of memory".to_string(),
            source: Box::new(std::io::Error::new(std::io::ErrorKind::Other, "test")),
        };
        assert!(!system_error.is_recoverable());
    }
}
```

## 品質目標

- **エラーハンドリング網羅率**: 100%
- **エラーメッセージ国際化**: 日本語・英語完全対応
- **エラー発生からログ記録まで**: 1ms以内
- **エラー回復成功率**: 95%以上（回復可能エラー）
- **エラー情報損失**: 0件
- **ユーザビリティスコア**: 8.0/10以上

包括的で型安全なエラーハンドリングにより、信頼性の高いアプリケーションを実現します。