# 設定管理コンポーネント詳細設計書 - Zoom Video Mover

## 文書概要
**文書ID**: DES-CONFIG-001  
**コンポーネント名**: 設定管理コンポーネント（Configuration Management Component）  
**作成日**: 2025-08-03  
  
**バージョン**: 1.0  

## コンポーネント概要

### 責任・役割
- **設定一元管理**: アプリケーション全体の設定情報の統合管理
- **永続化・復元**: TOML形式での設定ファイル保存・読み込み
- **検証・バリデーション**: 設定値の妥当性チェック・自動修正
- **変更通知**: 設定変更の全コンポーネントへのリアルタイム配信

### アーキテクチャ位置
```
┌─────────────────────────────────────────────────────────────────┐
│                   Application Layer                             │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │              Configuration Management Component              │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │ │
│  │  │   Config    │  │ Validation  │  │     Change          │ │ │
│  │  │   Manager   │  │   Engine    │  │  Notification       │ │ │
│  │  │             │  │             │  │     System          │ │ │
│  │  └─────────────┘  └─────────────┘  └─────────────────────┘ │ │
│  │  ┌─────────────────────────────────────────────────────────┐ │ │
│  │  │           Configuration Persistence Layer               │ │ │
│  │  └─────────────────────────────────────────────────────────┘ │ │
│  └─────────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                 Infrastructure Layer                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │   TOML      │  │ File System │  │    Backup               │  │
│  │  Parser     │  │   Manager   │  │    Manager              │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## モジュール構造設計

### 内部モジュール構成
```rust
pub mod config {
    /// 設定マネージャー
    pub mod config_manager;
    
    /// 検証エンジン
    pub mod validation_engine;
    
    /// 変更通知システム
    pub mod change_notification;
    
    /// 永続化レイヤー
    pub mod persistence;
    
    /// バックアップマネージャー
    pub mod backup_manager;
    
    /// デフォルト値管理
    pub mod defaults;
    
    /// マイグレーション
    pub mod migration;
    
    /// 設定スキーマ
    pub mod schema;
    
    /// エラー定義
    pub mod error;
    
    /// 設定・定数
    pub mod constants;
}
```

### モジュール依存関係
```
config_manager
    ├── → validation_engine
    ├── → change_notification
    ├── → persistence
    ├── → backup_manager
    ├── → defaults
    └── → error

validation_engine
    ├── → schema
    ├── → defaults
    └── → error

change_notification
    └── → error

persistence
    ├── → backup_manager
    ├── → migration
    └── → error

backup_manager
    └── → error

defaults
    └── → schema

migration
    ├── → schema
    └── → error

schema
    └── → error
```

## データ構造設計

### コアデータ構造

#### 1. アプリケーション設定
```rust
/// アプリケーション全体設定
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    /// OAuth認証設定
    pub auth: AuthConfig,
    
    /// フィルタリング設定
    pub filters: FilterConfig,
    
    /// ダウンロード設定
    pub download: DownloadConfig,
    
    /// UI設定
    pub ui: UiConfig,
    
    /// アプリケーション動作設定
    pub application: ApplicationConfig,
    
    /// ログ設定
    pub logging: LoggingConfig,
    
    /// 詳細設定（上級者向け）
    pub advanced: AdvancedConfig,
    
    /// 設定メタデータ
    pub metadata: ConfigMetadata,
}

/// OAuth認証設定
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthConfig {
    /// クライアントID
    pub client_id: String,
    
    /// クライアントシークレット（暗号化保存）
    #[serde(with = "encrypted_string")]
    pub client_secret: String,
    
    /// リダイレクトURI
    pub redirect_uri: String,
    
    /// 要求スコープ
    pub scopes: Vec<String>,
    
    /// PKCEサポート
    pub use_pkce: bool,
    
    /// 自動トークン更新
    pub auto_refresh: bool,
    
    /// トークン更新間隔（分）
    pub refresh_interval_minutes: u32,
    
    /// 認証タイムアウト（秒）
    pub auth_timeout_seconds: u32,
}

/// フィルタリング設定
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FilterConfig {
    /// デフォルトフィルタ
    pub default_filters: DefaultFilters,
    
    /// カスタムフィルタプリセット
    pub custom_presets: HashMap<String, CustomFilterPreset>,
    
    /// フィルタ履歴保持数
    pub history_size: u32,
    
    /// フィルタ履歴保持期間（日）
    pub history_retention_days: u32,
    
    /// 検索インデックス設定
    pub search_index: SearchIndexConfig,
}

/// ダウンロード設定
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DownloadConfig {
    /// 出力ディレクトリ
    pub output_directory: PathBuf,
    
    /// 同時ダウンロード数
    pub concurrent_downloads: u32,
    
    /// チャンクサイズ（MB）
    pub chunk_size_mb: u32,
    
    /// 自動リトライ設定
    pub retry_config: RetryConfig,
    
    /// ダウンロード完了後の動作
    pub post_download_actions: PostDownloadActions,
    
    /// ファイル名テンプレート
    pub filename_template: String,
    
    /// 重複ファイル処理方法
    pub duplicate_file_handling: DuplicateFileHandling,
    
    /// 帯域幅制限（Mbps、0で無制限）
    pub bandwidth_limit_mbps: u32,
}

/// UI設定
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UiConfig {
    /// テーマ設定
    pub theme: ThemeConfig,
    
    /// 言語設定
    pub language: LanguageConfig,
    
    /// ウィンドウ設定
    pub window: WindowConfig,
    
    /// フォント設定
    pub font: FontConfig,
    
    /// 通知設定
    pub notifications: NotificationConfig,
    
    /// アクセシビリティ設定
    pub accessibility: AccessibilityConfig,
}
```

#### 2. 設定値検証・制約
```rust
/// 設定値検証ルール
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    /// フィールドパス（例: "download.concurrent_downloads"）
    pub field_path: String,
    
    /// 検証タイプ
    pub validation_type: ValidationType,
    
    /// エラーメッセージ
    pub error_message: String,
    
    /// 修正提案
    pub suggested_fix: Option<String>,
    
    /// 重要度
    pub severity: ValidationSeverity,
}

/// 検証タイプ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationType {
    /// 範囲チェック
    Range {
        min: Option<f64>,
        max: Option<f64>,
    },
    
    /// 長さチェック
    Length {
        min_length: Option<usize>,
        max_length: Option<usize>,
    },
    
    /// 正規表現チェック
    Regex {
        pattern: String,
        description: String,
    },
    
    /// URL形式チェック
    Url {
        schemes: Vec<String>,
        require_host: bool,
    },
    
    /// ディレクトリ存在チェック
    DirectoryExists {
        create_if_missing: bool,
    },
    
    /// ファイル権限チェック
    FilePermissions {
        readable: bool,
        writable: bool,
    },
    
    /// カスタム検証関数
    Custom {
        validator_name: String,
        parameters: HashMap<String, serde_json::Value>,
    },
    
    /// 依存関係チェック
    Dependency {
        depends_on: String,
        condition: DependencyCondition,
    },
}

/// 検証重要度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidationSeverity {
    /// エラー（設定適用不可）
    Error,
    
    /// 警告（動作に影響する可能性）
    Warning,
    
    /// 情報（推奨設定と異なる）
    Info,
}

