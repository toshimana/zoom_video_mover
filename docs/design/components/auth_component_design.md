# 認証コンポーネント詳細設計書 - Zoom Video Mover

## 文書概要
**文書ID**: DES-AUTH-001  
**コンポーネント名**: 認証コンポーネント（Authentication Component）  
**作成日**: 2025-08-03  
  
**バージョン**: 1.0  

## コンポーネント概要

### 責任・役割
- **OAuth 2.0認証フロー**: Zoom APIアクセス用の認証処理
- **トークン管理**: アクセストークン・リフレッシュトークンの安全な管理
- **暗号化・復号化**: 認証情報の暗号化保存・復号化読み込み
- **認証状態管理**: 認証状態の監視・通知・自動更新

### アーキテクチャ位置
```
┌─────────────────────────────────────────────────────────────────┐
│                   Application Layer                             │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │              Authentication Component                        │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │ │
│  │  │   OAuth     │  │   Token     │  │     Secure          │ │ │
│  │  │  Client     │  │  Manager    │  │     Storage         │ │ │
│  │  │             │  │             │  │                     │ │ │
│  │  └─────────────┘  └─────────────┘  └─────────────────────┘ │ │
│  └─────────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                 Infrastructure Layer                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │   HTTP      │  │   Crypto    │  │    File System          │  │
│  │   Client    │  │   Provider  │  │    (Config Storage)     │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## モジュール構造設計

### 内部モジュール構成
```rust
pub mod auth {
    /// OAuth クライアント実装
    pub mod oauth_client;
    
    /// トークン管理
    pub mod token_manager;
    
    /// セキュアストレージ
    pub mod secure_storage;
    
    /// 認証状態管理
    pub mod auth_state;
    
    /// 暗号化ユーティリティ
    pub mod crypto_utils;
    
    /// エラー定義
    pub mod error;
    
    /// 設定・定数
    pub mod config;
}
```

### モジュール依存関係
```
oauth_client
    ├── → token_manager
    ├── → auth_state  
    └── → error

token_manager
    ├── → secure_storage
    ├── → crypto_utils
    ├── → auth_state
    └── → error

secure_storage
    ├── → crypto_utils
    └── → error

auth_state
    └── → error

crypto_utils
    └── → error
```

## データ構造設計

### コアデータ構造

#### 1. 認証トークン
```rust
/// OAuth アクセストークン
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessToken {
    /// トークン値
    pub token: String,
    
    /// トークン種別（通常 "Bearer"）
    pub token_type: String,
    
    /// 有効期限
    pub expires_at: chrono::DateTime<chrono::Utc>,
    
    /// スコープ
    pub scope: Vec<String>,
    
    /// 発行時刻
    pub issued_at: chrono::DateTime<chrono::Utc>,
}

/// OAuth リフレッシュトークン
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshToken {
    /// リフレッシュトークン値
    pub token: String,
    
    /// 有効期限
    pub expires_at: chrono::DateTime<chrono::Utc>,
    
    /// 発行時刻
    pub issued_at: chrono::DateTime<chrono::Utc>,
}

/// 認証情報セット
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthCredentials {
    /// アクセストークン
    pub access_token: AccessToken,
    
    /// リフレッシュトークン
    pub refresh_token: Option<RefreshToken>,
    
    /// ユーザー情報
    pub user_info: Option<UserInfo>,
}
```

#### 2. 認証状態
```rust
/// 認証状態
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthState {
    /// 未認証
    Unauthenticated,
    
    /// 認証処理中
    Authenticating,
    
    /// 認証済み
    Authenticated {
        user_id: String,
        expires_at: chrono::DateTime<chrono::Utc>,
    },
    
    /// トークン期限切れ
    TokenExpired {
        can_refresh: bool,
    },
    
    /// 認証エラー
    AuthenticationFailed {
        error: AuthError,
        retry_possible: bool,
    },
}

/// 認証状態変更イベント
#[derive(Debug, Clone)]
pub struct AuthStateEvent {
    /// 前の状態
    pub previous_state: AuthState,
    
    /// 新しい状態
    pub new_state: AuthState,
    
