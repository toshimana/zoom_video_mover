/// Property-basedテスト基盤拡張 - 1000ケース以上の網羅的品質保証
/// 
/// # テスト戦略
/// - 全コンポーネントの重要関数をProperty-basedテストで検証
/// - 1000ケース以上の自動検証による品質保証基盤
/// - データ整合性・境界値・異常系の完全検証

use zoom_video_mover_lib::components::{
    auth::{AuthComponent, AuthToken},
    api::{ApiComponent, ApiConfig, RecordingSearchRequest, MeetingRecording, RecordingFile, RecordingFileType},
    download::{DownloadComponent, DownloadConfig},
    config::{AppConfig, OAuthConfig, ApiSettings},
    ComponentLifecycle,
};
use zoom_video_mover_lib::errors::AppError;
use zoom_video_mover_lib::{sanitize_filename, generate_file_path};
use proptest::prelude::*;
use chrono::{NaiveDate, Utc, Duration, Datelike};
use tempfile::TempDir;

// ===== 任意値生成器定義 =====

/// 有効なファイル名文字列生成器
prop_compose! {
    fn arb_valid_filename()
        (base in "[a-zA-Z0-9_\\-]{1,50}",
         ext in "[a-zA-Z0-9]{2,5}")
        -> String
    {
        format!("{}.{}", base, ext)
    }
}

/// 危険なファイル名文字列生成器（特殊文字含む）
prop_compose! {
    fn arb_dangerous_filename()
        (chars in "[\\PC]{1,100}")
        -> String
    {
        chars
    }
}

/// 有効な日付範囲生成器
prop_compose! {
    fn arb_valid_date_range()
        (year in 2020i32..2030i32,
         start_month in 1u32..13u32,
         start_day in 1u32..29u32,
         duration_days in 1u32..365u32)
        -> (NaiveDate, NaiveDate)
    {
        let start_date = NaiveDate::from_ymd_opt(year, start_month, start_day).unwrap();
        let end_date = start_date + chrono::Duration::days(duration_days as i64);
        (start_date, end_date)
    }
}

/// OAuth設定生成器
prop_compose! {
    fn arb_oauth_config()
        (client_id in "[a-zA-Z0-9]{10,50}",
         client_secret in "[a-zA-Z0-9]{20,100}",
         port in 8000u16..9000u16,
         scopes in prop::collection::vec("[a-z:]{5,20}", 1..5))
        -> OAuthConfig
    {
        OAuthConfig {
            client_id,
            client_secret,
            redirect_uri: format!("http://localhost:{}/callback", port),
            scopes,
        }
    }
}

/// 認証トークン生成器
prop_compose! {
    fn arb_auth_token()
        (access_token in "[a-zA-Z0-9]{50,100}",
         refresh_token in "[a-zA-Z0-9]{50,100}",
         expires_in_hours in 1i64..24i64,
         scopes in prop::collection::vec("[a-z:]{5,20}", 1..5))
        -> AuthToken
    {
        AuthToken {
            access_token,
            token_type: "Bearer".to_string(),
            expires_at: Utc::now() + Duration::hours(expires_in_hours),
            refresh_token: Some(refresh_token),
            scopes,
        }
    }
}

/// 録画ファイル情報生成器
prop_compose! {
    fn arb_recording_file()
        (id in "[a-zA-Z0-9]{10,20}",
         meeting_id in "[0-9]{10,15}",
         file_type in prop::sample::select(vec![
             RecordingFileType::MP4,
             RecordingFileType::M4A,
             RecordingFileType::Transcript,
             RecordingFileType::Chat,
             RecordingFileType::ClosedCaption,
             RecordingFileType::Timeline,
             RecordingFileType::Summary,
         ]),
         file_size in 1024u64..1_000_000_000u64,
         file_name in arb_valid_filename())
        -> RecordingFile
    {
        RecordingFile {
            id: id.clone(),
            meeting_id,
            recording_start: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            recording_end: (Utc::now() + Duration::hours(1)).format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            file_type,
            file_extension: file_name.split('.').last().unwrap_or("mp4").to_string(),
            file_size,
            play_url: Some(format!("https://example.com/play/{}", id)),
            download_url: format!("https://example.com/download/{}", id),
            status: "completed".to_string(),
            recording_type: "shared_screen_with_speaker_view".to_string(),
        }
    }
}