/// 設定値制約
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigConstraints {
    /// 並列ダウンロード数制限
    pub max_concurrent_downloads: u32,
    
    /// チャンクサイズ制限（MB）
    pub max_chunk_size_mb: u32,
    
    /// 出力ディレクトリパス長制限
    pub max_path_length: usize,
    
    /// カスタムプリセット数制限
    pub max_custom_presets: u32,
    
    /// フィルタ履歴制限
    pub max_filter_history: u32,
    
    /// 設定ファイルサイズ制限（MB）
    pub max_config_file_size_mb: u32,
}
```

#### 3. 設定変更管理
```rust
/// 設定変更イベント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChangeEvent {
    /// 変更ID
    pub change_id: String,
    
    /// 変更時刻
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// 変更者（システム/ユーザー）
    pub changed_by: ChangeSource,
    
    /// 変更内容
    pub changes: Vec<ConfigFieldChange>,
    
    /// 変更理由
    pub reason: Option<String>,
    
    /// バリデーション結果
    pub validation_result: ValidationResult,
}

/// 個別フィールド変更
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFieldChange {
    /// フィールドパス
    pub field_path: String,
    
    /// 変更前の値
    pub old_value: Option<serde_json::Value>,
    
    /// 変更後の値
    pub new_value: serde_json::Value,
    
    /// 変更タイプ
    pub change_type: ChangeType,
}

/// 変更タイプ
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChangeType {
    /// 新規追加
    Added,
    
    /// 値変更
    Modified,
    
    /// 削除
    Removed,
    
    /// リセット（デフォルト値に戻す）
    Reset,
    
    /// マイグレーション
    Migrated,
}

/// 変更ソース
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChangeSource {
    /// ユーザー操作
    User,
    
    /// システム自動更新
    System,
    
    /// マイグレーション
    Migration,
    
    /// デフォルト値復元
    DefaultRestore,
    
    /// 外部ツール
    External {
        tool_name: String,
        tool_version: String,
    },
}

/// 設定バックアップ情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigBackup {
    /// バックアップID
    pub backup_id: String,
    
    /// 作成時刻
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// バックアップファイルパス
    pub file_path: PathBuf,
    
    /// バックアップトリガー
    pub trigger: BackupTrigger,
    
    /// 設定バージョン
    pub config_version: String,
    
    /// 圧縮サイズ（バイト）
    pub compressed_size: u64,
    
    /// 元サイズ（バイト）
    pub original_size: u64,
    
    /// ハッシュ値
    pub hash: String,
    
    /// メタデータ
    pub metadata: HashMap<String, String>,
}
```

## インターフェース設計

### 公開API

#### 1. 設定マネージャー
```rust
/// 設定管理マネージャー - コンポーネントのメインインターフェース
#[async_trait]
pub trait ConfigManager: Send + Sync {
    /// 設定の読み込み
    async fn load_config(&self) -> Result<AppConfig, ConfigError>;
    
    /// 設定の保存
    async fn save_config(&self, config: &AppConfig) -> Result<(), ConfigError>;
    
    /// 部分設定更新
    async fn update_field<T>(&self, field_path: &str, value: T) -> Result<(), ConfigError>
    where
        T: Serialize + Send + Sync;
    
    /// 設定値取得
    async fn get_field<T>(&self, field_path: &str) -> Result<T, ConfigError>
    where
        T: for<'de> Deserialize<'de> + Send;
    
    /// 設定検証
    async fn validate_config(&self, config: &AppConfig) -> ValidationResult;
    
    /// デフォルト設定取得
    fn get_default_config(&self) -> AppConfig;
    
    /// 設定リセット
    async fn reset_to_defaults(&self) -> Result<(), ConfigError>;
    
    /// 設定変更監視
    fn subscribe_changes(&self) -> broadcast::Receiver<ConfigChangeEvent>;
    
    /// バックアップ作成
    async fn create_backup(&self, trigger: BackupTrigger) -> Result<ConfigBackup, ConfigError>;
    
    /// バックアップから復元
    async fn restore_from_backup(&self, backup_id: &str) -> Result<(), ConfigError>;
    
    /// 設定マイグレーション
    async fn migrate_config(&self, from_version: &str, to_version: &str) -> Result<(), ConfigError>;
}
```

#### 2. 実装クラス
```rust
/// 設定管理マネージャー実装
pub struct FileBasedConfigManager {
    /// 設定ファイルパス
    config_file_path: PathBuf,
    
    /// 検証エンジン
    validation_engine: Arc<ValidationEngine>,
    
    /// 変更通知システム
    change_notifier: Arc<ChangeNotificationSystem>,
    
    /// 永続化レイヤー
    persistence_layer: Arc<ConfigPersistenceLayer>,
    
    /// バックアップマネージャー
    backup_manager: Arc<BackupManager>,
    
    /// デフォルト値プロバイダー
    defaults_provider: Arc<DefaultsProvider>,
    
    /// 現在の設定（キャッシュ）
    current_config: Arc<RwLock<AppConfig>>,
    
    /// 設定ロック（同時変更防止）
    config_lock: Arc<Mutex<()>>,
    
    /// 変更通知チャンネル
    change_tx: broadcast::Sender<ConfigChangeEvent>,
    
    /// 設定制約
    constraints: ConfigConstraints,
}

impl FileBasedConfigManager {
    /// 新しい設定マネージャーを作成
    pub fn new(config_file_path: PathBuf) -> Result<Self, ConfigError> {
        let validation_engine = Arc::new(ValidationEngine::new()?);
        let change_notifier = Arc::new(ChangeNotificationSystem::new());
        let persistence_layer = Arc::new(ConfigPersistenceLayer::new(&config_file_path)?);
        let backup_manager = Arc::new(BackupManager::new(&config_file_path)?);
        let defaults_provider = Arc::new(DefaultsProvider::new());
        
        // 初期設定読み込み
        let initial_config = Self::load_or_create_initial_config(
            &persistence_layer,
            &defaults_provider,
            &validation_engine,
        )?;
        
        let current_config = Arc::new(RwLock::new(initial_config));
        let config_lock = Arc::new(Mutex::new(()));
        let (change_tx, _) = broadcast::channel(1000);
        let constraints = ConfigConstraints::default();
        
        Ok(Self {
            config_file_path,
            validation_engine,
            change_notifier,
            persistence_layer,
            backup_manager,
            defaults_provider,
            current_config,
            config_lock,
            change_tx,
            constraints,
        })
    }
    
