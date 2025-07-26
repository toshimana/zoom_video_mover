use eframe::egui;
use std::sync::mpsc;
use std::thread;
use chrono::{Local, Datelike};
use zoom_video_mover_lib::{Config, ZoomRecordingDownloader};

#[derive(Debug)]
pub enum AppMessage {
    AuthUrlGenerated(String),
    AuthComplete(String),
    DownloadProgress(String),
    DownloadComplete(Vec<String>),
    Error(String),
}

pub struct ZoomDownloaderApp {
    // UI State
    client_id: String,
    client_secret: String,
    from_date: String,
    to_date: String,
    output_dir: String,
    auth_code: String,
    
    // App State
    config_loaded: bool,
    auth_url: Option<String>,
    is_authenticating: bool,
    is_downloading: bool,
    access_token: Option<String>,
    
    // Progress
    status_message: String,
    download_progress: Vec<String>,
    
    // Communication
    receiver: mpsc::Receiver<AppMessage>,
    sender: mpsc::Sender<AppMessage>,
}

impl Default for ZoomDownloaderApp {
    /// ZoomDownloaderAppの新しいインスタンスを作成する
    /// 
    /// 事前条件:
    /// - mpsc::channel() が正常に動作する
    /// 
    /// 事後条件:
    /// - 初期状態のZoomDownloaderAppインスタンスが作成される
    /// - 全てのフィールドが適切なデフォルト値で初期化される
    /// - 内部通信チャンネルが正常に設定される
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        
        Self {
            client_id: String::new(),
            client_secret: String::new(),
            from_date: String::new(),
            to_date: String::new(),
            output_dir: String::new(),
            auth_code: String::new(),
            config_loaded: false,
            auth_url: None,
            is_authenticating: false,
            is_downloading: false,
            access_token: None,
            status_message: "Ready".to_string(),
            download_progress: Vec::new(),
            receiver,
            sender,
        }
    }
}

impl ZoomDownloaderApp {
    /// メッセージを処理する（複雑度削減版）
    /// 
    /// 事前条件:
    /// - self は有効なZoomDownloaderAppインスタンスである
    /// 
    /// 事後条件:
    /// - 受信した全てのメッセージが処理される
    /// - アプリの状態が適切に更新される
    /// 
    /// 不変条件:
    /// - メッセージ処理中にアプリの状態が一貫性を保つ
    fn process_messages(&mut self) {
        while let Ok(msg) = self.receiver.try_recv() {
            match msg {
                AppMessage::AuthUrlGenerated(url) => {
                    self.auth_url = Some(url);
                    self.is_authenticating = true;
                    self.status_message = "Auth URL generated. Please complete authentication in browser.".to_string();
                }
                AppMessage::AuthComplete(token) => {
                    self.access_token = Some(token);
                    self.is_authenticating = false;
                    self.status_message = "Authentication completed.".to_string();
                }
                AppMessage::DownloadProgress(msg) => {
                    self.download_progress.push(msg.clone());
                    self.status_message = msg;
                }
                AppMessage::DownloadComplete(files) => {
                    self.is_downloading = false;
                    self.status_message = format!("Download completed: {} files", files.len());
                    self.download_progress.push(format!("Completed: Downloaded {} files", files.len()));
                }
                AppMessage::Error(err) => {
                    self.is_authenticating = false;
                    self.is_downloading = false;
                    self.status_message = format!("Error: {}", err);
                }
            }
        }
    }

    /// 設定セクションをレンダリングする（複雑度削減版）
    /// 
    /// 事前条件:
    /// - ui は有効なegui::Uiである
    /// 
    /// 事後条件:
    /// - 設定セクションが描画される
    /// - ユーザーの操作が適切に処理される
    /// 
    /// 不変条件:
    /// - UI の状態が一貫性を保つ
    fn render_config_section(&mut self, ui: &mut egui::Ui) {
        ui.heading("Configuration");
        
        if !self.config_loaded {
            if ui.button("Load Configuration").clicked() {
                self.load_config();
            }
        } else {
            ui.horizontal(|ui| {
                ui.label("Client ID:");
                ui.text_edit_singleline(&mut self.client_id);
            });
            
            ui.horizontal(|ui| {
                ui.label("Client Secret:");
                ui.add(egui::TextEdit::singleline(&mut self.client_secret).password(true));
            });
            
            if ui.button("Save Configuration").clicked() {
                self.save_config();
            }
        }
        
        ui.separator();
    }

