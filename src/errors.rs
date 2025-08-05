/// エラーハンドリング基盤 - thiserrorポリシー準拠
/// 
/// # 目的
/// - 統一エラー型による一貫したエラー処理
/// - thiserrorを使用したエラー定義
/// - エラー回復戦略の実装
use thiserror::Error;

/// Zoom Video Moverアプリケーションの統一エラー型
/// 
/// # 設計方針
/// - すべてのコンポーネントで使用する統一エラー型
/// - 適切なエラー分類とコンテキスト情報
/// - エラー回復戦略を考慮した設計
#[derive(Error, Debug)]
pub enum AppError {
    /// ネットワーク関連エラー
    #[error("Network error: {message}")]
    Network { 
        message: String,
        #[source] 
        source: Option<Box<dyn std::error::Error + Send + Sync>> 
    },

    /// 認証関連エラー
    #[error("Authentication error: {message}")]
    Authentication { 
        message: String,
        #[source] 
        source: Option<Box<dyn std::error::Error + Send + Sync>> 
    },

    /// ファイルシステム関連エラー
    #[error("File system error: {message}")]
    FileSystem { 
        message: String,
        #[source] 
        source: Option<Box<dyn std::error::Error + Send + Sync>> 
    },

    /// 設定関連エラー
    #[error("Configuration error: {message}")]
    Configuration { 
        message: String,
        #[source] 
        source: Option<Box<dyn std::error::Error + Send + Sync>> 
    },

    /// レート制限エラー
    #[error("Rate limit exceeded: {message}")]
    RateLimit { 
        message: String,
        retry_after: Option<u64> 
    },

    /// 無効なトークンエラー
    #[error("Invalid token: {message}")]
    InvalidToken { 
        message: String 
    },

    /// API関連エラー
    #[error("API error ({code}): {message}")]
    Api { 
        code: u16, 
        message: String,
        #[source] 
        source: Option<Box<dyn std::error::Error + Send + Sync>> 
    },

    /// バリデーションエラー
    #[error("Validation error: {message}")]
    Validation { 
        message: String,
        field: Option<String> 
    },

    /// シリアライゼーション/デシリアライゼーションエラー
    #[error("Serialization error: {message}")]
    Serialization { 
        message: String,
        #[source] 
        source: Option<Box<dyn std::error::Error + Send + Sync>> 
    },
}

/// Result型のエイリアス
pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    /// ネットワークエラーを作成
    pub fn network<E>(message: impl Into<String>, source: Option<E>) -> Self 
    where 
        E: std::error::Error + Send + Sync + 'static 
    {
        Self::Network {
            message: message.into(),
            source: source.map(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
        }
    }

    /// 認証エラーを作成
    pub fn authentication<E>(message: impl Into<String>, source: Option<E>) -> Self 
    where 
        E: std::error::Error + Send + Sync + 'static 
    {
        Self::Authentication {
            message: message.into(),
            source: source.map(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
        }
    }

    /// ファイルシステムエラーを作成
    pub fn file_system<E>(message: impl Into<String>, source: Option<E>) -> Self 
    where 
        E: std::error::Error + Send + Sync + 'static 
    {
        Self::FileSystem {
            message: message.into(),
            source: source.map(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
        }
    }

    /// 設定エラーを作成
    pub fn configuration<E>(message: impl Into<String>, source: Option<E>) -> Self 
    where 
        E: std::error::Error + Send + Sync + 'static 
    {
        Self::Configuration {
            message: message.into(),
            source: source.map(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
        }
    }

    /// APIエラーを作成
    pub fn api<E>(code: u16, message: impl Into<String>, source: Option<E>) -> Self 
    where 
        E: std::error::Error + Send + Sync + 'static 
    {
        Self::Api {
            code,
            message: message.into(),
            source: source.map(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
        }
    }

    /// バリデーションエラーを作成
    /// 
    /// # 事前条件
    /// - message は空でない有効なエラーメッセージである
    /// 
    /// # 事後条件
    /// - バリデーションエラーのAppErrorインスタンスが生成される
    /// - field が指定された場合は対象フィールド名が設定される
    /// 
    /// # 不変条件
    /// - message の内容は変更されない
    /// - field の内容は変更されない
    pub fn validation(message: impl Into<String>, field: Option<String>) -> Self {
        Self::Validation {
            message: message.into(),
            field,
        }
    }

    /// シリアライゼーションエラーを作成
    pub fn serialization<E>(message: impl Into<String>, source: Option<E>) -> Self 
    where 
        E: std::error::Error + Send + Sync + 'static 
    {
        Self::Serialization {
            message: message.into(),
            source: source.map(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
        }
    }

    /// エラーが回復可能かどうかを判定
    /// 
    /// # 事前条件
    /// - self は有効なAppErrorインスタンスである
    /// 
    /// # 事後条件
    /// - 回復可能な場合は true を返す
    /// - 回復不可能な場合は false を返す
    /// 
    /// # 不変条件
    /// - self の状態は変更されない
    /// - 判定ロジックは一貫している
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::Network { .. } => true,
            Self::RateLimit { .. } => true,
            Self::Api { code, .. } => *code >= 500, // サーバーエラーは回復可能
            _ => false,
        }
    }

    /// リトライ推奨待機時間を取得（秒）
    /// 
    /// # 事前条件
    /// - self は有効なAppErrorインスタンスである
    /// 
    /// # 事後条件
    /// - リトライ推奨時間がある場合は Some(秒数) を返す
    /// - リトライ推奨時間がない場合は None を返す
    /// 
    /// # 不変条件
    /// - self の状態は変更されない
    /// - 返される時間は0以上の妥当な値である
    pub fn retry_after(&self) -> Option<u64> {
        match self {
            Self::RateLimit { retry_after, .. } => *retry_after,
            Self::Network { .. } => Some(5),
            Self::Api { code, .. } if *code >= 500 => Some(10),
            _ => None,
        }
    }
}

/// 外部エラーからAppErrorへの変換実装
impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Self::network("Request timeout", Some(err))
        } else if err.is_connect() {
            Self::network("Connection failed", Some(err))
        } else {
            Self::network("HTTP request failed", Some(err))
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        Self::serialization("JSON serialization failed", Some(err))
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        Self::file_system("File system operation failed", Some(err))
    }
}

impl From<toml::de::Error> for AppError {
    fn from(err: toml::de::Error) -> Self {
        Self::configuration("TOML parsing failed", Some(err))
    }
}

impl From<toml::ser::Error> for AppError {
    fn from(err: toml::ser::Error) -> Self {
        Self::configuration("TOML serialization failed", Some(err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = AppError::network("Test network error", None::<std::io::Error>);
        assert!(matches!(err, AppError::Network { .. }));
        assert!(err.is_recoverable());
    }

    #[test]
    fn test_retry_after() {
        let err = AppError::RateLimit {
            message: "Rate limited".to_string(),
            retry_after: Some(60),
        };
        assert_eq!(err.retry_after(), Some(60));
    }

    #[test]
    fn test_validation_error() {
        let err = AppError::validation("Invalid field", Some("client_id".to_string()));
        assert!(matches!(err, AppError::Validation { .. }));
        assert!(!err.is_recoverable());
    }
}