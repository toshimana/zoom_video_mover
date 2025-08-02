# テスト方針 - Zoom Video Mover

## テストの基本方針

### テスト哲学・原則
- **早期発見**: 開発プロセスの早い段階でのバグ・問題の発見
- **自動化優先**: 手動テストを最小限に抑えた自動化テスト体制
- **包括性**: 機能・性能・セキュリティ・互換性の全面的なテスト
- **再現性**: 一貫した結果を提供する安定したテスト
- **継続改善**: テスト結果に基づく継続的な品質向上
- **リスクベース**: 重要度・リスクに応じたテスト優先度設定

### 品質目標
- **機能品質**: 全要件の正常動作保証・境界値・異常系の適切な処理
- **性能品質**: 応答時間・スループット・リソース効率の目標達成
- **信頼性品質**: エラー回復・長時間稼働・ストレス耐性の保証
- **セキュリティ品質**: 脆弱性・データ保護・認証の安全性確保
- **使用性品質**: ユーザビリティ・アクセシビリティの確保

## テスト戦略・分類

### テストピラミッド構造

```
           ┌─────────────────────┐
           │   Manual Tests      │ ← 最小限
           │  (Exploratory)      │
           └─────────────────────┘
         ┌───────────────────────────┐
         │     E2E Tests             │ ← 少数・重要シナリオ
         │  (Integration Tests)      │
         └───────────────────────────┘
       ┌─────────────────────────────────┐
       │        Unit Tests               │ ← 大多数・高速実行
       │   (Property-based Tests)        │
       └─────────────────────────────────┘
```

### テスト分類・責任範囲

#### 1. Property-based Testing（基盤テスト）
- **目的**: データ整合性・不変条件の自動検証
- **対象**: 純粋関数・データ変換・アルゴリズム
- **実行**: 1000ケース以上の自動生成入力での検証
- **優先度**: 最高（開発の基盤となる信頼性確保）

#### 2. Unit Testing（単体テスト）
- **目的**: 個別関数・メソッドの仕様確認
- **対象**: ビジネスロジック・計算処理・状態変更
- **実行**: 関数レベルの入出力検証
- **優先度**: 高（機能の正確性保証）

#### 3. Integration Testing（統合テスト）
- **目的**: コンポーネント間連携・外部システム接続
- **対象**: API呼び出し・ファイルI/O・GUI操作フロー
- **実行**: 実際の環境に近い条件での動作確認
- **優先度**: 高（システム全体の動作保証）

#### 4. Performance Testing（性能テスト）
- **目的**: 応答時間・スループット・リソース効率の測定
- **対象**: ダウンロード処理・API呼び出し・並行処理
- **実行**: 負荷条件下での性能測定
- **優先度**: 中（ユーザー体験の品質保証）

#### 5. Security Testing（セキュリティテスト）
- **目的**: 脆弱性・データ保護・認証の安全性検証
- **対象**: OAuth認証・設定ファイル・通信暗号化
- **実行**: セキュリティスキャン・侵入テスト
- **優先度**: 高（データ保護・プライバシー確保）

#### 6. Compatibility Testing（互換性テスト）
- **目的**: 様々な環境での動作確認
- **対象**: Windows バージョン・文字エンコーディング・画面解像度
- **実行**: 複数環境での自動テスト
- **優先度**: 中（幅広いユーザー対応）

## Property-based Testing戦略

### プロパティ設計原則

#### 1. 不変条件（Invariants）
```rust
// 設定ファイルの整合性プロパティ
proptest! {
    /// 設定ファイルのラウンドトリップ特性
    /// - 保存→読み込み→保存の結果が一致する
    /// - データの完全性が保たれる
    #[test]
    fn config_roundtrip_property(
        client_id in "[a-zA-Z0-9]{10,50}",
        client_secret in "[a-zA-Z0-9]{20,100}",
        output_dir in prop::option::of("[a-zA-Z0-9/\\\\:.]{5,100}")
    ) {
        let original_config = Config {
            client_id: client_id.clone(),
            client_secret: client_secret.clone(),
            redirect_uri: Some("http://localhost:8080/callback".to_string()),
            output_directory: PathBuf::from(output_dir.unwrap_or_else(|| ".".to_string())),
        };
        
        // TOML シリアライゼーション
        let toml_string = toml::to_string(&original_config).unwrap();
        
        // デシリアライゼーション
        let deserialized_config: Config = toml::from_str(&toml_string).unwrap();
        
        // ラウンドトリップ検証
        prop_assert_eq!(original_config.client_id, deserialized_config.client_id);
        prop_assert_eq!(original_config.client_secret, deserialized_config.client_secret);
        prop_assert_eq!(original_config.redirect_uri, deserialized_config.redirect_uri);
        prop_assert_eq!(original_config.output_directory, deserialized_config.output_directory);
        
        // 再シリアライゼーションの一致確認
        let re_serialized = toml::to_string(&deserialized_config).unwrap();
        prop_assert_eq!(toml_string, re_serialized);
    }
}
```

#### 2. 関数特性（Function Properties）
```rust
// ファイル名サニタイズのプロパティ
proptest! {
    /// ファイル名サニタイズの特性
    /// - 出力は常に有効なWindowsファイル名
    /// - 長さ制限内に収まる
    /// - 危険な文字が除去される
    #[test]
    fn filename_sanitization_property(
        filename in ".*{1,500}" // 任意の文字列（最大500文字）
    ) {
        prop_assume!(!filename.is_empty()); // 空文字列は除外
        
        let sanitized = sanitize_filename(&filename);
        
        // 基本特性
        prop_assert!(!sanitized.is_empty(), "sanitized filename must not be empty");
        prop_assert!(sanitized.len() <= 255, "must be within Windows limit");
        
        // 無効文字の除去確認
        let invalid_chars = ['<', '>', ':', '"', '|', '?', '*', '/', '\\'];
        for ch in invalid_chars {
            prop_assert!(!sanitized.contains(ch), "must not contain invalid char: {}", ch);
        }
        
        // 制御文字の除去確認
        prop_assert!(
            !sanitized.chars().any(|c| c.is_control()),
            "must not contain control characters"
        );
        
        // Windows予約語の回避確認
        let reserved_names = ["CON", "PRN", "AUX", "NUL"];
        let name_upper = sanitized.to_uppercase();
        let base_name = name_upper.split('.').next().unwrap_or(&name_upper);
        
        if reserved_names.contains(&base_name) {
            prop_assert!(
                sanitized.starts_with('_'),
                "reserved name must be prefixed with underscore"
            );
        }
        
        // 冪等性確認（サニタイズ済みファイル名の再サニタイズ）
        prop_assert_eq!(sanitized, sanitize_filename(&sanitized));
    }
}
```