    /// 変更時刻
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// 変更理由
    pub reason: StateChangeReason,
}
```

#### 3. OAuth 設定
```rust
/// OAuth 設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    /// クライアントID
    pub client_id: String,
    
    /// クライアントシークレット
    pub client_secret: String,
    
    /// リダイレクトURI
    pub redirect_uri: String,
    
    /// 認証エンドポイント
    pub auth_endpoint: String,
    
    /// トークンエンドポイント
    pub token_endpoint: String,
    
    /// 要求スコープ
    pub scopes: Vec<String>,
    
    /// PKCEサポート
    pub use_pkce: bool,
}
```

## インターフェース設計

### 公開API

#### 1. 認証クライアント
```rust
/// 認証クライアント - コンポーネントのメインインターフェース
#[async_trait]
pub trait AuthenticationClient: Send + Sync {
    /// OAuth認証フローを開始
    async fn start_oauth_flow(&self) -> Result<AuthFlowHandle, AuthError>;
    
    /// 認証コードを使用してトークンを取得
    async fn exchange_code_for_token(&self, auth_code: String) -> Result<AuthCredentials, AuthError>;
    
    /// リフレッシュトークンを使用してアクセストークンを更新
    async fn refresh_access_token(&self) -> Result<AccessToken, AuthError>;
    
    /// 現在の認証状態を取得
    fn get_auth_state(&self) -> AuthState;
    
    /// 有効なアクセストークンを取得（自動更新付き）
    async fn get_valid_access_token(&self) -> Result<AccessToken, AuthError>;
    
    /// 認証状態変更を監視
    fn subscribe_to_auth_events(&self) -> broadcast::Receiver<AuthStateEvent>;
    
    /// ログアウト（トークン削除）
    async fn logout(&self) -> Result<(), AuthError>;
}
```

#### 2. 実装クラス
```rust
/// 認証クライアント実装
pub struct ZoomAuthClient {
    /// OAuth設定
    config: OAuthConfig,
    
    /// トークンマネージャー
    token_manager: Arc<TokenManager>,
    
    /// HTTPクライアント
    http_client: Arc<HttpClient>,
    
    /// 認証状態管理
    state_manager: Arc<AuthStateManager>,
    
    /// セキュアストレージ
    secure_storage: Arc<SecureStorage>,
}

impl ZoomAuthClient {
    /// 新しい認証クライアントを作成
    pub fn new(config: OAuthConfig) -> Result<Self, AuthError> {
        let token_manager = Arc::new(TokenManager::new(&config)?);
        let http_client = Arc::new(HttpClient::new()?);
        let state_manager = Arc::new(AuthStateManager::new());
        let secure_storage = Arc::new(SecureStorage::new()?);
        
        Ok(Self {
            config,
            token_manager,
            http_client,
            state_manager,
            secure_storage,
        })
    }
    
    /// 初期化（保存された認証情報の読み込み）
    pub async fn initialize(&self) -> Result<(), AuthError> {
        // 1. 保存された認証情報を読み込み
        if let Ok(credentials) = self.secure_storage.load_credentials().await {
            // 2. トークンの有効性チェック
            if self.token_manager.is_token_valid(&credentials.access_token) {
                self.state_manager.set_authenticated(credentials).await;
            } else if let Some(refresh_token) = credentials.refresh_token {
                // 3. リフレッシュトークンで自動更新
                if let Ok(new_token) = self.refresh_token_internal(&refresh_token).await {
                    let updated_credentials = AuthCredentials {
                        access_token: new_token,
                        refresh_token: Some(refresh_token),
                        user_info: credentials.user_info,
                    };
                    self.state_manager.set_authenticated(updated_credentials).await;
                } else {
                    self.state_manager.set_token_expired(false).await;
                }
            } else {
                self.state_manager.set_token_expired(false).await;
            }
        } else {
            self.state_manager.set_unauthenticated().await;
        }
        
        Ok(())
    }
}
```

### 内部インターフェース

#### 1. トークンマネージャー
```rust
/// トークン管理インターフェース
#[async_trait]
pub trait TokenManager: Send + Sync {
    /// トークンの有効性チェック
    fn is_token_valid(&self, token: &AccessToken) -> bool;
    
    /// トークンの残り有効時間
    fn token_time_to_expiry(&self, token: &AccessToken) -> Duration;
    
    /// トークンの自動更新が必要かチェック
    fn needs_refresh(&self, token: &AccessToken) -> bool;
    
    /// トークンメタデータの更新
    async fn update_token_metadata(&self, token: &mut AccessToken) -> Result<(), AuthError>;
}

