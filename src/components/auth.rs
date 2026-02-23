//! 認証コンポーネント
//!
//! # 責任
//! - OAuth 2.0 + PKCE認証フローの実装
//! - アクセストークンの管理
//! - トークンの更新・無効化
//! - 認証状態の管理

use crate::errors::{AppError, AppResult};
use crate::components::{ComponentLifecycle, Configurable};
use crate::components::config::OAuthConfig;
use crate::components::crypto::{CryptoComponent, SecretData};
use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl,
    AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, Scope,
    TokenResponse, RefreshToken,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// OAuth認証トークン
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    /// アクセストークン
    pub access_token: String,
    /// トークンタイプ
    pub token_type: String,
    /// 有効期限
    pub expires_at: DateTime<Utc>,
    /// リフレッシュトークン
    pub refresh_token: Option<String>,
    /// 許可されたスコープ
    pub scopes: Vec<String>,
}

impl AuthToken {
    /// トークンが有効かどうか確認
    /// 
    /// # 事前条件
    /// - self は有効なAuthTokenインスタンスである
    /// 
    /// # 事後条件
    /// - トークンが有効期限内で、アクセストークンが空でない場合 true を返す
    /// - そうでなければ false を返す
    /// 
    /// # 不変条件
    /// - self の状態は変更されない
    pub fn is_valid(&self) -> bool {
        Utc::now() < self.expires_at && !self.access_token.is_empty()
    }
    
    /// 指定したスコープが含まれているか確認
    /// 
    /// # 事前条件
    /// - required_scope は空でない文字列である
    /// 
    /// # 事後条件
    /// - 指定したスコープが含まれている場合 true を返す
    /// - そうでなければ false を返す
    /// 
    /// # 不変条件
    /// - self の状態は変更されない
    pub fn has_scope(&self, required_scope: &str) -> bool {
        assert!(!required_scope.is_empty(), "required_scope must not be empty");
        self.scopes.iter().any(|scope| scope == required_scope)
    }
    
    /// 複数のスコープがすべて含まれているか確認
    /// 
    /// # 事前条件
    /// - required_scopes は空でないスライスである
    /// 
    /// # 事後条件
    /// - すべてのスコープが含まれている場合 true を返す
    /// - そうでなければ false を返す
    /// 
    /// # 不変条件
    /// - self の状態は変更されない
    pub fn has_all_scopes(&self, required_scopes: &[&str]) -> bool {
        assert!(!required_scopes.is_empty(), "required_scopes must not be empty");
        required_scopes.iter().all(|&scope| self.has_scope(scope))
    }
    
    /// トークンの残り有効時間を秒で取得
    /// 
    /// # 副作用
    /// - なし（純粋関数）
    /// 
    /// # 事前条件
    /// - expires_at が有効な DateTime<Utc> である
    /// 
    /// # 事後条件
    /// - 残り時間が秒単位で返される
    /// - 期限切れの場合は 0 が返される
    /// - 負の値は返されない
    /// 
    /// # 不変条件
    /// - self の状態は変更されない
    /// - システムの現在時刻が基準となる
    pub fn remaining_seconds(&self) -> i64 {
        (self.expires_at - Utc::now()).num_seconds().max(0)
    }
}

/// OAuth認証フロー状態
#[derive(Debug)]
pub struct AuthFlowState {
    /// 状態ID
    pub state_id: String,
    /// CSRF Token
    pub csrf_token: String,
    /// PKCE Code Verifier
    pub pkce_verifier: PkceCodeVerifier,
    /// 作成時刻
    pub created_at: DateTime<Utc>,
    /// 認証URL
    pub auth_url: String,
}

impl AuthFlowState {
    /// 認証フロー状態が有効か確認（10分間有効）
    /// 
    /// # 副作用
    /// - なし（純粋関数）
    /// 
    /// # 事前条件
    /// - created_at が有効な DateTime<Utc> である
    /// 
    /// # 事後条件
    /// - 作成から10分以内の場合 true を返す
    /// - 10分を超過している場合 false を返す
    /// 
    /// # 不変条件
    /// - self の状態は変更されない
    /// - 10分の有効期限は固定値として維持される
    pub fn is_valid(&self) -> bool {
        let now = Utc::now();
        let expiry = self.created_at + Duration::minutes(10);
        now < expiry
    }
}

