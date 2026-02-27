use super::helpers::create_test_app;
/// 状態遷移テスト
///
/// mpscチャネル経由でAppMessageを送信し、
/// process_messages_for_test()後の状態を検証する。
use zoom_video_mover_lib::gui::{AppMessage, AppScreen};
use zoom_video_mover_lib::RecordingSearchResponse;

/// ST-001: AuthUrlGenerated処理
#[test]
fn st001_auth_url_generated_sets_state() {
    let mut app = create_test_app();
    let url = "https://zoom.us/oauth/authorize?client_id=test".to_string();

    app.sender()
        .send(AppMessage::AuthUrlGenerated(url.clone()))
        .unwrap();
    app.process_messages_for_test();

    assert!(app.is_authenticating());
    assert!(app.status_message().contains("Auth URL generated"));
}

/// ST-002: AuthComplete処理
#[test]
fn st002_auth_complete_sets_token_and_screen() {
    let mut app = create_test_app();
    let token = "test_access_token_12345".to_string();

    app.sender()
        .send(AppMessage::AuthComplete(token.clone()))
        .unwrap();
    app.process_messages_for_test();

    assert_eq!(app.access_token(), &Some(token));
    assert_eq!(app.current_screen(), &AppScreen::Recordings);
    assert!(!app.is_authenticating());
}

/// ST-003: RecordingsLoaded処理
#[test]
fn st003_recordings_loaded_sets_recordings() {
    let mut app = create_test_app();
    let recordings = RecordingSearchResponse {
        from: "2025-01-01".to_string(),
        to: "2025-01-31".to_string(),
        page_count: 1,
        page_size: 30,
        total_records: 0,
        next_page_token: None,
        meetings: vec![],
    };

    app.sender()
        .send(AppMessage::RecordingsLoaded(recordings))
        .unwrap();
    app.process_messages_for_test();

    assert!(app.recordings().is_some());
    assert!(app.status_message().contains("Recordings loaded"));
}

/// ST-004: DownloadProgress処理
#[test]
fn st004_download_progress_updates_state() {
    let mut app = create_test_app();
    let msg = "Downloading file 1 of 3...".to_string();

    app.sender()
        .send(AppMessage::DownloadProgress(msg.clone()))
        .unwrap();
    app.process_messages_for_test();

    assert_eq!(app.current_screen(), &AppScreen::Progress);
    assert!(app.download_progress_log().contains(&msg));
}

/// ST-005: DownloadComplete処理
#[test]
fn st005_download_complete_resets_state() {
    let mut app = create_test_app();
    app.set_is_downloading(true);
    app.set_current_screen(AppScreen::Progress);

    let files = vec!["file1.mp4".to_string(), "file2.mp4".to_string()];
    app.sender()
        .send(AppMessage::DownloadComplete(files))
        .unwrap();
    app.process_messages_for_test();

    assert!(!app.is_downloading());
    assert_eq!(app.current_screen(), &AppScreen::Recordings);
    assert!(app.status_message().contains("2 files"));
}

/// ST-006: Error処理
#[test]
fn st006_error_resets_flags_and_shows_error() {
    let mut app = create_test_app();
    app.set_is_authenticating(true);
    app.set_is_downloading(true);

    let err = "Test error occurred".to_string();
    app.sender().send(AppMessage::Error(err.clone())).unwrap();
    app.process_messages_for_test();

    assert!(!app.is_authenticating());
    assert!(!app.is_downloading());
    assert_eq!(app.error_message(), err);
    assert_eq!(app.current_screen(), &AppScreen::Error);
}

/// ST-007: DownloadPaused処理
#[test]
fn st007_download_paused_sets_flag() {
    let mut app = create_test_app();

    app.sender().send(AppMessage::DownloadPaused).unwrap();
    app.process_messages_for_test();

    assert!(app.is_download_paused());
}

/// ST-008: DownloadResumed処理
#[test]
fn st008_download_resumed_clears_flag() {
    let mut app = create_test_app();

    // まずpauseした状態にする
    app.sender().send(AppMessage::DownloadPaused).unwrap();
    app.process_messages_for_test();
    assert!(app.is_download_paused());

    // resume
    app.sender().send(AppMessage::DownloadResumed).unwrap();
    app.process_messages_for_test();

    assert!(!app.is_download_paused());
}

/// ST-009: DownloadCancelled処理
#[test]
fn st009_download_cancelled_resets_state() {
    let mut app = create_test_app();
    app.set_is_downloading(true);
    app.set_current_screen(AppScreen::Progress);

    app.sender().send(AppMessage::DownloadCancelled).unwrap();
    app.process_messages_for_test();

    assert!(!app.is_downloading());
    assert!(!app.is_download_paused());
    assert_eq!(app.current_screen(), &AppScreen::Recordings);
}

/// ST-010: LogExported処理
#[test]
fn st010_log_exported_updates_status() {
    let mut app = create_test_app();
    let filepath = "/tmp/zoom_log_20250101.txt".to_string();

    app.sender()
        .send(AppMessage::LogExported(filepath.clone()))
        .unwrap();
    app.process_messages_for_test();

    assert!(app.status_message().contains("Log exported successfully"));
    assert!(app.status_message().contains(&filepath));
}

/// ST-011: Error後にauth/download状態リセット
#[test]
fn st011_error_resets_both_auth_and_download() {
    let mut app = create_test_app();
    app.set_is_authenticating(true);
    app.set_is_downloading(true);

    app.sender()
        .send(AppMessage::Error("test".to_string()))
        .unwrap();
    app.process_messages_for_test();

    assert!(!app.is_authenticating());
    assert!(!app.is_downloading());
}

/// ST-012: 連続メッセージ処理
#[test]
fn st012_sequential_messages_maintain_consistency() {
    let mut app = create_test_app();

    // 認証完了 → 録画ロード → ダウンロード開始 → ダウンロード完了
    app.sender()
        .send(AppMessage::AuthComplete("token".to_string()))
        .unwrap();
    app.sender()
        .send(AppMessage::RecordingsLoaded(RecordingSearchResponse {
            from: "2025-01-01".to_string(),
            to: "2025-01-31".to_string(),
            page_count: 1,
            page_size: 30,
            total_records: 1,
            next_page_token: None,
            meetings: vec![],
        }))
        .unwrap();
    app.sender()
        .send(AppMessage::DownloadProgress("Starting...".to_string()))
        .unwrap();
    app.sender()
        .send(AppMessage::DownloadComplete(vec!["a.mp4".to_string()]))
        .unwrap();

    app.process_messages_for_test();

    // 最終的にRecordings画面に戻り、ダウンロード完了状態
    assert_eq!(app.current_screen(), &AppScreen::Recordings);
    assert!(app.access_token().is_some());
    assert!(app.recordings().is_some());
    assert!(!app.is_downloading());
    assert!(app.status_message().contains("1 files"));
}
