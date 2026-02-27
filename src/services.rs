//! サービスtrait定義
//!
//! 外部依存を抽象化し、テスト時にMock化可能にするためのtrait群。
//! GUI層はこれらのtraitを通じて外部システムにアクセスする。

use crate::components::api::RecordingSearchResponse;
use crate::gui::AppMessage;
use crate::Config;
use std::sync::mpsc;

#[cfg(feature = "test-support")]
use mockall::automock;

/// 設定ファイルの読み書きを担当するサービス
#[cfg_attr(feature = "test-support", automock)]
pub trait ConfigService: Send + Sync {
    fn load_config(&self, path: &str) -> Result<Config, Box<dyn std::error::Error>>;
    fn save_config(&self, config: &Config, path: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn create_sample_config(&self, path: &str) -> Result<(), Box<dyn std::error::Error>>;
}

/// OAuth認証フローを担当するサービス
#[cfg_attr(feature = "test-support", automock)]
pub trait AuthService: Send + Sync + 'static {
    fn generate_auth_url(
        &self,
        client_id: &str,
        client_secret: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;

    fn exchange_code_for_token(
        &self,
        client_id: &str,
        client_secret: &str,
        auth_code: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
}

/// 録画データ取得を担当するサービス
#[cfg_attr(feature = "test-support", automock)]
pub trait RecordingService: Send + Sync + 'static {
    fn get_recordings(
        &self,
        access_token: &str,
        user_id: &str,
        from_date: &str,
        to_date: &str,
    ) -> Result<RecordingSearchResponse, Box<dyn std::error::Error + Send + Sync>>;
}

/// ブラウザ起動を担当するサービス
#[cfg_attr(feature = "test-support", automock)]
pub trait BrowserLauncher: Send + Sync {
    fn open_url(&self, url: &str) -> Result<(), Box<dyn std::error::Error>>;
}

/// ファイルダウンロードを担当するサービス
#[cfg_attr(feature = "test-support", automock)]
pub trait DownloadService: Send + Sync + 'static {
    fn download_files(
        &self,
        access_token: &str,
        recordings: &RecordingSearchResponse,
        selected_recordings: &[String],
        output_dir: &str,
        sender: mpsc::Sender<AppMessage>,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>>;
}
