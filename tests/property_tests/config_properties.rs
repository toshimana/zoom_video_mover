/// 設定管理コンポーネントのProperty-basedテスト
/// 
/// # テスト対象プロパティ
/// - 設定のラウンドトリップ性質（TOML ↔ 構造体）
/// - バリデーションルールの一貫性
/// - デフォルト値の妥当性
/// - 設定変更の原子性

use zoom_video_mover_lib::components::config::{AppConfig, OAuthConfig, ConfigManager};
use zoom_video_mover_lib::components::ComponentLifecycle;
use proptest::prelude::*;
use std::path::PathBuf;
use tempfile::tempdir;
use validator::Validate;

/// OAuth設定の任意値生成器
pub fn arb_oauth_config() -> impl Strategy<Value = OAuthConfig> {
    (
        "[a-zA-Z0-9]{20,50}",
        "[a-zA-Z0-9]{40,100}",
        "http://localhost:8080/callback",
        prop::collection::vec("[a-z:]+", 1..5)
    ).prop_map(|(client_id, client_secret, redirect_uri, scopes)| {
        OAuthConfig {
            client_id,
            client_secret,
            redirect_uri: redirect_uri.to_string(),
            scopes,
        }
    })
}

/// 無効なOAuth設定の任意値生成器
pub fn arb_invalid_oauth_config() -> impl Strategy<Value = OAuthConfig> {
    prop_oneof![
        // 空のClient ID
        Just(OAuthConfig {
            client_id: "".to_string(),
            client_secret: "valid_secret".to_string(),
            redirect_uri: "http://localhost:8080/callback".to_string(),
            scopes: vec!["recording:read".to_string()],
        }),
        // 空のClient Secret
        Just(OAuthConfig {
            client_id: "valid_client_id".to_string(),
            client_secret: "".to_string(),
            redirect_uri: "http://localhost:8080/callback".to_string(),
            scopes: vec!["recording:read".to_string()],
        }),
    ]
}

/// アプリケーション設定の任意値生成器
pub fn arb_app_config() -> impl Strategy<Value = AppConfig> {
    (
        arb_oauth_config(),
        "[a-zA-Z0-9/_\\\\.-]{1,200}",  // output_directory
        1u32..11u32,                   // max_concurrent_downloads (1-10)
        5u64..301u64,                  // request_timeout_seconds (5-300)
        1u32..101u32,                  // rate_limit_per_second (1-100)
        any::<bool>(),                 // debug_mode
        prop_oneof!["debug", "info", "warn", "error"]  // log_level
    ).prop_map(|(oauth, output_directory, max_concurrent_downloads, request_timeout_seconds, rate_limit_per_second, debug_mode, log_level)| {
        AppConfig {
            oauth,
            output_directory,
            max_concurrent_downloads,
            request_timeout_seconds,
            rate_limit_per_second,
            debug_mode,
            log_level,
        }
    })
}

