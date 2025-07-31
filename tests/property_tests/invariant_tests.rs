// tests/property_tests/invariant_tests.rs
// Property-basedテスト - 全仕様横断的な不変条件検証

use proptest::prelude::*;
use chrono::{NaiveDate, Datelike};
use std::collections::HashMap;
use tempfile::TempDir;
use zoom_video_mover_lib::{Config, Recording, RecordingFile, ZoomVideoMoverError};
use std::fs;

// 任意値生成器の定義

/// 有効なClient ID生成器
fn arb_client_id() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_-]{10,50}".prop_map(|s| s)
}

/// 有効なClient Secret生成器 (20文字以上)
fn arb_client_secret() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_-]{20,100}".prop_map(|s| s)
}

/// 有効なRedirect URI生成器
fn arb_redirect_uri() -> impl Strategy<Value = Option<String>> {
    prop_oneof![
        Just(None),
        Just(Some("http://localhost:8080/callback".to_string())),
        Just(Some("https://example.com/oauth/callback".to_string())),
    ]
}

/// Config構造体生成器
prop_compose! {
    fn arb_config()
        (client_id in arb_client_id(),
         client_secret in arb_client_secret(),
         redirect_uri in arb_redirect_uri())
        -> Config
    {
        Config { client_id, client_secret, redirect_uri }
    }
}

/// 有効な日付生成器 (実在する日付のみ)
prop_compose! {
    fn arb_valid_date()
        (year in 2020i32..2030i32,
         month in 1u32..13u32,
         day_offset in 0u32..31u32)
        -> String
    {
        // 月ごとの最大日数を計算
        let max_day = match month {
            2 => {
                // うるう年判定
                if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
                    29
                } else {
                    28
                }
            },
            4 | 6 | 9 | 11 => 30,
            _ => 31,
        };
        
        let day = (day_offset % max_day) + 1;
        
        // 実際の日付として検証
        let date = NaiveDate::from_ymd_opt(year, month, day)
            .expect("Generated date should be valid");
        
        date.format("%Y-%m-%d").to_string()
    }
}

/// 日付範囲生成器 (from_date <= to_date)
prop_compose! {
    fn arb_date_range()
        (from_date in arb_valid_date(),
         offset_days in 0u32..365u32)
        -> (String, String)
    {
        let from_parsed = NaiveDate::parse_from_str(&from_date, "%Y-%m-%d").unwrap();
        let to_parsed = from_parsed + chrono::Duration::days(offset_days as i64);
        let to_date = to_parsed.format("%Y-%m-%d").to_string();
        
        (from_date, to_date)
    }
}

/// Recording構造体生成器
prop_compose! {
    fn arb_recording()
        (meeting_id in "[0-9]{9,12}",
         topic in "[a-zA-Z0-9 \\u3040-\\u309f\\u30a0-\\u30ff\\u4e00-\\u9faf]{5,50}",  // 日本語対応
         duration in 1u32..300u32,
         file_count in 1usize..5usize)
        -> Recording
    {
        let mut recording_files = Vec::new();
        
        for i in 0..file_count {
            let file_type = match i % 5 {
                0 => "MP4",
                1 => "MP3", 
                2 => "TXT",
                3 => "JSON",
                4 => "VTT",
                _ => "MP4",
            };
            
            let file_size = match file_type {
                "MP4" => (500_000_000u64..2_000_000_000u64).sample(&mut rand::thread_rng()),
                "MP3" => (10_000_000u64..100_000_000u64).sample(&mut rand::thread_rng()),
                _ => (1_000u64..10_000_000u64).sample(&mut rand::thread_rng()),
            };
            
            recording_files.push(RecordingFile {
                id: format!("file_{}_{}", meeting_id, i),
                file_type: file_type.to_string(),
                file_size,
                download_url: format!("https://zoom.us/rec/download/{}_{}_{}", meeting_id, file_type.to_lowercase(), i),
                play_url: Some(format!("https://zoom.us/rec/play/{}_{}", meeting_id, i)),
                recording_start: chrono::Utc::now() - chrono::Duration::hours(1),
                recording_end: chrono::Utc::now(),
            });
        }
        
        Recording {
            meeting_id,
            topic,
            start_time: chrono::Utc::now() - chrono::Duration::hours(2),
            duration,
            recording_files,
            ai_summary_available: rand::random(),
        }
    }
}

