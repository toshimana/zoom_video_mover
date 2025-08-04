# Rustコーディング方針 - Zoom Video Mover

**技術要素**: Rust 1.70+  
**適用範囲**: 全Rustコード（src/, tests/, benches/）

## Rustコーディング原則

### 基本原則
- **型安全性**: Rustの型システムを最大限活用
- **メモリ安全性**: 所有権システムによる安全なメモリ管理
- **並行安全性**: データ競合のないマルチスレッド処理
- **パフォーマンス**: ゼロコスト抽象化の活用
- **エラー安全性**: Result型による明示的エラーハンドリング

### コード品質指標
- **関数の長さ**: 50行以内（推奨25行以内）
- **循環的複雑度**: 関数あたり10以下
- **ネストレベル**: 4レベル以内
- **クレート依存関係**: 最小限かつ明確な理由

## 関数設計規約

### 関数コメント必須要素
すべての public 関数には以下を含むコメントを記載：

```rust
/// 関数の目的と動作の説明
/// 
/// # 副作用
/// - ファイル操作、ネットワーク通信等の外部への影響
/// 
/// # 事前条件
/// - 引数の有効性、システム状態、依存関係等
/// 
/// # 事後条件  
/// - 戻り値の性質、システム状態の変化等
/// 
/// # 不変条件
/// - 実行中に常に維持される条件
/// 
/// # Examples
/// ```
/// let result = function_example("valid_input")?;
/// assert_eq!(result.len(), 5);
/// ```
pub fn example_function(input: &str) -> Result<String, Error> {
    // 事前条件のassertion
    assert!(!input.is_empty(), "input must not be empty");
    debug_assert!(input.is_ascii(), "input should be ASCII for performance");
    
    // 実装
    let result = process_input(input)?;
    
    // 事後条件のassertion
    debug_assert!(!result.is_empty(), "result must not be empty");
    debug_assert!(result.len() <= input.len() * 2, "result length should be reasonable");
    
    Ok(result)
}
```

### アサーション使い分け
- **`assert!`**: 常にチェックが必要な重要な条件（本番環境でも実行）
- **`debug_assert!`**: デバッグ時のみチェック（本番環境では無効化）
- **`unreachable!`**: 到達不可能なコードパス

```rust
// 重要なビジネスルールチェック
assert!(user_id > 0, "user_id must be positive");

// パフォーマンス重視のデバッグチェック  
debug_assert!(data.is_sorted(), "data should be pre-sorted for efficiency");