#### 3. 順序関係（Ordering Properties）
```rust
// 日付処理のプロパティ
proptest! {
    /// 日付範囲処理の特性
    /// - from_date <= to_date の関係が保たれる
    /// - 有効な日付のみが生成される
    /// - 日付計算の正確性
    #[test]
    fn date_range_property(
        year in 2020i32..2030i32,
        month1 in 1u32..13u32,
        month2 in 1u32..13u32,
        day_offset1 in 0u32..31u32,
        day_offset2 in 0u32..31u32
    ) {
        // 有効な日付生成
        let date1 = generate_valid_date(year, month1, day_offset1);
        let date2 = generate_valid_date(year, month2, day_offset2);
        
        let (from_date, to_date) = if date1 <= date2 {
            (date1, date2)
        } else {
            (date2, date1)
        };
        
        // API呼び出し形式に変換
        let from_str = from_date.format("%Y-%m-%d").to_string();
        let to_str = to_date.format("%Y-%m-%d").to_string();
        
        // 日付範囲検証関数のテスト
        let result = validate_date_range(&from_str, &to_str);
        prop_assert!(result.is_ok(), "valid date range should pass validation");
        
        // 順序関係の保持確認
        let parsed_from = NaiveDate::parse_from_str(&from_str, "%Y-%m-%d").unwrap();
        let parsed_to = NaiveDate::parse_from_str(&to_str, "%Y-%m-%d").unwrap();
        prop_assert!(parsed_from <= parsed_to, "from_date must be <= to_date");
        
        // 日付範囲計算の検証
        let duration = parsed_to.signed_duration_since(parsed_from);
        prop_assert!(duration.num_days() >= 0, "duration must be non-negative");
        prop_assert!(duration.num_days() <= 365, "duration must be within reasonable range");
    }
}

/// 月ごとの有効日数を考慮した日付生成
fn generate_valid_date(year: i32, month: u32, day_offset: u32) -> NaiveDate {
    let max_day = match month {
        2 => {
            if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
                29 // うるう年
            } else {
                28 // 平年
            }
        }
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    
    let day = (day_offset % max_day) + 1;
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
}
```

#### 4. エラー処理プロパティ
```rust
// エラーハンドリングのプロパティ
proptest! {
    /// エラー処理の特性
    /// - 不正入力に対する適切なエラー
    /// - エラーメッセージの有用性
    /// - システム状態の一貫性保持
    #[test]
    fn error_handling_property(
        invalid_url in prop::option::of(".*"),
        invalid_path in prop::option::of(".*"),
        invalid_size in prop::option::of(any::<u64>())
    ) {
        // 不正なダウンロードリクエストの生成
        let request = DownloadRequest {
            file_id: FileId::new("test_file".to_string()).unwrap(),
            url: invalid_url.unwrap_or_else(|| "".to_string()),
            output_path: PathBuf::from(invalid_path.unwrap_or_else(|| "".to_string())),
            file_size: invalid_size.unwrap_or(0),
        };
        
        // エラー処理のテスト
        let result = validate_download_request(&request);
        
        if request.url.is_empty() {
            // 空URLの場合は適切なエラーが返される
            prop_assert!(result.is_err(), "empty URL should cause error");
            
            let error = result.unwrap_err();
            prop_assert!(
                matches!(error, ZoomVideoMoverError::Validation { .. }),
                "should return validation error for empty URL"
            );
            
            // エラーメッセージの有用性確認
            let error_msg = error.to_string();
            prop_assert!(
                error_msg.to_lowercase().contains("url"),
                "error message should mention URL: {}",
                error_msg
            );
        }
        
        // システム状態の一貫性確認（エラー後も正常動作可能）
        let valid_request = create_valid_download_request();
        let valid_result = validate_download_request(&valid_request);
        prop_assert!(
            valid_result.is_ok(),
            "system should remain functional after error"
        );
    }
}
```

### Property-based Testing実行戦略

#### 実行設定
```rust
// プロパティテスト設定
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 1000,           // テストケース数
        max_shrink_iters: 1000, // 最小化試行回数
        timeout: 5000,         // タイムアウト（ミリ秒）
        max_global_rejects: 65536, // 最大リジェクト数
        source_file: Some("tests/property_tests.rs"), // ソースファイル
        ..ProptestConfig::default()
    })]
    
    // テスト関数
}

// 環境変数による実行制御
#[cfg(test)]
mod property_test_config {
    use super::*;
    
    pub fn get_test_cases() -> u32 {
        std::env::var("PROPTEST_CASES")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1000)
    }
    
    pub fn is_verbose_mode() -> bool {
        std::env::var("PROPTEST_VERBOSE").is_ok()
    }
}
```

#### 実行コマンド
```bash
# 基本実行
cargo test --test property_tests

# 詳細ログ付き実行
PROPTEST_VERBOSE=1 cargo test --test property_tests -- --nocapture

# テストケース数指定
PROPTEST_CASES=5000 cargo test --test property_tests

# 特定プロパティのみ実行
cargo test --test property_tests -- config_roundtrip_property

# 並列実行制御
cargo test --test property_tests -- --test-threads=4
```