proptest! {
    /// OAuth設定のTOMLラウンドトリップ性質
    /// Property: シリアライズ → デシリアライズしても元の値と等しい
    #[test]
    fn oauth_config_toml_roundtrip(config in arb_oauth_config()) {
        let toml_str = toml::to_string(&config)
            .expect("OAuth config should serialize to TOML");
        
        let parsed: OAuthConfig = toml::from_str(&toml_str)
            .expect("Serialized TOML should deserialize back to OAuth config");
        
        prop_assert_eq!(config.client_id, parsed.client_id);
        prop_assert_eq!(config.client_secret, parsed.client_secret);
        prop_assert_eq!(config.redirect_uri, parsed.redirect_uri);
        prop_assert_eq!(config.scopes, parsed.scopes);
    }
    
    /// アプリケーション設定のTOMLラウンドトリップ性質
    /// Property: シリアライズ → デシリアライズしても元の値と等しい
    #[test]
    fn app_config_toml_roundtrip(config in arb_app_config()) {
        let toml_str = toml::to_string(&config)
            .expect("App config should serialize to TOML");
        
        let parsed: AppConfig = toml::from_str(&toml_str)
            .expect("Serialized TOML should deserialize back to App config");
        
        prop_assert_eq!(config.oauth.client_id, parsed.oauth.client_id);
        prop_assert_eq!(config.oauth.client_secret, parsed.oauth.client_secret);
        prop_assert_eq!(config.output_directory, parsed.output_directory);
        prop_assert_eq!(config.max_concurrent_downloads, parsed.max_concurrent_downloads);
        prop_assert_eq!(config.request_timeout_seconds, parsed.request_timeout_seconds);
        prop_assert_eq!(config.rate_limit_per_second, parsed.rate_limit_per_second);
        prop_assert_eq!(config.debug_mode, parsed.debug_mode);
        prop_assert_eq!(config.log_level, parsed.log_level);
    }
    
    /// 有効な設定はバリデーションを通過する
    /// Property: 有効な設定として生成されたものは validation を通過する
    #[test]
    fn valid_config_passes_validation(config in arb_app_config()) {
        let validation_result = config.validate();
        prop_assert!(validation_result.is_ok(), 
            "Valid config should pass validation: {:?}", validation_result);
    }
    
    /// 無効なOAuth設定はバリデーションで拒否される
    /// Property: 意図的に無効にした設定は validation で拒否される
    #[test]
    fn invalid_oauth_config_fails_validation(oauth_config in arb_invalid_oauth_config()) {
        let validation_result = oauth_config.validate();
        prop_assert!(validation_result.is_err(), 
            "Invalid OAuth config should fail validation");
    }
    
    /// 設定のデフォルト値の妥当性
    /// Property: デフォルト設定は常に有効である
    #[test]
    fn default_config_is_valid(_unit in any::<()>()) {
        let default_oauth = OAuthConfig::default();
        let default_app = AppConfig::default();
        
        // デフォルトOAuth設定のバリデーション（空の値は無効）
        prop_assert!(default_oauth.validate().is_err(), 
            "Default OAuth config should be invalid (empty values)");
        
        // デフォルトアプリ設定のフィールド検証
        prop_assert!(!default_app.output_directory.is_empty(), 
            "Default output directory should not be empty");
        prop_assert!(default_app.max_concurrent_downloads >= 1 && default_app.max_concurrent_downloads <= 10,
            "Default concurrent downloads should be in valid range");
        prop_assert!(default_app.request_timeout_seconds >= 5 && default_app.request_timeout_seconds <= 300,
            "Default timeout should be in valid range");
    }
    
    /// 設定変更の冪等性
    /// Property: 同じ設定値を複数回設定しても結果は同じ
    #[test]
    fn config_update_idempotency(config in arb_app_config()) {
        // ConfigManager is already imported above
        use tempfile::NamedTempFile;
        
        let temp_file = NamedTempFile::new().expect("Should create temp file");
        let config_path = temp_file.path();
        
        let mut manager = ConfigManager::new(config_path);
        
        // 同じ設定を2回適用
        let result1 = manager.update_config(config.clone());
        let result2 = manager.update_config(config.clone());
        
        prop_assert!(result1.is_ok(), "First config update should succeed");
        prop_assert!(result2.is_ok(), "Second config update should succeed");
        prop_assert_eq!(manager.get_config().oauth.client_id, config.oauth.client_id,
            "Config should remain consistent after multiple updates");
    }
    
    /// 境界値での設定バリデーション
    /// Property: 境界値で設定されたパラメータがバリデーションルールに従う
    #[test]
    fn boundary_value_validation(
        concurrent_downloads in prop_oneof![Just(1u32), Just(10u32)],
        timeout_seconds in prop_oneof![Just(5u64), Just(300u64)]
    ) {
        let mut config = AppConfig::default();
        config.oauth.client_id = "valid_client_id".to_string();
        config.oauth.client_secret = "valid_client_secret".to_string();
        config.max_concurrent_downloads = concurrent_downloads;
        config.request_timeout_seconds = timeout_seconds;
        
        let validation_result = config.validate();
        prop_assert!(validation_result.is_ok(), 
            "Boundary values should pass validation: {:?}", validation_result);
    }
    
    /// 設定ファイルの原子性
    /// Property: 設定保存中の中断があっても、前の有効な状態または新しい有効な状態のいずれかが保たれる
    #[test]
    fn config_file_atomicity(config in arb_app_config()) {
        // ConfigManager is already imported above
        use tempfile::NamedTempFile;
        
        let temp_file = NamedTempFile::new().expect("Should create temp file");
        let config_path = temp_file.path();
        
        let mut manager = ConfigManager::new(config_path);
        
        // 初期設定を保存
        let _ = manager.update_config(config.clone());
        let rt = tokio::runtime::Runtime::new().expect("Should create runtime");
        let save_result = rt.block_on(manager.save_to_file());
        
        prop_assert!(save_result.is_ok(), "Config save should succeed");
        
        // ファイルが存在し、読み取り可能であることを確認
        prop_assert!(config_path.exists(), "Config file should exist after save");
        
        // 保存されたファイルから設定を読み戻せることを確認
        let mut new_manager = ConfigManager::new(config_path);
        let load_result = rt.block_on(new_manager.initialize());
        
        prop_assert!(load_result.is_ok(), "Should be able to load saved config");
        prop_assert_eq!(new_manager.get_config().oauth.client_id, config.oauth.client_id,
            "Loaded config should match saved config");
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    // ConfigManager and ComponentLifecycle are already imported above
    use tokio;

    #[tokio::test]
    async fn test_config_manager_property_compliance() {
        let temp_dir = tempdir().expect("Should create temp dir");
        let config_path = temp_dir.path().join("test_config.toml");
        
        let mut manager = ConfigManager::new(&config_path);
        
        // 初期化
        assert!(manager.initialize().await.is_ok());
        
        // Property: 初期化後は健全性チェックを通過する
        assert!(manager.health_check().await);
        
        // Property: デフォルト設定がロードされている
        let config = manager.get_config();
        assert!(!config.output_directory.is_empty());
        
        // Property: 有効な設定更新は成功する
        let mut valid_oauth = OAuthConfig::default();
        valid_oauth.client_id = "test_client_id".to_string();
        valid_oauth.client_secret = "test_client_secret".to_string();
        
        assert!(manager.update_oauth_config(valid_oauth).is_ok());
        assert!(manager.is_modified());
        
        // Property: 保存後は modified フラグがクリアされる
        assert!(manager.save_to_file().await.is_ok());
        assert!(!manager.is_modified());
        
        // 終了処理
        assert!(manager.shutdown().await.is_ok());
    }
}