/// トークンマネージャー実装
pub struct ZoomTokenManager {
    /// 更新しきい値（有効期限の5分前）
    refresh_threshold: Duration,
    
    /// 時刻プロバイダー（テスト容易性のため）
    time_provider: Arc<dyn TimeProvider>,
}

impl ZoomTokenManager {
    pub fn new() -> Self {
        Self {
            refresh_threshold: Duration::minutes(5),
            time_provider: Arc::new(SystemTimeProvider::new()),
        }
    }
}

#[async_trait]
impl TokenManager for ZoomTokenManager {
    fn is_token_valid(&self, token: &AccessToken) -> bool {
        let now = self.time_provider.now();
        token.expires_at > now
    }
    
    fn needs_refresh(&self, token: &AccessToken) -> bool {
        let now = self.time_provider.now();
        let threshold = token.expires_at - self.refresh_threshold;
        now >= threshold
    }
}
```

#### 2. セキュアストレージ
```rust
/// セキュアストレージインターフェース
#[async_trait]
pub trait SecureStorage: Send + Sync {
    /// 認証情報の暗号化保存
    async fn save_credentials(&self, credentials: &AuthCredentials) -> Result<(), StorageError>;
    
    /// 認証情報の復号化読み込み
    async fn load_credentials(&self) -> Result<AuthCredentials, StorageError>;
    
    /// 認証情報の削除
    async fn clear_credentials(&self) -> Result<(), StorageError>;
    
    /// ストレージの整合性確認
    async fn verify_integrity(&self) -> Result<bool, StorageError>;
}

/// セキュアストレージ実装
pub struct FileBasedSecureStorage {
    /// ストレージファイルパス
    storage_path: PathBuf,
    
    /// 暗号化キー
    encryption_key: Arc<EncryptionKey>,
    
    /// 暗号化実装
    cipher: Arc<dyn Cipher>,
}

impl FileBasedSecureStorage {
    pub fn new(storage_path: PathBuf) -> Result<Self, StorageError> {
        // 1. 暗号化キーの生成/読み込み
        let encryption_key = Arc::new(EncryptionKey::derive_from_system()?);
        
        // 2. AES-GCM暗号化の初期化
        let cipher = Arc::new(AesGcmCipher::new(encryption_key.clone())?);
        
        Ok(Self {
            storage_path,
            encryption_key,
            cipher,
        })
    }
}

#[async_trait]
impl SecureStorage for FileBasedSecureStorage {
    async fn save_credentials(&self, credentials: &AuthCredentials) -> Result<(), StorageError> {
        // 1. 認証情報のシリアライゼーション
        let plaintext = serde_json::to_vec(credentials)
            .map_err(|e| StorageError::Serialization(e))?;
        
        // 2. AES-GCM暗号化
        let ciphertext = self.cipher.encrypt(&plaintext)
            .map_err(|e| StorageError::Encryption(e))?;
        
        // 3. ファイルへの安全な書き込み
        self.write_encrypted_file(&ciphertext).await?;
        
        // 4. プレーンテキストのメモリクリア
        secure_zero_memory(&plaintext);
        
        Ok(())
    }
    
    async fn load_credentials(&self) -> Result<AuthCredentials, StorageError> {
        // 1. 暗号化ファイルの読み込み
        let ciphertext = self.read_encrypted_file().await?;
        
        // 2. AES-GCM復号化
        let plaintext = self.cipher.decrypt(&ciphertext)
            .map_err(|e| StorageError::Decryption(e))?;
        
        // 3. 認証情報のデシリアライゼーション
        let credentials = serde_json::from_slice(&plaintext)
            .map_err(|e| StorageError::Deserialization(e))?;
        
        // 4. プレーンテキストのメモリクリア
        secure_zero_memory(&plaintext);
        
        Ok(credentials)
    }
}
```

## アルゴリズム設計

### OAuth 2.0 フロー実装

#### 1. 認証フロー（PKCE対応）
```rust
impl ZoomAuthClient {
    /// OAuth認証フロー開始
    pub async fn start_oauth_flow(&self) -> Result<AuthFlowHandle, AuthError> {
        // 1. PKCE パラメータ生成
        let pkce = PkceParams::generate()?;
        
        // 2. state パラメータ生成（CSRF対策）
        let state = generate_secure_random_string(32)?;
        
        // 3. 認証URLの構築
        let auth_url = self.build_auth_url(&pkce, &state)?;
        
        // 4. フロー状態の保存
        let flow_handle = AuthFlowHandle {
            state: state.clone(),
            pkce_verifier: pkce.verifier,
            auth_url,
            expires_at: chrono::Utc::now() + chrono::Duration::minutes(10),
        };
        
        // 5. 状態を認証中に変更
        self.state_manager.set_authenticating().await;
        
        Ok(flow_handle)
    }
    