    /// 日付範囲セクションをレンダリングする（複雑度削減版）
    /// 
    /// 事前条件:
    /// - ui は有効なegui::Uiである
    /// 
    /// 事後条件:
    /// - 日付範囲セクションが描画される
    /// - デフォルト日付が設定される
    /// - ユーザーの操作が適切に処理される
    /// 
    /// 不変条件:
    /// - 日付形式が一貫している
    fn render_date_section(&mut self, ui: &mut egui::Ui) {
        ui.heading("Download Period");
        
        // Set default dates if empty
        if self.from_date.is_empty() {
            let today = Local::now().date_naive();
            let month_start = today.with_day(1).unwrap();
            self.from_date = month_start.format("%Y-%m-%d").to_string();
            self.to_date = today.format("%Y-%m-%d").to_string();
        }
        
        ui.horizontal(|ui| {
            ui.label("Start Date (YYYY-MM-DD):");
            ui.text_edit_singleline(&mut self.from_date);
            if ui.button("This Month Start").clicked() {
                let today = Local::now().date_naive();
                self.from_date = today.with_day(1).unwrap().format("%Y-%m-%d").to_string();
            }
        });
        
        ui.horizontal(|ui| {
            ui.label("End Date (YYYY-MM-DD):");
            ui.text_edit_singleline(&mut self.to_date);
            if ui.button("Today").clicked() {
                self.to_date = Local::now().date_naive().format("%Y-%m-%d").to_string();
            }
        });
        
        ui.separator();
    }

    /// 出力ディレクトリセクションをレンダリングする（複雑度削減版）
    /// 
    /// 事前条件:
    /// - ui は有効なegui::Uiである
    /// 
    /// 事後条件:
    /// - 出力ディレクトリセクションが描画される
    /// - デフォルトディレクトリが設定される
    /// 
    /// 不変条件:
    /// - ディレクトリパスが有効である
    fn render_output_directory_section(&mut self, ui: &mut egui::Ui) {
        ui.heading("Output Directory");
        
        if self.output_dir.is_empty() {
            self.output_dir = self.get_default_downloads_dir();
        }
        
        ui.horizontal(|ui| {
            ui.label("Output Folder:");
            ui.text_edit_singleline(&mut self.output_dir);
        });
        
        ui.separator();
    }

    /// 認証セクションをレンダリングする（複雑度削減版）
    /// 
    /// 事前条件:
    /// - ui は有効なegui::Uiである
    /// 
    /// 事後条件:
    /// - 認証セクションが描画される
    /// - 認証状態に応じた適切なUIが表示される
    /// 
    /// 不変条件:
    /// - 認証状態が一貫している
    fn render_authentication_section(&mut self, ui: &mut egui::Ui) {
        ui.heading("Authentication");
        
        if self.access_token.is_none() {
            if !self.is_authenticating {
                if ui.button("Start OAuth Authentication").clicked() {
                    self.start_authentication();
                }
            } else {
                self.render_authentication_in_progress(ui);
            }
        } else {
            ui.colored_label(egui::Color32::GREEN, "✓ Authenticated");
            if ui.button("Reset Authentication").clicked() {
                self.access_token = None;
                self.auth_url = None;
                self.auth_code.clear();
            }
        }
        
        ui.separator();
    }