/// 認証コンポーネント
pub struct AuthComponent {
    /// OAuth設定
    config: OAuthConfig,
    /// OAuth クライアント
    oauth_client: Option<BasicClient>,
    /// 現在のトークン（暗号化して保存）
    current_token: Option<AuthToken>,
    /// 進行中の認証フロー
    pending_flows: HashMap<String, AuthFlowState>,
    /// HTTPクライアント
    #[allow(dead_code)]
    http_client: reqwest::Client,
    /// 暗号化コンポーネント
    crypto: CryptoComponent,
}

impl AuthComponent {
    /// 新しい認証コンポーネントを作成
    /// 
    /// # 事前条件
    /// - config は有効なOAuth設定である
    /// 
    /// # 事後条件
    /// - AuthComponentインスタンスが作成される
    /// - 内部状態が適切に初期化される
    pub fn new(config: OAuthConfig) -> Self {
        Self {
            config,
            oauth_client: None,
            current_token: None,
            pending_flows: HashMap::new(),
            http_client: reqwest::Client::new(),
            crypto: CryptoComponent::new(),
        }
    }
    
    /// OAuth クライアントを初期化
    /// 
    /// # 事前条件
    /// - config が有効である
    /// 
    /// # 事後条件
    /// - OAuth クライアントが初期化される
    /// - 失敗時は適切なエラーが返される
    fn initialize_oauth_client(&mut self) -> AppResult<()> {
        let auth_url = AuthUrl::new("https://zoom.us/oauth/authorize".to_string())
            .map_err(|e| AppError::configuration("Invalid auth URL", Some(e)))?;
            
        let token_url = TokenUrl::new("https://zoom.us/oauth/token".to_string())
            .map_err(|e| AppError::configuration("Invalid token URL", Some(e)))?;
            
        let redirect_url = RedirectUrl::new(self.config.redirect_uri.clone())
            .map_err(|e| AppError::configuration("Invalid redirect URI", Some(e)))?;
        
        self.oauth_client = Some(
            BasicClient::new(
                ClientId::new(self.config.client_id.clone()),
                Some(ClientSecret::new(self.config.client_secret.clone())),
                auth_url,
                Some(token_url),
            )
            .set_redirect_uri(redirect_url)
        );
        
        log::info!("OAuth client initialized successfully");
        Ok(())
    }
    
    /// 認証URLを生成する
    /// 
    /// # 副作用
    /// - 新しい認証フロー状態が作成される
    /// 
    /// # 事前条件
    /// - OAuth クライアントが初期化されている
    /// 
    /// # 事後条件
    /// - 成功時: 認証URLと状態IDが返される
    /// - 失敗時: 適切なエラーが返される
    pub fn generate_auth_url(&mut self) -> AppResult<(String, String)> {
        let oauth_client = self.oauth_client.as_ref()
            .ok_or_else(|| AppError::authentication("OAuth client not initialized", None::<std::io::Error>))?;
        
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let csrf_token = CsrfToken::new_random();
        let state_id = Uuid::new_v4().to_string();
        
        let mut auth_url_builder = oauth_client
            .authorize_url(|| csrf_token.clone())
            .set_pkce_challenge(pkce_challenge);
        
        // スコープを追加
        for scope in &self.config.scopes {
            auth_url_builder = auth_url_builder.add_scope(Scope::new(scope.clone()));
        }
        
        let (auth_url, _) = auth_url_builder.url();
        
        // 認証フロー状態を保存
        let flow_state = AuthFlowState {
            state_id: state_id.clone(),
            csrf_token: csrf_token.secret().clone(),
            pkce_verifier,
            created_at: Utc::now(),
            auth_url: auth_url.to_string(),
        };
        
        self.pending_flows.insert(state_id.clone(), flow_state);
        
        // 古い認証フローをクリーンアップ
        self.cleanup_expired_flows();
        
        log::info!("Auth URL generated for state: {}", state_id);
        Ok((auth_url.to_string(), state_id))
    }
    