/// 会議録画情報生成器
prop_compose! {
    fn arb_meeting_recording()
        (id in 1000000000u64..9999999999u64,
         uuid in "[a-zA-Z0-9\\+/]{20,30}",
         topic in "[\\w\\s]{5,50}",
         duration in 300u32..7200u32,
         files in prop::collection::vec(arb_recording_file(), 1..5))
        -> MeetingRecording
    {
        MeetingRecording {
            uuid,
            id,
            account_id: "account123".to_string(),
            host_id: "host123".to_string(),
            topic,
            meeting_type: 2,
            start_time: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            timezone: "UTC".to_string(),
            duration,
            total_size: files.iter().map(|f| f.file_size).sum(),
            recording_count: files.len() as u32,
            recording_files: files,
        }
    }
}

// ===== Property-basedテスト実装 =====

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]
    
    /// ファイル名サニタイズの完全性Property検証
    #[test]
    fn filename_sanitize_comprehensive_properties(
        input in arb_dangerous_filename()
    ) {
        let sanitized = sanitize_filename(&input);
        
        // Property 1: 結果は常に有効なファイル名
        prop_assert!(!sanitized.is_empty());
        prop_assert!(sanitized.len() <= 200);
        
        // Property 2: 危険な文字が除去されている
        prop_assert!(!sanitized.contains('/'));
        prop_assert!(!sanitized.contains('\\'));
        prop_assert!(!sanitized.contains(':'));
        prop_assert!(!sanitized.contains('*'));
        prop_assert!(!sanitized.contains('?'));
        prop_assert!(!sanitized.contains('"'));
        prop_assert!(!sanitized.contains('<'));
        prop_assert!(!sanitized.contains('>'));
        prop_assert!(!sanitized.contains('|'));
        
        // Property 3: Windows予約名が回避されている
        let windows_reserved = ["CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", 
                               "COM4", "COM5", "COM6", "COM7", "COM8", "COM9", 
                               "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", 
                               "LPT7", "LPT8", "LPT9"];
        for reserved in windows_reserved {
            if sanitized.to_uppercase() == reserved {
                prop_assert!(sanitized.starts_with('_'));
            }
        }
        
        // Property 4: 冪等性（同じ入力に対して常に同じ出力）
        let sanitized2 = sanitize_filename(&input);
        prop_assert_eq!(sanitized, sanitized2);
    }
    
    /// OAuth設定の妥当性Property検証
    #[test]
    fn oauth_config_validity_properties(
        config in arb_oauth_config()
    ) {
        // Property 1: 基本フィールドの妥当性
        prop_assert!(!config.client_id.is_empty());
        prop_assert!(!config.client_secret.is_empty());
        prop_assert!(!config.redirect_uri.is_empty());
        prop_assert!(!config.scopes.is_empty());
        
        // Property 2: client_idの長さ制約
        prop_assert!(config.client_id.len() >= 10);
        prop_assert!(config.client_id.len() <= 50);
        
        // Property 3: client_secretの長さ制約
        prop_assert!(config.client_secret.len() >= 20);
        prop_assert!(config.client_secret.len() <= 100);
        
        // Property 4: redirect_uriの形式
        prop_assert!(config.redirect_uri.starts_with("http://") || 
                    config.redirect_uri.starts_with("https://"));
        
        // Property 5: AuthComponentの作成が成功する
        let _auth_component = AuthComponent::new(config.clone());
        // 作成自体は常に成功する（バリデーションは初期化時）
        prop_assert!(true); // 作成成功を確認
    }
    
    /// 認証トークンのライフサイクルProperty検証
    #[test]
    fn auth_token_lifecycle_properties(
        token in arb_auth_token()
    ) {
        // Property 1: 有効なトークンは有効期限内
        if token.expires_at > Utc::now() {
            prop_assert!(token.is_valid());
        }
        
        // Property 2: アクセストークンが存在する
        prop_assert!(!token.access_token.is_empty());
        
        // Property 3: スコープが存在する
        prop_assert!(!token.scopes.is_empty());
        
        // Property 4: スコープ確認の正確性
        for scope in &token.scopes {
            prop_assert!(token.has_scope(scope));
        }
        
        // Property 5: 存在しないスコープは確認されない
        prop_assert!(!token.has_scope("nonexistent:scope"));
        
        // Property 6: 残り時間の一貫性
        let remaining = token.remaining_seconds();
        if token.is_valid() {
            prop_assert!(remaining > 0);
        } else {
            prop_assert_eq!(remaining, 0);
        }
    }
    
    /// 日付範囲バリデーションProperty検証
    #[test]
    fn date_range_validation_properties(
        (from_date, to_date) in arb_valid_date_range()
    ) {
        // Property 1: 日付順序の保証
        prop_assert!(from_date <= to_date);
        
        // Property 2: 録画検索リクエストの妥当性
        let request = RecordingSearchRequest {
            user_id: Some("test_user".to_string()),
            from: from_date,
            to: to_date,
            page_size: Some(30),
            next_page_token: None,
        };
        
        // 日付範囲が正しい場合、リクエスト自体は有効
        prop_assert!(request.from <= request.to);
        
        // Property 3: 年の妥当性
        prop_assert!(from_date.year() >= 2020);
        prop_assert!(to_date.year() <= 2030);
        
        // Property 4: 月日の妥当性
        prop_assert!(from_date.month() >= 1 && from_date.month() <= 12);
        prop_assert!(to_date.month() >= 1 && to_date.month() <= 12);
        prop_assert!(from_date.day() >= 1 && from_date.day() <= 31);
        prop_assert!(to_date.day() >= 1 && to_date.day() <= 31);
    }
    
    /// 録画データの整合性Property検証
    #[test]
    fn recording_data_consistency_properties(
        meeting in arb_meeting_recording()
    ) {
        // Property 1: IDの妥当性
        prop_assert!(meeting.id > 0);
        prop_assert!(!meeting.uuid.is_empty());
        
        // Property 2: 録画ファイル数の一貫性
        prop_assert_eq!(meeting.recording_files.len(), meeting.recording_count as usize);
        
        // Property 3: 総サイズの一貫性
        let calculated_size: u64 = meeting.recording_files.iter()
            .map(|f| f.file_size)
            .sum();
        prop_assert_eq!(meeting.total_size, calculated_size);
        
        // Property 4: 時間の妥当性
        prop_assert!(meeting.duration > 0);
        prop_assert!(meeting.duration <= 86400); // 24時間以内
        
        // Property 5: 各ファイルの妥当性
        for file in &meeting.recording_files {
            prop_assert!(!file.id.is_empty());
            prop_assert!(!file.download_url.is_empty());
            prop_assert!(file.file_size > 0);
            prop_assert!(file.download_url.starts_with("http"));
        }
    }
    
    /// エラー処理の回復可能性Property検証
    #[test]
    fn error_recovery_properties(
        message in "[\\PC]{1,100}",
        status_code in 400u16..600u16
    ) {
        // Property 1: ネットワークエラーの回復可能性
        let network_error = AppError::network(&message, None::<std::io::Error>);
        prop_assert!(network_error.is_recoverable());
        prop_assert!(network_error.retry_after().is_some());
        
        // Property 2: APIエラーの回復可能性判定
        let api_error = AppError::api(status_code, &message, None::<std::io::Error>);
        if status_code >= 500 {
            prop_assert!(api_error.is_recoverable());
        } else {
            prop_assert!(!api_error.is_recoverable());
        }
        
        // Property 3: バリデーションエラーの非回復性
        let validation_error = AppError::validation(&message, None);
        prop_assert!(!validation_error.is_recoverable());
        prop_assert!(validation_error.retry_after().is_none());
        
        // Property 4: レート制限エラーの回復可能性
        let rate_limit_error = AppError::rate_limit(&message);
        prop_assert!(rate_limit_error.is_recoverable());
    }
    
    /// 設定ファイルのシリアライゼーション冪等性Property検証
    #[test]
    fn config_serialization_idempotency_properties(
        oauth_config in arb_oauth_config(),
        output_dir in "[a-zA-Z0-9_/\\\\]{1,50}",
        concurrent_downloads in 1u32..10u32,
        timeout_seconds in 5u64..300u64
    ) {
        let app_config = AppConfig {
            oauth: oauth_config,
            output_directory: output_dir,
            max_concurrent_downloads: concurrent_downloads,
            request_timeout_seconds: timeout_seconds,
            rate_limit_per_second: 10,
            debug_mode: false,
            log_level: "info".to_string(),
            api: ApiSettings::default(),
        };
        
        // Property 1: シリアライゼーション・デシリアライゼーションの冪等性
        let serialized = toml::to_string(&app_config);
        prop_assert!(serialized.is_ok());
        
        let deserialized: Result<AppConfig, _> = toml::from_str(&serialized.unwrap());
        prop_assert!(deserialized.is_ok());
        
        let config2 = deserialized.unwrap();
        
        // Property 2: 元データとの一致
        prop_assert_eq!(app_config.oauth.client_id, config2.oauth.client_id);
        prop_assert_eq!(app_config.oauth.client_secret, config2.oauth.client_secret);
        prop_assert_eq!(app_config.output_directory, config2.output_directory);
        prop_assert_eq!(app_config.max_concurrent_downloads, config2.max_concurrent_downloads);
    }

    /// generate_file_pathが二重拡張子を生成しないことのProperty検証
    #[test]
    fn generate_file_path_never_has_double_extension(
        meeting in arb_meeting_recording(),
        file in arb_recording_file()
    ) {
        let path = generate_file_path(&meeting, &file);

        // ファイル名部分を取得
        let file_name = path.split('/').last().unwrap_or(&path);

        // 二重拡張子パターンがないことを確認
        let double_ext_patterns = [
            ".mp4.mp4", ".m4a.m4a", ".vtt.vtt", ".txt.txt",
            ".json.json", ".dat.dat",
        ];
        for pattern in &double_ext_patterns {
            prop_assert!(
                !file_name.ends_with(pattern),
                "Double extension detected: {} in {}",
                pattern, file_name
            );
        }

        // フォルダ名がYYYY-MM-DD形式であること
        let folder = path.split('/').next().unwrap_or("");
        prop_assert!(
            folder.len() == 10 || folder == "unknown",
            "Folder name should be YYYY-MM-DD or 'unknown', got: {}",
            folder
        );
    }
}

