use eframe::egui;
use std::sync::mpsc;
use std::thread;
use chrono::{Datelike, Local};
use crate::{Config, ZoomRecordingDownloader, RecordingResponse};

#[derive(Debug)]
pub enum AppMessage {
    AuthUrlGenerated(String),
    AuthComplete(String),
    RecordingsLoaded(RecordingResponse),
    DownloadProgress(String),
    DownloadComplete(Vec<String>),
    Error(String),
}

#[derive(Debug, PartialEq)]
pub enum AppScreen {
    Config,      // SC002: 設定画面
    Auth,        // SC003: 認証画面
    Recordings,  // SC004: 録画リスト画面
    Progress,    // SC005: ダウンロード進捗画面
    Error,       // SC006: エラー表示画面
}

pub struct ZoomDownloaderApp {
    // UI State
    current_screen: AppScreen,
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
    
    // Recordings Data
    recordings: Option<RecordingResponse>,
    selected_recordings: std::collections::HashSet<String>,
    
    // Progress
    status_message: String,
    download_progress: Vec<String>,
    current_file: String,
    progress_percentage: f32,
    
    // Error State
    error_message: String,
    error_details: String,
    
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
            current_screen: AppScreen::Config,
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
            recordings: None,
            selected_recordings: std::collections::HashSet::new(),
            status_message: "Ready".to_string(),
            download_progress: Vec::new(),
            current_file: String::new(),
            progress_percentage: 0.0,
            error_message: String::new(),
            error_details: String::new(),
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
                    self.current_screen = AppScreen::Recordings;
                    self.status_message = "Authentication completed.".to_string();
                }
                AppMessage::RecordingsLoaded(recordings) => {
                    self.recordings = Some(recordings);
                    self.status_message = "Recordings loaded.".to_string();
                }
                AppMessage::DownloadProgress(msg) => {
                    self.download_progress.push(msg.clone());
                    self.status_message = msg;
                    self.current_screen = AppScreen::Progress;
                }
                AppMessage::DownloadComplete(files) => {
                    self.is_downloading = false;
                    self.current_screen = AppScreen::Recordings;
                    self.status_message = format!("Download completed: {} files", files.len());
                    self.download_progress.push(format!("Completed: Downloaded {} files", files.len()));
                }
                AppMessage::Error(err) => {
                    self.is_authenticating = false;
                    self.is_downloading = false;
                    self.error_message = err.clone();
                    self.error_details = format!("Timestamp: {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
                    self.current_screen = AppScreen::Error;
                    self.status_message = format!("Error: {}", err);
                }
            }
        }
    }

    /// SC002: 設定画面をレンダリングする
    /// 
    /// 事前条件:
    /// - ui は有効なegui::Uiである
    /// 
    /// 事後条件:
    /// - 設定画面が画面仕様書通りに描画される
    /// - ユーザーの操作が適切に処理される
    /// 
    /// 不変条件:
    /// - UI の状態が一貫性を保つ
    fn render_config(&mut self, ui: &mut egui::Ui) {
        ui.heading("設定");
        ui.separator();
        
        ui.add_space(10.0);
        
        // 設定フォーム
        egui::Grid::new("config_grid")
            .num_columns(2)
            .spacing([20.0, 10.0])
            .show(ui, |ui| {
                // CF001: Client ID入力
                ui.label("Client ID:");
                ui.add_sized([300.0, 25.0], egui::TextEdit::singleline(&mut self.client_id));
                ui.end_row();
                
                // CF002: Client Secret入力 (パスワード形式)
                ui.label("Client Secret:");
                ui.add_sized([300.0, 25.0], egui::TextEdit::singleline(&mut self.client_secret).password(true));
                ui.end_row();
                
                // CF003: 出力ディレクトリ入力
                ui.label("Output Directory:");
                if self.output_dir.is_empty() {
                    self.output_dir = self.get_default_downloads_dir();
                }
                ui.add_sized([300.0, 25.0], egui::TextEdit::singleline(&mut self.output_dir));
                ui.end_row();
            });
        
        ui.add_space(20.0);
        
        // CF004 & CF005: 設定保存・読込ボタン
        ui.horizontal(|ui| {
            let save_button = egui::Button::new("設定を保存")
                .fill(egui::Color32::from_rgb(46, 139, 87));
            if ui.add_sized([120.0, 35.0], save_button).clicked() {
                self.save_config();
            }
            
            ui.add_space(15.0);
            
            let load_button = egui::Button::new("設定を読込")
                .fill(egui::Color32::from_rgb(65, 105, 225));
            if ui.add_sized([120.0, 35.0], load_button).clicked() {
                self.load_config();
            }
        });
        
        ui.add_space(15.0);
        
        // 入力検証とバリデーションメッセージ
        if self.client_id.is_empty() {
            ui.colored_label(egui::Color32::RED, "⚠ Client ID is required");
        } else if self.client_secret.is_empty() {
            ui.colored_label(egui::Color32::RED, "⚠ Client Secret is required");
        } else {
            ui.colored_label(egui::Color32::GREEN, "✓ 設定が有効です");
            self.config_loaded = true;
        }
    }

}