    /// 認証コードをトークンに交換する
    /// 
    /// # 副作用
    /// - HTTPリクエストの送信
    /// - 認証状態の更新
    /// 
    /// # 事前条件
    /// - auth_code は有効な認証コードである
    /// - state_id は有効な状態IDである
    /// 
    /// # 事後条件
    /// - 成功時: 有効なトークンが設定される
    /// - 失敗時: 適切なエラーが返される
    pub async fn exchange_code_for_token(&mut self, auth_code: &str, state_id: &str) -> AppResult<AuthToken> {
        assert!(!auth_code.is_empty(), "auth_code must not be empty");
        assert!(!state_id.is_empty(), "state_id must not be empty");
        
        let oauth_client = self.oauth_client.as_ref()
            .ok_or_else(|| AppError::authentication("OAuth client not initialized", None::<std::io::Error>))?;
        
        let flow_state = self.pending_flows.remove(state_id)
            .ok_or_else(|| AppError::authentication("Invalid or expired state", None::<std::io::Error>))?;
        
        if !flow_state.is_valid() {
            return Err(AppError::authentication("Authentication flow expired", None::<std::io::Error>));
        }
        
        let token_result = oauth_client
            .exchange_code(AuthorizationCode::new(auth_code.to_string()))
            .set_pkce_verifier(flow_state.pkce_verifier)
            .request_async(oauth2::reqwest::async_http_client)
            .await
            .map_err(|e| AppError::authentication("Token exchange failed", Some(e)))?;
        
        let expires_at = Utc::now() + Duration::seconds(
            token_result.expires_in()
                .map(|d| d.as_secs() as i64)
                .unwrap_or(3600)
        );
        
        let token = AuthToken {
            access_token: token_result.access_token().secret().clone(),
            token_type: "Bearer".to_string(),
            expires_at,
            refresh_token: token_result.refresh_token().map(|t| t.secret().clone()),
            scopes: self.config.scopes.clone(),
        };
        
        // トークンの妥当性確認
        debug_assert!(!token.access_token.is_empty(), "access_token must not be empty");
        debug_assert!(token.expires_at > Utc::now(), "token must not be expired");
        
        self.current_token = Some(token.clone());
        
        // トークンを暗号化して保存
        if let Err(e) = self.save_token_securely().await {
            log::warn!("Failed to save token securely: {:?}", e);
            // 保存失敗は致命的ではない（メモリ内のトークンは利用可能）
        }
        
        log::info!("Token exchange completed successfully");
        Ok(token)
    }
    
    /// リフレッシュトークンを使用してアクセストークンを更新する
    /// 
    /// # 副作用
    /// - HTTPリクエストの送信
    /// - トークン状態の更新
    /// 
    /// # 事前条件
    /// - リフレッシュトークンが利用可能である
    /// 
    /// # 事後条件
    /// - 成功時: 新しいアクセストークンが設定される
    /// - 失敗時: 適切なエラーが返される
    pub async fn refresh_token(&mut self) -> AppResult<AuthToken> {
        let oauth_client = self.oauth_client.as_ref()
            .ok_or_else(|| AppError::authentication("OAuth client not initialized", None::<std::io::Error>))?;
        
        let current_token = self.current_token.as_ref()
            .ok_or_else(|| AppError::authentication("No current token available", None::<std::io::Error>))?;
        
        let refresh_token = current_token.refresh_token.as_ref()
            .ok_or_else(|| AppError::authentication("No refresh token available", None::<std::io::Error>))?;
        
        let token_result = oauth_client
            .exchange_refresh_token(&RefreshToken::new(refresh_token.clone()))
            .request_async(oauth2::reqwest::async_http_client)
            .await
            .map_err(|e| AppError::authentication("Token refresh failed", Some(e)))?;
        
        let expires_at = Utc::now() + Duration::seconds(
            token_result.expires_in()
                .map(|d| d.as_secs() as i64)
                .unwrap_or(3600)
        );
        
        let new_token = AuthToken {
            access_token: token_result.access_token().secret().clone(),
            token_type: "Bearer".to_string(),
            expires_at,
            refresh_token: token_result.refresh_token()
                .map(|t| t.secret().clone())
                .or_else(|| current_token.refresh_token.clone()),
            scopes: current_token.scopes.clone(),
        };
        
        self.current_token = Some(new_token.clone());
        
        log::info!("Token refreshed successfully");
        Ok(new_token)
    }
    
