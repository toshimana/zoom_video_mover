/// 設定管理コンポーネント
/// 
/// # 責任
/// - アプリケーション設定の読み込み・保存
/// - 設定値のバリデーション
/// - 設定変更の監視
/// - デフォルト設定の提供

use crate::errors::{AppError, AppResult};
use crate::components::{ComponentLifecycle, Configurable};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use validator::Validate;

/// Zoom OAuth設定
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct OAuthConfig {
    /// Client ID (必須)
    #[validate(length(min = 1, message = "Client ID is required"))]
    pub client_id: String,
    
    /// Client Secret (必須)
    #[validate(length(min = 1, message = "Client Secret is required"))]
    pub client_secret: String,
    
    /// リダイレクトURI
    #[validate(url(message = "Invalid redirect URI format"))]
    pub redirect_uri: String,
    
    /// 認証スコープ
    pub scopes: Vec<String>,
}

impl Default for OAuthConfig {
    fn default() -> Self {
        Self {
            client_id: String::new(),
            client_secret: String::new(),
            redirect_uri: "http://localhost:8080/callback".to_string(),
            scopes: vec![
                "recording:read".to_string(),
                "user:read".to_string(),
                "meeting:read".to_string(),
            ],
        }
    }
}

/// アプリケーション設定
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AppConfig {
    /// OAuth設定
    pub oauth: OAuthConfig,
    
    /// 出力ディレクトリ
    #[validate(length(min = 1, message = "Output directory is required"))]
    pub output_directory: String,
    
    /// 並列ダウンロード数
    #[validate(range(min = 1, max = 10, message = "Concurrent downloads must be between 1 and 10"))]
    pub max_concurrent_downloads: u32,
    
    /// リクエストタイムアウト（秒）
    #[validate(range(min = 5, max = 300, message = "Timeout must be between 5 and 300 seconds"))]
    pub request_timeout_seconds: u64,
    
    /// レート制限設定
    pub rate_limit_per_second: u32,
    
    /// デバッグモード
    pub debug_mode: bool,
    
    /// ログレベル
    pub log_level: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            oauth: OAuthConfig::default(),
            output_directory: get_default_output_directory(),
            max_concurrent_downloads: 3,
            request_timeout_seconds: 30,
            rate_limit_per_second: 10,
            debug_mode: false,
            log_level: "info".to_string(),
        }
    }
}

/// 設定管理コンポーネント
pub struct ConfigManager {
    /// 現在の設定
    config: AppConfig,
    /// 設定ファイルパス
    config_path: PathBuf,
    /// 設定が変更されたかどうか
    is_modified: bool,
}

impl ConfigManager {
    /// 新しい設定管理コンポーネントを作成
    /// 
    /// # 事前条件
    /// - config_path は有効なパスである
    /// 
    /// # 事後条件
    /// - ConfigManagerインスタンスが作成される
    /// - デフォルト設定で初期化される
    pub fn new<P: AsRef<Path>>(config_path: P) -> Self {
        Self {
            config: AppConfig::default(),
            config_path: config_path.as_ref().to_path_buf(),
            is_modified: false,
        }
    }
    
    /// 設定ファイルから設定を読み込む
    /// 
    /// # 副作用
    /// - ファイルシステムからの読み込み
    /// 
    /// # 事前条件
    /// - 設定ファイルが存在し、読み取り可能である
    /// 
    /// # 事後条件
    /// - 成功時: 設定が正常に読み込まれる
    /// - 失敗時: 適切なエラーが返される
    pub async fn load_from_file(&mut self) -> AppResult<()> {
        if !self.config_path.exists() {
            log::info!("Config file not found, creating default config: {:?}", self.config_path);
            self.create_default_config().await?;
            return Ok(());
        }
        
        let content = tokio::fs::read_to_string(&self.config_path).await
            .map_err(|e| AppError::file_system("Failed to read config file", Some(e)))?;
            
        let config: AppConfig = toml::from_str(&content)
            .map_err(|e| AppError::configuration("Failed to parse config file", Some(e)))?;
            
        // バリデーション実行
        config.validate()
            .map_err(|e| AppError::validation(format!("Config validation failed: {}", e), None))?;
            
        self.config = config;
        self.is_modified = false;
        
        log::info!("Configuration loaded successfully from: {:?}", self.config_path);
        Ok(())
    }
    