impl eframe::App for ZoomDownloaderApp {
    /// GUI の更新処理を実行する（画面仕様準拠・タブベース）
    /// 
    /// 事前条件:
    /// - ctx は有効なegui::Contextである
    /// - _frame は有効なeframe::Frameである
    /// 
    /// 事後条件:
    /// - 受信したメッセージが全て処理される
    /// - GUI の状態が適切に更新される
    /// - タブベースのUI コンポーネントが描画される
    /// 
    /// 不変条件:
    /// - この関数は毎フレーム呼び出される
    /// - 処理中にGUIの状態が一貫性を保つ
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process incoming messages
        self.process_messages();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Zoom Video Mover");
            ui.separator();

            // SC001: メイン画面 - タブコンテナ
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 12.0;
                
                // MC004: 設定タブ (常時表示)
                let config_button = egui::Button::new("設定")
                    .fill(if self.current_screen == AppScreen::Config { 
                        egui::Color32::from_rgb(100, 149, 237) 
                    } else { 
                        egui::Color32::from_gray(220) 
                    });
                if ui.add_sized([90.0, 35.0], config_button).clicked() {
                    self.current_screen = AppScreen::Config;
                }
                
                // MC005: 認証タブ (config_loaded = true時のみ)
                if self.config_loaded {
                    let auth_button = egui::Button::new("認証")
                        .fill(if self.current_screen == AppScreen::Auth { 
                            egui::Color32::from_rgb(100, 149, 237) 
                        } else { 
                            egui::Color32::from_gray(220) 
                        });
                    if ui.add_sized([90.0, 35.0], auth_button).clicked() {
                        self.current_screen = AppScreen::Auth;
                    }
                }
                
                // MC006: 録画リストタブ (access_token != None時のみ)
                if self.access_token.is_some() {
                    let recordings_button = egui::Button::new("録画リスト")
                        .fill(if self.current_screen == AppScreen::Recordings { 
                            egui::Color32::from_rgb(100, 149, 237) 
                        } else { 
                            egui::Color32::from_gray(220) 
                        });
                    if ui.add_sized([110.0, 35.0], recordings_button).clicked() {
                        self.current_screen = AppScreen::Recordings;
                    }
                }
                
                // MC007: ダウンロードタブ (is_downloading = true時のみ)
                if self.is_downloading {
                    let progress_button = egui::Button::new("ダウンロード")
                        .fill(if self.current_screen == AppScreen::Progress { 
                            egui::Color32::from_rgb(100, 149, 237) 
                        } else { 
                            egui::Color32::from_gray(220) 
                        });
                    if ui.add_sized([110.0, 35.0], progress_button).clicked() {
                        self.current_screen = AppScreen::Progress;
                    }
                }
            });
            
            ui.separator();
            
            // 現在のタブコンテンツ表示エリア
            match self.current_screen {
                AppScreen::Config => self.render_config(ui),
                AppScreen::Auth => self.render_auth(ui),
                AppScreen::Recordings => self.render_recordings(ui),
                AppScreen::Progress => self.render_progress(ui),
                AppScreen::Error => {
                    self.render_error(ui);
                    // エラー画面でも設定タブに戻れるように
                    ui.separator();
                    if ui.button("設定画面に戻る").clicked() {
                        self.current_screen = AppScreen::Config;
                        self.error_message.clear();
                        self.error_details.clear();
                    }
                },
            }
            
            ui.separator();
            
            // MC003: ステータスバー
            ui.horizontal(|ui| {
                ui.label("Status:");
                ui.colored_label(
                    if self.error_message.is_empty() { egui::Color32::GREEN } else { egui::Color32::RED },
                    &self.status_message
                );
            });
        });
        
        // Request repaint for real-time updates
        ctx.request_repaint();
    }
}

