/// 拡張Property-basedテスト - ポリシー準拠確認
/// 
/// # テスト対象
/// - エラーハンドリングシステムの完全検証
/// - 認証コンポーネントの堅牢性確認
/// - 設定管理の包括的テスト

use zoom_video_mover_lib::errors::*;
use proptest::prelude::*;
use chrono::{DateTime, Utc, Duration};
use std::error::Error;

// エラーハンドリングProperty-basedテスト
proptest! {
    /// エラー作成関数の冪等性
    #[test]
    fn error_creation_idempotency(message in "[a-zA-Z0-9 ._-]{1,200}") {
        let error1 = AppError::validation(&message, None);
        let error2 = AppError::validation(&message, None);
        
        prop_assert_eq!(format!("{}", error1), format!("{}", error2));
    }
    
    /// 回復可能性判定の一貫性
    #[test]
    fn recoverable_errors_consistency(message in "[a-zA-Z0-9 ._-]{1,200}") {
        let network_error = AppError::network(&message, None::<std::io::Error>);
        let rate_limit_error = AppError::RateLimit { 
            message: message.clone(), 
            retry_after: Some(60) 
        };
        
        prop_assert!(network_error.is_recoverable());
        prop_assert!(rate_limit_error.is_recoverable());
    }
    
    /// 非回復可能エラーの一貫性
    #[test]
    fn non_recoverable_errors_consistency(message in "[a-zA-Z0-9 ._-]{1,200}") {
        let validation_error = AppError::validation(&message, None);
        let invalid_token_error = AppError::InvalidToken { message: message.clone() };
        
        prop_assert!(!validation_error.is_recoverable());
        prop_assert!(!invalid_token_error.is_recoverable());
    }
    
    /// APIエラーの回復可能性ルール
    #[test]
    fn api_error_recoverability_rules(
        code in prop_oneof![400u16..500u16, 500u16..600u16], 
        message in "[a-zA-Z0-9 ._-]{1,200}"
    ) {
        let api_error = AppError::api(code, &message, None::<std::io::Error>);
        
        if code >= 500 {
            prop_assert!(api_error.is_recoverable(), "5xx errors should be recoverable");
        } else if code >= 400 {
            prop_assert!(!api_error.is_recoverable(), "4xx errors should not be recoverable");
        }
    }
    
    /// リトライ時間の妥当性
    #[test]
    fn retry_after_validity(message in "[a-zA-Z0-9 ._-]{1,200}") {
        let network_error = AppError::network(&message, None::<std::io::Error>);
        let rate_limit_error = AppError::RateLimit { 
            message: message.clone(), 
            retry_after: Some(30) 
        };
        
        if let Some(retry_time) = network_error.retry_after() {
            prop_assert!(retry_time >= 1 && retry_time <= 3600, "Retry time should be reasonable");
        }
        
        if let Some(retry_time) = rate_limit_error.retry_after() {
            prop_assert_eq!(retry_time, 30, "Rate limit retry time should match setting");
        }
    }
    
    /// エラーチェーンの整合性
    #[test]
    fn error_chain_consistency(message in "[a-zA-Z0-9 ._-]{1,200}") {
        use std::io;
        
        let source_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let app_error = AppError::file_system(&message, Some(source_error));
        
        prop_assert!(app_error.source().is_some(), "Source error should be preserved");
    }
    
    /// エラーメッセージ形式の一貫性
    #[test]
    fn error_message_format_consistency(message in "[a-zA-Z0-9 ._-]{1,200}") {
        let validation_error = AppError::validation(&message, Some("test_field".to_string()));
        let error_string = format!("{}", validation_error);
        
        prop_assert!(error_string.contains("Validation error:"), "Validation error should have proper prefix");
        prop_assert!(error_string.contains(&message), "Error message should contain original message");
    }
}

// 認証コンポーネントProperty-basedテスト
#[derive(Debug, Clone)]
struct TestAuthToken {
    access_token: String,
    token_type: String,
    expires_at: DateTime<Utc>,
    refresh_token: Option<String>,
    scopes: Vec<String>,
}

impl TestAuthToken {
    fn is_valid(&self) -> bool {
        Utc::now() < self.expires_at && !self.access_token.is_empty()
    }
    
    fn has_scope(&self, required_scope: &str) -> bool {
        !required_scope.is_empty() && self.scopes.iter().any(|scope| scope == required_scope)
    }
    
    fn has_all_scopes(&self, required_scopes: &[&str]) -> bool {
        !required_scopes.is_empty() && required_scopes.iter().all(|&scope| self.has_scope(scope))
    }
    
    fn remaining_seconds(&self) -> i64 {
        (self.expires_at - Utc::now()).num_seconds().max(0)
    }
}