## Unit Testing戦略

### 単体テスト設計原則

#### 1. 関数レベルテスト
```rust
#[cfg(test)]
mod unit_tests {
    use super::*;
    use tokio_test;
    
    /// OAuth認証URL生成のテスト
    #[test]
    fn test_generate_auth_url() {
        let downloader = ZoomRecordingDownloader::new_with_test_config();
        let client_id = "test_client_id";
        
        let auth_url = downloader.generate_auth_url_sync(client_id).unwrap();
        
        // URL形式の検証
        assert!(auth_url.starts_with("https://zoom.us/oauth/authorize"));
        assert!(auth_url.contains(&format!("client_id={}", client_id)));
        assert!(auth_url.contains("response_type=code"));
        assert!(auth_url.contains("scope="));
        
        // stateパラメータの存在確認（CSRF対策）
        assert!(auth_url.contains("state="));
        
        // URLの妥当性確認
        let parsed_url = url::Url::parse(&auth_url).unwrap();
        assert_eq!(parsed_url.scheme(), "https");
        assert_eq!(parsed_url.host_str(), Some("zoom.us"));
        assert_eq!(parsed_url.path(), "/oauth/authorize");
    }
    
    /// トークン有効性チェックのテスト
    #[test]
    fn test_token_validation() {
        let now = Utc::now();
        
        // 有効なトークン
        let valid_token = AuthToken {
            access_token: "valid_token".to_string(),
            token_type: "Bearer".to_string(),
            expires_at: now + chrono::Duration::hours(1),
            refresh_token: Some("refresh".to_string()),
            scope: "recording:read user:read".to_string(),
        };
        
        assert!(valid_token.is_valid());
        assert!(valid_token.has_scope("recording:read"));
        assert!(valid_token.has_all_scopes(&["recording:read", "user:read"]));
        
        // 期限切れトークン
        let expired_token = AuthToken {
            access_token: "expired_token".to_string(),
            token_type: "Bearer".to_string(),
            expires_at: now - chrono::Duration::hours(1),
            refresh_token: None,
            scope: "recording:read".to_string(),
        };
        
        assert!(!expired_token.is_valid());
        
        // 権限不足トークン
        let limited_token = AuthToken {
            access_token: "limited_token".to_string(),
            token_type: "Bearer".to_string(),
            expires_at: now + chrono::Duration::hours(1),
            refresh_token: None,
            scope: "user:read".to_string(),
        };
        
        assert!(!limited_token.has_scope("recording:read"));
        assert!(!limited_token.has_all_scopes(&["recording:read", "user:read"]));
    }
    
    /// ファイル名サニタイズのテスト
    #[test]
    fn test_filename_sanitization() {
        // 基本的なサニタイズ
        assert_eq!(sanitize_filename("normal_file.txt"), "normal_file.txt");
        
        // 無効文字の置換
        assert_eq!(sanitize_filename("file<>:\"|?*.txt"), "file___________.txt");
        
        // Windows予約語の回避
        assert_eq!(sanitize_filename("CON.txt"), "_CON.txt");
        assert_eq!(sanitize_filename("con.txt"), "_con.txt");
        assert_eq!(sanitize_filename("PRN"), "_PRN");
        
        // 長さ制限
        let long_name = "a".repeat(300);
        let sanitized = sanitize_filename(&long_name);
        assert!(sanitized.len() <= 255);
        
        // 日本語ファイル名の保持
        assert_eq!(sanitize_filename("会議録画_2024年1月.mp4"), "会議録画_2024年1月.mp4");
        
        // 制御文字の除去
        let control_chars = "file\x00\x01\x1f.txt";
        let sanitized = sanitize_filename(control_chars);
        assert!(!sanitized.chars().any(|c| c.is_control()));
        assert_eq!(sanitized, "file.txt");
        
        // 末尾の空白・ピリオド除去
        assert_eq!(sanitize_filename("file. "), "file");
        assert_eq!(sanitize_filename("file..."), "file");
    }
}
```

#### 2. 非同期処理テスト
```rust
#[cfg(test)]
mod async_tests {
    use super::*;
    use tokio_test;
    use std::time::Duration;
    
    /// 非同期API呼び出しのテスト
    #[tokio::test]
    async fn test_api_call_with_timeout() {
        let mut mock_client = MockHttpClient::new();
        
        // 正常レスポンスのモック
        mock_client.add_response(
            "https://api.zoom.us/v2/users/me/recordings",
            Ok(create_mock_recordings_response())
        );
        
        let downloader = ZoomRecordingDownloader::new_with_mock(mock_client);
        
        let result = downloader.get_recordings("2024-01-01", "2024-01-31").await;
        assert!(result.is_ok());
        
        let recordings = result.unwrap();
        assert!(!recordings.is_empty());
        assert_eq!(recordings[0].topic, "Test Meeting");
    }
    
    /// タイムアウト処理のテスト
    #[tokio::test]
    async fn test_api_timeout() {
        let mut mock_client = MockHttpClient::new();
        
        // 遅延レスポンスのモック
        mock_client.add_delayed_response(
            "https://api.zoom.us/v2/users/me/recordings",
            Duration::from_secs(10), // 10秒遅延
            Ok(create_mock_recordings_response())
        );
        
        let mut downloader = ZoomRecordingDownloader::new_with_mock(mock_client);
        downloader.set_timeout(Duration::from_secs(5)); // 5秒タイムアウト
        
        let result = downloader.get_recordings("2024-01-01", "2024-01-31").await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            ZoomVideoMoverError::Timeout { .. } => {}, // 期待されるエラー
            other => panic!("Expected timeout error, got: {:?}", other),
        }
    }
    
    /// 並列ダウンロードのテスト
    #[tokio::test]
    async fn test_parallel_downloads() {
        let mut mock_client = MockHttpClient::new();
        
        // 複数ファイルのダウンロードレスポンス
        for i in 1..=5 {
            mock_client.add_response(
                &format!("https://example.com/file{}.mp4", i),
                Ok(MockResponse::with_body(vec![i as u8; 1024])) // 1KB のモックデータ
            );
        }
        
        let downloader = ZoomRecordingDownloader::new_with_mock(mock_client);
        
        // ダウンロードリクエスト作成
        let requests: Vec<DownloadRequest> = (1..=5).map(|i| {
            DownloadRequest {
                file_id: FileId::new(format!("file{}", i)).unwrap(),
                url: format!("https://example.com/file{}.mp4", i),
                output_path: PathBuf::from(format!("./test_output/file{}.mp4", i)),
                file_size: 1024,
            }
        }).collect();
        
        let start_time = Instant::now();
        let results = downloader.download_files_parallel(requests).await;
        let elapsed = start_time.elapsed();
        
        // 結果検証
        assert!(results.is_ok());
        let paths = results.unwrap();
        assert_eq!(paths.len(), 5);
        
        // 並列処理効果の確認（逐次実行より高速）
        assert!(elapsed < Duration::from_secs(5)); // 理論的には1秒未満
        
        // ファイル存在確認
        for path in paths {
            assert!(path.exists());
            assert_eq!(tokio::fs::metadata(&path).await.unwrap().len(), 1024);
        }
    }
}
```

