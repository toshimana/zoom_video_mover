/// 各画面のスクリーンショット生成テスト
///
/// `cargo test --features test-support --test gui_tests screenshot -- --nocapture`
/// で実行し、tests/snapshots/ にPNG画像を出力する。

use std::path::PathBuf;
use zoom_video_mover_lib::gui::AppScreen;
use super::helpers::create_test_app;
use super::screenshot::render_app_to_png;

/// スナップショット出力ディレクトリ
fn snapshots_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("snapshots")
}

#[test]
fn screenshot_config_screen() {
    let mut app = create_test_app();
    app.set_current_screen(AppScreen::Config);
    render_app_to_png(&mut app, &snapshots_dir().join("config_screen.png"));
}

#[test]
fn screenshot_auth_screen() {
    let mut app = create_test_app();
    app.set_config_loaded(true);
    app.set_current_screen(AppScreen::Auth);
    render_app_to_png(&mut app, &snapshots_dir().join("auth_screen.png"));
}

#[test]
fn screenshot_recordings_screen() {
    let mut app = create_test_app();
    app.set_config_loaded(true);
    app.set_access_token(Some("test_token".to_string()));
    app.set_current_screen(AppScreen::Recordings);
    render_app_to_png(&mut app, &snapshots_dir().join("recordings_screen.png"));
}

#[test]
fn screenshot_progress_screen() {
    let mut app = create_test_app();
    app.set_is_downloading(true);
    app.set_current_screen(AppScreen::Progress);
    render_app_to_png(&mut app, &snapshots_dir().join("progress_screen.png"));
}

#[test]
fn screenshot_error_screen() {
    let mut app = create_test_app();
    app.set_error_message("Test error: connection timeout".to_string());
    app.set_current_screen(AppScreen::Error);
    render_app_to_png(&mut app, &snapshots_dir().join("error_screen.png"));
}
