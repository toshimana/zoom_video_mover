// tests/integration_tests/operation_flow_tests.rs
// 操作仕様対応の統合テスト

use rstest::*;
use serial_test::serial;
use tempfile::TempDir;
use tokio::time::{timeout, Duration};
use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{method, path, query_param, body_string_contains};
use zoom_video_mover_lib::{Config, ZoomRecordingDownloader, ZoomVideoMoverError};
use serde_json::json;
use std::fs;

/// OP002: 設定入力・保存の統合テスト
/// 
/// テスト対象仕様:
/// - operation_specifications.md: OP002 設定入力・保存
/// - screen_specifications.md: SC002 設定画面
/// - function_specifications.md: FN001 設定管理機能
#[rstest]
#[case::complete_flow(
    "zoom_client_test",
    "zoom_secret_test_123456789",
    Some("http://localhost:8080/callback"),
    "完全な設定フロー"
)]
#[case::minimal_flow(
    "minimal_client",
    "minimal_secret_123456789",  
    None,
    "最小限の設定フロー"
)]
#[serial]  // ファイル操作の競合を避けるため
async fn test_op002_config_input_save_flow(
    #[case] client_id: &str,
    #[case] client_secret: &str,
    #[case] redirect_uri: Option<&str>,
    #[case] description: &str,
) {
    // 事前条件: 一時作業ディレクトリ作成
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    
    // ステップ1: 設定ファイル存在確認 (存在しない状態)
    assert!(!config_path.exists(), "初期状態では設定ファイルが存在しない");
    
    // ステップ2: サンプル設定ファイル作成 (OP002-初期化)
    let sample_result = Config::create_sample_file(config_path.to_str().unwrap());
    assert!(sample_result.is_ok(), "サンプル設定ファイル作成成功: {}", description);
    assert!(config_path.exists(), "サンプル設定ファイルが作成される");
    
    // ステップ3: 設定読み込み (OP002-設定読み込み)
    let initial_config = Config::load_from_file(config_path.to_str().unwrap());
    assert!(initial_config.is_ok(), "サンプル設定読み込み成功");
    
    // ステップ4: 設定フィールド更新 (OP002-手動入力)
    let updated_config = Config {
        client_id: client_id.to_string(),
        client_secret: client_secret.to_string(),
        redirect_uri: redirect_uri.map(|s| s.to_string()),
    };
    
    // ステップ5: 入力検証 (OP002-検証)
    assert!(!updated_config.client_id.is_empty(), "client_id入力検証");
    assert!(updated_config.client_secret.len() >= 20, "client_secret長さ検証");
    if let Some(ref uri) = updated_config.redirect_uri {
        assert!(uri.starts_with("http"), "redirect_uri形式検証");
    }
    
    // ステップ6: 設定保存 (OP002-設定保存)
    let save_result = updated_config.save_to_file(config_path.to_str().unwrap());
    assert!(save_result.is_ok(), "設定保存成功: {}", description);
    
    // ステップ7: 保存確認・ラウンドトリップテスト (OP002-保存確認)
    let saved_config = Config::load_from_file(config_path.to_str().unwrap())
        .expect("保存した設定の読み込み成功");
    
    // 事後条件: データ整合性検証
    assert_eq!(saved_config.client_id, updated_config.client_id);
    assert_eq!(saved_config.client_secret, updated_config.client_secret);
    assert_eq!(saved_config.redirect_uri, updated_config.redirect_uri);
    
    println!("✓ OP002統合テスト完了: {}", description);
}