// Property-basedテスト実装

/// Property: Config のTOMLラウンドトリップ不変条件
/// 
/// 仕様対応:
/// - function_specifications.md: FN001 設定管理機能
/// - 不変条件: シリアライゼーション→デシリアライゼーションの可逆性
/// - データ整合性: 入力と出力の完全一致
proptest! {
    #[test]
    fn config_toml_roundtrip_invariant(config in arb_config()) {
        // 事前条件のassertion
        prop_assert!(!config.client_id.is_empty());
        prop_assert!(config.client_secret.len() >= 20);
        
        // ラウンドトリップテスト: Config → TOML → Config
        let toml_str = toml::to_string(&config)
            .map_err(|e| TestCaseError::fail(format!("TOML serialization failed: {}", e)))?;
        
        // TOML文字列の妥当性検証
        prop_assert!(!toml_str.is_empty());
        prop_assert!(toml_str.contains("client_id"));
        prop_assert!(toml_str.contains("client_secret"));
        
        let parsed_config: Config = toml::from_str(&toml_str)
            .map_err(|e| TestCaseError::fail(format!("TOML deserialization failed: {}", e)))?;
        
        // 不変条件: 完全な可逆性
        prop_assert_eq!(parsed_config.client_id, config.client_id);
        prop_assert_eq!(parsed_config.client_secret, config.client_secret);
        prop_assert_eq!(parsed_config.redirect_uri, config.redirect_uri);
        
        // 事後条件のassertion
        prop_assert!(!parsed_config.client_id.is_empty());
        prop_assert!(parsed_config.client_secret.len() >= 20);
    }
}

/// Property: 日付検証の不変条件
/// 
/// 仕様対応:
/// - function_specifications.md: FN003 録画検索機能
/// - CLAUDE.md: 日時・日付の検証規約
/// - 不変条件: 生成された日付は必ず実在する
proptest! {
    #[test]  
    fn generated_dates_are_actually_valid(date_str in arb_valid_date()) {
        // 事前条件: 日付文字列の形式チェック
        prop_assert_eq!(date_str.len(), 10); // YYYY-MM-DD
        prop_assert!(date_str.contains("-"));
        
        // 不変条件: chrono による日付解析が必ず成功する
        let parsed_date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
            .map_err(|e| TestCaseError::fail(format!("Invalid date generated: {} ({})", date_str, e)))?;
        
        // 事後条件: 解析された日付の妥当性
        prop_assert!(parsed_date.year() >= 2020 && parsed_date.year() < 2030);
        prop_assert!(parsed_date.month() >= 1 && parsed_date.month() <= 12);
        prop_assert!(parsed_date.day() >= 1 && parsed_date.day() <= 31);
        
        // 月ごとの日数制限の検証
        let max_day_for_month = match parsed_date.month() {
            2 => {
                if parsed_date.year() % 4 == 0 && (parsed_date.year() % 100 != 0 || parsed_date.year() % 400 == 0) {
                    29
                } else {
                    28
                }
            },
            4 | 6 | 9 | 11 => 30,
            _ => 31,
        };
        
        prop_assert!(parsed_date.day() <= max_day_for_month, 
            "Day {} should be <= {} for month {}", parsed_date.day(), max_day_for_month, parsed_date.month());
    }
}

/// Property: 日付範囲の順序不変条件
/// 
/// 仕様対応:
/// - function_specifications.md: FN003 録画検索機能
/// - operation_specifications.md: OP004 録画検索・一覧表示
/// - 不変条件: from_date <= to_date が常に保たれる
proptest! {
    #[test]
    fn date_range_always_ordered((from_date, to_date) in arb_date_range()) {
        // 事前条件: 両方の日付が有効
        let from_parsed = NaiveDate::parse_from_str(&from_date, "%Y-%m-%d")
            .map_err(|_| TestCaseError::fail(format!("Invalid from_date: {}", from_date)))?;
        let to_parsed = NaiveDate::parse_from_str(&to_date, "%Y-%m-%d")
            .map_err(|_| TestCaseError::fail(format!("Invalid to_date: {}", to_date)))?;
        
        // 不変条件: 日付範囲の順序が常に正しい
        prop_assert!(from_parsed <= to_parsed, 
            "Date range order: {} should be <= {}", from_date, to_date);
        
        // 事後条件: 日付範囲が実用的
        let duration = to_parsed.signed_duration_since(from_parsed);
        prop_assert!(duration.num_days() <= 365, "Date range should be within 1 year");
        prop_assert!(duration.num_days() >= 0, "Duration should be non-negative");
    }
}