    /// 現在のトークンを取得する
    /// 
    /// # 事前条件
    /// - なし
    /// 
    /// # 事後条件
    /// - トークンが存在する場合は Some(AuthToken) を返す
    /// - 存在しない場合は None を返す
    /// 
    /// # 不変条件
    /// - self の状態は変更されない
    pub fn get_current_token(&self) -> Option<&AuthToken> {
        self.current_token.as_ref()
    }
    
    /// 認証状態をクリアする
    /// 
    /// # 副作用
    /// - 内部状態のクリア
    /// - 永続化されたトークンの削除
    /// 
    /// # 事前条件
    /// - なし
    /// 
    /// # 事後条件
    /// - 認証状態がクリアされる
    /// - 進行中の認証フローが削除される
    pub fn clear_auth_state(&mut self) {
        self.current_token = None;
        self.pending_flows.clear();
        
        // 永続化されたトークンファイルを削除
        if let Err(e) = self.delete_stored_token() {
            log::warn!("Failed to delete stored token: {:?}", e);
        }
        
        log::info!("Authentication state cleared");
    }
    
    /// トークンが自動更新可能か確認する
    /// 
    /// # 副作用
    /// - なし（純粋関数）
    /// 
    /// # 事前条件
    /// - なし
    /// 
    /// # 事後条件
    /// - リフレッシュトークンが存在する場合 true を返す
    /// - 現在のトークンが存在しない、またはリフレッシュトークンが存在しない場合 false を返す
    /// 
    /// # 不変条件
    /// - self の状態は変更されない
    /// - トークンの構造は変更されない
    pub fn can_auto_refresh(&self) -> bool {
        self.current_token.as_ref()
            .and_then(|t| t.refresh_token.as_ref())
            .is_some()
    }
    
    /// 期限切れの認証フローをクリーンアップ
    fn cleanup_expired_flows(&mut self) {
        let now = Utc::now();
        self.pending_flows.retain(|_, flow| {
            let expiry = flow.created_at + Duration::minutes(10);
            now < expiry
        });
    }
    
