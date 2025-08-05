/// Property-basedテスト基盤戦略実装
/// 
/// # 目的
/// - 1000+ケースの自動検証による網羅的品質保証
/// - データ整合性・不変条件・境界値の完全自動検証
/// - 手動テストでは困難な網羅的品質保証を実現

pub mod config_properties;
pub mod auth_properties;
pub mod date_properties;
pub mod errors_properties;

use proptest::prelude::*;

/// Property-basedテストの共通設定
pub struct PropertyTestConfig {
    /// テストケース数
    pub test_cases: u32,
    /// 最大シュリンク回数
    pub max_shrink_iters: u32,
    /// タイムアウト（ミリ秒）
    pub timeout_ms: u32,
}

impl Default for PropertyTestConfig {
    fn default() -> Self {
        Self {
            test_cases: 1000,
            max_shrink_iters: 100,
            timeout_ms: 5000,
        }
    }
}

/// 日付文字列生成器（YYYY-MM-DD形式）
/// 
/// # 生成ルール
/// - 2020年〜2030年の範囲
/// - 各月の適切な日数制限
/// - うるう年対応
pub fn arb_valid_date() -> impl Strategy<Value = String> {
    (2020i32..2030i32, 1u32..13u32, 0u32..31u32)
        .prop_map(|(year, month, day_offset)| {
            let max_day = match month {
                2 => {
                    if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
                        29 // うるう年の2月
                    } else {
                        28 // 平年の2月
                    }
                },
                4 | 6 | 9 | 11 => 30,
                _ => 31,
            };
            let day = (day_offset % max_day) + 1;
            format!("{:04}-{:02}-{:02}", year, month, day)
        })
}

/// 日付範囲生成器（from_date <= to_date）
pub fn arb_date_range() -> impl Strategy<Value = (String, String)> {
    arb_valid_date().prop_flat_map(|from_date| {
        let from_parsed = chrono::NaiveDate::parse_from_str(&from_date, "%Y-%m-%d")
            .expect("Generated date should be valid");
        
        // from_dateから1年以内の範囲で生成
        let max_days_ahead = 365;
        (0u32..max_days_ahead)
            .prop_map(move |days_offset| {
                let to_date = from_parsed + chrono::Duration::days(days_offset as i64);
                (from_date.clone(), to_date.format("%Y-%m-%d").to_string())
            })
    })
}

/// 有効なファイル名生成器
pub fn arb_valid_filename() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_\\-\\. ]{1,200}".prop_map(|s| s.trim().to_string())
        .prop_filter("non-empty after trim", |s| !s.is_empty())
}

/// 無効なファイル名生成器
pub fn arb_invalid_filename() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("".to_string()),                    // 空文字列
        Just("/\\:*?\"<>|".to_string()),        // 特殊文字のみ
        Just("CON".to_string()),                 // Windows予約名
        Just("a".repeat(300)),                   // 長すぎる名前
    ]
}

/// Client ID生成器
pub fn arb_client_id() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9]{20,50}"
}

/// Client Secret生成器  
pub fn arb_client_secret() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9]{40,100}"
}

/// 有効なURL生成器
pub fn arb_valid_url() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("http://localhost:8080/callback".to_string()),
        Just("https://example.com/oauth/callback".to_string()),
        Just("http://127.0.0.1:3000/auth".to_string()),
    ]
}

/// 無効なURL生成器
pub fn arb_invalid_url() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("".to_string()),
        Just("not-a-url".to_string()),
        Just("ftp://invalid".to_string()),
        Just("http://".to_string()),
    ]
}

/// Property-basedテストの実行マクロ
#[macro_export]
macro_rules! run_property_test {
    ($test_name:ident, $property:expr, $strategy:expr) => {
        proptest! {
            #![proptest_config(ProptestConfig {
                cases: 1000,
                max_shrink_iters: 100,
                timeout: 5000,
                .. ProptestConfig::default()
            })]
            
            #[test]
            fn $test_name(input in $strategy) {
                $property(input)?;
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    proptest! {
        /// 生成された日付が実際に存在することを検証
        #[test]
        fn generated_dates_are_actually_valid(date_str in arb_valid_date()) {
            // 実際の日付として解析可能であることを確認
            let parsed = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d");
            prop_assert!(parsed.is_ok(), "Generated date should be parseable: {}", date_str);
            
            // 日付形式の検証
            prop_assert_eq!(date_str.len(), 10, "Date should be YYYY-MM-DD format");
            prop_assert!(date_str.chars().nth(4) == Some('-'), "Year-month separator missing");
            prop_assert!(date_str.chars().nth(7) == Some('-'), "Month-day separator missing");
        }
        
        /// 日付範囲が常に順序を保つことを検証
        #[test]
        fn date_range_always_ordered((from_date, to_date) in arb_date_range()) {
            let from_parsed = NaiveDate::parse_from_str(&from_date, "%Y-%m-%d").unwrap();
            let to_parsed = NaiveDate::parse_from_str(&to_date, "%Y-%m-%d").unwrap();
            
            prop_assert!(from_parsed <= to_parsed, 
                "From date {} should be <= to date {}", from_date, to_date);
        }
        
        /// 有効なファイル名が実際に有効であることを検証
        #[test]
        fn valid_filenames_are_actually_valid(filename in arb_valid_filename()) {
            prop_assert!(!filename.is_empty(), "Valid filename should not be empty");
            prop_assert!(filename.len() <= 200, "Valid filename should not exceed 200 chars");
            
            // 禁止文字が含まれていないことを確認
            let forbidden_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
            for ch in forbidden_chars.iter() {
                prop_assert!(!filename.contains(*ch), 
                    "Valid filename should not contain forbidden char: {}", ch);
            }
        }
    }
}