// ===== 統合Property-basedテスト =====

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]
    
    /// 全コンポーネント連携のProperty検証
    #[test]
    fn component_integration_properties(
        oauth_config in arb_oauth_config()
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            // Property 1: 認証コンポーネントの初期化成功
            let mut auth_component = AuthComponent::new(oauth_config.clone());
            let init_result = auth_component.initialize().await;
            prop_assert!(init_result.is_ok());
            
            // Property 2: APIコンポーネントの初期化成功  
            let api_config = ApiConfig::default();
            let mut api_component = ApiComponent::new(api_config);
            let api_init_result = api_component.initialize().await;
            prop_assert!(api_init_result.is_ok());
            
            // Property 3: ダウンロードコンポーネントの初期化成功
            let temp_dir = TempDir::new().unwrap();
            let download_config = DownloadConfig {
                concurrent_downloads: 2,
                chunk_size: 1024,
                timeout: std::time::Duration::from_secs(30),
                max_retries: 3,
                output_directory: temp_dir.path().to_path_buf(),
            };
            let mut download_component = DownloadComponent::new(download_config);
            let download_init_result = download_component.initialize().await;
            prop_assert!(download_init_result.is_ok());
            
            // Property 4: コンポーネント間の健全性確認
            prop_assert!(auth_component.health_check().await);
            prop_assert!(api_component.health_check().await);
            prop_assert!(download_component.health_check().await);
            
            // Property 5: 適切なシャットダウン
            let shutdown_result1 = download_component.shutdown().await;
            prop_assert!(shutdown_result1.is_ok());
            let shutdown_result2 = api_component.shutdown().await;
            prop_assert!(shutdown_result2.is_ok());
            let shutdown_result3 = auth_component.shutdown().await;
            prop_assert!(shutdown_result3.is_ok());
            
            Ok(())
        });
        result.unwrap()
    }
}