    /// 初期設定の読み込みまたは作成
    fn load_or_create_initial_config(
        persistence_layer: &ConfigPersistenceLayer,
        defaults_provider: &DefaultsProvider,
        validation_engine: &ValidationEngine,
    ) -> Result<AppConfig, ConfigError> {
        // 1. 既存設定ファイル読み込み試行
        match persistence_layer.load_config() {
            Ok(mut config) => {
                // 2. 設定検証・自動修正
                let validation_result = validation_engine.validate_and_fix(&mut config)?;
                
                if validation_result.has_errors() {
                    log::warn!("Configuration validation found errors: {:?}", validation_result.errors);
                }
                
                if validation_result.was_modified {
                    // 修正された設定を保存
                    persistence_layer.save_config(&config)?;
                    log::info!("Configuration was automatically corrected and saved");
                }
                
                Ok(config)
            },
            Err(ConfigError::FileNotFound { .. }) => {
                // 3. 設定ファイルが存在しない場合はデフォルト作成
                log::info!("Configuration file not found, creating default configuration");
                let default_config = defaults_provider.get_default_config();
                persistence_layer.save_config(&default_config)?;
                Ok(default_config)
            },
            Err(error) => {
                // 4. その他のエラーの場合
                log::error!("Failed to load configuration: {:?}", error);
                
                // バックアップからの復元試行
                if let Ok(backup) = Self::find_latest_valid_backup(&persistence_layer) {
                    log::info!("Attempting to restore from backup: {}", backup.backup_id);
                    return persistence_layer.restore_from_backup(&backup.backup_id);
                }
                
                // 最終手段：デフォルト設定を使用
                log::warn!("Using default configuration as fallback");
                Ok(defaults_provider.get_default_config())
            }
        }
    }
}

#[async_trait]
impl ConfigManager for FileBasedConfigManager {
    async fn load_config(&self) -> Result<AppConfig, ConfigError> {
        let config = self.current_config.read().await;
        Ok(config.clone())
    }
    
    async fn save_config(&self, config: &AppConfig) -> Result<(), ConfigError> {
        // 1. 並行書き込み防止
        let _lock = self.config_lock.lock().await;
        
        // 2. 設定検証
        let validation_result = self.validation_engine.validate_config(config).await?;
        if validation_result.has_errors() {
            return Err(ConfigError::ValidationFailed {
                errors: validation_result.errors,
            });
        }
        
        // 3. 変更差分計算
        let current_config = self.current_config.read().await;
        let changes = self.calculate_config_changes(&*current_config, config);
        drop(current_config);
        
        // 4. バックアップ作成（重要な変更の場合）
        if Self::requires_backup(&changes) {
            self.backup_manager.create_backup(BackupTrigger::BeforeImportantChange).await?;
        }
        
        // 5. 設定保存
        self.persistence_layer.save_config(config).await?;
        
        // 6. メモリ内設定更新
        {
            let mut current_config = self.current_config.write().await;
            *current_config = config.clone();
        }
        
        // 7. 変更通知
        if !changes.is_empty() {
            let change_event = ConfigChangeEvent {
                change_id: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                changed_by: ChangeSource::User,
                changes,
                reason: None,
                validation_result,
            };
            
            self.change_tx.send(change_event.clone()).ok();
            self.change_notifier.notify_change(change_event).await?;
        }
        
        Ok(())
    }
    
    async fn update_field<T>(&self, field_path: &str, value: T) -> Result<(), ConfigError>
    where
        T: Serialize + Send + Sync,
    {
        // 1. 値をJSONに変換
        let json_value = serde_json::to_value(value)
            .map_err(|e| ConfigError::SerializationError {
                field: field_path.to_string(),
                source: e,
            })?;
        
        // 2. フィールドパス検証
        self.validation_engine.validate_field_path(field_path)?;
        
        // 3. 設定取得・更新
        let _lock = self.config_lock.lock().await;
        let mut config = self.load_config().await?;
        
        // 4. フィールド値更新
        Self::set_field_value(&mut config, field_path, json_value)?;
        
        // 5. 部分的設定検証
        self.validation_engine.validate_field(&config, field_path).await?;
        
        // 6. 設定保存
        self.save_config(&config).await?;
        
        Ok(())
    }
    
    async fn get_field<T>(&self, field_path: &str) -> Result<T, ConfigError>
    where
        T: for<'de> Deserialize<'de> + Send,
    {
        // 1. 設定取得
        let config = self.load_config().await?;
        
        // 2. フィールド値抽出
        let json_value = Self::get_field_value(&config, field_path)?;
        
        // 3. 型変換
        serde_json::from_value(json_value)
            .map_err(|e| ConfigError::DeserializationError {
                field: field_path.to_string(),
                source: e,
            })
    }
    
    async fn validate_config(&self, config: &AppConfig) -> ValidationResult {
        self.validation_engine.validate_config(config).await
            .unwrap_or_else(|e| ValidationResult {
                is_valid: false,
                errors: vec![ValidationError {
                    field_path: "global".to_string(),
                    error_type: ValidationErrorType::SystemError,
                    message: format!("Validation engine error: {}", e),
                    severity: ValidationSeverity::Error,
                    suggested_fix: None,
                }],
                warnings: Vec::new(),
                info: Vec::new(),
                was_modified: false,
            })
    }
    
    async fn create_backup(&self, trigger: BackupTrigger) -> Result<ConfigBackup, ConfigError> {
        self.backup_manager.create_backup(trigger).await
    }
    
    async fn restore_from_backup(&self, backup_id: &str) -> Result<(), ConfigError> {
        // 1. バックアップから設定復元
        let restored_config = self.backup_manager.restore_config(backup_id).await?;
        
        // 2. 復元設定の検証
        let validation_result = self.validate_config(&restored_config).await;
        if validation_result.has_errors() {
            return Err(ConfigError::BackupRestoreValidationFailed {
                backup_id: backup_id.to_string(),
                validation_errors: validation_result.errors,
            });
        }
        
        // 3. 現在の設定をバックアップ（復元前）
        self.create_backup(BackupTrigger::BeforeRestore).await?;
        
        // 4. 復元設定適用
        self.save_config(&restored_config).await?;
        
        Ok(())
    }
}
```

### 内部インターフェース

#### 1. 検証エンジン
```rust
/// 設定検証エンジンインターフェース
#[async_trait]
pub trait ValidationEngine: Send + Sync {
    /// 設定全体の検証
    async fn validate_config(&self, config: &AppConfig) -> Result<ValidationResult, ValidationError>;
    
    /// 特定フィールドの検証
    async fn validate_field(&self, config: &AppConfig, field_path: &str) -> Result<(), ValidationError>;
    
    /// 設定検証・自動修正
    async fn validate_and_fix(&self, config: &mut AppConfig) -> Result<ValidationResult, ValidationError>;
    
    /// フィールドパス妥当性確認
    fn validate_field_path(&self, field_path: &str) -> Result<(), ValidationError>;
    
    /// カスタム検証ルール追加
    fn add_custom_validator(&self, validator: Box<dyn CustomValidator>) -> Result<(), ValidationError>;
}

/// 設定検証エンジン実装
pub struct RuleBasedValidationEngine {
    /// 検証ルール
    validation_rules: Vec<ValidationRule>,
    
    /// カスタムバリデーター
    custom_validators: HashMap<String, Box<dyn CustomValidator>>,
    
    /// 制約定義
    constraints: ConfigConstraints,
    
    /// 自動修正有効フラグ
    auto_fix_enabled: bool,
}

impl RuleBasedValidationEngine {
    pub fn new() -> Result<Self, ValidationError> {
        let validation_rules = Self::load_builtin_validation_rules()?;
        let custom_validators = HashMap::new();
        let constraints = ConfigConstraints::default();
        let auto_fix_enabled = true;
        
        Ok(Self {
            validation_rules,
            custom_validators,
            constraints,
            auto_fix_enabled,
        })
    }
    