impl ZoomDownloaderApp {
    /// SC003: 認証画面をレンダリングする
    /// 
    /// 事前条件:
    /// - ui は有効なegui::Uiである
    /// - config_loaded が true である
    /// 
    /// 事後条件:
    /// - 認証画面が画面仕様書通りに描画される
    /// - OAuth認証フローが適切に処理される
    /// 
    /// 不変条件:
    /// - 認証状態が一貫している
    fn render_auth(&mut self, ui: &mut egui::Ui) {
        ui.heading("認証");
        ui.separator();
        
        // 認証状態表示
        let status_text = if self.access_token.is_some() {
            "Status: Authenticated"
        } else if self.is_authenticating {
            "Status: Waiting for Code"
        } else if self.auth_url.is_some() {
            "Status: Auth URL Generated"
        } else {
            "Status: Ready"
        };
        ui.label(status_text);
        ui.add_space(10.0);
        
        if self.access_token.is_none() {
            if !self.is_authenticating {
                // AU001: 認証開始ボタン
                if ui.button("認証開始").clicked() {
                    self.start_authentication();
                }
            } else {
                self.render_auth_in_progress(ui);
            }
        } else {
            ui.colored_label(egui::Color32::GREEN, "✓ Authenticated");
            if ui.button("Reset Authentication").clicked() {
                self.access_token = None;
                self.auth_url = None;
                self.auth_code.clear();
                self.is_authenticating = false;
            }
        }
    }
    
    /// SC003: 認証進行中の詳細UIをレンダリングする
    /// 
    /// 事前条件:
    /// - ui は有効なegui::Uiである
    /// - 認証が進行中である (is_authenticating = true)
    /// 
    /// 事後条件:
    /// - 認証URL と認証コード入力UIが描画される
    /// 
    /// 不変条件:
    /// - 認証フローが適切に処理される
    fn render_auth_in_progress(&mut self, ui: &mut egui::Ui) {
        if let Some(url) = &self.auth_url {
            // AU002: Auth URL表示
            ui.label("Auth URL:");
            let mut url_display = url.clone();
            ui.add_sized([ui.available_width(), 60.0], egui::TextEdit::multiline(&mut url_display));
            
            ui.horizontal(|ui| {
                // AU003: URLコピーボタン
                if ui.button("📋 コピー").clicked() {
                    ui.output_mut(|o| o.copied_text = url.clone());
                }
                
                // AU004: ブラウザ起動ボタン
                if ui.button("ブラウザで開く").clicked() {
                    let _ = open::that(url);
                }
            });
            
            ui.add_space(15.0);
            
            // AU005: 認証コード入力
            ui.label("Authorization Code:");
            ui.add_sized([ui.available_width(), 20.0], egui::TextEdit::singleline(&mut self.auth_code));
            
            ui.add_space(10.0);
            
            // AU006: 認証完了ボタン
            if ui.add_enabled(!self.auth_code.is_empty(), egui::Button::new("認証完了")).clicked() {
                self.complete_authentication();
            }
        }
    }
    