    /// 設定をファイルに保存する
    /// 
    /// # 副作用
    /// - ファイルシステムへの書き込み
    /// 
    /// # 事前条件
    /// - 書き込み権限がある
    /// - 親ディレクトリが存在する
    /// 
    /// # 事後条件
    /// - 成功時: 設定がファイルに保存される
    /// - 失敗時: 適切なエラーが返される
    pub async fn save_to_file(&mut self) -> AppResult<()> {
        // バリデーション実行
        self.config.validate()
            .map_err(|e| AppError::validation(format!("Config validation failed: {}", e), None))?;
            
        let content = toml::to_string_pretty(&self.config)
            .map_err(|e| AppError::configuration("Failed to serialize config", Some(e)))?;
            
        // 親ディレクトリを作成
        if let Some(parent) = self.config_path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| AppError::file_system("Failed to create config directory", Some(e)))?;
        }
        
        tokio::fs::write(&self.config_path, content).await
            .map_err(|e| AppError::file_system("Failed to write config file", Some(e)))?;
            
        self.is_modified = false;
        
        log::info!("Configuration saved successfully to: {:?}", self.config_path);
        Ok(())
    }
    
    /// デフォルト設定ファイルを作成する
    /// 
    /// # 副作用
    /// - ファイルシステムへの書き込み
    /// 
    /// # 事前条件
    /// - 書き込み権限がある
    /// 
    /// # 事後条件
    /// - デフォルト設定ファイルが作成される
    async fn create_default_config(&mut self) -> AppResult<()> {
        self.config = AppConfig::default();
        self.save_to_file().await?;
        log::info!("Default configuration file created: {:?}", self.config_path);
        Ok(())
    }
    
    /// 設定が変更されているかチェック
    pub fn is_modified(&self) -> bool {
        self.is_modified
    }
    
    /// OAuth設定を更新する
    /// 
    /// # 事前条件
    /// - oauth_config は有効な設定である
    /// 
    /// # 事後条件
    /// - OAuth設定が更新される
    /// - バリデーションエラーの場合はエラーが返される
    pub fn update_oauth_config(&mut self, oauth_config: OAuthConfig) -> AppResult<()> {
        oauth_config.validate()
            .map_err(|e| AppError::validation(format!("OAuth config validation failed: {}", e), None))?;
            
        self.config.oauth = oauth_config;
        self.is_modified = true;
        Ok(())
    }
    
    /// 出力ディレクトリを更新する
    /// 
    /// # 事前条件
    /// - output_dir は有効なパスである
    /// 
    /// # 事後条件
    /// - 出力ディレクトリが更新される
    pub fn update_output_directory<P: AsRef<Path>>(&mut self, output_dir: P) -> AppResult<()> {
        let path_str = output_dir.as_ref().to_string_lossy().to_string();
        
        if path_str.is_empty() {
            return Err(AppError::validation("Output directory cannot be empty", Some("output_directory".to_string())));
        }
        
        self.config.output_directory = path_str;
        self.is_modified = true;
        Ok(())
    }
}

#[async_trait]
impl ComponentLifecycle for ConfigManager {
    async fn initialize(&mut self) -> AppResult<()> {
        log::info!("Initializing ConfigManager");
        self.load_from_file().await?;
        log::info!("ConfigManager initialized successfully");
        Ok(())
    }
    
    async fn shutdown(&mut self) -> AppResult<()> {
        log::info!("Shutting down ConfigManager");
        
        if self.is_modified {
            log::info!("Saving modified configuration before shutdown");
            self.save_to_file().await?;
        }
        
        log::info!("ConfigManager shut down successfully");
        Ok(())
    }
    
    async fn health_check(&self) -> bool {
        // 設定の基本的な妥当性をチェック
        self.config.validate().is_ok() && self.config_path.parent().map_or(false, |p| p.exists())
    }
}

impl Configurable<AppConfig> for ConfigManager {
    fn update_config(&mut self, config: AppConfig) -> AppResult<()> {
        config.validate()
            .map_err(|e| AppError::validation(format!("Config validation failed: {}", e), None))?;
            
        self.config = config;
        self.is_modified = true;
        Ok(())
    }
    
    fn get_config(&self) -> &AppConfig {
        &self.config
    }
}

/// デフォルトの出力ディレクトリを取得
fn get_default_output_directory() -> String {
    if cfg!(windows) {
        dirs::download_dir()
            .map(|p| p.join("ZoomRecordings").to_string_lossy().to_string())
            .unwrap_or_else(|| r".\downloads".to_string())
    } else {
        "./downloads".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_config_manager_lifecycle() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");
        
        let mut manager = ConfigManager::new(&config_path);
        
        // 初期化テスト
        assert!(manager.initialize().await.is_ok());
        assert!(config_path.exists());
        
        // 設定更新テスト
        let mut oauth_config = OAuthConfig::default();
        oauth_config.client_id = "test_client_id".to_string();
        oauth_config.client_secret = "test_client_secret".to_string();
        
        assert!(manager.update_oauth_config(oauth_config).is_ok());
        assert!(manager.is_modified());
        
        // 保存テスト
        assert!(manager.save_to_file().await.is_ok());
        assert!(!manager.is_modified());
        
        // 終了処理テスト
        assert!(manager.shutdown().await.is_ok());
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();
        
        // 有効な設定
        config.oauth.client_id = "valid_client_id".to_string();
        config.oauth.client_secret = "valid_client_secret".to_string();
        assert!(config.oauth.validate().is_ok());
        
        // 無効な設定
        config.oauth.client_id = String::new();
        assert!(config.oauth.validate().is_err());
    }
}