    /// 組み込み検証ルール読み込み
    fn load_builtin_validation_rules() -> Result<Vec<ValidationRule>, ValidationError> {
        let mut rules = Vec::new();
        
        // OAuth設定検証ルール
        rules.push(ValidationRule {
            field_path: "auth.client_id".to_string(),
            validation_type: ValidationType::Length {
                min_length: Some(10),
                max_length: Some(100),
            },
            error_message: "Client ID must be between 10-100 characters".to_string(),
            suggested_fix: Some("Check your Zoom app configuration".to_string()),
            severity: ValidationSeverity::Error,
        });
        
        rules.push(ValidationRule {
            field_path: "auth.redirect_uri".to_string(),
            validation_type: ValidationType::Url {
                schemes: vec!["http".to_string(), "https".to_string()],
                require_host: true,
            },
            error_message: "Redirect URI must be a valid HTTP/HTTPS URL".to_string(),
            suggested_fix: Some("Use format: http://localhost:PORT or https://your-domain.com/callback".to_string()),
            severity: ValidationSeverity::Error,
        });
        
        // ダウンロード設定検証ルール
        rules.push(ValidationRule {
            field_path: "download.concurrent_downloads".to_string(),
            validation_type: ValidationType::Range {
                min: Some(1.0),
                max: Some(10.0),
            },
            error_message: "Concurrent downloads must be between 1-10".to_string(),
            suggested_fix: Some("Recommended: 3-5 for optimal performance".to_string()),
            severity: ValidationSeverity::Warning,
        });
        
        rules.push(ValidationRule {
            field_path: "download.output_directory".to_string(),
            validation_type: ValidationType::DirectoryExists {
                create_if_missing: true,
            },
            error_message: "Output directory must exist and be writable".to_string(),
            suggested_fix: Some("Create directory or check permissions".to_string()),
            severity: ValidationSeverity::Error,
        });
        
        rules.push(ValidationRule {
            field_path: "download.chunk_size_mb".to_string(),
            validation_type: ValidationType::Range {
                min: Some(1.0),
                max: Some(100.0),
            },
            error_message: "Chunk size must be between 1-100 MB".to_string(),
            suggested_fix: Some("Recommended: 5-10 MB for most connections".to_string()),
            severity: ValidationSeverity::Warning,
        });
        
        // UI設定検証ルール
        rules.push(ValidationRule {
            field_path: "ui.font.size".to_string(),
            validation_type: ValidationType::Range {
                min: Some(8.0),
                max: Some(32.0),
            },
            error_message: "Font size must be between 8-32 points".to_string(),
            suggested_fix: Some("Recommended: 12-16 points for readability".to_string()),
            severity: ValidationSeverity::Warning,
        });
        
        Ok(rules)
    }
}

#[async_trait]
impl ValidationEngine for RuleBasedValidationEngine {
    async fn validate_config(&self, config: &AppConfig) -> Result<ValidationResult, ValidationError> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut info = Vec::new();
        
        // 1. 各検証ルール適用
        for rule in &self.validation_rules {
            match self.apply_validation_rule(config, rule).await {
                Ok(ValidationRuleResult::Valid) => {
                    // 検証成功
                },
                Ok(ValidationRuleResult::Invalid { message, severity }) => {
                    let validation_error = ValidationError {
                        field_path: rule.field_path.clone(),
                        error_type: ValidationErrorType::RuleViolation,
                        message,
                        severity: severity.clone(),
                        suggested_fix: rule.suggested_fix.clone(),
                    };
                    
                    match severity {
                        ValidationSeverity::Error => errors.push(validation_error),
                        ValidationSeverity::Warning => warnings.push(validation_error),
                        ValidationSeverity::Info => info.push(validation_error),
                    }
                },
                Err(e) => {
                    errors.push(ValidationError {
                        field_path: rule.field_path.clone(),
                        error_type: ValidationErrorType::SystemError,
                        message: format!("Validation rule execution failed: {}", e),
                        severity: ValidationSeverity::Error,
                        suggested_fix: None,
                    });
                }
            }
        }
        
        // 2. カスタムバリデーター実行
        for (name, validator) in &self.custom_validators {
            match validator.validate(config).await {
                Ok(custom_result) => {
                    errors.extend(custom_result.errors);
                    warnings.extend(custom_result.warnings);
                    info.extend(custom_result.info);
                },
                Err(e) => {
                    errors.push(ValidationError {
                        field_path: format!("custom.{}", name),
                        error_type: ValidationErrorType::SystemError,
                        message: format!("Custom validator failed: {}", e),
                        severity: ValidationSeverity::Error,
                        suggested_fix: None,
                    });
                }
            }
        }
        
        // 3. 制約チェック
        self.validate_constraints(config, &mut errors, &mut warnings).await;
        
        Ok(ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            info,
            was_modified: false,
        })
    }
    
    async fn validate_and_fix(&self, config: &mut AppConfig) -> Result<ValidationResult, ValidationError> {
        let mut was_modified = false;
        let mut validation_result = self.validate_config(config).await?;
        
        if self.auto_fix_enabled && !validation_result.errors.is_empty() {
            // 自動修正試行
            for error in &validation_result.errors {
                if let Some(fixed_value) = self.attempt_auto_fix(config, error).await? {
                    Self::set_field_value(config, &error.field_path, fixed_value)?;
                    was_modified = true;
                }
            }
            
            // 修正後再検証
            if was_modified {
                validation_result = self.validate_config(config).await?;
                validation_result.was_modified = true;
            }
        }
        
        Ok(validation_result)
    }
    
    /// 自動修正試行
    async fn attempt_auto_fix(&self, config: &AppConfig, error: &ValidationError) -> Result<Option<serde_json::Value>, ValidationError> {
        match error.field_path.as_str() {
            "download.concurrent_downloads" => {
                // 並列数が範囲外の場合、制限内に修正
                if let Ok(current_value) = Self::get_field_value(config, &error.field_path) {
                    if let Some(current_num) = current_value.as_u64() {
                        let fixed_value = current_num.max(1).min(self.constraints.max_concurrent_downloads as u64);
                        return Ok(Some(serde_json::Value::Number(serde_json::Number::from(fixed_value))));
                    }
                }
            },
            
            "download.output_directory" => {
                // 出力ディレクトリが存在しない場合、デフォルトディレクトリに修正
                let default_dir = self.get_default_output_directory();
                return Ok(Some(serde_json::Value::String(default_dir.to_string_lossy().to_string())));
            },
            
            "download.chunk_size_mb" => {
                // チャンクサイズが範囲外の場合、推奨値に修正
                return Ok(Some(serde_json::Value::Number(serde_json::Number::from(5)))); // 5MB
            },
            
            _ => {
                // その他のフィールドはデフォルト値で修正試行
                if let Some(default_value) = self.get_default_field_value(&error.field_path) {
                    return Ok(Some(default_value));
                }
            }
        }
        
        Ok(None)
    }
}
```

#### 2. 永続化レイヤー
```rust
/// 設定永続化レイヤーインターフェース
#[async_trait]
pub trait ConfigPersistenceLayer: Send + Sync {
    /// 設定読み込み
    async fn load_config(&self) -> Result<AppConfig, ConfigError>;
    
    /// 設定保存
    async fn save_config(&self, config: &AppConfig) -> Result<(), ConfigError>;
    