    /// 認証コード→トークン交換
    pub async fn exchange_code_for_token(&self, auth_code: String) -> Result<AuthCredentials, AuthError> {
        // 1. 認証状態確認
        if !matches!(self.get_auth_state(), AuthState::Authenticating) {
            return Err(AuthError::InvalidState("Not in authenticating state"));
        }
        
        // 2. トークンリクエストの構築
        let token_request = self.build_token_request(&auth_code)?;
        
        // 3. Zoom API へのトークンリクエスト
        let response = self.http_client
            .post(&self.config.token_endpoint)
            .form(&token_request)
            .send()
            .await
            .map_err(|e| AuthError::NetworkError(e))?;
        
        // 4. レスポンス解析
        if !response.status().is_success() {
            let error_response: OAuthErrorResponse = response.json().await?;
            return Err(AuthError::OAuthError(error_response));
        }
        
        let token_response: TokenResponse = response.json().await?;
        
        // 5. 認証情報の構築
        let credentials = self.build_auth_credentials(token_response)?;
        
        // 6. 認証情報の安全な保存
        self.secure_storage.save_credentials(&credentials).await?;
        
        // 7. 認証状態を認証済みに変更
        self.state_manager.set_authenticated(credentials.clone()).await;
        
        Ok(credentials)
    }
}
```

#### 2. トークン自動更新
```rust
impl ZoomAuthClient {
    /// トークン自動更新ロジック
    async fn auto_refresh_token(&self) -> Result<AccessToken, AuthError> {
        // 1. 現在の認証情報取得
        let current_credentials = self.secure_storage.load_credentials().await?;
        
        // 2. リフレッシュトークンの確認
        let refresh_token = current_credentials.refresh_token
            .ok_or(AuthError::NoRefreshToken)?;
        
        // 3. リフレッシュトークンの有効性確認
        if !self.token_manager.is_token_valid(&refresh_token) {
            self.state_manager.set_token_expired(false).await;
            return Err(AuthError::RefreshTokenExpired);
        }
        
        // 4. トークン更新リクエスト
        let refresh_request = RefreshTokenRequest {
            grant_type: "refresh_token".to_string(),
            refresh_token: refresh_token.token.clone(),
            client_id: self.config.client_id.clone(),
            client_secret: self.config.client_secret.clone(),
        };
        
        // 5. Zoom API コール
        let response = self.http_client
            .post(&self.config.token_endpoint)
            .json(&refresh_request)
            .send()
            .await?;
        
        // 6. 新しいトークン取得
        let token_response: TokenResponse = response.json().await?;
        let new_access_token = self.parse_access_token(token_response)?;
        
        // 7. 認証情報更新・保存
        let updated_credentials = AuthCredentials {
            access_token: new_access_token.clone(),
            refresh_token: Some(refresh_token),
            user_info: current_credentials.user_info,
        };
        
        self.secure_storage.save_credentials(&updated_credentials).await?;
        self.state_manager.update_authenticated_state(updated_credentials).await;
        
        Ok(new_access_token)
    }
}
```

### 暗号化アルゴリズム

#### AES-GCM実装
```rust
/// AES-GCM暗号化実装
pub struct AesGcmCipher {
    /// AES-256-GCM暗号化キー
    key: Key,
    
    /// 乱数生成器
    rng: ThreadRng,
}

impl AesGcmCipher {
    pub fn new(key: Key) -> Result<Self, CryptoError> {
        Ok(Self {
            key,
            rng: thread_rng(),
        })
    }
}