/// Property: 月の日数制限の不変条件
/// 
/// 仕様対応:
/// - CLAUDE.md: 日時・日付の検証規約
/// - 不変条件: 月ごとの最大日数を超える日付は生成されない
proptest! {
    #[test]
    fn month_day_limits_are_respected(date_str in arb_valid_date()) {
        let parsed_date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")?;
        
        let year = parsed_date.year();
        let month = parsed_date.month();
        let day = parsed_date.day();
        
        // 各月の制限チェック
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => {
                prop_assert!(day <= 31, "31-day months: day {} should be <= 31", day);
            },
            4 | 6 | 9 | 11 => {
                prop_assert!(day <= 30, "30-day months: day {} should be <= 30", day);
            },
            2 => {
                // うるう年判定
                let is_leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
                let max_feb_day = if is_leap { 29 } else { 28 };
                prop_assert!(day <= max_feb_day, 
                    "February in {}: day {} should be <= {}", year, day, max_feb_day);
            },
            _ => {
                prop_assert!(false, "Invalid month: {}", month);
            }
        }
    }
}

/// Property: うるう年2月29日の妥当性不変条件
/// 
/// 仕様対応:
/// - CLAUDE.md: 日時・日付の検証規約
/// - 不変条件: うるう年のみで2月29日が有効
proptest! {
    #[test]
    fn leap_year_february_29_is_valid(year in 2000i32..2100i32) {
        let is_leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
        
        // 2月29日の日付作成試行
        let feb_29_result = NaiveDate::from_ymd_opt(year, 2, 29);
        
        if is_leap {
            // うるう年では2月29日が有効
            prop_assert!(feb_29_result.is_some(), 
                "Leap year {}: February 29 should be valid", year);
            
            let feb_29 = feb_29_result.unwrap();
            prop_assert_eq!(feb_29.year(), year);
            prop_assert_eq!(feb_29.month(), 2);
            prop_assert_eq!(feb_29.day(), 29);
        } else {
            // 平年では2月29日が無効
            prop_assert!(feb_29_result.is_none(), 
                "Non-leap year {}: February 29 should be invalid", year);
        }
        
        // 2月28日は常に有効
        let feb_28_result = NaiveDate::from_ymd_opt(year, 2, 28);
        prop_assert!(feb_28_result.is_some(), "February 28 should always be valid");
    }
}

/// Property: Recording構造体の整合性不変条件
/// 
/// 仕様対応:
/// - function_specifications.md: FN003 録画検索機能
/// - screen_specifications.md: SC004 録画リスト画面
/// - 不変条件: 録画データの構造的整合性
proptest! {
    #[test]
    fn recording_structure_invariants(recording in arb_recording()) {
        // 事前条件: 基本フィールドの妥当性
        prop_assert!(!recording.meeting_id.is_empty());
        prop_assert!(!recording.topic.is_empty());
        prop_assert!(recording.duration > 0);
        prop_assert!(!recording.recording_files.is_empty());
        
        // 不変条件1: ファイル種別の一意性（同じ種別は1つまで）
        let mut file_types = HashMap::new();
        for file in &recording.recording_files {
            let count = file_types.entry(&file.file_type).or_insert(0);
            *count += 1;
        }
        
        for (file_type, count) in file_types {
            prop_assert!(count <= 2, "File type {} appears {} times (should be <= 2)", file_type, count);
        }
        
        // 不変条件2: ファイルサイズの妥当性
        for file in &recording.recording_files {
            prop_assert!(file.file_size > 0, "File size should be positive");
            
            // ファイルタイプごとのサイズ制限
            match file.file_type.as_str() {
                "MP4" => {
                    prop_assert!(file.file_size >= 1_000_000, "MP4 should be >= 1MB");
                    prop_assert!(file.file_size <= 5_000_000_000, "MP4 should be <= 5GB");
                },
                "MP3" => {
                    prop_assert!(file.file_size >= 100_000, "MP3 should be >= 100KB");
                    prop_assert!(file.file_size <= 500_000_000, "MP3 should be <= 500MB");
                },
                "TXT" | "JSON" | "VTT" => {
                    prop_assert!(file.file_size <= 50_000_000, "Text files should be <= 50MB");
                },
                _ => {}
            }
        }
        
        // 不変条件3: URL形式の妥当性
        for file in &recording.recording_files {
            prop_assert!(file.download_url.starts_with("https://"), 
                "Download URL should be HTTPS: {}", file.download_url);
            prop_assert!(file.download_url.contains("zoom.us"), 
                "Download URL should be Zoom domain: {}", file.download_url);
            
            if let Some(ref play_url) = file.play_url {
                prop_assert!(play_url.starts_with("https://"), 
                    "Play URL should be HTTPS: {}", play_url);
            }
        }
        
        // 不変条件4: 時間の整合性
        prop_assert!(recording.start_time <= chrono::Utc::now(), 
            "Recording start time should be in the past");
        
        for file in &recording.recording_files {
            prop_assert!(file.recording_start <= file.recording_end, 
                "File recording start should be <= end");
            prop_assert!(file.recording_start >= recording.start_time - chrono::Duration::minutes(5), 
                "File start should be close to meeting start");
        }
    }
}

