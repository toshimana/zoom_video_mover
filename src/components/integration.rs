/// 統合コンポーネント - 各コンポーネントの連携を管理
/// 
/// # 責任
/// - コンポーネント間の依存関係管理
/// - ワークフローの実行
/// - イベントの仲介
/// - エラーハンドリングの統合

use crate::errors::{AppError, AppResult};
use crate::components::{ComponentLifecycle, Configurable};
use crate::components::auth::{AuthComponent, AuthToken};
use crate::components::api::{ApiComponent, ApiConfig, RecordingSearchRequest, MeetingRecording};
use crate::components::download::{DownloadComponent, DownloadConfig, DownloadEvent};
use crate::components::config::{AppConfig, OAuthConfig};
use async_trait::async_trait;
use tokio::sync::mpsc;
use chrono::NaiveDate;
use std::sync::Arc;
use std::path::PathBuf;
use log;

/// 統合コンポーネント設定
#[derive(Debug, Clone)]
pub struct IntegrationConfig {
    /// 出力ディレクトリ
    pub output_directory: PathBuf,
    /// 同時ダウンロード数
    pub concurrent_downloads: usize,
    /// ダウンロード対象ファイルタイプ
    pub download_file_types: Vec<String>,
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        Self {
            output_directory: PathBuf::from("downloads"),
            concurrent_downloads: 3,
            download_file_types: vec![
                "MP4".to_string(),
                "M4A".to_string(),
                "TRANSCRIPT".to_string(),
                "CHAT".to_string(),
            ],
        }
    }
}

/// 統合コンポーネント
pub struct IntegrationComponent {
    /// ファイルパス
    config_path: String,
    /// 認証管理
    auth_component: AuthComponent,
    /// API通信
    api_component: ApiComponent,
    /// ダウンロード実行
    download_component: DownloadComponent,
    /// 統合設定
    integration_config: IntegrationConfig,
    /// イベントチャネル
    event_receiver: Option<mpsc::UnboundedReceiver<DownloadEvent>>,
}

impl IntegrationComponent {
    /// 新しい統合コンポーネントを作成
    /// 
    /// # 事前条件
    /// - config_path は有効な設定ファイルパスである
    /// - integration_config は有効な統合設定である
    /// 
    /// # 事後条件
    /// - IntegrationComponentインスタンスが作成される
    /// - 各サブコンポーネントが初期化される
    pub async fn new(config_path: &str, integration_config: IntegrationConfig) -> AppResult<Self> {
        // 設定ファイルの読み込み
        let app_config = AppConfig::load_from_file(config_path)?;
        
        // OAuth設定の構築
        let oauth_config = OAuthConfig {
            client_id: app_config.oauth.client_id.clone(),
            client_secret: app_config.oauth.client_secret.clone(),
            redirect_uri: if app_config.oauth.redirect_uri.is_empty() {
                "http://localhost:8080/callback".to_string()
            } else {
                app_config.oauth.redirect_uri.clone()
            },
            scopes: vec![
                "recording:read".to_string(),
                "user:read".to_string(),
                "meeting:read".to_string(),
            ],
        };
        
        // 認証コンポーネントの初期化
        let auth_component = AuthComponent::new(oauth_config);
        
        // API設定の構築
        let api_config = ApiConfig {
            base_url: app_config.api.base_url.clone(),
            rate_limit: Default::default(),
            timeout: std::time::Duration::from_secs(app_config.api.timeout_seconds),
            max_retries: app_config.api.max_retries,
            default_page_size: app_config.api.default_page_size,
        };
        
        // APIコンポーネントの初期化
        let api_component = ApiComponent::new(api_config);
        
        // ダウンロード設定の構築
        let download_config = DownloadConfig {
            concurrent_downloads: integration_config.concurrent_downloads,
            chunk_size: 8192 * 1024,
            timeout: std::time::Duration::from_secs(300),
            max_retries: 3,
            output_directory: integration_config.output_directory.clone(),
        };
        
        // ダウンロードコンポーネントの初期化
        let mut download_component = DownloadComponent::new(download_config);
        
        // イベントチャネルの設定
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        download_component.set_event_listener(event_sender);
        
        Ok(Self {
            config_path: config_path.to_string(),
            auth_component,
            api_component,
            download_component,
            integration_config,
            event_receiver: Some(event_receiver),
        })
    }
    
    /// 認証フローを実行
    /// 
    /// # 副作用
    /// - ブラウザを開いて認証を行う
    /// - トークンを取得・保存する
    /// 
    /// # 事前条件
    /// - コンポーネントが初期化されている
    /// 
    /// # 事後条件
    /// - 成功時: 有効な認証トークンが設定される
    /// - 失敗時: 適切なエラーが返される
    pub async fn authenticate(&mut self) -> AppResult<AuthToken> {
        log::info!("Starting authentication flow");
        
        // 認証URL生成
        let (auth_url, state_id) = self.auth_component.generate_auth_url()?;
        
        log::info!("Please visit the following URL to authenticate:");
        log::info!("{}", auth_url);
        
        // TODO: 実際の実装では、ローカルHTTPサーバーを起動してコールバックを受け取る
        // ここでは簡略化のため、手動でコードを入力してもらう想定
        
        // 仮の認証コード（実際の実装では、コールバックから取得）
        let auth_code = "dummy_auth_code";
        
        // トークン交換
        let token = self.auth_component.exchange_code_for_token(auth_code, &state_id).await?;
        
        // APIコンポーネントにトークンを設定
        self.api_component.set_auth_token(token.clone()).await;
        
        // 設定ファイルに保存（オプション）
        // TODO: トークンの永続化
        
        log::info!("Authentication successful");
        Ok(token)
    }
    