impl Cipher for AesGcmCipher {
    fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptedData, CryptoError> {
        // 1. ランダムnonce生成（96bit）
        let mut nonce_bytes = [0u8; 12];
        self.rng.fill(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // 2. AES-GCM暗号化実行
        let cipher = Aes256Gcm::new(&self.key);
        let ciphertext = cipher.encrypt(nonce, plaintext)
            .map_err(|e| CryptoError::EncryptionFailed(e))?;
        
        // 3. nonce + ciphertext の結合
        let mut encrypted_data = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
        encrypted_data.extend_from_slice(&nonce_bytes);
        encrypted_data.extend_from_slice(&ciphertext);
        
        Ok(EncryptedData {
            data: encrypted_data,
            algorithm: "AES-256-GCM".to_string(),
        })
    }
    
    fn decrypt(&self, encrypted_data: &EncryptedData) -> Result<Vec<u8>, CryptoError> {
        // 1. データ形式確認
        if encrypted_data.algorithm != "AES-256-GCM" {
            return Err(CryptoError::UnsupportedAlgorithm(encrypted_data.algorithm.clone()));
        }
        
        if encrypted_data.data.len() < 12 {
            return Err(CryptoError::InvalidDataFormat);
        }
        
        // 2. nonce と ciphertext の分離
        let (nonce_bytes, ciphertext) = encrypted_data.data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        // 3. AES-GCM復号化実行
        let cipher = Aes256Gcm::new(&self.key);
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| CryptoError::DecryptionFailed(e))?;
        
        Ok(plaintext)
    }
}
```

## エラー処理設計

### エラー階層構造
```rust
/// 認証エラー定義
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    /// OAuth固有エラー
    #[error("OAuth error: {error_code} - {description}")]
    OAuthError {
        error_code: String,
        description: String,
        error_uri: Option<String>,
    },
    
    /// ネットワークエラー
    #[error("Network error during authentication: {source}")]
    NetworkError {
        #[from]
        source: reqwest::Error,
    },
    
    /// 暗号化エラー
    #[error("Cryptographic error: {source}")]
    CryptographicError {
        #[from]
        source: CryptoError,
    },
    
    /// ストレージエラー
    #[error("Storage error: {source}")]
    StorageError {
        #[from]
        source: StorageError,
    },
    
    /// 設定エラー
    #[error("Configuration error: {message}")]
    ConfigurationError {
        message: String,
        field: String,
    },
    
    /// 状態エラー
    #[error("Invalid authentication state: {message}")]
    InvalidState {
        message: String,
    },
    
    /// トークン期限切れ
    #[error("Token expired: {token_type}")]
    TokenExpired {
        token_type: String,
        expired_at: chrono::DateTime<chrono::Utc>,
    },
}

/// エラー回復戦略
impl AuthError {
    /// エラーが回復可能かどうか
    pub fn is_recoverable(&self) -> bool {
        match self {
            AuthError::NetworkError { .. } => true,
            AuthError::TokenExpired { .. } => true,
            AuthError::OAuthError { error_code, .. } => {
                matches!(error_code.as_str(), "invalid_grant" | "temporarily_unavailable")
            },
            _ => false,
        }
    }
    