    /// 認証進行中のUIをレンダリングする（複雑度削減版）
    /// 
    /// 事前条件:
    /// - ui は有効なegui::Uiである
    /// - 認証が進行中である
    /// 
    /// 事後条件:
    /// - 認証URL と認証コード入力UIが描画される
    /// 
    /// 不変条件:
    /// - 認証フローが適切に処理される
    fn render_authentication_in_progress(&mut self, ui: &mut egui::Ui) {
        if let Some(url) = &self.auth_url {
            ui.label("Please open the following URL in your browser and complete authentication:");
            ui.text_edit_multiline(&mut url.clone());
            
            if ui.button("Copy URL to Clipboard").clicked() {
                ui.output_mut(|o| o.copied_text = url.clone());
            }
            
            if ui.button("Open in Browser").clicked() {
                let _ = open::that(url);
            }
            
            ui.separator();
            
            ui.label("After authentication, please enter the authorization code:");
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.auth_code);
                if ui.button("Submit Authorization Code").clicked() && !self.auth_code.is_empty() {
                    self.complete_authentication();
                }
            });
        }
    }

    /// ダウンロードセクションをレンダリングする（複雑度削減版）
    /// 
    /// 事前条件:
    /// - ui は有効なegui::Uiである
    /// 
    /// 事後条件:
    /// - ダウンロードセクションが描画される
    /// - ダウンロード可能性が適切に制御される
    /// 
    /// 不変条件:
    /// - ダウンロード状態が一貫している
    fn render_download_section(&mut self, ui: &mut egui::Ui) {
        ui.heading("Download");
        
        let can_download = self.access_token.is_some() 
            && !self.from_date.is_empty() 
            && !self.to_date.is_empty() 
            && !self.is_downloading;
        
        ui.add_enabled_ui(can_download, |ui| {
            if ui.button("Start Download").clicked() {
                self.start_download();
            }
        });
        
        if self.is_downloading {
            ui.spinner();
            ui.label("Downloading...");
        }
        
        ui.separator();
    }

    /// ステータスセクションをレンダリングする（複雑度削減版）
    /// 
    /// 事前条件:
    /// - ui は有効なegui::Uiである
    /// 
    /// 事後条件:
    /// - ステータスと進行状況が描画される
    /// 
    /// 不変条件:
    /// - ステータス情報が適切に表示される
    fn render_status_section(&self, ui: &mut egui::Ui) {
        ui.heading("Status");
        ui.label(&self.status_message);
        
        if !self.download_progress.is_empty() {
            ui.separator();
            ui.heading("Progress");
            egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                for msg in &self.download_progress {
                    ui.label(msg);
                }
            });
        }
    }
}

impl eframe::App for ZoomDownloaderApp {
    /// GUI の更新処理を実行する（リファクタリング版・複雑度削減）
    /// 
    /// 事前条件:
    /// - ctx は有効なegui::Contextである
    /// - _frame は有効なeframe::Frameである
    /// 
    /// 事後条件:
    /// - 受信したメッセージが全て処理される
    /// - GUI の状態が適切に更新される
    /// - UI コンポーネントが描画される
    /// 
    /// 不変条件:
    /// - この関数は毎フレーム呼び出される
    /// - 処理中にGUIの状態が一貫性を保つ
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process incoming messages
        self.process_messages();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Zoom Recording Downloader");
            ui.separator();

            // Render each section using separate functions
            self.render_config_section(ui);
            self.render_date_section(ui);
            self.render_output_directory_section(ui);
            self.render_authentication_section(ui);
            self.render_download_section(ui);
            self.render_status_section(ui);
        });
        
        // Request repaint for real-time updates
        ctx.request_repaint();
    }
}

impl ZoomDownloaderApp {
    /// 設定ファイルを読み込み、GUI状態を更新する
    /// 
    /// # 副作用
    /// - ファイルシステムからの読み込み
    /// - ファイルが存在しない場合はサンプルファイルを作成
    /// - GUI内部状態の変更
    fn load_config(&mut self) {
        match Config::load_from_file("config.toml") {
            Ok(config) => {
                self.client_id = config.client_id;
                self.client_secret = config.client_secret;
                self.config_loaded = true;
                self.status_message = "Configuration loaded".to_string();
            }
            Err(_) => {
                // Create sample config
                let _ = Config::create_sample_file("config.toml");
                self.status_message = "Configuration file not found. Created config.toml.".to_string();
            }
        }
    }
    
    /// 現在のGUI設定をファイルに保存する
    /// 
    /// # 副作用
    /// - ファイルシステムへの書き込み
    /// - GUI内部状態の変更（ステータスメッセージの更新）
    fn save_config(&mut self) {
        let config = Config {
            client_id: self.client_id.clone(),
            client_secret: self.client_secret.clone(),
            redirect_uri: Some("http://localhost:8080/callback".to_string()),
        };
        
        match config.save_to_file("config.toml") {
            Ok(_) => {
                self.status_message = "Configuration saved".to_string();
            }
            Err(e) => {
                self.status_message = format!("Failed to save configuration: {}", e);
            }
        }
    }
    
    fn get_default_downloads_dir(&self) -> String {
        if cfg!(windows) {
            match dirs::download_dir() {
                Some(path) => path.join("ZoomRecordings").to_string_lossy().to_string(),
                None => ".\\downloads".to_string(),
            }
        } else {
            "./downloads".to_string()
        }
    }
    