    /// トークンを暗号化して永続化する
    /// 
    /// # セキュリティ要件
    /// - AES-256-GCM暗号化
    /// - Windows DPAPI保護
    /// - アクセストークン・リフレッシュトークンの暗号化
    /// 
    /// # 副作用
    /// - ファイルシステムへの暗号化データ書き込み
    /// 
    /// # 事前条件
    /// - 暗号化コンポーネントが初期化済み
    /// - current_token が Some である
    /// 
    /// # 事後条件
    /// - トークンが暗号化されて保存される
    /// - 失敗時は適切なエラーが返される
    pub async fn save_token_securely(&self) -> AppResult<()> {
        let token = self.current_token.as_ref()
            .ok_or_else(|| AppError::authentication("No token to save", None::<std::io::Error>))?;
        
        if !self.crypto.is_initialized() {
            return Err(AppError::authentication("Crypto component not initialized", None::<std::io::Error>));
        }
        
        // トークンをJSONにシリアライズ
        let token_json = serde_json::to_string(token)
            .map_err(|e| AppError::serialization("Failed to serialize token", Some(e)))?;
        
        // 暗号化
        let secret_data = SecretData::from_string(token_json);
        let encrypted_json = self.crypto.encrypt_to_json(&secret_data)?;
        
        // ファイルに保存
        let token_file_path = Self::get_token_storage_path()?;
        
        // ディレクトリ作成
        if let Some(parent) = token_file_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AppError::file_system("Failed to create token directory", Some(e)))?;
        }
        
        std::fs::write(&token_file_path, encrypted_json)
            .map_err(|e| AppError::file_system("Failed to save encrypted token", Some(e)))?;
        
        log::info!("Token saved securely to: {:?}", token_file_path);
        Ok(())
    }
    
    /// 暗号化されたトークンを読み込む
    /// 
    /// # セキュリティ要件
    /// - 暗号化データの認証タグ検証
    /// - 復号化後の機密データ自動ゼロ化
    /// 
    /// # 副作用
    /// - ファイルシステムからの読み取り
    /// - トークン状態の更新
    /// 
    /// # 事前条件
    /// - 暗号化コンポーネントが初期化済み
    /// - 暗号化されたトークンファイルが存在する
    /// 
    /// # 事後条件
    /// - 有効なトークンが復号化される
    /// - トークンの状態が更新される
    pub async fn load_token_securely(&mut self) -> AppResult<Option<AuthToken>> {
        if !self.crypto.is_initialized() {
            return Err(AppError::authentication("Crypto component not initialized", None::<std::io::Error>));
        }
        
        let token_file_path = Self::get_token_storage_path()?;
        
        if !token_file_path.exists() {
            log::debug!("No stored token file found");
            return Ok(None);
        }
        
        // 暗号化されたデータを読み込み
        let encrypted_json = std::fs::read_to_string(&token_file_path)
            .map_err(|e| AppError::file_system("Failed to read encrypted token file", Some(e)))?;
        
        // 復号化
        let secret_data = self.crypto.decrypt_from_json(&encrypted_json)?;
        let token_json = secret_data.expose_secret_string()
            .map_err(|e| AppError::serialization("Decrypted token is not valid UTF-8", Some(e)))?;
        
        // JSONからトークンをデシリアライズ
        let token: AuthToken = serde_json::from_str(token_json)
            .map_err(|e| AppError::serialization("Failed to deserialize token", Some(e)))?;
        
        // トークンの有効性確認
        if token.is_valid() {
            self.current_token = Some(token.clone());
            log::info!("Valid token loaded from secure storage");
            Ok(Some(token))
        } else {
            log::warn!("Loaded token is expired, deleting");
            if let Err(e) = self.delete_stored_token() {
                log::warn!("Failed to delete expired token: {:?}", e);
            }
            Ok(None)
        }
    }
    
    /// 保存されたトークンファイルを削除する
    /// 
    /// # 副作用
    /// - ファイルシステムからの削除
    /// 
    /// # 事前条件
    /// - なし
    /// 
    /// # 事後条件
    /// - トークンファイルが削除される
    /// - ファイルが存在しない場合はエラーなし
    pub fn delete_stored_token(&self) -> AppResult<()> {
        let token_file_path = Self::get_token_storage_path()?;
        
        if token_file_path.exists() {
            std::fs::remove_file(&token_file_path)
                .map_err(|e| AppError::file_system("Failed to delete token file", Some(e)))?;
            log::info!("Stored token file deleted: {:?}", token_file_path);
        }
        
        Ok(())
    }
    
    /// トークン保存パスを取得
    fn get_token_storage_path() -> AppResult<std::path::PathBuf> {
        // Windows: %APPDATA%\ZoomVideoMover\auth_token.encrypted
        #[cfg(target_os = "windows")]
        {
            let mut path = dirs::config_dir()
                .ok_or_else(|| AppError::file_system("Could not determine config directory", None::<std::io::Error>))?;
            path.push("ZoomVideoMover");
            path.push("auth_token.encrypted");
            Ok(path)
        }
        
        // Unix-like: ~/.config/zoom-video-mover/auth_token.encrypted
        #[cfg(not(target_os = "windows"))]
        {
            let mut path = dirs::config_dir()
                .ok_or_else(|| AppError::file_system("Could not determine config directory", None::<std::io::Error>))?;
            path.push("zoom-video-mover");
            path.push("auth_token.encrypted");
            Ok(path)
        }
    }
    
    /// 自動トークン更新とともに現在のトークンを取得
    /// 
    /// # セキュリティ要件
    /// - 期限切れ5分前での自動更新
    /// - 更新失敗時の既存トークン保持
    /// 
    /// # 副作用
    /// - 必要に応じてトークンの更新
    /// - HTTPリクエストの送信
    /// 
    /// # 事前条件
    /// - なし
    /// 
    /// # 事後条件
    /// - 有効なトークンが返される（自動更新込み）
    /// - トークンが存在しない場合は None が返される
    pub async fn get_valid_token(&mut self) -> AppResult<Option<&AuthToken>> {
        if let Some(token) = &self.current_token {
            // 5分以内に期限切れの場合、自動更新を試行
            if token.remaining_seconds() < 300 && self.can_auto_refresh() {
                log::info!("Token expires soon, attempting auto-refresh");
                match self.refresh_token().await {
                    Ok(_) => {
                        log::info!("Token auto-refresh successful");
                        // 更新されたトークンを保存
                        if let Err(e) = self.save_token_securely().await {
                            log::warn!("Failed to save refreshed token: {:?}", e);
                        }
                    },
                    Err(e) => {
                        log::warn!("Token auto-refresh failed: {:?}", e);
                        // リフレッシュ失敗時は既存トークンをそのまま返す
                        // （ユーザーに再認証を促す）
                    }
                }
            }
        }
        
        Ok(self.current_token.as_ref())
    }
}

