/// テスト用ヘルパー・Mock設定
use std::sync::Arc;
use zoom_video_mover_lib::gui::ZoomDownloaderApp;
use zoom_video_mover_lib::services::{
    MockAuthService, MockBrowserLauncher, MockConfigService, MockDownloadService,
    MockRecordingService,
};
use zoom_video_mover_lib::services_impl::AppServices;

/// デフォルトMockサービスを構築する
/// 各Mockは何も呼ばれないことを期待する
pub fn mock_services() -> AppServices {
    AppServices {
        config_service: Box::new(MockConfigService::new()),
        auth_service: Arc::new(MockAuthService::new()),
        recording_service: Arc::new(MockRecordingService::new()),
        browser_launcher: Box::new(MockBrowserLauncher::new()),
        download_service: Arc::new(MockDownloadService::new()),
    }
}

/// テスト用アプリインスタンスを作成する
pub fn create_test_app() -> ZoomDownloaderApp {
    ZoomDownloaderApp::new_with_services(mock_services())
}