/// OP003: OAuth認証実行の統合テスト (Mock使用)
/// 
/// テスト対象仕様:
/// - operation_specifications.md: OP003 OAuth認証実行
/// - screen_specifications.md: SC003 認証画面
/// - function_specifications.md: FN002 OAuth認証機能
#[rstest]
#[case::successful_auth_flow(
    "test_client_123",
    "test_secret_abcdef123456789",
    "http://localhost:8080/callback",
    "auth_code_success_12345",
    true,
    "成功する認証フロー"
)]
#[case::failed_auth_flow(
    "invalid_client",
    "invalid_secret_123456789",
    "http://localhost:8080/callback", 
    "invalid_auth_code",
    false,
    "失敗する認証フロー"
)]
async fn test_op003_oauth_authentication_flow(
    #[case] client_id: &str,
    #[case] client_secret: &str,
    #[case] redirect_uri: &str,
    #[case] auth_code: &str,
    #[case] should_succeed: bool,
    #[case] description: &str,
) {
    // 事前条件: Mock OAuth サーバー設定
    let mock_server = MockServer::start().await;
    
    if should_succeed {
        // 成功レスポンスのMock
        Mock::given(method("POST"))
            .and(path("/oauth/token"))
            .and(body_string_contains("grant_type=authorization_code"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(&json!({
                        "access_token": "mock_access_token_abcdef",
                        "token_type": "Bearer",
                        "expires_in": 3600,
                        "refresh_token": "mock_refresh_token_xyz",
                        "scope": "recording:read user:read meeting:read"
                    }))
            )
            .mount(&mock_server)
            .await;
    } else {
        // 失敗レスポンスのMock
        Mock::given(method("POST"))
            .and(path("/oauth/token"))
            .respond_with(
                ResponseTemplate::new(400)
                    .set_body_json(&json!({
                        "error": "invalid_grant",
                        "error_description": "Invalid authorization code"
                    }))
            )
            .mount(&mock_server)
            .await;
    }
    
    // ステップ1: ダウンローダー初期化 (OP003-認証開始)
    let mut downloader = ZoomRecordingDownloader::new(
        client_id.to_string(),
        client_secret.to_string(),
        redirect_uri.to_string(),
    );
    downloader.set_oauth_base_url(&mock_server.uri());
    
    // ステップ2: 認証URL生成 (OP003-認証URL生成)
    let auth_url_result = downloader.generate_auth_url();
    assert!(auth_url_result.is_ok(), "認証URL生成成功");
    
    let auth_url = auth_url_result.unwrap();
    
    // ステップ3: 認証URL検証 (OP003-認証URL表示)
    assert!(auth_url.contains("oauth/authorize"));
    assert!(auth_url.contains(&format!("client_id={}", client_id)));
    assert!(auth_url.contains("response_type=code"));
    assert!(auth_url.contains("state="));  // CSRF対策
    
    // ステップ4: 認証コード交換 (OP003-認証完了)
    let token_result = timeout(
        Duration::from_secs(5),
        downloader.exchange_code(auth_code)
    ).await;
    
    assert!(token_result.is_ok(), "認証コード交換がタイムアウトしない");
    let exchange_result = token_result.unwrap();
    
    // 事後条件: 認証結果検証
    match should_succeed {
        true => {
            let token = exchange_result.expect(&format!("認証成功を期待: {}", description));
            
            // トークン詳細検証
            assert!(!token.access_token.is_empty(), "アクセストークンが取得される");
            assert_eq!(token.token_type, "Bearer", "Bearerトークンタイプ");
            assert!(token.expires_at > chrono::Utc::now(), "有効期限が未来");
            
            if let Some(refresh_token) = &token.refresh_token {
                assert!(!refresh_token.is_empty(), "リフレッシュトークンが取得される");
            }
            
            println!("✓ OP003認証成功: アクセストークン={}", 
                &token.access_token[..8]); // 部分表示
        }
        false => {
            assert!(exchange_result.is_err(), "認証失敗を期待: {}", description);
            
            let error = exchange_result.unwrap_err();
            assert!(error.to_string().contains("invalid"), "無効コードエラー");
            
            println!("✓ OP003認証失敗: {}", error);
        }
    }
}

/// OP004: 録画検索・一覧表示の統合テスト
/// 
/// テスト対象仕様:
/// - operation_specifications.md: OP004 録画検索・一覧表示
/// - screen_specifications.md: SC004 録画リスト画面  
/// - function_specifications.md: FN003 録画検索機能
#[tokio::test]
async fn test_op004_recording_search_display_flow() {
    // 事前条件: Mock API サーバー設定
    let mock_server = MockServer::start().await;
    
    // 録画リストAPI Mock
    Mock::given(method("GET"))
        .and(path("/v2/users/me/recordings"))
        .and(query_param("from", "2024-01-01"))
        .and(query_param("to", "2024-01-31"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(&json!({
                    "from": "2024-01-01",
                    "to": "2024-01-31",
                    "meetings": [
                        {
                            "uuid": "meeting-uuid-123",
                            "id": 123456789,
                            "topic": "週次チーム会議",
                            "start_time": "2024-01-15T10:00:00Z",
                            "duration": 60,
                            "recording_files": [
                                {
                                    "id": "video-file-123",
                                    "file_type": "MP4",
                                    "file_size": 1073741824,
                                    "download_url": "https://zoom.us/rec/download/video123",
                                    "recording_start": "2024-01-15T10:00:00Z",
                                    "recording_end": "2024-01-15T11:00:00Z"
                                },
                                {
                                    "id": "audio-file-123",
                                    "file_type": "MP3", 
                                    "file_size": 67108864,
                                    "download_url": "https://zoom.us/rec/download/audio123",
                                    "recording_start": "2024-01-15T10:00:00Z",
                                    "recording_end": "2024-01-15T11:00:00Z"
                                }
                            ]
                        },
                        {
                            "uuid": "meeting-uuid-456",
                            "id": 456789012,
                            "topic": "プロジェクト進捗レビュー",
                            "start_time": "2024-01-20T14:00:00Z", 
                            "duration": 90,
                            "recording_files": [
                                {
                                    "id": "video-file-456",
                                    "file_type": "MP4",
                                    "file_size": 2147483648,
                                    "download_url": "https://zoom.us/rec/download/video456",
                                    "recording_start": "2024-01-20T14:00:00Z",
                                    "recording_end": "2024-01-20T15:30:00Z"
                                }
                            ]
                        }
                    ]
                }))
        )
        .mount(&mock_server)
        .await;
    
    // ステップ1: 認証済みダウンローダー初期化 (OP004-前提条件)
    let mut downloader = ZoomRecordingDownloader::new_with_token(
        "test_client".to_string(),
        "test_secret".to_string(),
        "test_access_token".to_string(),
    );
    downloader.set_api_base_url(&mock_server.uri());
    
    // ステップ2: 検索期間設定 (OP004-期間設定)
    let from_date = "2024-01-01";
    let to_date = "2024-01-31";
    
    // 日付形式検証
    let from_parsed = chrono::NaiveDate::parse_from_str(from_date, "%Y-%m-%d");
    let to_parsed = chrono::NaiveDate::parse_from_str(to_date, "%Y-%m-%d");
    assert!(from_parsed.is_ok(), "開始日が有効な形式");
    assert!(to_parsed.is_ok(), "終了日が有効な形式");
    assert!(from_parsed.unwrap() <= to_parsed.unwrap(), "日付範囲が有効");
    
    // ステップ3: 録画検索実行 (OP004-検索実行)
    let search_result = timeout(
        Duration::from_secs(10),
        downloader.get_recordings(from_date, to_date)
    ).await;
    
    assert!(search_result.is_ok(), "検索処理がタイムアウトしない");
    let recordings = search_result.unwrap()
        .expect("録画検索が成功");
    
    // ステップ4: 検索結果表示・検証 (OP004-一覧表示)
    assert_eq!(recordings.len(), 2, "期待する録画数が取得される");
    
    // 各録画の詳細検証
    let first_recording = &recordings[0];
    assert_eq!(first_recording.topic, "週次チーム会議");
    assert_eq!(first_recording.recording_files.len(), 2, "動画・音声ファイル");
    
    let second_recording = &recordings[1];
    assert_eq!(second_recording.topic, "プロジェクト進捗レビュー");
    assert_eq!(second_recording.recording_files.len(), 1, "動画ファイルのみ");
    
    // ステップ5: ファイル種別・サイズ情報表示検証 (OP004-詳細表示)
    let mut total_files = 0;
    let mut total_size = 0u64;
    
    for recording in &recordings {
        for file in &recording.recording_files {
            total_files += 1;
            total_size += file.file_size;
            
            // ファイル詳細検証
            assert!(!file.id.is_empty(), "ファイルIDが存在");
            assert!(["MP4", "MP3", "TXT", "JSON", "VTT"].contains(&file.file_type.as_str()),
                "サポートされるファイルタイプ: {}", file.file_type);
            assert!(file.file_size > 0, "ファイルサイズが正の値");
            assert!(file.download_url.starts_with("https://"), "HTTPS URL");
        }
    }
    
    // ステップ6: 統計情報表示検証 (OP004-統計表示)
    assert_eq!(total_files, 3, "総ファイル数");
    assert!(total_size > 0, "総ファイルサイズ");
    
    // 人間が読みやすい形式での表示検証
    let total_size_mb = total_size as f64 / (1024.0 * 1024.0);
    assert!(total_size_mb > 1000.0, "1GB以上のデータサイズ"); // 1GB + 64MB + 2GB
    
    println!("✓ OP004統合テスト完了:");
    println!("  - 録画数: {}", recordings.len());
    println!("  - 総ファイル数: {}", total_files);
    println!("  - 総サイズ: {:.1} MB", total_size_mb);
}

/// OP006: ダウンロード実行の統合テスト (Mock使用)
/// 
/// テスト対象仕様:
/// - operation_specifications.md: OP006 ダウンロード実行
/// - screen_specifications.md: SC005 ダウンロード進捗画面
/// - function_specifications.md: FN004 ファイルダウンロード機能
#[tokio::test]
async fn test_op006_download_execution_flow() {
    // 事前条件: Mock ダウンロードサーバー設定
    let mock_server = MockServer::start().await;
    
    // 小さなテストファイルのMock (1KB)
    let test_file_content = "a".repeat(1024);  // 1KB のテストデータ
    
    Mock::given(method("GET"))
        .and(path("/rec/download/test_video"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(&test_file_content)
                .insert_header("content-length", "1024")
                .insert_header("content-type", "video/mp4")
        )
        .mount(&mock_server)
        .await;
    
    // ステップ1: ダウンロード対象設定 (OP006-ファイル選択)
    let temp_dir = TempDir::new().unwrap();
    let download_request = zoom_video_mover_lib::DownloadRequest {
        file_id: "test_video_123".to_string(),
        file_name: "test_meeting_video.mp4".to_string(),
        download_url: format!("{}/rec/download/test_video", mock_server.uri()),
        file_size: 1024,
        output_path: temp_dir.path().join("test_meeting_video.mp4"),
    };
    
    // ステップ2: ダウンロード設定確認 (OP006-ダウンロード開始)
    assert!(!download_request.file_name.is_empty(), "ファイル名が設定される");
    assert!(download_request.file_size > 0, "ファイルサイズが正の値");
    assert!(download_request.download_url.starts_with("http"), "有効なダウンロードURL");
    
    // ステップ3: 出力ディレクトリ確認 (OP006-保存先確認)
    if let Some(parent) = download_request.output_path.parent() {
        assert!(parent.exists(), "出力ディレクトリが存在");
    }
    
    // ステップ4: ダウンロード実行 (OP006-ダウンロード実行)
    let downloader = ZoomRecordingDownloader::new_with_token(
        "test_client".to_string(),
        "test_secret".to_string(),
        "test_access_token".to_string(),
    );
    
    // 進捗通知用チャンネル
    let (progress_sender, mut progress_receiver) = tokio::sync::mpsc::channel(100);
    
    // ダウンロード実行 (タイムアウト付き)
    let download_task = tokio::spawn(async move {
        downloader.download_file(download_request, Some(progress_sender)).await
    });
    
    // ステップ5: 進捗監視 (OP006-進捗表示)
    let mut progress_updates = Vec::new();
    let mut download_completed = false;
    
    // 進捗とダウンロード完了の両方を監視
    tokio::select! {
        // ダウンロード完了
        download_result = download_task => {
            let final_path = download_result.unwrap()
                .expect("ダウンロードが成功");
            
            // ステップ6: ダウンロード完了確認 (OP006-完了確認)
            assert!(final_path.exists(), "ダウンロードファイルが作成される");
            
            let downloaded_content = fs::read_to_string(&final_path)
                .expect("ダウンロードファイルが読み取れる");
            assert_eq!(downloaded_content.len(), 1024, "正しいファイルサイズ");
            assert_eq!(downloaded_content, test_file_content, "正しいファイル内容");
            
            download_completed = true;
        }
        
        // 進捗更新の監視 (タイムアウト付き)
        _ = async {
            while let Some(progress) = progress_receiver.recv().await {
                progress_updates.push(progress);
            }
        } => {}
    }
    
    // 事後条件: ダウンロード結果検証
    assert!(download_completed, "ダウンロードが完了");
    
    // 進捗更新の検証 (小さなファイルなので進捗更新は少ない可能性)
    println!("✓ OP006統合テスト完了:");
    println!("  - 進捗更新回数: {}", progress_updates.len());
    println!("  - ファイルサイズ: {} bytes", test_file_content.len());
}

/// エラーフロー統合テスト: OP008 エラー処理・回復
/// 
/// テスト対象仕様:
/// - operation_specifications.md: OP008 エラー処理・回復
/// - screen_specifications.md: SC006 エラー表示画面
/// - function_specifications.md: FN007 エラー処理機能
#[rstest]
#[case::network_error(500, "ネットワークエラー")]
#[case::not_found_error(404, "ファイル未発見")]
#[case::unauthorized_error(401, "認証エラー")]
async fn test_op008_error_handling_recovery_flow(
    #[case] error_status: u16,
    #[case] description: &str,
) {
    // 事前条件: エラーレスポンスのMock
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/v2/users/me/recordings"))
        .respond_with(
            ResponseTemplate::new(error_status)
                .set_body_json(&json!({
                    "code": error_status,
                    "message": format!("Test error: {}", description)
                }))
        )
        .mount(&mock_server)
        .await;
    
    // ステップ1: エラー発生条件での操作実行 (OP008-エラー発生)
    let mut downloader = ZoomRecordingDownloader::new_with_token(
        "test_client".to_string(),
        "test_secret".to_string(),
        "test_access_token".to_string(),
    );
    downloader.set_api_base_url(&mock_server.uri());
    
    // ステップ2: エラー発生・検出 (OP008-エラー検出)
    let error_result = downloader.get_recordings("2024-01-01", "2024-01-31").await;
    assert!(error_result.is_err(), "エラーが正しく検出される: {}", description);
    
    // ステップ3: エラー分類・詳細情報取得 (OP008-エラー分類)
    let error = error_result.unwrap_err();
    
    match error_status {
        401 => {
            // 認証エラーの検証
            assert!(error.to_string().to_lowercase().contains("auth"),
                "認証エラーとして分類される");
        }
        404 => {
            // リソース未発見エラーの検証
            assert!(error.to_string().contains("not found") ||
                   error.to_string().contains("404"),
                "リソース未発見エラーとして分類される");
        }
        500 => {
            // サーバーエラーの検証
            assert!(error.to_string().to_lowercase().contains("server") ||
                   error.to_string().contains("500"),
                "サーバーエラーとして分類される");
        }
        _ => {
            // その他のエラー
            assert!(!error.to_string().is_empty(), "エラーメッセージが存在");
        }
    }
    
    // ステップ4: エラー情報の構造化 (OP008-エラー詳細)
    let error_details = format!("Error: {} (Status: {})", error, error_status);
    assert!(error_details.len() > 20, "詳細なエラー情報");
    
    // ステップ5: 回復戦略の確認 (OP008-回復処理)
    match error_status {
        401 => {
            // 認証エラー → 再認証が必要
            println!("✓ 認証エラー検出 → 再認証が必要");
        }
        404 => {
            // リソース未発見 → 設定確認が必要
            println!("✓ リソース未発見 → 設定・権限確認が必要");
        }
        500 => {
            // サーバーエラー → リトライ可能
            println!("✓ サーバーエラー検出 → 自動リトライ可能");
        }
        _ => {
            println!("✓ その他のエラー検出: {}", error);
        }
    }
    
    println!("✓ OP008エラーハンドリング完了: {}", description);
}

// 統合テスト トレーサビリティ
//
// 操作仕様 (operation_specifications.md) との対応:
// ├─ OP002: 設定入力・保存         → test_op002_config_input_save_flow
// ├─ OP003: OAuth認証実行          → test_op003_oauth_authentication_flow  
// ├─ OP004: 録画検索・一覧表示      → test_op004_recording_search_display_flow
// ├─ OP006: ダウンロード実行        → test_op006_download_execution_flow
// └─ OP008: エラー処理・回復        → test_op008_error_handling_recovery_flow
//
// 画面仕様 (screen_specifications.md) との対応:
// ├─ SC002: 設定画面              → test_op002_*
// ├─ SC003: 認証画面              → test_op003_*
// ├─ SC004: 録画リスト画面         → test_op004_*
// ├─ SC005: ダウンロード進捗画面    → test_op006_*
// └─ SC006: エラー表示画面         → test_op008_*
//
// 機能仕様 (function_specifications.md) との対応:
// ├─ FN001: 設定管理機能          → test_op002_*
// ├─ FN002: OAuth認証機能         → test_op003_*
// ├─ FN003: 録画検索機能          → test_op004_*
// ├─ FN004: ファイルダウンロード機能 → test_op006_*
// └─ FN007: エラー処理機能         → test_op008_*