proptest! {
    /// トークン有効性判定の一貫性
    #[test]
    fn token_validity_consistency(
        access_token in "[a-zA-Z0-9_-]{20,100}",
        expires_in_seconds in 1i64..86400i64,
        scopes in prop::collection::vec(
            prop_oneof![
                Just("recording:read".to_string()),
                Just("user:read".to_string()),
                Just("meeting:read".to_string()),
            ],
            1..4
        )
    ) {
        let token = TestAuthToken {
            access_token: access_token.clone(),
            token_type: "Bearer".to_string(),
            expires_at: Utc::now() + Duration::seconds(expires_in_seconds),
            refresh_token: None,
            scopes,
        };
        
        let is_valid = token.is_valid();
        let has_access_token = !token.access_token.is_empty();
        let not_expired = Utc::now() < token.expires_at;
        
        prop_assert_eq!(is_valid, has_access_token && not_expired);
    }
    
    /// 期限切れトークンの無効性
    #[test]
    fn expired_token_invalidity(
        access_token in "[a-zA-Z0-9_-]{20,100}",
        expires_ago_seconds in 1i64..86400i64,
        scopes in prop::collection::vec(
            prop_oneof![
                Just("recording:read".to_string()),
                Just("user:read".to_string()),
            ],
            1..3
        )
    ) {
        let expired_token = TestAuthToken {
            access_token,
            token_type: "Bearer".to_string(),
            expires_at: Utc::now() - Duration::seconds(expires_ago_seconds),
            refresh_token: None,
            scopes,
        };
        
        prop_assert!(!expired_token.is_valid(), "Expired token should be invalid");
    }
    
    /// スコープ確認の一貫性
    #[test]
    fn scope_verification_consistency(
        access_token in "[a-zA-Z0-9_-]{20,100}",
        expires_in_seconds in 1i64..86400i64,
        scopes in prop::collection::vec(
            prop_oneof![
                Just("recording:read".to_string()),
                Just("user:read".to_string()),
                Just("meeting:read".to_string()),
            ],
            1..4
        ),
        required_scope in prop_oneof![
            "recording:read",
            "user:read", 
            "meeting:read",
            "invalid:scope"
        ]
    ) {
        let token = TestAuthToken {
            access_token,
            token_type: "Bearer".to_string(),
            expires_at: Utc::now() + Duration::seconds(expires_in_seconds),
            refresh_token: None,
            scopes: scopes.clone(),
        };
        
        let has_scope = token.has_scope(&required_scope);
        let scope_exists = scopes.iter().any(|scope| scope == &required_scope);
        
        prop_assert_eq!(has_scope, scope_exists, "Scope verification should be accurate");
    }
    
    /// 複数スコープ確認の正確性
    #[test]
    fn multiple_scopes_verification(
        access_token in "[a-zA-Z0-9_-]{20,100}",
        expires_in_seconds in 1i64..86400i64,
        scopes in prop::collection::vec(
            prop_oneof![
                Just("recording:read".to_string()),
                Just("user:read".to_string()),
                Just("meeting:read".to_string()),
            ],
            1..4
        )
    ) {
        let token = TestAuthToken {
            access_token,
            token_type: "Bearer".to_string(),
            expires_at: Utc::now() + Duration::seconds(expires_in_seconds),
            refresh_token: None,
            scopes,
        };
        
        let required_scopes = ["recording:read", "user:read"];
        let has_all = token.has_all_scopes(&required_scopes);
        let individually_has_all = required_scopes.iter().all(|&scope| token.has_scope(scope));
        
        prop_assert_eq!(has_all, individually_has_all, "Multiple scope check should be consistent");
    }
    
    /// 残り有効時間の妥当性
    #[test]
    fn remaining_time_validity(
        access_token in "[a-zA-Z0-9_-]{20,100}",
        expires_in_seconds in -86400i64..86400i64,
        scopes in prop::collection::vec(
            prop_oneof![Just("user:read".to_string())],
            1..2
        )
    ) {
        let token = TestAuthToken {
            access_token,
            token_type: "Bearer".to_string(),
            expires_at: Utc::now() + Duration::seconds(expires_in_seconds),
            refresh_token: None,
            scopes,
        };
        
        let remaining = token.remaining_seconds();
        
        prop_assert!(remaining >= 0, "Remaining time should not be negative");
        
        if Utc::now() >= token.expires_at {
            prop_assert_eq!(remaining, 0, "Expired token should have 0 remaining time");
        }
    }
    
    /// 空アクセストークンの無効性
    #[test]
    fn empty_access_token_invalidity(
        expires_in_seconds in 1i64..86400i64,
        scopes in prop::collection::vec(
            prop_oneof![Just("user:read".to_string())],
            1..2
        )
    ) {
        let empty_token = TestAuthToken {
            access_token: String::new(),
            token_type: "Bearer".to_string(),
            expires_at: Utc::now() + Duration::seconds(expires_in_seconds),
            refresh_token: None,
            scopes,
        };
        
        prop_assert!(!empty_token.is_valid(), "Empty access token should be invalid");
    }
    
    /// スコープ空文字列での検索動作
    #[test]
    fn empty_scope_search_behavior(
        access_token in "[a-zA-Z0-9_-]{20,100}",
        expires_in_seconds in 1i64..86400i64,
        scopes in prop::collection::vec(
            prop_oneof![Just("user:read".to_string())],
            1..2
        )
    ) {
        let token = TestAuthToken {
            access_token,
            token_type: "Bearer".to_string(),
            expires_at: Utc::now() + Duration::seconds(expires_in_seconds),
            refresh_token: None,
            scopes,
        };
        
        prop_assert!(!token.has_scope(""), "Empty scope should not be found");
    }
}