#[async_trait]
impl ComponentLifecycle for AuthComponent {
    async fn initialize(&mut self) -> AppResult<()> {
        log::info!("Initializing AuthComponent");
        
        // 暗号化コンポーネントの初期化
        self.crypto.initialize_master_key().await?;
        log::info!("Crypto component initialized");
        
        // OAuth クライアントの初期化
        self.initialize_oauth_client()?;
        
        // 保存されたトークンの読み込み試行
        match self.load_token_securely().await {
            Ok(Some(_)) => log::info!("Existing token loaded from secure storage"),
            Ok(None) => log::info!("No existing token found"),
            Err(e) => {
                log::warn!("Failed to load existing token: {:?}", e);
                // 読み込み失敗は致命的ではない（新規認証で継続可能）
            }
        }
        
        log::info!("AuthComponent initialized successfully");
        Ok(())
    }
    
    async fn shutdown(&mut self) -> AppResult<()> {
        log::info!("Shutting down AuthComponent");
        self.clear_auth_state();
        log::info!("AuthComponent shut down successfully");
        Ok(())
    }
    
    async fn health_check(&self) -> bool {
        // OAuth クライアントが初期化されており、設定が有効であることを確認
        self.oauth_client.is_some() && 
        !self.config.client_id.is_empty() && 
        !self.config.client_secret.is_empty()
    }
}

impl Configurable<OAuthConfig> for AuthComponent {
    fn update_config(&mut self, config: OAuthConfig) -> AppResult<()> {
        self.config = config;
        self.initialize_oauth_client()?;
        // 設定変更時は認証状態をクリア
        self.clear_auth_state();
        Ok(())
    }
    
    fn get_config(&self) -> &OAuthConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_token_validity() {
        let mut token = AuthToken {
            access_token: "test_token".to_string(),
            token_type: "Bearer".to_string(),
            expires_at: Utc::now() + Duration::hours(1),
            refresh_token: None,
            scopes: vec!["recording:read".to_string()],
        };
        
        assert!(token.is_valid());
        assert!(token.has_scope("recording:read"));
        assert!(!token.has_scope("invalid_scope"));
        
        // 期限切れトークン
        token.expires_at = Utc::now() - Duration::hours(1);
        assert!(!token.is_valid());
    }
    
    #[tokio::test]
    async fn test_auth_component_lifecycle() {
        let config = OAuthConfig {
            client_id: "test_client_id".to_string(),
            client_secret: "test_client_secret".to_string(),
            redirect_uri: "http://localhost:8080/callback".to_string(),
            scopes: vec!["recording:read".to_string()],
        };
        
        let mut auth_component = AuthComponent::new(config);
        
        // 初期化テスト
        assert!(auth_component.initialize().await.is_ok());
        assert!(auth_component.health_check().await);
        
        // 認証URL生成テスト
        let result = auth_component.generate_auth_url();
        assert!(result.is_ok());
        
        let (auth_url, state_id) = result.unwrap();
        assert!(!auth_url.is_empty());
        assert!(!state_id.is_empty());
        assert!(auth_url.contains("zoom.us/oauth/authorize"));
        
        // 終了処理テスト
        assert!(auth_component.shutdown().await.is_ok());
    }
}