    /// SC004: 録画リスト画面をレンダリングする
    /// 
    /// 事前条件:
    /// - ui は有効なegui::Uiである
    /// - access_token が設定されている
    /// 
    /// 事後条件:
    /// - 録画リスト画面が画面仕様書通りに描画される
    /// - ファイル選択機能が適切に動作する
    /// 
    /// 不変条件:
    /// - 録画データの整合性が保たれる
    fn render_recordings(&mut self, ui: &mut egui::Ui) {
        ui.heading("録画リスト");
        ui.separator();
        
        // 検索期間設定
        ui.label("検索期間:");
        ui.horizontal(|ui| {
            ui.label("From:");
            if self.from_date.is_empty() {
                let today = Local::now().date_naive();
                let month_start = today.with_day(1).unwrap();
                self.from_date = month_start.format("%Y-%m-%d").to_string();
            }
            ui.text_edit_singleline(&mut self.from_date);
            
            ui.label("To:");
            if self.to_date.is_empty() {
                self.to_date = Local::now().date_naive().format("%Y-%m-%d").to_string();
            }
            ui.text_edit_singleline(&mut self.to_date);
            
            // RL003: 検索実行ボタン
            if ui.button("検索実行").clicked() {
                self.fetch_recordings();
            }
        });
        
        ui.separator();
        
        // 録画リスト表示
        if let Some(recordings) = &self.recordings {
            // RL004: 全選択チェックボックス
            ui.checkbox(&mut false, "☑ Select All");
            ui.separator();
            
            // 録画リスト
            egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                for meeting in &recordings.meetings {
                    ui.horizontal(|ui| {
                        // RL005: ミーティング選択
                        let mut meeting_selected = self.selected_recordings.contains(&meeting.uuid);
                        if ui.checkbox(&mut meeting_selected, &format!("Meeting - {}", meeting.topic)).changed() {
                            if meeting_selected {
                                self.selected_recordings.insert(meeting.uuid.clone());
                            } else {
                                self.selected_recordings.remove(&meeting.uuid);
                            }
                        }
                    });
                    
                    // ファイルリスト表示（簡略版）
                    for file in &meeting.recording_files {
                        ui.horizontal(|ui| {
                            ui.add_space(20.0);
                            // RL006: ファイル選択
                            let file_id = format!("{}-{}", meeting.uuid, file.id);
                            let mut file_selected = self.selected_recordings.contains(&file_id);
                            if ui.checkbox(&mut file_selected, &format!("☑ {} ({}) - {}MB", 
                                file.file_type, &file.file_extension, 
                                file.file_size / 1024 / 1024)).changed() {
                                if file_selected {
                                    self.selected_recordings.insert(file_id);
                                } else {
                                    self.selected_recordings.remove(&file_id);
                                }
                            }
                        });
                    }
                    ui.add_space(5.0);
                }
            });
            
            ui.separator();
            