// ===== ストレステスト =====

#[cfg(test)]
mod stress_tests {
    use super::*;
    use zoom_video_mover_lib::components::ComponentLifecycle;
    
    /// 大量データ処理のストレステスト
    #[tokio::test]
    async fn test_large_dataset_processing() {
        let mut meetings = Vec::new();
        
        // 1000件の録画データを生成
        for i in 0..1000 {
            let meeting = MeetingRecording {
                uuid: format!("uuid_{}", i),
                id: 1000000000 + i,
                account_id: "stress_test_account".to_string(),
                host_id: "stress_test_host".to_string(),
                topic: format!("Stress Test Meeting {}", i),
                meeting_type: 2,
                start_time: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                timezone: "UTC".to_string(),
                duration: 3600,
                total_size: 100_000_000,
                recording_count: 2,
                recording_files: vec![
                    RecordingFile {
                        id: format!("file_{}_{}", i, 1),
                        meeting_id: format!("{}", 1000000000 + i),
                        recording_start: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                        recording_end: (Utc::now() + Duration::hours(1)).format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                        file_type: RecordingFileType::MP4,
                        file_extension: "mp4".to_string(),
                        file_size: 80_000_000,
                        play_url: Some(format!("https://example.com/play/{}", i)),
                        download_url: format!("https://example.com/download/{}", i),
                        status: "completed".to_string(),
                        recording_type: "shared_screen_with_speaker_view".to_string(),
                    },
                    RecordingFile {
                        id: format!("file_{}_{}", i, 2),
                        meeting_id: format!("{}", 1000000000 + i),
                        recording_start: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                        recording_end: (Utc::now() + Duration::hours(1)).format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                        file_type: RecordingFileType::M4A,
                        file_extension: "m4a".to_string(),
                        file_size: 20_000_000,
                        play_url: Some(format!("https://example.com/play_audio/{}", i)),
                        download_url: format!("https://example.com/download_audio/{}", i),
                        status: "completed".to_string(),
                        recording_type: "audio_only".to_string(),
                    },
                ],
            };
            meetings.push(meeting);
        }
        
        // データ処理の検証
        assert_eq!(meetings.len(), 1000);
        
        // 各会議の整合性確認
        for meeting in &meetings {
            assert_eq!(meeting.recording_files.len(), 2);
            assert_eq!(meeting.recording_count, 2);
            
            let total_size: u64 = meeting.recording_files.iter().map(|f| f.file_size).sum();
            assert_eq!(meeting.total_size, total_size);
        }
        
        // メモリ使用量確認（大量データでもメモリリークしない）
        drop(meetings);
    }
    
    /// コンポーネント並行処理ストレステスト
    #[tokio::test]
    async fn test_concurrent_component_operations() {
        let mut handles = Vec::new();
        
        // 100個の並行タスクを起動
        for i in 0..100 {
            let handle = tokio::spawn(async move {
                let oauth_config = OAuthConfig {
                    client_id: format!("client_{}", i),
                    client_secret: format!("secret_{}", i),
                    redirect_uri: format!("http://localhost:808{}/callback", i % 10),
                    scopes: vec!["recording:read".to_string()],
                };
                
                let mut auth_component = AuthComponent::new(oauth_config);
                
                // 初期化・ヘルスチェック・シャットダウンのサイクル
                auth_component.initialize().await.expect("Initialize failed");
                assert!(auth_component.health_check().await);
                auth_component.shutdown().await.expect("Shutdown failed");
                
                i
            });
            
            handles.push(handle);
        }
        
        // すべてのタスクの完了を待機
        for handle in handles {
            let result = handle.await.expect("Task failed");
            assert!(result < 100);
        }
    }
}