#### 3. エラーケーステスト
```rust
#[cfg(test)]
mod error_tests {
    use super::*;
    
    /// 認証エラーのテスト
    #[tokio::test]
    async fn test_authentication_errors() {
        let mut mock_client = MockHttpClient::new();
        
        // 401 Unauthorized レスポンス
        mock_client.add_response(
            "https://zoom.us/oauth/token",
            Err(ZoomVideoMoverError::Http { 
                status: 401, 
                message: "Invalid client credentials".to_string() 
            })
        );
        
        let downloader = ZoomRecordingDownloader::new_with_mock(mock_client);
        
        let result = downloader.exchange_code("invalid_id", "invalid_secret", "auth_code").await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            ZoomVideoMoverError::Authentication { message } => {
                assert!(message.contains("Invalid client credentials"));
            }
            other => panic!("Expected authentication error, got: {:?}", other),
        }
    }
    
    /// ネットワークエラーのテスト
    #[tokio::test]
    async fn test_network_errors() {
        let mut mock_client = MockHttpClient::new();
        
        // 接続エラーのシミュレーション
        mock_client.add_response(
            "https://api.zoom.us/v2/users/me/recordings",
            Err(ZoomVideoMoverError::Network { 
                source: reqwest::Error::from(std::io::Error::new(
                    std::io::ErrorKind::ConnectionRefused,
                    "Connection refused"
                ))
            })
        );
        
        let downloader = ZoomRecordingDownloader::new_with_mock(mock_client);
        
        let result = downloader.get_recordings("2024-01-01", "2024-01-31").await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            ZoomVideoMoverError::Network { .. } => {}, // 期待されるエラー
            other => panic!("Expected network error, got: {:?}", other),
        }
    }
    
    /// ファイルシステムエラーのテスト
    #[tokio::test]
    async fn test_filesystem_errors() {
        let downloader = ZoomRecordingDownloader::new();
        
        // 書き込み権限のないディレクトリ
        let invalid_path = PathBuf::from("/root/cannot_write_here.txt");
        
        let request = DownloadRequest {
            file_id: FileId::new("test_file".to_string()).unwrap(),
            url: "https://example.com/file.mp4".to_string(),
            output_path: invalid_path,
            file_size: 1024,
        };
        
        let result = downloader.download_file(request).await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            ZoomVideoMoverError::FileSystem { operation, path, .. } => {
                assert_eq!(operation, "write");
                assert!(path.to_string_lossy().contains("cannot_write_here"));
            }
            other => panic!("Expected filesystem error, got: {:?}", other),
        }
    }
}
```

## Integration Testing戦略

### 統合テスト設計

