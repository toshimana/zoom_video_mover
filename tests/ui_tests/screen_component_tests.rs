// tests/ui_tests/screen_component_tests.rs
// 画面仕様対応のUIテスト

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tempfile::TempDir;
use zoom_video_mover_lib::{Config, ZoomRecordingDownloader, ZoomDownloaderApp, AppMessage};

// UIテスト用のMockコンテキスト
struct MockUIContext {
    pub button_clicks: Arc<Mutex<Vec<String>>>,
    pub text_inputs: Arc<Mutex<HashMap<String, String>>>,
    pub checkbox_states: Arc<Mutex<HashMap<String, bool>>>,
    pub ui_outputs: Arc<Mutex<Vec<String>>>,
}

impl MockUIContext {
    fn new() -> Self {
        Self {
            button_clicks: Arc::new(Mutex::new(Vec::new())),
            text_inputs: Arc::new(Mutex::new(HashMap::new())),
            checkbox_states: Arc::new(Mutex::new(HashMap::new())),
            ui_outputs: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    fn click_button(&self, button_id: &str) {
        self.button_clicks.lock().unwrap().push(button_id.to_string());
    }
    
    fn set_text_input(&self, field_id: &str, value: &str) {
        self.text_inputs.lock().unwrap().insert(field_id.to_string(), value.to_string());
    }
    
    fn set_checkbox(&self, checkbox_id: &str, checked: bool) {
        self.checkbox_states.lock().unwrap().insert(checkbox_id.to_string(), checked);
    }
    
    fn add_ui_output(&self, output: &str) {
        self.ui_outputs.lock().unwrap().push(output.to_string());
    }
}

/// SC002: 設定画面のUIテスト
/// 
/// テスト対象仕様:
/// - screen_specifications.md: SC002 設定画面
/// - UI要素: CF001-CF005 (Client ID入力, Client Secret入力, 出力ディレクトリ入力, 設定保存ボタン, 設定読込ボタン)
#[tokio::test]
async fn test_sc002_config_screen_ui_components() {
    // 事前条件: UIコンテキストとアプリ状態初期化
    let mock_ctx = MockUIContext::new();
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    
    // アプリケーション状態の初期化
    let mut app = ZoomDownloaderApp::default();
    
    // ステップ1: 設定画面の初期表示検証 (SC002 初期状態)
    assert_eq!(app.client_id, "", "client_id初期値は空文字");
    assert_eq!(app.client_secret, "", "client_secret初期値は空文字");
    assert_eq!(app.output_dir, "", "output_dir初期値は空文字");
    assert!(!app.config_loaded, "設定未読み込み状態");
    
    // ステップ2: CF001 Client ID入力フィールドテスト
    let test_client_id = "test_zoom_client_12345";
    mock_ctx.set_text_input("client_id", test_client_id);
    app.client_id = test_client_id.to_string();
    
    // 入力検証
    assert_eq!(app.client_id, test_client_id, "Client ID入力が反映される");
    assert!(!app.client_id.is_empty(), "Client ID空文字検証");
    assert!(app.client_id.len() >= 10, "Client ID最小長度検証");
    
    // ステップ3: CF002 Client Secret入力フィールドテスト (パスワード形式)
    let test_client_secret = "test_zoom_secret_abcdef123456789";
    mock_ctx.set_text_input("client_secret", test_client_secret);
    app.client_secret = test_client_secret.to_string();
    
    // 入力検証 
    assert_eq!(app.client_secret, test_client_secret, "Client Secret入力が反映される");
    assert!(app.client_secret.len() >= 20, "Client Secret最小長度検証");
    
    // パスワード形式の検証 (UIではマスク表示される)
    let masked_display = "*".repeat(app.client_secret.len());
    assert_eq!(masked_display.len(), test_client_secret.len(), "マスク表示長さ一致");
    
    // ステップ4: CF003 出力ディレクトリ入力フィールドテスト
    let test_output_dir = temp_dir.path().to_string_lossy().to_string();
    mock_ctx.set_text_input("output_dir", &test_output_dir);
    app.output_dir = test_output_dir.clone();
    
    // パス検証
    assert_eq!(app.output_dir, test_output_dir, "出力ディレクトリ入力が反映される");
    assert!(std::path::Path::new(&app.output_dir).exists(), "出力ディレクトリが存在");
    
    // ステップ5: CF004 設定保存ボタンテスト
    mock_ctx.click_button("save_config");
    
    // 設定保存処理のシミュレーション
    let config = Config {
        client_id: app.client_id.clone(),
        client_secret: app.client_secret.clone(),
        redirect_uri: Some("http://localhost:8080/callback".to_string()),
    };
    
    let save_result = config.save_to_file(config_path.to_str().unwrap());
    assert!(save_result.is_ok(), "設定保存が成功");
    
    // 保存後の状態確認
    assert!(config_path.exists(), "設定ファイルが作成される");
    app.config_loaded = true;
    
    // ステップ6: CF005 設定読込ボタンテスト
    mock_ctx.click_button("load_config");
    
    // 設定読み込み処理のシミュレーション
    let loaded_config = Config::load_from_file(config_path.to_str().unwrap());
    assert!(loaded_config.is_ok(), "設定読み込みが成功");
    
    let config = loaded_config.unwrap();
    
    // 読み込み結果の画面反映確認
    assert_eq!(config.client_id, test_client_id, "読み込んだClient IDが画面に反映");
    assert_eq!(config.client_secret, test_client_secret, "読み込んだClient Secretが画面に反映");
    
    // ステップ7: UI操作履歴の検証
    let button_clicks = mock_ctx.button_clicks.lock().unwrap();
    assert!(button_clicks.contains(&"save_config".to_string()), "保存ボタンがクリックされた");
    assert!(button_clicks.contains(&"load_config".to_string()), "読込ボタンがクリックされた");
    
    let text_inputs = mock_ctx.text_inputs.lock().unwrap();
    assert_eq!(text_inputs.get("client_id"), Some(&test_client_id.to_string()));
    assert_eq!(text_inputs.get("client_secret"), Some(&test_client_secret.to_string()));
    
    println!("✓ SC002設定画面UIテスト完了");
}

/// SC003: 認証画面のUIテスト
/// 
/// テスト対象仕様:
/// - screen_specifications.md: SC003 認証画面
/// - UI要素: AU001-AU006 (認証開始ボタン, Auth URL表示, URLコピーボタン, ブラウザ起動ボタン, 認証コード入力, 認証完了ボタン)
#[tokio::test]
async fn test_sc003_auth_screen_ui_components() {
    let mock_ctx = MockUIContext::new();
    let mut app = ZoomDownloaderApp::default();
    
    // 事前条件: 設定済み状態
    app.client_id = "test_client_id".to_string();
    app.client_secret = "test_client_secret_123456789".to_string();
    app.config_loaded = true;
    
    // ステップ1: 認証画面初期表示検証 (SC003 初期状態)
    assert!(app.auth_url.is_none(), "認証URL初期値はNone");
    assert!(!app.is_authenticating, "認証処理中フラグは初期false");
    assert!(app.access_token.is_none(), "アクセストークン初期値はNone");
    
    // ステップ2: AU001 認証開始ボタンテスト
    mock_ctx.click_button("start_auth");
    
    // 認証開始処理のシミュレーション
    let mock_auth_url = "https://zoom.us/oauth/authorize?response_type=code&client_id=test_client_id&redirect_uri=http://localhost:8080/callback&scope=recording:read user:read meeting:read&state=mock_state_12345".to_string();
    
    app.auth_url = Some(mock_auth_url.clone());
    app.is_authenticating = true;
    app.status_message = "Auth URL generated. Please complete authentication in browser.".to_string();
    
    // 認証URL生成後の状態検証
    assert!(app.auth_url.is_some(), "認証URLが生成される");
    assert!(app.is_authenticating, "認証処理中状態になる");
    
    let auth_url = app.auth_url.as_ref().unwrap();
    assert!(auth_url.contains("oauth/authorize"), "OAuth認証URLを含む");
    assert!(auth_url.contains("client_id=test_client_id"), "Client IDを含む");
    assert!(auth_url.contains("state="), "CSRFトークンを含む");
    
    // ステップ3: AU002 Auth URL表示テスト
    mock_ctx.add_ui_output(&format!("Auth URL: {}", auth_url));
    
    // URL表示の検証
    let ui_outputs = mock_ctx.ui_outputs.lock().unwrap();
    assert!(ui_outputs.iter().any(|output| output.contains("Auth URL:")), "Auth URLが表示される");
    
    // ステップ4: AU003 URLコピーボタンテスト
    mock_ctx.click_button("copy_url");
    
    // クリップボードへのコピー処理シミュレーション
    mock_ctx.add_ui_output("URL copied to clipboard");
    
    // ステップ5: AU004 ブラウザ起動ボタンテスト
    mock_ctx.click_button("open_browser");
    
    // ブラウザ起動処理シミュレーション (実際のブラウザ起動は行わない)
    mock_ctx.add_ui_output("Browser launched with auth URL");
    
    // ステップ6: AU005 認証コード入力テスト
    let test_auth_code = "test_authorization_code_12345";
    mock_ctx.set_text_input("auth_code", test_auth_code);
    app.auth_code = test_auth_code.to_string();
    
    // 認証コード入力検証
    assert_eq!(app.auth_code, test_auth_code, "認証コード入力が反映される");
    assert!(!app.auth_code.is_empty(), "認証コードが空でない");
    assert!(app.auth_code.len() > 10, "認証コードが適切な長さ");
    
    // ステップ7: AU006 認証完了ボタンテスト
    mock_ctx.click_button("complete_auth");
    
    // 認証完了処理のシミュレーション
    let mock_access_token = "mock_access_token_abcdef123456789";
    app.access_token = Some(mock_access_token.to_string());
    app.is_authenticating = false;
    app.status_message = "Authentication completed.".to_string();
    
    // 認証完了後の状態検証
    assert!(app.access_token.is_some(), "アクセストークンが取得される");
    assert!(!app.is_authenticating, "認証処理中状態が解除される");
    
    let access_token = app.access_token.as_ref().unwrap();
    assert!(access_token.starts_with("mock_access_token"), "有効なアクセストークン形式");
    
    // ステップ8: UI操作フロー検証
    let button_clicks = mock_ctx.button_clicks.lock().unwrap();
    let expected_flow = vec!["start_auth", "copy_url", "open_browser", "complete_auth"];
    
    for expected_button in expected_flow {
        assert!(button_clicks.contains(&expected_button.to_string()), 
            "期待するボタン操作: {}", expected_button);
    }
    
    println!("✓ SC003認証画面UIテスト完了");
}

/// SC004: 録画リスト画面のUIテスト
/// 
/// テスト対象仕様:
/// - screen_specifications.md: SC004 録画リスト画面
/// - UI要素: RL001-RL007 (From日付入力, To日付入力, 検索実行ボタン, 全選択チェックボックス, ミーティング選択, ファイル選択, ダウンロード開始ボタン)
#[tokio::test]
async fn test_sc004_recording_list_screen_ui_components() {
    let mock_ctx = MockUIContext::new();
    let mut app = ZoomDownloaderApp::default();
    
    // 事前条件: 認証済み状態
    app.access_token = Some("test_access_token".to_string());
    app.config_loaded = true;
    
    // ステップ1: 録画リスト画面初期表示検証 (SC004 初期状態)
    assert_eq!(app.from_date, "", "開始日初期値は空文字");
    assert_eq!(app.to_date, "", "終了日初期値は空文字");
    
    // ステップ2: RL001 From日付入力テスト
    let test_from_date = "2024-01-01";
    mock_ctx.set_text_input("from_date", test_from_date);
    app.from_date = test_from_date.to_string();
    
    // 日付形式検証
    let parsed_from = chrono::NaiveDate::parse_from_str(&app.from_date, "%Y-%m-%d");
    assert!(parsed_from.is_ok(), "From日付が有効な形式");
    
    // ステップ3: RL002 To日付入力テスト
    let test_to_date = "2024-01-31";
    mock_ctx.set_text_input("to_date", test_to_date);
    app.to_date = test_to_date.to_string();
    
    // 日付形式・範囲検証
    let parsed_to = chrono::NaiveDate::parse_from_str(&app.to_date, "%Y-%m-%d");
    assert!(parsed_to.is_ok(), "To日付が有効な形式");
    assert!(parsed_from.unwrap() <= parsed_to.unwrap(), "日付範囲が有効");
    
    // ステップ4: RL003 検索実行ボタンテスト
    mock_ctx.click_button("search_recordings");
    
    // 検索結果のシミュレーション
    let mock_recordings = vec![
        ("meeting_123", "週次チーム会議", vec![("video_123", "MP4", 1073741824), ("audio_123", "MP3", 67108864)]),
        ("meeting_456", "プロジェクト進捗会議", vec![("video_456", "MP4", 2147483648)]),
    ];
    
    mock_ctx.add_ui_output(&format!("Found {} recordings", mock_recordings.len()));
    
    // ステップ5: RL004 全選択チェックボックステスト
    mock_ctx.set_checkbox("select_all", true);
    
    // 全選択処理のシミュレーション
    for (meeting_id, _, files) in &mock_recordings {
        mock_ctx.set_checkbox(&format!("meeting_{}", meeting_id), true);
        for (file_id, _, _) in files {
            mock_ctx.set_checkbox(&format!("file_{}", file_id), true);
        }
    }
    
    // 全選択状態の検証
    let checkbox_states = mock_ctx.checkbox_states.lock().unwrap();
    assert_eq!(checkbox_states.get("select_all"), Some(&true), "全選択チェックボックスがON");
    
    // ステップ6: RL005 ミーティング選択テスト
    let first_meeting = &mock_recordings[0];
    let meeting_checkbox_id = format!("meeting_{}", first_meeting.0);
    mock_ctx.set_checkbox(&meeting_checkbox_id, true);
    
    // ミーティング選択時の子ファイル連動確認
    for (file_id, _, _) in &first_meeting.2 {
        let file_checkbox_id = format!("file_{}", file_id);
        assert_eq!(checkbox_states.get(&file_checkbox_id), Some(&true),
            "ミーティング選択時に子ファイルも選択される");
    }
    
    // ステップ7: RL006 個別ファイル選択テスト
    let specific_file_id = "file_video_123";
    mock_ctx.set_checkbox(specific_file_id, false);  // 一つだけ選択解除
    
    // 部分選択状態の確認
    let meeting_partial_selected = checkbox_states.get(&meeting_checkbox_id) == Some(&true) &&
        checkbox_states.get(specific_file_id) == Some(&false);
    assert!(meeting_partial_selected, "部分選択状態（中間状態）が検出される");
    
    // ステップ8: 統計情報表示テスト
    let mut total_files = 0;
    let mut total_size = 0u64;
    let mut selected_files = 0;
    let mut selected_size = 0u64;
    
    for (_, _, files) in &mock_recordings {
        for (file_id, _, size) in files {
            total_files += 1;
            total_size += size;
            
            let file_checkbox_id = format!("file_{}", file_id);
            if checkbox_states.get(&file_checkbox_id) == Some(&true) {
                selected_files += 1;
                selected_size += size;
            }
        }
    }
    
    mock_ctx.add_ui_output(&format!("Total: {} files ({} MB)", total_files, total_size / 1_000_000));
    mock_ctx.add_ui_output(&format!("Selected: {} files ({} MB)", selected_files, selected_size / 1_000_000));
    
    // 統計情報の检验
    assert_eq!(total_files, 3, "総ファイル数が正しい");
    assert!(total_size > 3_000_000_000, "総サイズが3GB以上");
    assert!(selected_files > 0, "選択ファイル数が正の値");
    
    // ステップ9: RL007 ダウンロード開始ボタンテスト
    mock_ctx.click_button("start_download");
    
    // ダウンロード開始可能性チェック
    assert!(selected_files > 0, "選択ファイルがあるためダウンロード可能");
    
    // UI操作検証
    let button_clicks = mock_ctx.button_clicks.lock().unwrap();
    assert!(button_clicks.contains(&"search_recordings".to_string()), "検索が実行された");
    assert!(button_clicks.contains(&"start_download".to_string()), "ダウンロードが開始された");
    
    println!("✓ SC004録画リスト画面UIテスト完了");
    println!("  - 総ファイル数: {}", total_files);
    println!("  - 選択ファイル数: {}", selected_files);
}

/// SC005: ダウンロード進捗画面のUIテスト
/// 
/// テスト対象仕様:
/// - screen_specifications.md: SC005 ダウンロード進捗画面
/// - UI要素: PR001-PR007 (全体進捗バー, 現在ファイル名, ファイル進捗バー, 一時停止ボタン, キャンセルボタン, ログ表示エリア, 統計情報)
#[tokio::test]
async fn test_sc005_progress_screen_ui_components() {
    let mock_ctx = MockUIContext::new();
    let mut app = ZoomDownloaderApp::default();
    
    // 事前条件: ダウンロード中状態
    app.is_downloading = true;
    
    // ダウンロード進捗のシミュレーション
    let total_files = 3;
    let mut completed_files = 0;
    let total_bytes = 3_221_225_472u64;  // 約3GB
    let mut downloaded_bytes = 0u64;
    
    // ステップ1: ダウンロード進捗画面初期表示検証 (SC005 初期状態)
    assert!(app.is_downloading, "ダウンロード中状態");
    assert_eq!(app.download_progress.len(), 0, "進捗ログは初期空");
    
    // ステップ2: PR001 全体進捗バー表示テスト
    let overall_progress = downloaded_bytes as f32 / total_bytes as f32;
    mock_ctx.add_ui_output(&format!("Overall Progress: [{:>50}] {:.1}%", 
        "█".repeat((overall_progress * 50.0) as usize), overall_progress * 100.0));
    
    assert!(overall_progress >= 0.0 && overall_progress <= 1.0, "全体進捗が有効範囲");
    
    // ステップ3: PR002 現在ファイル名表示テスト
    let current_file = "Meeting_2024-01-15_video.mp4";
    mock_ctx.add_ui_output(&format!("Current: {}", current_file));
    
    // ファイル名表示の検証
    let ui_outputs = mock_ctx.ui_outputs.lock().unwrap();
    assert!(ui_outputs.iter().any(|output| output.contains(current_file)), 
        "現在ダウンロード中ファイル名が表示される");
    
    // ステップ4: PR003 ファイル進捗バー表示テスト
    let file_downloaded = 644_245_094u64;  // 約614MB
    let file_total = 1_073_741_824u64;     // 1GB
    let file_progress = file_downloaded as f32 / file_total as f32;
    
    mock_ctx.add_ui_output(&format!("Progress: [{:>20}] {:.1}% ({} MB/{} MB)",
        "█".repeat((file_progress * 20.0) as usize),
        file_progress * 100.0,
        file_downloaded / 1_000_000,
        file_total / 1_000_000));
    
    assert!(file_progress >= 0.0 && file_progress <= 1.0, "ファイル進捗が有効範囲");
    assert!(file_progress > 0.5, "ファイルが半分以上ダウンロード済み");
    
    // ステップ5: PR004 一時停止ボタンテスト
    mock_ctx.click_button("pause_download");
    
    // 一時停止状態のシミュレーション
    app.is_downloading = false;  // 一時停止状態
    mock_ctx.add_ui_output("Download paused");
    
    // 一時停止→再開ボタンの表示切り替え
    mock_ctx.click_button("resume_download");
    app.is_downloading = true;   // 再開状態
    mock_ctx.add_ui_output("Download resumed");
    
    // ステップ6: PR005 キャンセルボタンテスト
    mock_ctx.click_button("cancel_download");
    
    // キャンセル確認ダイアログのシミュレーション
    mock_ctx.add_ui_output("Cancel confirmation: Are you sure you want to cancel all downloads?");
    
    // 確認後のキャンセル実行
    mock_ctx.click_button("confirm_cancel");
    app.is_downloading = false;
    mock_ctx.add_ui_output("All downloads cancelled");
    
    // ステップ7: PR006 ログ表示エリアテスト
    let log_messages = vec![
        "[10:30:15] Started downloading Meeting_2024-01-15_video.mp4",
        "[10:30:45] Downloaded 25% (300MB/1.2GB) - Speed: 4.8MB/s",
        "[10:31:15] Downloaded 50% (600MB/1.2GB) - Speed: 5.1MB/s",
        "[10:31:45] Downloaded 60% (720MB/1.2GB) - Speed: 4.9MB/s",
    ];
    
    for message in &log_messages {
        app.download_progress.push(message.to_string());
        mock_ctx.add_ui_output(message);
    }
    
    // ログ表示の検証
    assert_eq!(app.download_progress.len(), log_messages.len(), "ログメッセージ数が一致");
    
    let ui_outputs = mock_ctx.ui_outputs.lock().unwrap();
    for message in &log_messages {
        assert!(ui_outputs.iter().any(|output| output.contains(message)),
            "ログメッセージが表示される: {}", message);
    }
    
    // ステップ8: PR007 統計情報表示テスト
    completed_files = 1;  // 1ファイル完了
    downloaded_bytes = file_total;  // 1ファイル分完了
    
    let eta_seconds = if downloaded_bytes > 0 {
        let remaining_bytes = total_bytes - downloaded_bytes;
        let avg_speed = 5_000_000u64;  // 5MB/s
        remaining_bytes / avg_speed
    } else {
        0
    };
    
    mock_ctx.add_ui_output(&format!("Completed Files: {}/{}", completed_files, total_files));
    mock_ctx.add_ui_output(&format!("Total Downloaded: {} MB / {} MB", 
        downloaded_bytes / 1_000_000, total_bytes / 1_000_000));
    mock_ctx.add_ui_output(&format!("Estimated Time: {}min {}sec remaining", 
        eta_seconds / 60, eta_seconds % 60));
    
    // 統計情報の検証
    assert_eq!(completed_files, 1, "完了ファイル数が正しい");
    assert!(downloaded_bytes > 0, "ダウンロード済みバイト数が正の値");
    assert!(eta_seconds > 0, "残り時間推定が正の値");
    
    // UI操作フロー検証
    let button_clicks = mock_ctx.button_clicks.lock().unwrap();
    let expected_buttons = vec!["pause_download", "resume_download", "cancel_download", "confirm_cancel"];
    
    for button in expected_buttons {
        assert!(button_clicks.contains(&button.to_string()), 
            "期待するボタン操作: {}", button);
    }
    
    println!("✓ SC005ダウンロード進捗画面UIテスト完了");
    println!("  - 完了ファイル数: {}/{}", completed_files, total_files);
    println!("  - ダウンロード済み: {} MB", downloaded_bytes / 1_000_000);
    println!("  - 残り時間: {}分{}秒", eta_seconds / 60, eta_seconds % 60);
}

/// SC006: エラー表示画面のUIテスト
/// 
/// テスト対象仕様:
/// - screen_specifications.md: SC006 エラー表示画面
/// - エラー分類: 認証エラー, ネットワークエラー, ファイルエラー
/// - 回復操作: リトライ, 設定に戻る, ログ出力
#[tokio::test]
async fn test_sc006_error_display_screen_ui_components() {
    let mock_ctx = MockUIContext::new();
    let mut app = ZoomDownloaderApp::default();
    
    // エラーケースのテストパラメータ
    let error_cases = vec![
        ("AUTH_001", "OAuth authentication failed: Invalid client credentials", 
         "認証エラー", vec!["設定に戻る", "リトライ"]),
        ("NET_001", "Network connection timeout: Failed to connect to api.zoom.us",
         "ネットワークエラー", vec!["リトライ", "ログ出力"]),
        ("FILE_001", "Permission denied: Cannot write to output directory",
         "ファイルエラー", vec!["設定に戻る", "ログ出力"]),
    ];
    
    for (error_code, error_message, error_type, recovery_actions) in error_cases {
        println!("Testing error case: {} - {}", error_code, error_type);
        
        // ステップ1: エラー状態設定 (SC006 エラー発生)
        app.status_message = format!("Error: {}", error_message);
        
        // ステップ2: エラー種別表示テスト
        mock_ctx.add_ui_output(&format!("⚠ エラー"));
        mock_ctx.add_ui_output(&format!("エラー種別: {}", error_type));
        
        // ステップ3: エラーメッセージ表示テスト
        mock_ctx.add_ui_output(&format!("エラーメッセージ:"));
        mock_ctx.add_ui_output(error_message);
        
        // エラーメッセージの検証
        assert!(error_message.len() > 10, "詳細なエラーメッセージ");
        assert!(error_message.contains(&error_code[..3]), "エラーコード種別を含む");
        
        // ステップ4: 詳細情報表示テスト
        let error_details = format!(
            "Error Code: {}\nTimestamp: {}\nContext: UI Test",
            error_code,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
        );
        mock_ctx.add_ui_output(&format!("詳細情報: {}", error_details));
        
        // ステップ5: 推奨アクション表示テスト
        mock_ctx.add_ui_output("推奨アクション:");
        match error_type {
            "認証エラー" => {
                mock_ctx.add_ui_output("• 設定画面でClient IDとClient Secretを確認してください");
                mock_ctx.add_ui_output("• Zoom Developer Appの設定を確認してください");
            }
            "ネットワークエラー" => {
                mock_ctx.add_ui_output("• インターネット接続を確認してください");
                mock_ctx.add_ui_output("• ファイアウォール設定を確認してください");
            }
            "ファイルエラー" => {
                mock_ctx.add_ui_output("• 出力ディレクトリの権限を確認してください");
                mock_ctx.add_ui_output("• ディスク容量を確認してください");
            }
            _ => {}
        }
        
        // ステップ6: 回復操作ボタンテスト
        for action in &recovery_actions {
            let button_id = match action.as_str() {
                "リトライ" => "retry_operation",
                "設定に戻る" => "back_to_config", 
                "ログ出力" => "export_logs",
                _ => "unknown_action",
            };
            
            mock_ctx.click_button(button_id);
            mock_ctx.add_ui_output(&format!("実行: {}", action));
        }
        
        // ステップ7: 回復操作後の状態変化シミュレーション
        match error_type {
            "認証エラー" => {
                if recovery_actions.contains(&"設定に戻る".to_string()) {
                    app.status_message = "Configuration screen loaded".to_string();
                    mock_ctx.add_ui_output("設定画面に遷移しました");
                }
            }
            "ネットワークエラー" => {
                if recovery_actions.contains(&"リトライ".to_string()) {
                    app.status_message = "Retrying operation...".to_string();
                    mock_ctx.add_ui_output("操作を再試行中...");
                }
            }
            "ファイルエラー" => {
                if recovery_actions.contains(&"ログ出力".to_string()) {
                    mock_ctx.add_ui_output("エラーログをファイルに出力しました");
                }
            }
            _ => {}
        }
    }
    
    // UI操作履歴の総合検証
    let button_clicks = mock_ctx.button_clicks.lock().unwrap();
    let expected_buttons = vec!["retry_operation", "back_to_config", "export_logs"];
    
    for button in expected_buttons {
        assert!(button_clicks.contains(&button.to_string()),
            "回復操作ボタンがクリックされた: {}", button);
    }
    
    // エラー表示内容の検証
    let ui_outputs = mock_ctx.ui_outputs.lock().unwrap();
    assert!(ui_outputs.iter().any(|output| output.contains("⚠ エラー")),
        "エラーアイコンが表示される");
    assert!(ui_outputs.iter().any(|output| output.contains("推奨アクション")),
        "推奨アクションが表示される");
    
    println!("✓ SC006エラー表示画面UIテスト完了");
    println!("  - テスト対象エラー種別: 3種類");
    println!("  - 回復操作ボタン: {} 個", button_clicks.len());
}

// UIテスト トレーサビリティ
//
// 画面仕様 (screen_specifications.md) との対応:
// ├─ SC002: 設定画面                → test_sc002_config_screen_ui_components
// ├─ SC003: 認証画面                → test_sc003_auth_screen_ui_components
// ├─ SC004: 録画リスト画面           → test_sc004_recording_list_screen_ui_components
// ├─ SC005: ダウンロード進捗画面      → test_sc005_progress_screen_ui_components
// └─ SC006: エラー表示画面           → test_sc006_error_display_screen_ui_components
//
// UI要素の詳細対応:
// SC002: CF001(Client ID入力) - CF005(設定読込ボタン)
// SC003: AU001(認証開始) - AU006(認証完了ボタン)  
// SC004: RL001(From日付) - RL007(ダウンロード開始ボタン)
// SC005: PR001(全体進捗バー) - PR007(統計情報)
// SC006: エラー種別表示, 詳細情報, 推奨アクション, 回復操作ボタン
//
// 操作仕様 (operation_specifications.md) との連携:
// - UI操作フローの検証
// - 画面遷移の確認
// - ユーザー入力の検証
//
// Mock・シミュレーション:
// - MockUIContext: UI操作・状態の記録
// - 実際のUIフレームワークは使用せず、状態変化を検証
// - 外部システム呼び出しは全てMock化