    /// 録画データをダウンロード
    /// 
    /// # 副作用
    /// - APIリクエストの送信
    /// - ファイルのダウンロード
    /// - ディスクへの書き込み
    /// 
    /// # 事前条件
    /// - 認証が完了している
    /// - from_date <= to_date である
    /// 
    /// # 事後条件
    /// - 成功時: 録画ファイルがダウンロードされる
    /// - 失敗時: 適切なエラーが返される
    pub async fn download_recordings(
        &mut self,
        user_id: Option<String>,
        from_date: NaiveDate,
        to_date: NaiveDate,
    ) -> AppResult<()> {
        assert!(from_date <= to_date, "from_date must be before or equal to to_date");
        
        log::info!("Searching recordings from {} to {}", from_date, to_date);
        
        // 録画データの検索
        let search_request = RecordingSearchRequest {
            user_id,
            from: from_date,
            to: to_date,
            page_size: Some(30),
            next_page_token: None,
        };
        
        let meetings = self.api_component.get_all_recordings(search_request).await?;
        
        log::info!("Found {} meetings with recordings", meetings.len());
        
        // ダウンロードワーカーの起動
        self.download_component.start_downloads().await?;
        
        // 各録画ファイルをダウンロードタスクとして追加
        let mut task_count = 0;
        for meeting in meetings {
            for recording_file in &meeting.recording_files {
                // ファイルタイプフィルタ
                if !self.integration_config.download_file_types.contains(&recording_file.file_type) {
                    continue;
                }
                
                let task_id = format!("{}_{}", meeting.id, recording_file.id);
                let file_name = self.generate_file_name(&meeting, &recording_file);
                
                self.download_component.add_download_task(
                    task_id,
                    recording_file.download_url.clone(),
                    file_name,
                    Some(recording_file.file_size),
                ).await?;
                
                task_count += 1;
            }
        }
        
        log::info!("Added {} download tasks", task_count);
        
        // イベントの処理
        if let Some(mut receiver) = self.event_receiver.take() {
            tokio::spawn(async move {
                while let Some(event) = receiver.recv().await {
                    match event {
                        DownloadEvent::TaskStarted { task_id } => {
                            log::info!("Download started: {}", task_id);
                        }
                        DownloadEvent::ProgressUpdate { task_id, progress } => {
                            log::debug!("Download progress {}: {:.1}%", task_id, progress.percentage * 100.0);
                        }
                        DownloadEvent::TaskCompleted { task_id, output_path } => {
                            log::info!("Download completed: {} -> {:?}", task_id, output_path);
                        }
                        DownloadEvent::TaskFailed { task_id, error } => {
                            log::error!("Download failed: {} - {}", task_id, error);
                        }
                        DownloadEvent::OverallProgressUpdate(overall) => {
                            log::info!("Overall progress: {}/{} completed", overall.completed_tasks, overall.total_tasks);
                        }
                    }
                }
            });
        }
        
        Ok(())
    }
    
    /// ファイル名を生成
    fn generate_file_name(&self, meeting: &MeetingRecording, recording_file: &crate::components::api::RecordingFile) -> String {
        use crate::sanitize_filename;
        
        let date_str = meeting.start_time.split('T').next().unwrap_or("unknown");
        let topic_safe = sanitize_filename(&meeting.topic);
        let file_type = recording_file.file_type.to_lowercase();
        let extension = &recording_file.file_extension;
        
        format!("{}_{}.{}.{}", date_str, topic_safe, file_type, extension)
    }
}

#[async_trait]
impl ComponentLifecycle for IntegrationComponent {
    async fn initialize(&mut self) -> AppResult<()> {
        log::info!("Initializing IntegrationComponent");
        
        // 各コンポーネントの初期化
        self.auth_component.initialize().await?;
        self.api_component.initialize().await?;
        self.download_component.initialize().await?;
        
        log::info!("IntegrationComponent initialized successfully");
        Ok(())
    }
    
    async fn shutdown(&mut self) -> AppResult<()> {
        log::info!("Shutting down IntegrationComponent");
        
        // 各コンポーネントのシャットダウン
        self.download_component.shutdown().await?;
        self.api_component.shutdown().await?;
        self.auth_component.shutdown().await?;
        
        log::info!("IntegrationComponent shut down successfully");
        Ok(())
    }
    
    async fn health_check(&self) -> bool {
        self.auth_component.health_check().await &&
        self.api_component.health_check().await &&
        self.download_component.health_check().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_integration_component_lifecycle() {
        // テスト用の設定ファイルパス
        let config_path = "test_config.toml";
        let integration_config = IntegrationConfig::default();
        
        // TODO: テスト用の設定ファイルを作成
        
        // let mut component = IntegrationComponent::new(config_path, integration_config).await.unwrap();
        
        // assert!(component.initialize().await.is_ok());
        // assert!(component.health_check().await);
        // assert!(component.shutdown().await.is_ok());
    }
}