    /// ファイル整合性確認
    async fn verify_file_integrity(&self) -> Result<bool, ConfigError>;
    
    /// 設定ファイル存在確認
    fn config_file_exists(&self) -> bool;
    
    /// 設定ファイル情報取得
    async fn get_file_info(&self) -> Result<ConfigFileInfo, ConfigError>;
}

/// TOML ベース永続化実装
pub struct TomlPersistenceLayer {
    /// 設定ファイルパス
    config_file_path: PathBuf,
    
    /// 一時ファイル用ディレクトリ
    temp_directory: PathBuf,
    
    /// ファイルロック
    file_lock: Arc<Mutex<()>>,
    
    /// エンコーディング設定
    encoding: FileEncoding,
}

impl TomlPersistenceLayer {
    pub fn new(config_file_path: &PathBuf) -> Result<Self, ConfigError> {
        let temp_directory = config_file_path.parent()
            .unwrap_or_else(|| Path::new("."))
            .join(".tmp");
        
        // 一時ディレクトリ作成
        std::fs::create_dir_all(&temp_directory)
            .map_err(|e| ConfigError::DirectoryCreateError {
                path: temp_directory.clone(),
                source: e,
            })?;
        
        let file_lock = Arc::new(Mutex::new(()));
        let encoding = FileEncoding::Utf8;
        
        Ok(Self {
            config_file_path: config_file_path.clone(),
            temp_directory,
            file_lock,
            encoding,
        })
    }
}

#[async_trait]
impl ConfigPersistenceLayer for TomlPersistenceLayer {
    async fn load_config(&self) -> Result<AppConfig, ConfigError> {
        let _lock = self.file_lock.lock().await;
        
        // 1. ファイル存在確認
        if !self.config_file_path.exists() {
            return Err(ConfigError::FileNotFound {
                path: self.config_file_path.clone(),
            });
        }
        
        // 2. ファイル読み込み
        let config_content = tokio::fs::read_to_string(&self.config_file_path).await
            .map_err(|e| ConfigError::FileReadError {
                path: self.config_file_path.clone(),
                source: e,
            })?;
        
        // 3. TOML パース
        let config: AppConfig = toml::from_str(&config_content)
            .map_err(|e| ConfigError::ParseError {
                format: "TOML".to_string(),
                source: e.to_string(),
                line: e.line_col().map(|(line, _)| line),
                column: e.line_col().map(|(_, col)| col),
            })?;
        
        Ok(config)
    }
    
    async fn save_config(&self, config: &AppConfig) -> Result<(), ConfigError> {
        let _lock = self.file_lock.lock().await;
        
        // 1. TOML シリアライゼーション
        let toml_content = toml::to_string_pretty(config)
            .map_err(|e| ConfigError::SerializationError {
                field: "config".to_string(),
                source: serde_json::Error::custom(e.to_string()),
            })?;
        
        // 2. 一時ファイルに書き込み（アトミック操作）
        let temp_file_path = self.temp_directory.join(format!(
            "config_{}.toml.tmp",
            uuid::Uuid::new_v4()
        ));
        
        tokio::fs::write(&temp_file_path, &toml_content).await
            .map_err(|e| ConfigError::FileWriteError {
                path: temp_file_path.clone(),
                source: e,
            })?;
        
        // 3. ファイル同期
        let temp_file = tokio::fs::File::open(&temp_file_path).await
            .map_err(|e| ConfigError::FileOpenError {
                path: temp_file_path.clone(),
                source: e,
            })?;
        
        temp_file.sync_all().await
            .map_err(|e| ConfigError::FileSyncError {
                path: temp_file_path.clone(),
                source: e,
            })?;
        
        // 4. 元ファイルのバックアップ（存在する場合）
        if self.config_file_path.exists() {
            let backup_path = self.config_file_path.with_extension("toml.backup");
            tokio::fs::copy(&self.config_file_path, &backup_path).await
                .map_err(|e| ConfigError::BackupCreationError {
                    source_path: self.config_file_path.clone(),
                    backup_path,
                    source: e,
                })?;
        }
        
        // 5. 一時ファイルを本ファイルに移動（アトミック操作）
        tokio::fs::rename(&temp_file_path, &self.config_file_path).await
            .map_err(|e| ConfigError::FileRenameError {
                from_path: temp_file_path,
                to_path: self.config_file_path.clone(),
                source: e,
            })?;
        
        // 6. ファイル権限設定（読み取り専用化）
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = tokio::fs::metadata(&self.config_file_path).await
                .map_err(|e| ConfigError::FileMetadataError {
                    path: self.config_file_path.clone(),
                    source: e,
                })?.permissions();
            
            perms.set_mode(0o644); // rw-r--r--
            tokio::fs::set_permissions(&self.config_file_path, perms).await
                .map_err(|e| ConfigError::FilePermissionError {
                    path: self.config_file_path.clone(),
                    source: e,
                })?;
        }
        
        Ok(())
    }
    
    async fn verify_file_integrity(&self) -> Result<bool, ConfigError> {
        if !self.config_file_path.exists() {
            return Ok(false);
        }
        
        // 1. ファイル読み込み・パーステスト
        match self.load_config().await {
            Ok(_) => Ok(true),
            Err(ConfigError::ParseError { .. }) => Ok(false),
            Err(e) => Err(e),
        }
    }
}
```

## アルゴリズム設計

### 設定変更差分計算アルゴリズム

#### 構造化差分検出
```rust
impl ConfigChangeDetector {
    /// 設定変更差分計算
    pub fn calculate_config_changes(
        &self,
        old_config: &AppConfig,
        new_config: &AppConfig,
    ) -> Vec<ConfigFieldChange> {
        let mut changes = Vec::new();
        
        // 1. 設定を平坦化（フィールドパス → 値のマッピング）
        let old_flat = self.flatten_config(old_config);
        let new_flat = self.flatten_config(new_config);
        
        // 2. 全フィールドパスの統合
        let all_paths: HashSet<String> = old_flat.keys()
            .chain(new_flat.keys())
            .cloned()
            .collect();
        
        // 3. フィールド別差分検出
        for path in all_paths {
            let change = match (old_flat.get(&path), new_flat.get(&path)) {
                (None, Some(new_value)) => {
                    // 新規追加
                    Some(ConfigFieldChange {
                        field_path: path,
                        old_value: None,
                        new_value: new_value.clone(),
                        change_type: ChangeType::Added,
                    })
                },
                
                (Some(old_value), None) => {
                    // 削除
                    Some(ConfigFieldChange {
                        field_path: path,
                        old_value: Some(old_value.clone()),
                        new_value: serde_json::Value::Null,
                        change_type: ChangeType::Removed,
                    })
                },
                
                (Some(old_value), Some(new_value)) => {
                    // 値変更チェック
                    if old_value != new_value {
                        Some(ConfigFieldChange {
                            field_path: path,
                            old_value: Some(old_value.clone()),
                            new_value: new_value.clone(),
                            change_type: ChangeType::Modified,
                        })
                    } else {
                        None // 変更なし
                    }
                },
                
                (None, None) => None, // あり得ない
            };
            
            if let Some(change) = change {
                changes.push(change);
            }
        }
        
        // 4. 変更の重要度順ソート
        changes.sort_by(|a, b| {
            self.get_field_importance(&a.field_path)
                .cmp(&self.get_field_importance(&b.field_path))
                .reverse()
        });
        
        changes
    }
    