#### 1. API統合テスト
```rust
// tests/integration_tests.rs
use zoom_video_mover::*;
use std::time::Duration;

#[tokio::test]
async fn test_oauth_integration() {
    // 実際のOAuth設定（テスト用）
    let config = load_test_config().await;
    let downloader = ZoomRecordingDownloader::new_with_config(config);
    
    // 認証URL生成
    let auth_url = downloader.generate_auth_url().await.unwrap();
    assert!(auth_url.starts_with("https://zoom.us/oauth/authorize"));
    
    // ここでは実際の認証は行わず、モック認証コードを使用
    let mock_auth_code = "test_auth_code";
    
    // モックサーバーでトークン交換をシミュレート
    let mock_server = setup_mock_oauth_server().await;
    
    let result = downloader.exchange_code(mock_auth_code).await;
    
    // モックサーバーが適切なトークンを返すことを確認
    assert!(result.is_ok());
    let token = result.unwrap();
    assert!(!token.access_token.is_empty());
    assert!(token.expires_at > chrono::Utc::now());
    
    teardown_mock_server(mock_server).await;
}

#[tokio::test]
async fn test_recording_retrieval_integration() {
    let downloader = setup_authenticated_downloader().await;
    
    // 録画リスト取得
    let recordings = downloader.get_recordings("2024-01-01", "2024-01-31").await.unwrap();
    
    // 基本検証
    assert!(!recordings.is_empty());
    
    for recording in &recordings {
        // 必須フィールドの確認
        assert!(!recording.meeting_id.as_str().is_empty());
        assert!(!recording.topic.is_empty());
        assert!(!recording.recording_files.is_empty());
        
        // 日付の妥当性確認
        assert!(recording.start_time.year() >= 2020);
        assert!(recording.duration > 0);
        
        // ファイル情報の検証
        for file in &recording.recording_files {
            assert!(!file.id.as_str().is_empty());
            assert!(!file.download_url.is_empty());
            assert!(file.download_url.starts_with("https://"));
            assert!(file.file_size > 0);
        }
    }
}

#[tokio::test]
async fn test_end_to_end_download() {
    let downloader = setup_authenticated_downloader().await;
    let output_dir = setup_test_output_directory().await;
    
    // 録画検索
    let recordings = downloader.get_recordings("2024-01-01", "2024-01-31").await.unwrap();
    assert!(!recordings.is_empty());
    
    // 最初の録画から最初のファイルを選択
    let recording = &recordings[0];
    let file = &recording.recording_files[0];
    
    // ダウンロードリクエスト作成
    let request = DownloadRequest {
        file_id: file.id.clone(),
        url: file.download_url.clone(),
        output_path: output_dir.join(format!("{}_{}.{}", 
            recording.topic.clone(),
            file.id.as_str(),
            file.file_type.to_lowercase()
        )),
        file_size: file.file_size,
    };
    
    // 進捗監視設定
    let (progress_sender, mut progress_receiver) = tokio::sync::mpsc::channel(100);
    
    // ダウンロード実行
    let download_task = downloader.download_file(request.clone(), Some(progress_sender));
    let progress_monitor = tokio::spawn(async move {
        let mut last_progress = 0.0;
        while let Some(progress) = progress_receiver.recv().await {
            let progress_ratio = progress.bytes_downloaded as f64 / progress.bytes_total as f64;
            assert!(progress_ratio >= last_progress, "Progress should not decrease");
            assert!(progress_ratio <= 1.0, "Progress should not exceed 100%");
            last_progress = progress_ratio;
        }
    });
    
    let downloaded_path = download_task.await.unwrap();
    progress_monitor.await.unwrap();
    
    // ダウンロード結果の検証
    assert!(downloaded_path.exists());
    let file_metadata = tokio::fs::metadata(&downloaded_path).await.unwrap();
    assert_eq!(file_metadata.len(), file.file_size);
    
    // ファイル内容の基本検証（ファイル形式に応じて）
    if request.output_path.extension().and_then(|s| s.to_str()) == Some("mp4") {
        // MP4ファイルの基本ヘッダーチェック
        let file_content = tokio::fs::read(&downloaded_path).await.unwrap();
        assert!(file_content.len() > 100, "MP4 file should have substantial content");
        // MP4のftypヘッダー確認（簡易チェック）
        assert!(file_content.windows(4).any(|w| w == b"ftyp"), "Should contain MP4 ftyp header");
    }
    
    cleanup_test_files(&output_dir).await;
}
```

#### 2. GUI統合テスト
```rust
// tests/gui_integration_tests.rs
use zoom_video_mover::gui::*;
use egui_testing::*;

#[tokio::test]
async fn test_gui_workflow() {
    let mut app = ZoomDownloaderApp::new_with_test_config();
    let mut egui_ctx = create_test_egui_context();
    
    // 初期状態の確認
    assert_eq!(app.current_tab(), TabType::Config);
    assert!(!app.is_authenticated());
    
    // 設定入力のシミュレーション
    app.set_config_input("test_client_id", "test_client_secret", "./test_output");
    
    // 設定保存
    let save_result = app.save_configuration().await;
    assert!(save_result.is_ok());
    
    // 認証タブへの遷移
    app.set_current_tab(TabType::Authentication);
    
    // 認証URL生成
    let auth_url_result = app.generate_auth_url().await;
    assert!(auth_url_result.is_ok());
    
    // モック認証コード入力
    app.set_auth_code("mock_auth_code");
    
    // トークン交換（モックサーバー使用）
    let token_result = app.complete_authentication().await;
    assert!(token_result.is_ok());
    assert!(app.is_authenticated());
    
    // 録画リストタブへの自動遷移確認
    assert_eq!(app.current_tab(), TabType::Recordings);
    
    // 録画検索
    app.set_date_range("2024-01-01", "2024-01-31");
    let search_result = app.search_recordings().await;
    assert!(search_result.is_ok());
    
    let recordings = app.get_recordings();
    assert!(!recordings.is_empty());
    
    // ファイル選択
    let first_recording = &recordings[0];
    app.select_recording_file(&first_recording.meeting_id, &first_recording.recording_files[0].id);
    
    // ダウンロード開始
    let download_result = app.start_download().await;
    assert!(download_result.is_ok());
    
    // 進捗タブへの自動遷移確認
    assert_eq!(app.current_tab(), TabType::Progress);
    assert!(app.is_downloading());
    
    // ダウンロード完了まで待機（モック環境では即座に完了）
    app.wait_for_download_completion().await;
    assert!(!app.is_downloading());
}

#[test]
fn test_gui_component_rendering() {
    let mut app = ZoomDownloaderApp::new_with_test_config();
    let egui_ctx = create_test_egui_context();
    
    egui_ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            // 設定画面のレンダリングテスト
            app.render_config_panel(ui);
            
            // 必要なUI要素の存在確認
            assert!(ui.memory(|m| m.has_focus(egui::Id::new("client_id_input"))));
        });
    });
}
```

## Performance Testing戦略

### 性能テスト設計