    fn start_authentication(&mut self) {
        let client_id = self.client_id.clone();
        let client_secret = self.client_secret.clone();
        let sender = self.sender.clone();
        
        thread::spawn(move || {
            let rt = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    let _ = sender.send(AppMessage::Error(format!("Runtime creation error: {}", e)));
                    return;
                }
            };
            
            rt.block_on(async {
                match generate_auth_url(&client_id, &client_secret).await {
                    Ok(url) => {
                        let _ = sender.send(AppMessage::AuthUrlGenerated(url));
                    }
                    Err(e) => {
                        let _ = sender.send(AppMessage::Error(format!("Auth URL generation error: {}", e)));
                    }
                }
            });
        });
    }
    
    fn complete_authentication(&mut self) {
        let client_id = self.client_id.clone();
        let client_secret = self.client_secret.clone();
        let auth_code = self.auth_code.clone();
        let sender = self.sender.clone();
        
        thread::spawn(move || {
            let rt = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    let _ = sender.send(AppMessage::Error(format!("Runtime creation error: {}", e)));
                    return;
                }
            };
            
            rt.block_on(async {
                match exchange_code_for_token(&client_id, &client_secret, &auth_code).await {
                    Ok(token) => {
                        let _ = sender.send(AppMessage::AuthComplete(token));
                    }
                    Err(e) => {
                        let _ = sender.send(AppMessage::Error(format!("Token acquisition error: {}", e)));
                    }
                }
            });
        });
    }
    
    fn start_download(&mut self) {
        if let Some(access_token) = &self.access_token {
            self.is_downloading = true;
            self.download_progress.clear();
            
            let access_token = access_token.clone();
            let from_date = self.from_date.clone();
            let to_date = self.to_date.clone();
            let output_dir = self.output_dir.clone();
            let sender = self.sender.clone();
            
            thread::spawn(move || {
                let rt = match tokio::runtime::Runtime::new() {
                    Ok(rt) => rt,
                    Err(e) => {
                        let _ = sender.send(AppMessage::Error(format!("Runtime creation error: {}", e)));
                        return;
                    }
                };
                
                rt.block_on(async {
                    let _ = sender.send(AppMessage::DownloadProgress("Fetching recording list...".to_string()));
                    
                    let downloader = ZoomRecordingDownloader::new(access_token);
                    
                    match downloader.download_all_recordings("me", &from_date, &to_date, &output_dir).await {
                        Ok(files) => {
                            let _ = sender.send(AppMessage::DownloadComplete(files));
                        }
                        Err(e) => {
                            let _ = sender.send(AppMessage::Error(format!("Download error: {}", e)));
                        }
                    }
                });
            });
        }
    }
}

async fn generate_auth_url(client_id: &str, client_secret: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, Scope, CsrfToken, PkceCodeChallenge, TokenUrl};
    
    let oauth_client = BasicClient::new(
        ClientId::new(client_id.to_string()),
        Some(ClientSecret::new(client_secret.to_string())),
        AuthUrl::new("https://zoom.us/oauth/authorize".to_string())?,
        Some(TokenUrl::new("https://zoom.us/oauth/token".to_string())?),
    )
    .set_redirect_uri(RedirectUrl::new("http://localhost:8080/callback".to_string())?);
    
    let (pkce_challenge, _pkce_verifier) = PkceCodeChallenge::new_random_sha256();
    
    let (auth_url, _csrf_token) = oauth_client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("recording:read".to_string()))
        .add_scope(Scope::new("user:read".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();
    
    Ok(auth_url.to_string())
}

async fn exchange_code_for_token(client_id: &str, client_secret: &str, auth_code: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl, AuthorizationCode, TokenResponse};
    
    let oauth_client = BasicClient::new(
        ClientId::new(client_id.to_string()),
        Some(ClientSecret::new(client_secret.to_string())),
        AuthUrl::new("https://zoom.us/oauth/authorize".to_string())?,
        Some(TokenUrl::new("https://zoom.us/oauth/token".to_string())?),
    )
    .set_redirect_uri(RedirectUrl::new("http://localhost:8080/callback".to_string())?);
    
    // Note: In a real implementation, you'd need to store and retrieve the PKCE verifier
    // For now, we'll use a dummy verifier
    let (_, pkce_verifier) = oauth2::PkceCodeChallenge::new_random_sha256();
    
    let token_result = oauth_client
        .exchange_code(AuthorizationCode::new(auth_code.to_string()))
        .set_pkce_verifier(pkce_verifier)
        .request_async(oauth2::reqwest::async_http_client)
        .await?;
    
    Ok(token_result.access_token().secret().to_string())
}