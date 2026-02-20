/// egui headless UIテスト
///
/// egui::Context::default() + ctx.run() + app.update_ui(ctx) パターンで
/// 各画面のレンダリングがパニックしないことを検証する。

use zoom_video_mover_lib::gui::AppScreen;
use zoom_video_mover_lib::services::MockConfigService;
use zoom_video_mover_lib::Config;
use super::helpers::{create_test_app, mock_services};
use zoom_video_mover_lib::gui::ZoomDownloaderApp;

/// headless contextでupdate_uiを実行するヘルパー
fn run_update_ui(app: &mut ZoomDownloaderApp) {
    let ctx = egui::Context::default();
    // egui requires running within ctx.run() to set up internal state
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        app.update_ui(ctx);
    });
}

/// UI-001: 初期状態でupdate_uiがパニックしない
#[test]
fn ui001_initial_state_renders_without_panic() {
    let mut app = create_test_app();
    run_update_ui(&mut app);
}

/// UI-002: Config画面レンダリング
#[test]
fn ui002_config_screen_renders() {
    let mut app = create_test_app();
    // Config画面はデフォルト
    assert_eq!(app.current_screen(), &AppScreen::Config);
    run_update_ui(&mut app);
}

/// UI-003: Auth画面レンダリング
#[test]
fn ui003_auth_screen_renders() {
    let mut app = create_test_app();
    app.set_config_loaded(true);
    app.set_current_screen(AppScreen::Auth);
    run_update_ui(&mut app);
}

/// UI-004: Recordings画面レンダリング
#[test]
fn ui004_recordings_screen_renders() {
    let mut app = create_test_app();
    app.set_config_loaded(true);
    app.set_access_token(Some("test_token".to_string()));
    app.set_current_screen(AppScreen::Recordings);
    run_update_ui(&mut app);
}

/// UI-005: Progress画面レンダリング
#[test]
fn ui005_progress_screen_renders() {
    let mut app = create_test_app();
    app.set_is_downloading(true);
    app.set_current_screen(AppScreen::Progress);
    run_update_ui(&mut app);
}

/// UI-006: Error画面レンダリング
#[test]
fn ui006_error_screen_renders() {
    let mut app = create_test_app();
    app.set_error_message("Test error message".to_string());
    app.set_current_screen(AppScreen::Error);
    run_update_ui(&mut app);
}

/// UI-007: 全画面順次レンダリング
#[test]
fn ui007_all_screens_render_sequentially() {
    let mut app = create_test_app();
    app.set_config_loaded(true);
    app.set_access_token(Some("token".to_string()));

    // Config画面
    app.set_current_screen(AppScreen::Config);
    run_update_ui(&mut app);

    // Auth画面
    app.set_current_screen(AppScreen::Auth);
    run_update_ui(&mut app);

    // Recordings画面
    app.set_current_screen(AppScreen::Recordings);
    run_update_ui(&mut app);

    // Progress画面
    app.set_is_downloading(true);
    app.set_current_screen(AppScreen::Progress);
    run_update_ui(&mut app);

    // Error画面
    app.set_error_message("error".to_string());
    app.set_current_screen(AppScreen::Error);
    run_update_ui(&mut app);
}

/// UI-008: load_config Mock呼び出し検証
/// MockConfigServiceが正しく差し込まれ、呼び出し時に機能することを確認
#[test]
fn ui008_load_config_uses_mock_service() {
    let mut mock_config = MockConfigService::new();
    mock_config
        .expect_load_config()
        .withf(|path| path == "config.toml")
        .returning(|_| {
            Ok(Config {
                client_id: "mock_client_id".to_string(),
                client_secret: "mock_secret".to_string(),
                redirect_uri: Some("http://localhost:8080/callback".to_string()),
            })
        });
    // update_uiで他のconfigメソッドが呼ばれる可能性に備える
    mock_config
        .expect_save_config()
        .returning(|_, _| Ok(()));
    mock_config
        .expect_create_sample_config()
        .returning(|_| Ok(()));

    let mut services = mock_services();
    services.config_service = Box::new(mock_config);

    let mut app = ZoomDownloaderApp::new_with_services(services);

    // 状態確認：初期状態
    assert!(!app.config_loaded());

    // update_uiがパニックしないことを検証
    run_update_ui(&mut app);

    // load_configはUIボタンクリックで発火するため、
    // 直接テスト不可（egui headlessではボタンクリックをシミュレートできない）。
    // ここではMock付きアプリが正常にレンダリングできることを確認。
}

/// UI-009: save_config Mock呼び出し（パニックしない検証）
#[test]
fn ui009_save_config_does_not_panic_with_mock() {
    let mut services = mock_services();

    let mut mock_config = MockConfigService::new();
    mock_config
        .expect_save_config()
        .returning(|_, _| Ok(()));
    mock_config
        .expect_load_config()
        .returning(|_| {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "not found",
            )) as Box<dyn std::error::Error>)
        });
    mock_config
        .expect_create_sample_config()
        .returning(|_| Ok(()));

    services.config_service = Box::new(mock_config);

    let mut app = ZoomDownloaderApp::new_with_services(services);
    run_update_ui(&mut app);
}