    /// 設定の平坦化（ネストした構造を "parent.child" 形式のパスに展開）
    fn flatten_config(&self, config: &AppConfig) -> HashMap<String, serde_json::Value> {
        let mut flat_map = HashMap::new();
        
        // 設定全体をJSON値に変換
        let config_json = serde_json::to_value(config)
            .expect("AppConfig should always be serializable");
        
        // 再帰的に平坦化
        self.flatten_json_recursive(&config_json, String::new(), &mut flat_map);
        
        flat_map
    }
    
    /// JSON値の再帰的平坦化
    fn flatten_json_recursive(
        &self,
        value: &serde_json::Value,
        path_prefix: String,
        result: &mut HashMap<String, serde_json::Value>,
    ) {
        match value {
            serde_json::Value::Object(map) => {
                for (key, val) in map {
                    let new_path = if path_prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", path_prefix, key)
                    };
                    self.flatten_json_recursive(val, new_path, result);
                }
            },
            serde_json::Value::Array(arr) => {
                for (index, val) in arr.iter().enumerate() {
                    let new_path = format!("{}[{}]", path_prefix, index);
                    self.flatten_json_recursive(val, new_path, result);
                }
            },
            _ => {
                // プリミティブ値は直接格納
                result.insert(path_prefix, value.clone());
            }
        }
    }
    
    /// フィールドの重要度取得（バックアップトリガー判定等に使用）
    fn get_field_importance(&self, field_path: &str) -> u32 {
        match field_path {
            // 認証関連は最高重要度
            path if path.starts_with("auth.") => 100,
            
            // ダウンロード設定は高重要度
            path if path.starts_with("download.") => 80,
            
            // UI設定は中重要度
            path if path.starts_with("ui.") => 50,
            
            // フィルタ設定は中重要度
            path if path.starts_with("filters.") => 60,
            
            // その他は低重要度
            _ => 30,
        }
    }
}
```

### バックアップ管理アルゴリズム

#### 世代管理・自動削除
```rust
impl BackupManager {
    /// バックアップ作成
    pub async fn create_backup(&self, trigger: BackupTrigger) -> Result<ConfigBackup, ConfigError> {
        // 1. 現在の設定読み込み
        let current_config = self.load_current_config().await?;
        
        // 2. バックアップメタデータ生成
        let backup_id = format!("backup_{}_{}", 
            chrono::Utc::now().format("%Y%m%d_%H%M%S"),
            uuid::Uuid::new_v4().to_string()[..8].to_string()
        );
        
        let backup_file_name = format!("{}.toml.gz", backup_id);
        let backup_file_path = self.backup_directory.join(&backup_file_name);
        
        // 3. 設定の圧縮保存
        let (compressed_size, original_size, hash) = self.compress_and_save_config(
            &current_config,
            &backup_file_path,
        ).await?;
        
        // 4. バックアップ情報作成
        let backup_info = ConfigBackup {
            backup_id: backup_id.clone(),
            created_at: chrono::Utc::now(),
            file_path: backup_file_path.clone(),
            trigger,
            config_version: env!("CARGO_PKG_VERSION").to_string(),
            compressed_size,
            original_size,
            hash,
            metadata: HashMap::new(),
        };
        
        // 5. バックアップインデックス更新
        self.update_backup_index(&backup_info).await?;
        
        // 6. 古いバックアップの自動削除
        self.cleanup_old_backups().await?;
        
        Ok(backup_info)
    }
    
    /// 設定の圧縮保存
    async fn compress_and_save_config(
        &self,
        config: &AppConfig,
        backup_file_path: &PathBuf,
    ) -> Result<(u64, u64, String), ConfigError> {
        // 1. 設定をTOML形式にシリアライズ
        let toml_content = toml::to_string_pretty(config)
            .map_err(|e| ConfigError::SerializationError {
                field: "config".to_string(),
                source: serde_json::Error::custom(e.to_string()),
            })?;
        
        let original_size = toml_content.len() as u64;
        
        // 2. GZIP圧縮
        let compressed_data = self.compress_data(toml_content.as_bytes())?;
        let compressed_size = compressed_data.len() as u64;
        
        // 3. ハッシュ計算
        let hash = sha2::Sha256::digest(&compressed_data);
        let hash_string = format!("{:x}", hash);
        
        // 4. ファイル保存
        tokio::fs::write(backup_file_path, &compressed_data).await
            .map_err(|e| ConfigError::BackupWriteError {
                path: backup_file_path.clone(),
                source: e,
            })?;
        
        Ok((compressed_size, original_size, hash_string))
    }
    
    /// 古いバックアップの自動削除
    async fn cleanup_old_backups(&self) -> Result<(), ConfigError> {
        // 1. 現在のバックアップ一覧取得
        let mut backups = self.list_all_backups().await?;
        
        // 2. 作成日時でソート（新しい順）
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        // 3. 保持ポリシー適用
        let retention_policy = self.get_retention_policy();
        let mut backups_to_delete = Vec::new();
        
        // 世代数制限
        if backups.len() > retention_policy.max_backup_count {
            let excess_backups = &backups[retention_policy.max_backup_count..];
            backups_to_delete.extend(excess_backups.iter().cloned());
        }
        
        // 期間制限
        let cutoff_date = chrono::Utc::now() - chrono::Duration::days(retention_policy.max_age_days as i64);
        let expired_backups = backups.iter()
            .filter(|backup| backup.created_at < cutoff_date)
            .cloned();
        backups_to_delete.extend(expired_backups);
        
        // 重複除去
        backups_to_delete.sort_by(|a, b| a.backup_id.cmp(&b.backup_id));
        backups_to_delete.dedup_by(|a, b| a.backup_id == b.backup_id);
        
        // 4. バックアップ削除実行
        for backup in backups_to_delete {
            self.delete_backup(&backup.backup_id).await?;
        }
        
        Ok(())
    }
    
    /// バックアップからの復元
    pub async fn restore_config(&self, backup_id: &str) -> Result<AppConfig, ConfigError> {
        // 1. バックアップ情報取得
        let backup_info = self.get_backup_info(backup_id).await?;
        
        // 2. バックアップファイル読み込み
        let compressed_data = tokio::fs::read(&backup_info.file_path).await
            .map_err(|e| ConfigError::BackupReadError {
                path: backup_info.file_path.clone(),
                source: e,
            })?;
        
        // 3. ハッシュ検証
        let actual_hash = format!("{:x}", sha2::Sha256::digest(&compressed_data));
        if actual_hash != backup_info.hash {
            return Err(ConfigError::BackupIntegrityError {
                backup_id: backup_id.to_string(),
                expected_hash: backup_info.hash,
                actual_hash,
            });
        }
        
        // 4. 解凍
        let decompressed_data = self.decompress_data(&compressed_data)?;
        let toml_content = String::from_utf8(decompressed_data)
            .map_err(|e| ConfigError::BackupDecodingError {
                backup_id: backup_id.to_string(),
                source: e,
            })?;
        
        // 5. 設定パース
        let config: AppConfig = toml::from_str(&toml_content)
            .map_err(|e| ConfigError::BackupParseError {
                backup_id: backup_id.to_string(),
                source: e.to_string(),
            })?;
        
        Ok(config)
    }
    
