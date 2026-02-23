//! 本番用サービス実装
//!
//! services.rsで定義されたtraitの本番実装。
//! 既存のgui.rsにハードコードされていた外部呼び出しをラップする。

use std::sync::{mpsc, Arc};
use crate::{Config, RecordingResponse, ZoomRecordingDownloader};
use crate::gui::AppMessage;
use crate::services::{AuthService, BrowserLauncher, ConfigService, DownloadService, RecordingService};

/// 本番用設定サービス
pub struct RealConfigService;

impl ConfigService for RealConfigService {
    fn load_config(&self, path: &str) -> Result<Config, Box<dyn std::error::Error>> {
        Config::load_from_file(path)
    }

    fn save_config(&self, config: &Config, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        config.save_to_file(path)
    }

    fn create_sample_config(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        Config::create_sample_file(path)
    }
}

/// 本番用認証サービス
pub struct RealAuthService;

impl AuthService for RealAuthService {
    fn generate_auth_url(
        &self,
        client_id: &str,
        client_secret: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;
        let client_id = client_id.to_string();
        let client_secret = client_secret.to_string();
        rt.block_on(async {
            crate::gui::generate_auth_url_async(&client_id, &client_secret).await
        })
    }

    fn exchange_code_for_token(
        &self,
        client_id: &str,
        client_secret: &str,
        auth_code: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;
        let client_id = client_id.to_string();
        let client_secret = client_secret.to_string();
        let auth_code = auth_code.to_string();
        rt.block_on(async {
            crate::gui::exchange_code_for_token_async(&client_id, &client_secret, &auth_code).await
        })
    }
}

/// 本番用録画取得サービス
pub struct RealRecordingService;

impl RecordingService for RealRecordingService {
    fn get_recordings(
        &self,
        access_token: &str,
        user_id: &str,
        from_date: &str,
        to_date: &str,
    ) -> Result<RecordingResponse, Box<dyn std::error::Error + Send + Sync>> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;
        let access_token = access_token.to_string();
        let user_id = user_id.to_string();
        let from_date = from_date.to_string();
        let to_date = to_date.to_string();
        rt.block_on(async {
            let mut downloader = ZoomRecordingDownloader::new_with_token(
                "dummy_client_id".to_string(),
                "dummy_client_secret".to_string(),
                access_token,
            );
            downloader
                .get_recordings(Some(&user_id), &from_date, &to_date, None)
                .await
        })
    }
}

/// 本番用ブラウザ起動サービス
pub struct RealBrowserLauncher;

impl BrowserLauncher for RealBrowserLauncher {
    fn open_url(&self, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        open::that(url)?;
        Ok(())
    }
}

/// 本番用ダウンロードサービス
pub struct RealDownloadService;

impl DownloadService for RealDownloadService {
    fn download_files(
        &self,
        _access_token: &str,
        _selected_recordings: &[String],
        _output_dir: &str,
        sender: mpsc::Sender<AppMessage>,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let _ = sender.send(AppMessage::DownloadProgress(
            "Fetching recording list...".to_string(),
        ));
        // TODO: 実際のダウンロードロジック実装
        let files = vec!["placeholder.mp4".to_string()];
        let _ = sender.send(AppMessage::DownloadComplete(files.clone()));
        Ok(files)
    }
}

/// サービスコンテナ - 全サービスをまとめて保持
pub struct AppServices {
    pub config_service: Box<dyn ConfigService>,
    pub auth_service: Arc<dyn AuthService>,
    pub recording_service: Arc<dyn RecordingService>,
    pub browser_launcher: Box<dyn BrowserLauncher>,
    pub download_service: Arc<dyn DownloadService>,
}

impl Default for AppServices {
    fn default() -> Self {
        Self {
            config_service: Box::new(RealConfigService),
            auth_service: Arc::new(RealAuthService),
            recording_service: Arc::new(RealRecordingService),
            browser_launcher: Box::new(RealBrowserLauncher),
            download_service: Arc::new(RealDownloadService),
        }
    }
}