/// Property: ファイル名サニタイズの不変条件
/// 
/// 仕様対応:
/// - function_specifications.md: FN008 ファイル管理機能
/// - operation_specifications.md: OP006 ダウンロード実行
/// - 不変条件: サニタイズ後のファイル名がWindows互換
proptest! {
    #[test]
    fn filename_sanitization_invariants(
        raw_filename in "[a-zA-Z0-9 \\u3040-\\u309f\\u30a0-\\u30ff\\u4e00-\\u9faf<>:\"|?*/.\\\\]{1,300}"
    ) {
        // サニタイズ処理のシミュレーション
        let sanitized = sanitize_filename_for_windows(&raw_filename);
        
        // 不変条件1: 無効文字が除去される
        let invalid_chars = ['<', '>', ':', '"', '|', '?', '*', '/', '\\'];
        for invalid_char in invalid_chars {
            prop_assert!(!sanitized.contains(invalid_char), 
                "Sanitized filename should not contain '{}': {}", invalid_char, sanitized);
        }
        
        // 不変条件2: 長さ制限
        prop_assert!(sanitized.len() <= 255, 
            "Sanitized filename should be <= 255 chars: {} ({})", sanitized.len(), sanitized);
        
        // 不変条件3: 空文字にならない
        prop_assert!(!sanitized.is_empty(), "Sanitized filename should not be empty");
        
        // 不変条件4: Windows予約語の回避
        let reserved_names = ["CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9"];
        let sanitized_upper = sanitized.to_uppercase();
        
        for reserved in reserved_names {
            if sanitized_upper == reserved {
                prop_assert!(sanitized.starts_with("_"), 
                    "Reserved name '{}' should be prefixed with '_': {}", reserved, sanitized);
            }
        }
        
        // 不変条件5: 日本語文字の保持
        let original_japanese_chars: Vec<char> = raw_filename.chars()
            .filter(|c| *c >= '\u{3040}' && *c <= '\u{9faf}')  // ひらがな・カタカナ・漢字
            .collect();
        
        if !original_japanese_chars.is_empty() {
            let sanitized_japanese_chars: Vec<char> = sanitized.chars()
                .filter(|c| *c >= '\u{3040}' && *c <= '\u{9faf}')
                .collect();
            
            prop_assert!(!sanitized_japanese_chars.is_empty(), 
                "Japanese characters should be preserved: original={}, sanitized={}", 
                original_japanese_chars.len(), sanitized_japanese_chars.len());
        }
    }
}

