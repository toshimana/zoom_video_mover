/// 日付・日時処理のProperty-basedテスト
/// 
/// # テスト対象プロパティ
/// - 実際の日付のみ生成される
/// - 日付範囲の順序が保持される
/// - うるう年判定の正確性

use proptest::prelude::*;
use chrono::{NaiveDate, Datelike};

/// 実際に存在する日付のみを生成するProperty-basedテスト
proptest! {
    /// 生成された日付が実際に存在することを検証
    #[test]
    fn generated_dates_are_actually_valid(date_str in crate::property_tests::arb_valid_date()) {
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
    fn date_range_always_ordered((from_date, to_date) in crate::property_tests::arb_date_range()) {
        let from_parsed = NaiveDate::parse_from_str(&from_date, "%Y-%m-%d").unwrap();
        let to_parsed = NaiveDate::parse_from_str(&to_date, "%Y-%m-%d").unwrap();
        
        prop_assert!(from_parsed <= to_parsed, 
            "From date {} should be <= to date {}", from_date, to_date);
    }
    
    /// 月ごとの日数制限が正しく適用されることを検証
    #[test]
    fn month_day_limits_are_respected(date_str in crate::property_tests::arb_valid_date()) {
        let parsed = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").unwrap();
        let year = parsed.year();
        let month = parsed.month();
        let day = parsed.day();
        
        let expected_max_day = match month {
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
        
        prop_assert!(day <= expected_max_day, 
            "Day {} should not exceed max day {} for month {} in year {}", 
            day, expected_max_day, month, year);
    }
    
    /// うるう年判定の正確性を検証
    #[test]
    fn leap_year_february_29_is_valid(year in 2020i32..2030i32) {
        let is_leap_year = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
        
        if is_leap_year {
            // うるう年の場合、2月29日が有効であることを確認
            let feb_29 = NaiveDate::from_ymd_opt(year, 2, 29);
            prop_assert!(feb_29.is_some(), "Leap year {} should have valid Feb 29", year);
        } else {
            // 平年の場合、2月29日が無効であることを確認
            let feb_29 = NaiveDate::from_ymd_opt(year, 2, 29);
            prop_assert!(feb_29.is_none(), "Non-leap year {} should not have Feb 29", year);
        }
    }
    
    /// 日付文字列の正規化が一貫していることを検証
    #[test]
    fn date_string_normalization_consistency(date_str in crate::property_tests::arb_valid_date()) {
        let parsed = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").unwrap();
        let reformatted = parsed.format("%Y-%m-%d").to_string();
        
        prop_assert_eq!(date_str, reformatted, 
            "Date string should be consistently formatted");
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    
    #[test]
    fn test_leap_year_logic() {
        // 既知のうるう年と平年でテスト
        assert!(2020 % 4 == 0 && (2020 % 100 != 0 || 2020 % 400 == 0)); // うるう年
        assert!(!(2021 % 4 == 0 && (2021 % 100 != 0 || 2021 % 400 == 0))); // 平年
        assert!(2000 % 4 == 0 && (2000 % 100 != 0 || 2000 % 400 == 0)); // うるう年（400で割り切れる）
        assert!(!(1900 % 4 == 0 && (1900 % 100 != 0 || 1900 % 400 == 0))); // 平年（100で割り切れるが400では割り切れない）
    }
    
    #[test]
    fn test_date_ranges() {
        // 基本的な日付範囲テスト
        let from_date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let to_date = NaiveDate::from_ymd_opt(2023, 12, 31).unwrap();
        
        assert!(from_date <= to_date);
        assert_eq!((to_date - from_date).num_days(), 364); // 2023年は平年
    }
}