#### 1. ベンチマークテスト
```rust
// benches/performance_benchmarks.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use zoom_video_mover::*;
use std::time::Duration;

fn benchmark_config_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_operations");
    
    // 設定ファイル読み込み性能
    group.bench_function("load_config", |b| {
        let config_path = "./test_data/benchmark_config.toml";
        setup_benchmark_config(config_path);
        
        b.iter(|| {
            let config = black_box(Config::load_from_file(config_path)).unwrap();
            black_box(config);
        });
    });
    
    // 設定ファイル保存性能
    group.bench_function("save_config", |b| {
        let config = create_benchmark_config();
        let config_path = "./test_data/benchmark_save_config.toml";
        
        b.iter(|| {
            black_box(config.save_to_file(config_path)).unwrap();
        });
    });
    
    group.finish();
}

fn benchmark_file_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_operations");
    
    // ファイル名サニタイズ性能
    for input_size in [10, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("sanitize_filename", input_size),
            input_size,
            |b, &size| {
                let filename = "a".repeat(size) + "<>:\"|?*" + &"b".repeat(size);
                b.iter(|| {
                    let sanitized = black_box(sanitize_filename(&filename));
                    black_box(sanitized);
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_download_operations(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("download_operations");
    group.measurement_time(Duration::from_secs(30)); // 長めの測定時間
    
    // 並列ダウンロード性能
    for concurrent_count in [1, 3, 5, 10].iter() {
        group.bench_with_input(
            BenchmarkId::new("parallel_download", concurrent_count),
            concurrent_count,
            |b, &count| {
                let downloader = setup_benchmark_downloader_with_mocks(count);
                let requests = create_benchmark_download_requests(count);
                
                b.to_async(&rt).iter(|| async {
                    let results = black_box(
                        downloader.download_files_parallel(requests.clone()).await
                    ).unwrap();
                    black_box(results);
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_config_operations,
    benchmark_file_operations,
    benchmark_download_operations
);
criterion_main!(benches);
```

#### 2. 負荷テスト
```rust
// tests/load_tests.rs
use zoom_video_mover::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::Instant;

#[tokio::test]
async fn test_concurrent_api_calls() {
    let downloader = Arc::new(setup_test_downloader_with_rate_limiting());
    let concurrent_requests = 20;
    let mut tasks = Vec::new();
    
    let start_time = Instant::now();
    
    // 同時API呼び出し
    for i in 0..concurrent_requests {
        let downloader_clone = downloader.clone();
        let task = tokio::spawn(async move {
            let from_date = format!("2024-{:02}-01", (i % 12) + 1);
            let to_date = format!("2024-{:02}-28", (i % 12) + 1);
            
            downloader_clone.get_recordings(&from_date, &to_date).await
        });
        tasks.push(task);
    }
    
    // 全タスクの完了を待機
    let results = futures::future::join_all(tasks).await;
    let elapsed = start_time.elapsed();
    
    // 結果検証
    let mut success_count = 0;
    let mut rate_limit_count = 0;
    
    for result in results {
        match result.unwrap() {
            Ok(_) => success_count += 1,
            Err(ZoomVideoMoverError::RateLimit { .. }) => rate_limit_count += 1,
            Err(other) => panic!("Unexpected error: {:?}", other),
        }
    }
    
    // パフォーマンス検証
    assert!(success_count > 0, "At least some requests should succeed");
    assert!(elapsed < Duration::from_secs(60), "Should complete within reasonable time");
    
    // レート制限の適切な処理確認
    if rate_limit_count > 0 {
        println!("Rate limited requests: {}/{}", rate_limit_count, concurrent_requests);
    }
}

#[tokio::test]
async fn test_memory_usage_under_load() {
    let initial_memory = get_current_memory_usage();
    let downloader = setup_test_downloader();
    
    // 大量の小さなファイルダウンロード
    let mut download_tasks = Vec::new();
    for i in 0..100 {
        let request = create_small_file_download_request(i);
        let task = downloader.download_file(request);
        download_tasks.push(task);
    }
    
    // メモリ使用量監視
    let memory_monitor = tokio::spawn(async {
        let mut max_memory = 0;
        for _ in 0..30 { // 30秒間監視
            let current_memory = get_current_memory_usage();
            max_memory = max_memory.max(current_memory);
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        max_memory
    });
    
    // ダウンロード実行
    let download_results = futures::future::try_join_all(download_tasks).await;
    let max_memory = memory_monitor.await.unwrap();
    
    // 結果検証
    assert!(download_results.is_ok());
    
    // メモリ使用量検証（1GB以内）
    let memory_increase = max_memory - initial_memory;
    assert!(memory_increase < 1024 * 1024 * 1024, 
            "Memory usage should stay within limit: {} bytes", memory_increase);
    
    println!("Peak memory usage: {} MB", memory_increase / 1024 / 1024);
}

#[tokio::test]
async fn test_long_running_stability() {
    let downloader = setup_test_downloader();
    let start_time = Instant::now();
    let test_duration = Duration::from_secs(300); // 5分間
    
    let mut operation_count = 0;
    let mut error_count = 0;
    
    while start_time.elapsed() < test_duration {
        // 様々な操作を順次実行
        let operations = vec![
            test_config_load_save(),
            test_auth_url_generation(),
            test_recording_list_fetch(),
        ];
        
        for operation in operations {
            match operation.await {
                Ok(_) => operation_count += 1,
                Err(_) => error_count += 1,
            }
        }
        
        // メモリリーク検出
        if operation_count % 100 == 0 {
            let current_memory = get_current_memory_usage();
            println!("Operations: {}, Memory: {} MB", 
                    operation_count, current_memory / 1024 / 1024);
        }
        
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    // 安定性検証
    let error_rate = error_count as f64 / operation_count as f64;
    assert!(error_rate < 0.05, "Error rate should be under 5%: {:.2}%", error_rate * 100.0);
    
    println!("Stability test completed: {} operations, {:.2}% error rate", 
             operation_count, error_rate * 100.0);
}
```

## Security Testing戦略

### セキュリティテスト設計