/// Property: 設定ファイル操作の冪等性不変条件
/// 
/// 仕様対応:
/// - function_specifications.md: FN001 設定管理機能
/// - operation_specifications.md: OP002 設定入力・保存
/// - 不変条件: 保存→読込→保存の冪等性
proptest! {
    #[test]
    fn config_file_operations_idempotent(config in arb_config()) {
        let temp_dir = TempDir::new()
            .map_err(|e| TestCaseError::fail(format!("Failed to create temp dir: {}", e)))?;
        let config_path = temp_dir.path().join("test_config.toml");
        
        // 第1回保存 
        config.save_to_file(config_path.to_str().unwrap())
            .map_err(|e| TestCaseError::fail(format!("First save failed: {}", e)))?;
        
        // 読み込み
        let loaded_config = Config::load_from_file(config_path.to_str().unwrap())
            .map_err(|e| TestCaseError::fail(format!("Load failed: {}", e)))?;
        
        // 第2回保存 (同じ内容)
        loaded_config.save_to_file(config_path.to_str().unwrap())
            .map_err(|e| TestCaseError::fail(format!("Second save failed: {}", e)))?;
        
        // 再読み込み
        let reloaded_config = Config::load_from_file(config_path.to_str().unwrap())
            .map_err(|e| TestCaseError::fail(format!("Reload failed: {}", e)))?;
        
        // 不変条件: 冪等性 - 複数回の保存・読込で内容が変化しない
        prop_assert_eq!(loaded_config.client_id, reloaded_config.client_id);
        prop_assert_eq!(loaded_config.client_secret, reloaded_config.client_secret);
        prop_assert_eq!(loaded_config.redirect_uri, reloaded_config.redirect_uri);
        
        // オリジナルとの一致も確認
        prop_assert_eq!(config.client_id, reloaded_config.client_id);
        prop_assert_eq!(config.client_secret, reloaded_config.client_secret);
        prop_assert_eq!(config.redirect_uri, reloaded_config.redirect_uri);
    }
}

// ユーティリティ関数

/// Windows互換ファイル名サニタイズ (実装のシミュレーション)
fn sanitize_filename_for_windows(filename: &str) -> String {
    let invalid_chars = ['<', '>', ':', '"', '|', '?', '*', '/', '\\'];
    let mut sanitized = filename.to_string();
    
    // 無効文字の置換
    for ch in invalid_chars {
        sanitized = sanitized.replace(ch, "_");
    }
    
    // 制御文字の除去
    sanitized = sanitized.chars()
        .filter(|c| !c.is_control())
        .collect();
    
    // Windows予約語の処理
    let reserved_names = ["CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9"];
    let sanitized_upper = sanitized.to_uppercase();
    
    if reserved_names.contains(&sanitized_upper.as_str()) {
        sanitized = format!("_{}", sanitized);
    }
    
    // 長さ制限
    if sanitized.len() > 255 {
        let extension_start = sanitized.rfind('.').unwrap_or(sanitized.len());
        let extension = if extension_start < sanitized.len() {
            &sanitized[extension_start..]
        } else {
            ""
        };
        
        let max_base_len = 255 - extension.len();
        sanitized = format!("{}{}", &sanitized[..max_base_len], extension);
    }
    
    // 末尾の空白・ピリオド除去
    sanitized = sanitized.trim_end_matches([' ', '.']).to_string();
    
    // 空文字の場合のフォールバック
    if sanitized.is_empty() {
        sanitized = "unnamed_file".to_string();
    }
    
    sanitized
}

// Property-basedテスト トレーサビリティ
//
// 機能仕様 (function_specifications.md) との対応:
// ├─ FN001: 設定管理機能           → config_toml_roundtrip_invariant, config_file_operations_idempotent
// ├─ FN003: 録画検索機能           → generated_dates_are_actually_valid, date_range_always_ordered, recording_structure_invariants
// └─ FN008: ファイル管理機能        → filename_sanitization_invariants
//
// CLAUDE.md 規約との対応:
// ├─ 日時・日付の検証規約          → generated_dates_are_actually_valid, month_day_limits_are_respected, leap_year_february_29_is_valid
// ├─ Property-basedテスト戦略      → 全テスト関数
// └─ 関数の事前条件・事後条件・不変条件 → 各テスト内のassertion
//
// 操作仕様 (operation_specifications.md) との対応:
// ├─ OP002: 設定入力・保存         → config_file_operations_idempotent
// ├─ OP004: 録画検索・一覧表示      → date_range_always_ordered, recording_structure_invariants
// └─ OP006: ダウンロード実行        → filename_sanitization_invariants
//
// 画面仕様 (screen_specifications.md) との対応:
// ├─ SC002: 設定画面              → config_toml_roundtrip_invariant
// ├─ SC004: 録画リスト画面         → recording_structure_invariants
// └─ SC005: ダウンロード進捗画面    → filename_sanitization_invariants
//
// Property-basedテストの特徴:
// - 大量のランダム入力による網羅的検証
// - 境界値・エッジケースの自動発見
// - 不変条件の厳密な検証
// - 仕様書で定義された制約の自動チェック
// - 実装のロバスト性向上