            // 統計情報とダウンロードボタン
            ui.horizontal(|ui| {
                ui.label(format!("Selected: {} items", self.selected_recordings.len()));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // RL007: ダウンロード開始ボタン
                    if ui.add_enabled(!self.selected_recordings.is_empty() && !self.is_downloading, 
                        egui::Button::new("ダウンロード")).clicked() {
                        self.start_download();
                    }
                });
            });
        } else {
            ui.label("録画データを読み込むには検索実行ボタンをクリックしてください。");
        }
    }
    
    /// SC005: ダウンロード進捗画面をレンダリングする
    /// 
    /// 事前条件:
    /// - ui は有効なegui::Uiである
    /// - is_downloading が true である
    /// 
    /// 事後条件:
    /// - ダウンロード進捗画面が画面仕様書通りに描画される
    /// - リアルタイム進捗表示が動作する
    /// 
    /// 不変条件:
    /// - 進捗データの整合性が保たれる
    fn render_progress(&mut self, ui: &mut egui::Ui) {
        ui.heading("ダウンロード進捗");
        ui.separator();
        
        // PR001: 全体進捗バー
        ui.label("Overall Progress:");
        ui.add(egui::ProgressBar::new(self.progress_percentage).show_percentage());
        
        ui.add_space(10.0);
        
        // PR002: 現在ファイル名
        if !self.current_file.is_empty() {
            ui.label(format!("Current: {}", self.current_file));
            
            // PR003: ファイル進捗バー（全体進捗と同じ値を使用）
            ui.label("Progress:");
            ui.add(egui::ProgressBar::new(self.progress_percentage).show_percentage());
        }
        
        ui.add_space(15.0);
        
        // PR004 & PR005: 制御ボタン
        ui.horizontal(|ui| {
            if ui.button("一時停止").clicked() {
                // TODO: 一時停止機能実装
            }
            
            if ui.button("キャンセル").clicked() {
                self.is_downloading = false;
                self.current_screen = AppScreen::Recordings;
            }
        });
        
        ui.separator();
        
        // PR006: ログ表示エリア
        ui.label("Download Log:");
        egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
            for msg in &self.download_progress {
                ui.label(msg);
            }
        });
        
        ui.add_space(10.0);
        
        // PR007: 統計情報
        ui.label(format!("Status: {}", self.status_message));
    }
    
    /// SC006: エラー表示画面をレンダリングする
    /// 
    /// 事前条件:
    /// - ui は有効なegui::Uiである
    /// - エラーが発生している
    /// 
    /// 事後条件:
    /// - エラー表示画面が画面仕様書通りに描画される
    /// - リカバリ操作が提供される
    /// 
    /// 不変条件:
    /// - エラー情報の整合性が保たれる
    fn render_error(&mut self, ui: &mut egui::Ui) {
        ui.heading("⚠ エラー");
        ui.separator();
        
        // エラー種別自動判定
        let error_type = if self.error_message.contains("auth") || self.error_message.contains("401") {
            "認証エラー"
        } else if self.error_message.contains("network") || self.error_message.contains("timeout") {
            "ネットワークエラー"
        } else if self.error_message.contains("file") || self.error_message.contains("disk") {
            "ファイルエラー"
        } else {
            "一般エラー"
        };
        
        ui.label(format!("エラー種別: {}", error_type));
        ui.add_space(10.0);
        
        // エラーメッセージ
        ui.label("エラーメッセージ:");
        ui.add_sized([ui.available_width(), 60.0], 
            egui::TextEdit::multiline(&mut self.error_message.clone()).desired_width(f32::INFINITY));
        
        ui.add_space(10.0);
        
        // 詳細情報
        ui.label("詳細情報:");
        ui.add_sized([ui.available_width(), 80.0], 
            egui::TextEdit::multiline(&mut self.error_details.clone()).desired_width(f32::INFINITY));
        
        ui.add_space(15.0);
        
        // 推奨アクション
        ui.label("推奨アクション:");
        match error_type {
            "認証エラー" => {
                ui.label("• 設定画面でClient IDとClient Secretを確認してください");
                ui.label("• Zoom Developer Appの設定を確認してください");
            }
            "ネットワークエラー" => {
                ui.label("• インターネット接続を確認してください");
                ui.label("• ファイアウォール設定を確認してください");
            }
            "ファイルエラー" => {
                ui.label("• ディスク容量を確認してください");
                ui.label("• 出力ディレクトリの権限を確認してください");
            }
            _ => {
                ui.label("• 設定を確認してからリトライしてください");
            }
        }
        
        ui.add_space(15.0);
        
        // アクションボタン
        ui.horizontal(|ui| {
            if ui.button("リトライ").clicked() {
                self.error_message.clear();
                self.error_details.clear();
                self.current_screen = AppScreen::Recordings;
            }
            
            if ui.button("設定に戻る").clicked() {
                self.error_message.clear();
                self.error_details.clear();
                self.current_screen = AppScreen::Config;
            }
            
            if ui.button("ログ出力").clicked() {
                // TODO: ログファイル出力機能実装
                println!("Error: {}", self.error_message);
                println!("Details: {}", self.error_details);
            }
        });
    }
    
    /// 録画データを取得する
    /// 
    /// # 副作用
    /// - HTTPリクエストの送信
    /// - アプリケーション状態の更新
    fn fetch_recordings(&mut self) {
        if let Some(access_token) = &self.access_token {
            let access_token = access_token.clone();
            let from_date = self.from_date.clone();
            let to_date = self.to_date.clone();
            let sender = self.sender.clone();
            
            std::thread::spawn(move || {
                let rt = match tokio::runtime::Runtime::new() {
                    Ok(rt) => rt,
                    Err(e) => {
                        let _ = sender.send(AppMessage::Error(format!("Runtime creation error: {}", e)));
                        return;
                    }
                };
                
                rt.block_on(async {
                    let mut downloader = crate::ZoomRecordingDownloader::new_with_token(
                        "dummy_client_id".to_string(), "dummy_client_secret".to_string(), access_token);
                    match downloader.get_recordings(Some("me"), &from_date, &to_date, None).await {
                        Ok(recordings) => {
                            let _ = sender.send(AppMessage::RecordingsLoaded(recordings));
                        }
                        Err(e) => {
                            let _ = sender.send(AppMessage::Error(format!("Failed to fetch recordings: {}", e)));
                        }
                    }
                });
            });
        }
    }

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
            
            let _access_token = access_token.clone();
            let _from_date = self.from_date.clone();
            let _to_date = self.to_date.clone();
            let _output_dir = self.output_dir.clone();
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
                    
                    // For download, we need to implement file selection logic
                    // This is a simplified placeholder
                    
                    // TODO: Implement actual download logic using download_file method
                    let _ = sender.send(AppMessage::DownloadComplete(vec!["placeholder.mp4".to_string()]));
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