#### 1. 認証セキュリティテスト
```rust
// tests/security_tests.rs
use zoom_video_mover::*;
use std::time::Duration;

#[tokio::test]
async fn test_oauth_security() {
    let downloader = ZoomRecordingDownloader::new();
    
    // CSRF攻撃対策テスト
    let auth_url1 = downloader.generate_auth_url().await.unwrap();
    let auth_url2 = downloader.generate_auth_url().await.unwrap();
    
    // 異なるstateパラメータが生成されることを確認
    let state1 = extract_state_parameter(&auth_url1);
    let state2 = extract_state_parameter(&auth_url2);
    assert_ne!(state1, state2, "State parameters should be unique");
    assert!(state1.len() >= 16, "State parameter should be sufficiently long");
    
    // 不正なstateパラメータでの認証試行
    let invalid_auth_code = "invalid_code_with_wrong_state";
    let result = downloader.exchange_code(invalid_auth_code).await;
    assert!(result.is_err(), "Should reject invalid state parameter");
}

#[tokio::test]
async fn test_config_file_security() {
    let config = SecureConfig::new(
        "test_client_id".to_string(),
        "test_client_secret".to_string(),
        Some("http://localhost:8080/callback".to_string())
    );
    
    let temp_config_path = "./test_data/secure_config_test.toml";
    
    // 暗号化保存
    let storage = EncryptedConfigStorage::new(PathBuf::from(temp_config_path)).unwrap();
    storage.save_config(&config).await.unwrap();
    
    // ファイル権限確認
    let metadata = tokio::fs::metadata(temp_config_path).await.unwrap();
    let permissions = metadata.permissions();
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = permissions.mode();
        assert_eq!(mode & 0o777, 0o600, "Config file should be readable/writable by owner only");
    }
    
    // ファイル内容の暗号化確認
    let raw_content = tokio::fs::read(temp_config_path).await.unwrap();
    let content_str = String::from_utf8_lossy(&raw_content);
    
    // 平文の機密情報が含まれていないことを確認
    assert!(!content_str.contains("test_client_secret"), 
            "Config file should not contain plaintext secrets");
    assert!(!content_str.contains("client_id"), 
            "Config file should not contain plaintext field names");
    
    // 正常な復号化確認
    let loaded_config = storage.load_config().await.unwrap();
    assert_eq!(loaded_config.client_id, config.client_id);
    assert_eq!(loaded_config.client_secret(), config.client_secret());
    
    // クリーンアップ
    tokio::fs::remove_file(temp_config_path).await.ok();
}

#[tokio::test]
async fn test_https_enforcement() {
    let downloader = ZoomRecordingDownloader::new();
    
    // HTTP URLでのダウンロード試行（セキュリティ違反）
    let insecure_request = DownloadRequest {
        file_id: FileId::new("test".to_string()).unwrap(),
        url: "http://insecure.example.com/file.mp4".to_string(), // HTTP
        output_path: PathBuf::from("./test_output/insecure.mp4"),
        file_size: 1024,
    };
    
    let result = downloader.download_file(insecure_request).await;
    assert!(result.is_err(), "Should reject HTTP URLs");
    
    match result.unwrap_err() {
        ZoomVideoMoverError::Security { .. } => {}, // 期待されるエラー
        other => panic!("Expected security error, got: {:?}", other),
    }
    
    // HTTPS URLは正常処理（モック環境）
    let secure_request = DownloadRequest {
        file_id: FileId::new("test".to_string()).unwrap(),
        url: "https://secure.example.com/file.mp4".to_string(), // HTTPS
        output_path: PathBuf::from("./test_output/secure.mp4"),
        file_size: 1024,
    };
    
    let mock_downloader = setup_mock_downloader_for_https_test();
    let result = mock_downloader.download_file(secure_request).await;
    assert!(result.is_ok(), "HTTPS URLs should be accepted");
}

#[test]
fn test_input_validation() {
    // SQLインジェクション対策（該当する場合）
    let malicious_inputs = vec![
        "'; DROP TABLE users; --",
        "<script>alert('xss')</script>",
        "../../etc/passwd",
        "\x00\x01\x02", // NULL バイト攻撃
        "A".repeat(10000), // バッファオーバーフロー試行
    ];
    
    for malicious_input in malicious_inputs {
        // ファイル名サニタイズの安全性確認
        let sanitized = sanitize_filename(malicious_input);
        assert!(!sanitized.contains('<'), "Should remove dangerous characters");
        assert!(!sanitized.contains('>'), "Should remove dangerous characters");
        assert!(!sanitized.contains('\x00'), "Should remove null bytes");
        assert!(sanitized.len() <= 255, "Should enforce length limits");
        
        // 設定値の検証
        let config_result = Config::new()
            .client_id(malicious_input.to_string())
            .build();
        
        if malicious_input.is_empty() || malicious_input.contains('\x00') {
            assert!(config_result.is_err(), "Should reject invalid input");
        }
    }
}
```

#### 2. データ保護テスト
```rust
#[test]
fn test_memory_security() {
    use std::ptr;
    
    // 機密情報の自動クリア確認
    let secret = SecretString::new("sensitive_data".to_string());
    let secret_ptr = secret.expose_secret().as_ptr();
    
    // スコープを抜けて自動クリアされることを確認
    {
        let temp_secret = secret.clone();
        assert_eq!(temp_secret.expose_secret(), "sensitive_data");
    } // ここで temp_secret が Drop される
    
    drop(secret); // 明示的にDrop
    
    // メモリが実際にクリアされたかの検証は困難だが、
    // zeroize クレートの動作に依存
}

#[tokio::test]
async fn test_log_security() {
    // ログ出力から機密情報の漏洩を防ぐテスト
    let config = SecureConfig::new(
        "test_client_id".to_string(),
        "secret_client_secret".to_string(),
        None
    );
    
    let downloader = ZoomRecordingDownloader::new_with_secure_config(config);
    
    // 認証処理でのログ確認
    let _result = downloader.authenticate().await;
    
    // ログ出力を確認（実際の実装では専用のログキャプチャが必要）
    let log_content = capture_log_output().await;
    
    // 機密情報がログに含まれていないことを確認
    assert!(!log_content.contains("secret_client_secret"), 
            "Logs should not contain client secret");
    assert!(!log_content.contains("access_token"), 
            "Logs should not contain access tokens");
    
    // マスクされた情報が含まれることを確認
    assert!(log_content.contains("***"), 
            "Logs should contain masked sensitive data");
}
```

