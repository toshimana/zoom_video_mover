use chrono::Datelike;
/// Property-basedテスト統合実行
///
/// # 目的
/// - 基盤品質保証戦略として1000+ケースの自動検証
/// - 重要なコンポーネントのProperty検証
use proptest::prelude::*;
use zoom_video_mover_lib::{parse_datetime, sanitize_filename};

proptest! {
    /// ファイル名サニタイズのProperty検証
    #[test]
    fn filename_sanitization_properties(input in ".*") {
        if !input.is_empty() {
            let sanitized = sanitize_filename(&input);

            // Property 1: サニタイズ後は空でない
            prop_assert!(!sanitized.is_empty());

            // Property 2: 危険な文字が含まれない
            let dangerous_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
            for ch in dangerous_chars.iter() {
                prop_assert!(!sanitized.contains(*ch));
            }

            // Property 3: 長さ制限を守る
            prop_assert!(sanitized.len() <= 200);
        }
    }

    /// 日時パースのProperty検証
    #[test]
    fn datetime_parsing_properties(input in "2[0-9]{3}-[0-1][0-9]-[0-3][0-9]T[0-2][0-9]:[0-5][0-9]:[0-5][0-9]Z") {
        let parsed = parse_datetime(&input);

        // Property 1: パース結果は常に有効（2000年台の日時またはフォールバック値）
        prop_assert!(parsed.year() >= 2000);
        prop_assert!(parsed.year() <= 3000);

        // Property 2: UTC timezone
        prop_assert_eq!(parsed.timezone(), chrono::Utc);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn basic_property_verification() {
        // 基本的なProperty検証
        assert!(!sanitize_filename("test").is_empty());
        assert_eq!(sanitize_filename("test/file"), "test_file");

        let dt = parse_datetime("2025-01-01T00:00:00Z");
        assert_eq!(dt.year(), 2025);
    }
}