// 論理的に到達不可能
match status {
    Status::Active => process_active(),
    Status::Inactive => process_inactive(),
    _ => unreachable!("Invalid status should never reach here"),
}
```

## データ型設計

### 構造体設計
```rust
/// ユーザー認証情報
/// 
/// # 不変条件
/// - user_id は常に正の値
/// - email は有効な形式
/// - expires_at は現在時刻より未来
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthInfo {
    pub user_id: UserId,
    pub email: EmailAddress,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

impl AuthInfo {
    /// 新しい認証情報を作成
    /// 
    /// # 事前条件
    /// - user_id > 0
    /// - email は有効形式
    /// - expires_at > chrono::Utc::now()
    pub fn new(user_id: UserId, email: EmailAddress, expires_at: chrono::DateTime<chrono::Utc>) -> Result<Self, ValidationError> {
        // バリデーション
        if user_id.as_u64() == 0 {
            return Err(ValidationError::InvalidUserId);
        }
        if expires_at <= chrono::Utc::now() {
            return Err(ValidationError::ExpiredToken);
        }
        
        Ok(Self { user_id, email, expires_at })
    }
}
```

### エラー型設計
```rust
/// アプリケーション固有エラー
#[derive(Debug, thiserror::Error)]
pub enum ZoomVideoMoverError {
    #[error("認証エラー: {message}")]
    Authentication { message: String },
    
    #[error("ネットワークエラー: {source}")]
    Network {
        #[from]
        source: reqwest::Error,
    },
    
    #[error("ファイルシステムエラー: {operation} failed for {path}: {source}")]
    FileSystem {
        operation: String,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}
```

## 並行性・非同期処理

### async/await使用原則
```rust
/// 非同期HTTP呼び出しの例
/// 
/// # 事前条件
/// - client は初期化済み
/// - url は有効なHTTPS URL
/// 
/// # 事後条件
/// - 成功時: 有効なレスポンスを返す
/// - 失敗時: 適切なエラーを返す
pub async fn fetch_data(client: &reqwest::Client, url: &str) -> Result<String, NetworkError> {
    assert!(url.starts_with("https://"), "URL must use HTTPS");
    
    let response = client
        .get(url)
        .timeout(Duration::from_secs(30))
        .send()
        .await?;
    
    let text = response.text().await?;
    
    debug_assert!(!text.is_empty(), "Response should not be empty");
    Ok(text)
}
```

### Channelパターン
```rust
/// 非同期メッセージパッシング
pub async fn setup_message_channels() -> (Sender<Command>, Receiver<Event>) {
    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel(100);
    let (event_tx, event_rx) = tokio::sync::mpsc::channel(100);
    
    // メッセージ処理タスク起動
    tokio::spawn(async move {
        process_commands(cmd_rx, event_tx).await;
    });
    
    (cmd_tx, event_rx)
}
```

## メモリ管理・性能

### スマートポインタ使用指針
```rust
use std::sync::Arc;
use std::sync::RwLock;

/// 共有状態の管理
pub type SharedState = Arc<RwLock<AppState>>;

/// 読み取り優先のアクセスパターン
pub async fn read_state(state: &SharedState) -> Result<String, StateError> {
    let guard = state.read().map_err(|_| StateError::LockPoisoned)?;
    Ok(guard.current_status.clone())
}

/// 書き込みが必要な状態更新
pub async fn update_state(state: &SharedState, new_status: String) -> Result<(), StateError> {
    let mut guard = state.write().map_err(|_| StateError::LockPoisoned)?;
    guard.current_status = new_status;
    guard.last_updated = chrono::Utc::now();
    Ok(())
}
```

### ライフタイム管理
```rust
/// ライフタイム明示の例
pub fn process_data<'a>(input: &'a str, buffer: &'a mut String) -> &'a str {
    buffer.clear();
    buffer.push_str("processed: ");
    buffer.push_str(input);
    buffer.as_str()
}
```

## テスタビリティ

### 依存性注入パターン
```rust
/// テスト可能な設計
#[async_trait]
pub trait HttpClient: Send + Sync {
    async fn get(&self, url: &str) -> Result<String, HttpError>;
}

/// 本番実装
pub struct ReqwestClient {
    client: reqwest::Client,
}

/// テスト用モック
pub struct MockHttpClient {
    responses: HashMap<String, Result<String, HttpError>>,
}
```

### テスト記述
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_auth_token_validation() {
        // 有効なトークンテスト
        let valid_token = AuthToken::new(
            "valid_token".to_string(),
            chrono::Utc::now() + chrono::Duration::hours(1)
        ).unwrap();
        
        assert!(valid_token.is_valid());
        
        // 無効なトークンテスト
        let invalid_token = AuthToken::new(
            "".to_string(),
            chrono::Utc::now() - chrono::Duration::hours(1)
        );
        
        assert!(invalid_token.is_err());
    }
}
```

## コードフォーマット・リント

### Rustfmt設定
```toml
# rustfmt.toml
max_width = 100
hard_tabs = false
tab_spaces = 4
use_small_heuristics = "Default"
```

### Clippy設定
```toml
# Cargo.toml
[lints.clippy]
all = "warn"
pedantic = "warn"
unwrap_used = "deny"
expect_used = "warn"
```

## ドキュメント記述

### モジュールレベル文書
```rust
//! # Zoom API クライアント
//!
//! このモジュールはZoom Cloud APIとの通信を担当します。
//!
//! ## 主要機能
//! - OAuth 2.0 認証
//! - 録画データ取得
//! - ファイルダウンロード
//!
//! ## 使用例
//! ```
//! let client = ZoomApiClient::new("client_id", "client_secret")?;
//! let recordings = client.get_recordings("2024-01-01", "2024-01-31").await?;
//! ```
```

### パブリックAPI文書
```rust
/// ダウンロード進捗情報
///
/// ファイルダウンロードの進捗状況を表します。
/// 
/// # Examples
/// ```
/// let progress = DownloadProgress::new(1024, 2048);
/// assert_eq!(progress.percentage(), 50.0);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct DownloadProgress {
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
}
```

## セキュリティ考慮事項

### 機密情報処理
```rust
use secrecy::{Secret, SecretString};

/// 機密情報の安全な取り扱い
pub struct ApiCredentials {
    pub client_id: String,
    pub client_secret: SecretString,
}

impl ApiCredentials {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client_id,
            client_secret: SecretString::new(client_secret),
        }
    }
}

impl Drop for ApiCredentials {
    fn drop(&mut self) {
        // SecretStringが自動的にメモリをクリア
    }
}
```

### 入力検証
```rust
/// 安全な入力検証
pub fn validate_user_input(input: &str) -> Result<&str, ValidationError> {
    // 長さチェック
    if input.is_empty() || input.len() > 1000 {
        return Err(ValidationError::InvalidLength);
    }
    
    // 文字種チェック
    if !input.chars().all(|c| c.is_alphanumeric() || c.is_whitespace()) {
        return Err(ValidationError::InvalidCharacters);
    }
    
    Ok(input)
}
```

## パフォーマンス最適化

### 効率的なデータ構造
```rust
use std::collections::HashMap;
use fnv::FnvHashMap;

/// 高速ハッシュマップの使用
pub type FastMap<K, V> = FnvHashMap<K, V>;

/// パフォーマンス重視の文字列処理
pub fn efficient_string_processing(inputs: &[&str]) -> String {
    let total_len: usize = inputs.iter().map(|s| s.len()).sum();
    let mut result = String::with_capacity(total_len + inputs.len());
    
    for (i, input) in inputs.iter().enumerate() {
        if i > 0 {
            result.push(' ');
        }
        result.push_str(input);
    }
    
    result
}
```

## 品質目標

- **単体テストカバレッジ**: 90%以上
- **Property-basedテスト**: 重要関数100%
- **Clippyワーニング**: 0件
- **メモリリーク**: 0件
- **並行性バグ**: 0件

Rustの特性を最大限活用し、安全で高性能なコードを目指します。

---

**承認**:  
**ポリシー版本**: 1.0  
**最終更新**: 2025-08-04  
**適用範囲**: 全Rustコード（src/, tests/, benches/）  
**承認日**: ___________