## Test Automation戦略

### CI/CD パイプライン統合

#### 1. GitHub Actions設定
```yaml
# .github/workflows/test.yml
name: Test Suite

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  test:
    name: Test Suite
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        rust: [stable, beta]
        
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: ${{ matrix.rust }}
        profile: minimal
        override: true
        components: rustfmt, clippy
    
    - name: Cache dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target/
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Check formatting
      run: cargo fmt --all -- --check
    
    - name: Lint with clippy
      run: cargo clippy --all-targets --all-features -- -D warnings
    
    - name: Run unit tests
      run: cargo test --lib
    
    - name: Run property-based tests
      run: cargo test --test property_tests
      env:
        PROPTEST_CASES: 1000
    
    - name: Run integration tests
      run: cargo test --test integration_tests
    
    - name: Run security tests
      run: cargo test --test security_tests
    
    - name: Generate test coverage
      run: |
        cargo install cargo-tarpaulin
        cargo tarpaulin --out Xml
    
    - name: Upload coverage to Codecov
      uses: codecov/codecov-action@v3
      with:
        file: ./cobertura.xml
        fail_ci_if_error: true

  performance:
    name: Performance Tests
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        profile: minimal
        override: true
    
    - name: Run benchmarks
      run: cargo bench
    
    - name: Store benchmark results
      uses: benchmark-action/github-action-benchmark@v1
      with:
        tool: 'cargo'
        output-file-path: target/criterion/benchmark.json
        github-token: ${{ secrets.GITHUB_TOKEN }}
        auto-push: true

  security:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    
    - name: Security audit
      run: |
        cargo install cargo-audit
        cargo audit
    
    - name: Dependency check
      run: |
        cargo install cargo-outdated
        cargo outdated
```

#### 2. テスト実行スクリプト
```bash
#!/bin/bash
# scripts/run_tests.sh

set -e

echo "🧪 Running Zoom Video Mover Test Suite"

# 基本的な品質チェック
echo "📋 Code formatting check..."
cargo fmt --all -- --check

echo "📋 Linting with clippy..."
cargo clippy --all-targets --all-features -- -D warnings

echo "🔍 Type checking..."
cargo check --all-targets

# テスト実行
echo "🧪 Unit tests..."
cargo test --lib

echo "🧪 Property-based tests..."
PROPTEST_CASES=1000 cargo test --test property_tests -- --nocapture

echo "🧪 Integration tests..."
cargo test --test integration_tests

echo "🔒 Security tests..."
cargo test --test security_tests

echo "⚡ Performance tests..."
cargo test --test performance_tests --release

# カバレッジ生成
echo "📊 Generating test coverage..."
cargo tarpaulin --out Html

# ベンチマーク実行
echo "🏃 Running benchmarks..."
cargo bench

echo "✅ All tests completed successfully!"
```

### テスト品質メトリクス

#### 1. カバレッジ測定
```toml
# Cargo.toml - tarpaulin設定
[package.metadata.tarpaulin]
exclude_files = [
    "src/main.rs",
    "tests/*",
    "benches/*",
]
ignore_panics = true
run_types = ["Tests", "Doctests"]
timeout = 300
```

#### 2. 品質ゲート定義
```rust
// tests/quality_gates.rs
#[test]
fn test_coverage_requirements() {
    let coverage_report = load_coverage_report();
    
    // 最低カバレッジ要求
    assert!(coverage_report.line_coverage >= 0.90, 
            "Line coverage must be at least 90%: {:.1}%", 
            coverage_report.line_coverage * 100.0);
    
    assert!(coverage_report.branch_coverage >= 0.85,
            "Branch coverage must be at least 85%: {:.1}%",
            coverage_report.branch_coverage * 100.0);
    
    // 重要モジュールの高カバレッジ要求
    let oauth_coverage = coverage_report.module_coverage.get("oauth").unwrap();
    assert!(oauth_coverage >= &0.95,
            "OAuth module coverage must be at least 95%: {:.1}%",
            oauth_coverage * 100.0);
}

#[test]
fn test_performance_requirements() {
    let benchmark_results = load_benchmark_results();
    
    // パフォーマンス要求
    assert!(benchmark_results.config_load_time < Duration::from_millis(100),
            "Config loading should complete within 100ms");
    
    assert!(benchmark_results.auth_url_generation_time < Duration::from_millis(50),
            "Auth URL generation should complete within 50ms");
    
    assert!(benchmark_results.parallel_download_efficiency > 0.8,
            "Parallel download efficiency should be above 80%");
}
```

## 結論

本テスト方針は、**包括的・自動化・継続的**な品質保証を実現する現代的なテスト戦略を提供します。

### テスト方針の特徴
- **Property-based Testing中心**: データ整合性の自動検証による高信頼性
- **多層テスト構造**: Unit・Integration・Performance・Securityの全面的カバー
- **自動化優先**: CI/CDパイプライン統合による継続的品質保証
- **リスクベース**: 重要度に応じた適切なテスト優先度設定
- **セキュリティ重視**: 認証・暗号化・入力検証の包括的テスト
- **性能保証**: ベンチマーク・負荷テストによる性能品質確保

### 期待効果
- **品質向上**: 多角的なテストによる高品質ソフトウェア実現
- **開発効率**: 自動化によるテスト工数削減・早期問題発見
- **信頼性確保**: Property-basedテストによる予期しないバグの防止
- **セキュリティ強化**: 包括的セキュリティテストによる脆弱性排除
- **性能保証**: 継続的な性能監視による最適なユーザー体験
- **保守性向上**: 回帰テストによる安全な機能追加・修正

この包括的なテスト方針により、**ユーザーの期待を超える高品質・高性能・高セキュリティ**のソフトウェアシステムを実現します。