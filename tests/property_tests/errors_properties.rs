/// エラーハンドリングシステムのProperty-basedテスト
/// 
/// # テスト対象プロパティ
/// - エラー作成関数の一貫性
/// - 回復可能性判定の正確性
/// - リトライ時間計算の妥当性

use zoom_video_mover_lib::errors::*;
use proptest::prelude::*;

/// エラーメッセージ生成器
pub fn arb_error_message() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 ._-]{1,200}"
}

/// HTTP ステータスコード生成器
pub fn arb_http_status_code() -> impl Strategy<Value = u16> {
    prop_oneof![
        400u16..500u16,  // Client errors
        500u16..600u16,  // Server errors
        200u16..300u16,  // Success codes
    ]
}

/// フィールド名生成器
pub fn arb_field_name() -> impl Strategy<Value = Option<String>> {
    prop_oneof![
        Just(None),
        Just(Some("client_id".to_string())),
        Just(Some("client_secret".to_string())),
        Just(Some("output_directory".to_string())),
    ]
}

proptest! {
    /// エラー作成関数の冪等性
    /// Property: 同じパラメータで複数回作成したエラーは等しい
    #[test]
    fn error_creation_idempotency(message in arb_error_message()) {
        let error1 = AppError::validation(&message, None);
        let error2 = AppError::validation(&message, None);
        
        // エラーメッセージが同じことを確認
        prop_assert_eq!(format!("{}", error1), format!("{}", error2));
    }
    
    /// 回復可能性判定の一貫性
    /// Property: 回復可能なエラータイプは常に true を返す
    #[test]
    fn recoverable_errors_consistency(message in arb_error_message()) {
        let network_error = AppError::network(&message, None::<std::io::Error>);
        let rate_limit_error = AppError::RateLimit { 
            message: message.clone(), 
            retry_after: Some(60) 
        };
        
        // ネットワークエラーとレート制限エラーは常に回復可能
        prop_assert!(network_error.is_recoverable());
        prop_assert!(rate_limit_error.is_recoverable());
    }
    
    /// 非回復可能エラーの一貫性
    /// Property: 非回復可能なエラータイプは常に false を返す
    #[test]
    fn non_recoverable_errors_consistency(message in arb_error_message(), field in arb_field_name()) {
        let validation_error = AppError::validation(&message, field);
        let invalid_token_error = AppError::InvalidToken { message: message.clone() };
        
        // バリデーションエラーと無効トークンエラーは回復不可能
        prop_assert!(!validation_error.is_recoverable());
        prop_assert!(!invalid_token_error.is_recoverable());
    }
    
    /// APIエラーの回復可能性ルール
    /// Property: 5xx系エラーは回復可能、4xx系エラーは回復不可能
    #[test]
    fn api_error_recoverability_rules(code in arb_http_status_code(), message in arb_error_message()) {
        let api_error = AppError::api(code, &message, None::<std::io::Error>);
        
        if code >= 500 {
            prop_assert!(api_error.is_recoverable(), "5xx errors should be recoverable");
        } else {
            prop_assert!(!api_error.is_recoverable(), "4xx errors should not be recoverable");
        }
    }
    
    /// リトライ時間の妥当性
    /// Property: リトライ時間は0以上の妥当な値である
    #[test]
    fn retry_after_validity(message in arb_error_message()) {
        let network_error = AppError::network(&message, None::<std::io::Error>);
        let rate_limit_error = AppError::RateLimit { 
            message: message.clone(), 
            retry_after: Some(30) 
        };
        
        // リトライ時間は妥当な範囲内
        if let Some(retry_time) = network_error.retry_after() {
            prop_assert!(retry_time >= 1 && retry_time <= 3600, "Retry time should be reasonable");
        }
        
        if let Some(retry_time) = rate_limit_error.retry_after() {
            prop_assert_eq!(retry_time, 30, "Rate limit retry time should match setting");
        }
    }
    
    /// エラーチェーンの整合性
    /// Property: ソースエラーがある場合、エラーチェーンが正しく構築される
    #[test]
    fn error_chain_consistency(message in arb_error_message()) {
        use std::io;
        
        let source_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let app_error = AppError::file_system(&message, Some(source_error));
        
        // ソースエラーが設定されていることを確認
        prop_assert!(app_error.source().is_some(), "Source error should be preserved");
    }
    
    /// エラーメッセージ形式の一貫性
    /// Property: エラーメッセージは一貫した形式で出力される
    #[test]
    fn error_message_format_consistency(message in arb_error_message()) {
        let validation_error = AppError::validation(&message, Some("test_field".to_string()));
        let error_string = format!("{}", validation_error);
        
        // バリデーションエラーのメッセージ形式確認
        prop_assert!(error_string.contains("Validation error:"), "Validation error should have proper prefix");
        prop_assert!(error_string.contains(&message), "Error message should contain original message");
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    
    #[test]
    fn test_error_categories() {
        // 各エラーカテゴリの基本動作確認
        let network_err = AppError::network("Connection failed", None::<std::io::Error>);
        let auth_err = AppError::authentication("Invalid credentials", None::<std::io::Error>);
        let validation_err = AppError::validation("Invalid input", Some("field".to_string()));
        
        assert!(network_err.is_recoverable());
        assert!(!auth_err.is_recoverable());
        assert!(!validation_err.is_recoverable());
        
        assert!(network_err.retry_after().is_some());
        assert!(auth_err.retry_after().is_none());
        assert!(validation_err.retry_after().is_none());
    }
}