    /// 推奨される回復アクション
    pub fn suggested_recovery(&self) -> RecoveryAction {
        match self {
            AuthError::NetworkError { .. } => RecoveryAction::Retry {
                max_attempts: 3,
                backoff: Duration::seconds(1),
            },
            AuthError::TokenExpired { .. } => RecoveryAction::RefreshToken,
            AuthError::OAuthError { error_code: "invalid_grant", .. } => RecoveryAction::ReAuthenticate,
            _ => RecoveryAction::UserIntervention,
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
    
    // モックオブジェクト定義
    mock! {
        HttpClient {}
        
        #[async_trait]
        impl HttpClientTrait for HttpClient {
            async fn post(&self, url: &str) -> Result<Response, reqwest::Error>;
        }
    }
    
    mock! {
        SecureStorage {}
        
        #[async_trait]
        impl SecureStorage for SecureStorage {
            async fn save_credentials(&self, credentials: &AuthCredentials) -> Result<(), StorageError>;
            async fn load_credentials(&self) -> Result<AuthCredentials, StorageError>;
        }
    }
    
    /// OAuth フロー正常系テスト
    #[tokio::test]
    async fn test_oauth_flow_success() {
        // Arrange
        let mut mock_http = MockHttpClient::new();
        let mut mock_storage = MockSecureStorage::new();
        
        // トークンレスポンスのモック設定
        mock_http
            .expect_post()
            .returning(|_| Ok(create_mock_token_response()));
        
        // ストレージ保存のモック設定
        mock_storage
            .expect_save_credentials()
            .returning(|_| Ok(()));
        
        let auth_client = ZoomAuthClient::new_with_mocks(mock_http, mock_storage);
        
        // Act
        let auth_code = "test_auth_code".to_string();
        let result = auth_client.exchange_code_for_token(auth_code).await;
        
        // Assert
        assert!(result.is_ok());
        let credentials = result.unwrap();
        assert!(!credentials.access_token.token.is_empty());
        assert_eq!(auth_client.get_auth_state(), AuthState::Authenticated { .. });
    }
    
    /// トークン自動更新テスト
    #[tokio::test]
    async fn test_automatic_token_refresh() {
        // Arrange
        let auth_client = create_test_auth_client().await;
        let expired_token = create_expired_token();
        
        // Act
        let result = auth_client.get_valid_access_token().await;
        
        // Assert
        assert!(result.is_ok());
        let new_token = result.unwrap();
        assert!(new_token.expires_at > chrono::Utc::now());
    }
}
```

### Property-basedテスト
```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    
    /// 暗号化の可逆性テスト
    proptest! {
        #[test]
        fn test_encryption_roundtrip(
            credentials in arb_auth_credentials()
        ) {
            let cipher = AesGcmCipher::new(generate_test_key())?;
            
            // Serialize → Encrypt → Decrypt → Deserialize
            let plaintext = serde_json::to_vec(&credentials)?;
            let encrypted = cipher.encrypt(&plaintext)?;
            let decrypted = cipher.decrypt(&encrypted)?;
            let restored: AuthCredentials = serde_json::from_slice(&decrypted)?;
            
            prop_assert_eq!(credentials, restored);
        }
        
        #[test]
        fn test_token_expiry_logic(
            expires_in_seconds in 0u64..86400u64
        ) {
            let token = AccessToken {
                token: "test_token".to_string(),
                token_type: "Bearer".to_string(),
                expires_at: chrono::Utc::now() + chrono::Duration::seconds(expires_in_seconds as i64),
                scope: vec!["recording:read".to_string()],
                issued_at: chrono::Utc::now(),
            };
            
            let token_manager = ZoomTokenManager::new();
            
            // 有効期限チェックの一貫性
            let is_valid = token_manager.is_token_valid(&token);
            let needs_refresh = token_manager.needs_refresh(&token);
            
            if expires_in_seconds > 300 {  // 5分以上
                prop_assert!(is_valid);
                prop_assert!(!needs_refresh);
            } else {
                prop_assert_eq!(is_valid, expires_in_seconds > 0);
                if is_valid {
                    prop_assert!(needs_refresh);
                }
            }
        }
    }
    
    /// 任意の認証情報生成
    fn arb_auth_credentials() -> impl Strategy<Value = AuthCredentials> {
        (
            "[a-zA-Z0-9]{32,128}",  // access_token
            1u64..3600u64,          // expires_in
            prop::option::of("[a-zA-Z0-9]{32,128}"),  // refresh_token
        ).prop_map(|(token, expires_in, refresh_token)| {
            AuthCredentials {
                access_token: AccessToken {
                    token,
                    token_type: "Bearer".to_string(),
                    expires_at: chrono::Utc::now() + chrono::Duration::seconds(expires_in as i64),
                    scope: vec!["recording:read".to_string()],
                    issued_at: chrono::Utc::now(),
                },
                refresh_token: refresh_token.map(|rt| RefreshToken {
                    token: rt,
                    expires_at: chrono::Utc::now() + chrono::Duration::days(30),
                    issued_at: chrono::Utc::now(),
                }),
                user_info: None,
            }
        })
    }
}
```

## 性能・セキュリティ考慮事項

### 性能最適化
1. **メモリ効率**: セキュアメモリ管理・プレーンテキストの即座クリア
2. **並行性**: 非同期処理でUIブロッキング回避
3. **キャッシュ**: トークン検証結果の適切なキャッシュ
4. **ネットワーク**: HTTP接続プーリング・Keep-Alive

### セキュリティ強化
1. **暗号化**: AES-256-GCM による強力な暗号化
2. **PKCE**: OAuth 2.0 PKCE拡張による認証強化
3. **メモリ保護**: セキュアメモリ・ゼロ化処理
4. **入力検証**: 全ての外部入力の厳格な検証

---

**承認**:  
**品質基準適合**: [ ] 確認済  
**ポリシー準拠**: [ ] 確認済  
**承認日**: ___________