    /// データ圧縮
    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>, ConfigError> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;
        
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data)
            .map_err(|e| ConfigError::CompressionError {
                source: e,
            })?;
        
        encoder.finish()
            .map_err(|e| ConfigError::CompressionError {
                source: e,
            })
    }
    
    /// データ解凍
    fn decompress_data(&self, compressed_data: &[u8]) -> Result<Vec<u8>, ConfigError> {
        use flate2::read::GzDecoder;
        use std::io::Read;
        
        let mut decoder = GzDecoder::new(compressed_data);
        let mut decompressed = Vec::new();
        
        decoder.read_to_end(&mut decompressed)
            .map_err(|e| ConfigError::DecompressionError {
                source: e,
            })?;
        
        Ok(decompressed)
    }
}
```

## エラー処理設計

### エラー階層構造
```rust
/// 設定管理エラー定義
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// ファイル操作エラー
    #[error("File operation failed: {operation} on {path}")]
    FileOperationError {
        operation: String,
        path: PathBuf,
        source: std::io::Error,
    },
    
    /// 設定解析エラー
    #[error("Configuration parsing failed: {format} format error")]
    ParseError {
        format: String,
        source: String,
        line: Option<usize>,
        column: Option<usize>,
    },
    
    /// 設定検証エラー
    #[error("Configuration validation failed: {error_count} errors found")]
    ValidationFailed {
        errors: Vec<ValidationError>,
    },
    
    /// フィールドアクセスエラー
    #[error("Field access error: {field_path} - {reason}")]
    FieldAccessError {
        field_path: String,
        reason: String,
    },
    
    /// バックアップ操作エラー
    #[error("Backup operation failed: {operation} for backup {backup_id}")]
    BackupOperationError {
        operation: String,
        backup_id: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    /// 設定マイグレーションエラー
    #[error("Configuration migration failed: from version {from_version} to {to_version}")]
    MigrationError {
        from_version: String,
        to_version: String,
        migration_step: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    /// 設定ロックエラー
    #[error("Configuration lock error: {reason}")]
    LockError {
        reason: String,
    },
    
    /// 設定制約違反
    #[error("Configuration constraint violation: {constraint} - {details}")]
    ConstraintViolation {
        constraint: String,
        details: String,
        field_path: Option<String>,
    },
    
    /// 暗号化・復号化エラー
    #[error("Encryption/Decryption error: {operation}")]
    CryptographyError {
        operation: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    /// 設定変更通知エラー
    #[error("Change notification failed: {recipient}")]
    NotificationError {
        recipient: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

/// エラー回復戦略実装
pub struct ConfigErrorRecoveryStrategy {
    /// バックアップマネージャー参照
    backup_manager: Arc<BackupManager>,
    
    /// デフォルト値プロバイダー
    defaults_provider: Arc<DefaultsProvider>,
    
    /// 回復試行履歴
    recovery_history: Arc<Mutex<HashMap<String, RecoveryAttempt>>>,
}

impl ConfigErrorRecoveryStrategy {
    /// エラー種別に基づく自動回復
    pub async fn attempt_recovery(&self, error: &ConfigError, context: &ConfigContext) -> RecoveryResult {
        match error {
            ConfigError::FileOperationError { operation, path, .. } => {
                match operation.as_str() {
                    "read" => {
                        // 読み込みエラー: バックアップからの復元試行
                        if let Ok(latest_backup) = self.backup_manager.get_latest_backup().await {
                            RecoveryResult::RestoreFromBackup {
                                backup_id: latest_backup.backup_id,
                                confidence: 0.8,
                            }
                        } else {
                            // バックアップ無し: デフォルト設定使用
                            RecoveryResult::UseDefaults {
                                reason: "No valid backup found, using default configuration".to_string(),
                            }
                        }
                    },
                    
                    "write" => {
                        // 書き込みエラー: 権限・容量確認後リトライ
                        if let Ok(diagnosis) = self.diagnose_write_failure(path).await {
                            RecoveryResult::RetryWithFix {
                                fix_description: diagnosis.fix_description,
                                estimated_success_rate: diagnosis.success_probability,
                            }
                        } else {
                            RecoveryResult::RequiresUserIntervention
                        }
                    },
                    
                    _ => RecoveryResult::RequiresUserIntervention,
                }
            },
            
            ConfigError::ParseError { format, line, column, .. } => {
                // 解析エラー: 構文修正試行
                if format == "TOML" {
                    RecoveryResult::AutoFix {
                        fix_description: format!(
                            "Attempt to fix TOML syntax error at line {}, column {}",
                            line.unwrap_or(0),
                            column.unwrap_or(0)
                        ),
                        confidence: 0.6,
                    }
                } else {
                    RecoveryResult::RestoreFromBackup {
                        backup_id: self.backup_manager.get_latest_backup().await
                            .map(|b| b.backup_id)
                            .unwrap_or_default(),
                        confidence: 0.9,
                    }
                }
            },
            
            ConfigError::ValidationFailed { errors } => {
                // 検証エラー: 自動修正可能なエラー数に基づく判定
                let fixable_errors = errors.iter()
                    .filter(|e| self.is_auto_fixable(e))
                    .count();
                
                if fixable_errors == errors.len() {
                    RecoveryResult::AutoFix {
                        fix_description: format!("Auto-fix {} validation errors", fixable_errors),
                        confidence: 0.85,
                    }
                } else if fixable_errors > errors.len() / 2 {
                    RecoveryResult::PartialAutoFix {
                        fixable_count: fixable_errors,
                        total_count: errors.len(),
                        remaining_issues: errors.iter()
                            .filter(|e| !self.is_auto_fixable(e))
                            .map(|e| e.message.clone())
                            .collect(),
                    }
                } else {
                    RecoveryResult::RequiresUserIntervention
                }
            },
            
            ConfigError::BackupOperationError { operation, .. } => {
                match operation.as_str() {
                    "restore" => {
                        // 復元失敗: 他のバックアップ試行
                        if let Ok(alternative_backups) = self.backup_manager.list_valid_backups().await {
                            if alternative_backups.len() > 1 {
                                RecoveryResult::TryAlternativeBackup {
                                    backup_candidates: alternative_backups.into_iter()
                                        .take(3)
                                        .map(|b| b.backup_id)
                                        .collect(),
                                }
                            } else {
                                RecoveryResult::UseDefaults {
                                    reason: "No alternative backups available".to_string(),
                                }
                            }
                        } else {
                            RecoveryResult::UseDefaults {
                                reason: "Backup system unavailable".to_string(),
                            }
                        }
                    },
                    
                    _ => RecoveryResult::RequiresUserIntervention,
                }
            },
            
            _ => RecoveryResult::RequiresUserIntervention,
        }
    }
    
    /// エラーの自動修正可能性判定
    fn is_auto_fixable(&self, error: &ValidationError) -> bool {
        match error.error_type {
            ValidationErrorType::RangeViolation => true,
            ValidationErrorType::FormatError => true,
            ValidationErrorType::MissingField => true,
            ValidationErrorType::DirectoryNotFound => true,
            ValidationErrorType::PermissionError => false, // 管理者権限が必要
            ValidationErrorType::DependencyError => false, // 複雑な依存関係
            ValidationErrorType::SystemError => false,
            _ => false,
        }
    }
}
```

## テスト設計

### Property-basedテスト
```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        /// 設定保存・読み込みラウンドトリップ検証
        #[test]
        fn test_config_roundtrip_consistency(
            config in arb_app_config()
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let temp_dir = tempfile::TempDir::new().unwrap();
                let config_path = temp_dir.path().join("test_config.toml");
                
                let persistence_layer = TomlPersistenceLayer::new(&config_path).unwrap();
                
                // 保存 → 読み込み → 保存 のラウンドトリップ
                persistence_layer.save_config(&config).await.unwrap();
                let loaded_config = persistence_layer.load_config().await.unwrap();
                persistence_layer.save_config(&loaded_config).await.unwrap();
                let reloaded_config = persistence_layer.load_config().await.unwrap();
                
                // Property: ラウンドトリップで設定が変化しない
                prop_assert_eq!(config, loaded_config);
                prop_assert_eq!(loaded_config, reloaded_config);
            });
        }
        
        /// フィールドアクセスの完全性検証
        #[test]
        fn test_field_access_completeness(
            config in arb_app_config(),
            field_path in arb_valid_field_path()
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let config_manager = create_test_config_manager().await;
                
                // 設定を保存
                config_manager.save_config(&config).await.unwrap();
                
                // フィールド値取得・更新・再取得
                let original_value: serde_json::Value = config_manager.get_field(&field_path).await.unwrap();
                
                // 異なる値で更新
                let new_value = generate_different_value(&original_value);
                config_manager.update_field(&field_path, &new_value).await.unwrap();
                
                let updated_value: serde_json::Value = config_manager.get_field(&field_path).await.unwrap();
                
                // Property: 更新された値が正しく取得される
                prop_assert_eq!(updated_value, new_value);
                prop_assert_ne!(updated_value, original_value);
                
                // 元の値に戻す
                config_manager.update_field(&field_path, &original_value).await.unwrap();
                let restored_value: serde_json::Value = config_manager.get_field(&field_path).await.unwrap();
                
                // Property: 元の値に正しく戻る
                prop_assert_eq!(restored_value, original_value);
            });
        }
        
        /// バックアップ・復元の完全性検証
        #[test]
        fn test_backup_restore_integrity(
            config in arb_app_config()
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let temp_dir = tempfile::TempDir::new().unwrap();
                let config_path = temp_dir.path().join("test_config.toml");
                let backup_manager = BackupManager::new(&config_path).unwrap();
                
                // 設定保存
                let persistence_layer = TomlPersistenceLayer::new(&config_path).unwrap();
                persistence_layer.save_config(&config).await.unwrap();
                
                // バックアップ作成
                let backup = backup_manager.create_backup(BackupTrigger::Manual).await.unwrap();
                
                // 設定変更
                let mut modified_config = config.clone();
                modified_config.download.concurrent_downloads = 99;
                persistence_layer.save_config(&modified_config).await.unwrap();
                
                // バックアップから復元
                let restored_config = backup_manager.restore_config(&backup.backup_id).await.unwrap();
                
                // Property: 復元された設定が元の設定と一致
                prop_assert_eq!(restored_config, config);
                
                // Property: バックアップファイルが存在し、ハッシュが一致
                prop_assert!(backup.file_path.exists());
                let backup_data = std::fs::read(&backup.file_path).unwrap();
                let actual_hash = format!("{:x}", sha2::Sha256::digest(&backup_data));
                prop_assert_eq!(actual_hash, backup.hash);
            });
        }
        
        /// 設定検証の一貫性検証
        #[test]
        fn test_validation_consistency(
            config in arb_app_config()
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let validation_engine = RuleBasedValidationEngine::new().unwrap();
                
                // 複数回検証実行
                let result1 = validation_engine.validate_config(&config).await.unwrap();
                let result2 = validation_engine.validate_config(&config).await.unwrap();
                let result3 = validation_engine.validate_config(&config).await.unwrap();
                
                // Property: 検証結果は決定的
                prop_assert_eq!(result1.is_valid, result2.is_valid);
                prop_assert_eq!(result2.is_valid, result3.is_valid);
                prop_assert_eq!(result1.errors.len(), result2.errors.len());
                prop_assert_eq!(result2.errors.len(), result3.errors.len());
                
                // Property: 有効な設定は常に有効
                if result1.is_valid {
                    prop_assert!(result2.is_valid);
                    prop_assert!(result3.is_valid);
                }
                
                // Property: エラーがある場合は無効
                if !result1.errors.is_empty() {
                    prop_assert!(!result1.is_valid);
                }
            });
        }
    }
    
    /// 任意の設定生成
    fn arb_app_config() -> impl Strategy<Value = AppConfig> {
        (
            arb_auth_config(),
            arb_download_config(),
            arb_ui_config(),
        ).prop_map(|(auth, download, ui)| {
            AppConfig {
                auth,
                filters: FilterConfig::default(),
                download,
                ui,
                application: ApplicationConfig::default(),
                logging: LoggingConfig::default(),
                advanced: AdvancedConfig::default(),
                metadata: ConfigMetadata::default(),
            }
        })
    }
    
    /// 任意の認証設定生成
    fn arb_auth_config() -> impl Strategy<Value = AuthConfig> {
        (
            "[a-zA-Z0-9]{20,50}",  // client_id
            "[a-zA-Z0-9]{30,100}", // client_secret
            "https?://[a-zA-Z0-9\\.]+/[a-zA-Z0-9_/]*", // redirect_uri
        ).prop_map(|(client_id, client_secret, redirect_uri)| {
            AuthConfig {
                client_id,
                client_secret,
                redirect_uri,
                scopes: vec!["recording:read".to_string(), "user:read".to_string()],
                use_pkce: true,
                auto_refresh: true,
                refresh_interval_minutes: 55,
                auth_timeout_seconds: 300,
            }
        })
    }
    
    /// 任意の有効フィールドパス生成
    fn arb_valid_field_path() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("auth.client_id".to_string()),
            Just("auth.redirect_uri".to_string()),
            Just("download.concurrent_downloads".to_string()),
            Just("download.chunk_size_mb".to_string()),
            Just("ui.theme.dark_mode".to_string()),
            Just("ui.font.size".to_string()),
        ]
    }
}
```

## 性能・セキュリティ考慮事項

### 性能最適化
1. **設定キャッシュ**: メモリ内キャッシュによる高速アクセス
2. **差分更新**: 変更されたフィールドのみの部分更新
3. **非同期I/O**: tokio非同期ファイル操作による応答性向上
4. **設定圧縮**: バックアップファイルのGZIP圧縮

### セキュリティ強化
1. **機密情報暗号化**: OAuth秘密鍵の暗号化保存
2. **ファイル権限**: 設定ファイルの適切な権限設定
3. **整合性検証**: ハッシュによるファイル改ざん検出
4. **アトミック更新**: 設定変更の原子性保証

---

**承認**:  
**品質基準適合**: [ ] 確認済  
**ポリシー準拠**: [ ] 確認済  
**承認日**: ___________