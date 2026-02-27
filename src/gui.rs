use crate::components::api::RecordingSearchResponse;
use crate::services_impl::AppServices;
use crate::Config;
use chrono::{Datelike, Local};
use eframe::egui;
use egui::Color32;
use std::sync::{mpsc, Arc};
use std::thread;

// === Modern Color Palette ===
// Primary
const PRIMARY: Color32 = Color32::from_rgb(59, 130, 246); // Blue-500
const PRIMARY_HOVER: Color32 = Color32::from_rgb(37, 99, 235); // Blue-600

// Semantic
const SUCCESS_BG: Color32 = Color32::from_rgb(220, 252, 231); // Green-100
const SUCCESS_TEXT: Color32 = Color32::from_rgb(22, 101, 52); // Green-800
const WARNING_BG: Color32 = Color32::from_rgb(254, 243, 199); // Amber-100
const WARNING_TEXT: Color32 = Color32::from_rgb(146, 64, 14); // Amber-800
const ERROR: Color32 = Color32::from_rgb(239, 68, 68); // Red-500
const ERROR_BG: Color32 = Color32::from_rgb(254, 226, 226); // Red-100
const ERROR_TEXT: Color32 = Color32::from_rgb(153, 27, 27); // Red-800

// Button
const BTN_SAVE: Color32 = Color32::from_rgb(34, 197, 94); // Green
const BTN_LOAD: Color32 = Color32::from_rgb(59, 130, 246); // Blue
const BTN_PAUSE: Color32 = Color32::from_rgb(245, 158, 11); // Amber
const BTN_CANCEL: Color32 = Color32::from_rgb(239, 68, 68); // Red

// Neutral / Background
const BG_BASE: Color32 = Color32::from_rgb(241, 245, 249); // Slate-100
const BG_CARD: Color32 = Color32::from_rgb(255, 255, 255); // White
const BORDER_DEFAULT: Color32 = Color32::from_rgb(226, 232, 240); // Slate-200

// Text
const TEXT_PRIMARY: Color32 = Color32::from_rgb(15, 23, 42); // Slate-900
const TEXT_SECONDARY: Color32 = Color32::from_rgb(100, 116, 139); // Slate-500
const TEXT_ON_PRIMARY: Color32 = Color32::from_rgb(255, 255, 255); // White

// Progress
const PROGRESS_FILL: Color32 = Color32::from_rgb(59, 130, 246); // Blue

/// GUI表示設定（フォント・テーマ・スタイル）
/// テストのスクリーンショット生成でも使用
///
/// 事前条件:
/// - ctx は有効なegui::Contextである
///
/// 事後条件:
/// - 日本語フォント（NotoSansJP）が設定される
/// - ライトテーマが適用される
/// - フォントサイズ・スペーシング・色が読みやすく調整される
pub fn setup_gui_appearance(ctx: &egui::Context) {
    // 日本語フォント（NotoSansJP）を埋め込み設定
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "NotoSansJP".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/NotoSansJP-Regular.ttf")),
    );
    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, "NotoSansJP".to_owned());
    fonts
        .families
        .get_mut(&egui::FontFamily::Monospace)
        .unwrap()
        .insert(1, "NotoSansJP".to_owned());
    ctx.set_fonts(fonts);

    // ライトテーマを明示的に設定（カラーパレット適用）
    ctx.set_visuals(egui::Visuals::light());

    // フォントとスタイルの設定
    ctx.style_mut(|style| {
        // フォントサイズを大幅に拡大（日本語の可読性向上）
        for (_, font_id) in style.text_styles.iter_mut() {
            font_id.size = (font_id.size * 1.5).max(20.0);
        }

        // スペーシングの改善
        style.spacing.item_spacing.x = 12.0;
        style.spacing.item_spacing.y = 10.0;
        style.spacing.button_padding.x = 20.0;
        style.spacing.button_padding.y = 10.0;

        let rounding = egui::Rounding::same(8.0);

        // noninteractive
        style.visuals.widgets.noninteractive.bg_fill = BG_CARD;
        style.visuals.widgets.noninteractive.rounding = rounding;
        style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, TEXT_PRIMARY);

        // inactive
        style.visuals.widgets.inactive.bg_fill = Color32::from_gray(245);
        style.visuals.widgets.inactive.weak_bg_fill = Color32::from_gray(240);
        style.visuals.widgets.inactive.rounding = rounding;
        style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, TEXT_PRIMARY);
        style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, BORDER_DEFAULT);

        // hovered
        style.visuals.widgets.hovered.bg_fill = Color32::from_gray(230);
        style.visuals.widgets.hovered.weak_bg_fill = Color32::from_gray(225);
        style.visuals.widgets.hovered.rounding = rounding;
        style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, TEXT_PRIMARY);

        // active
        style.visuals.widgets.active.bg_fill = PRIMARY;
        style.visuals.widgets.active.weak_bg_fill = PRIMARY_HOVER;
        style.visuals.widgets.active.rounding = rounding;
        style.visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, TEXT_ON_PRIMARY);

        // パネル・ウィンドウ背景
        style.visuals.panel_fill = BG_BASE;
        style.visuals.window_fill = BG_CARD;
        style.visuals.window_shadow = egui::Shadow {
            offset: [0.0, 4.0].into(),
            blur: 12.0,
            spread: 0.0,
            color: Color32::from_black_alpha(20),
        };

        // 選択色
        style.visuals.selection.bg_fill = PRIMARY.gamma_multiply(0.3);
    });
}

/// カード型フレーム（各セクションの囲み用）
fn card_frame() -> egui::Frame {
    egui::Frame::none()
        .fill(BG_CARD)
        .rounding(egui::Rounding::same(12.0))
        .shadow(egui::Shadow {
            offset: [0.0, 2.0].into(),
            blur: 8.0,
            spread: 0.0,
            color: Color32::from_black_alpha(15),
        })
        .stroke(egui::Stroke::new(1.0, BORDER_DEFAULT))
        .inner_margin(egui::Margin::same(20.0))
        .outer_margin(egui::Margin::symmetric(0.0, 6.0))
}

/// エラー表示用カードフレーム
fn error_card_frame() -> egui::Frame {
    egui::Frame::none()
        .fill(ERROR_BG)
        .rounding(egui::Rounding::same(12.0))
        .stroke(egui::Stroke::new(2.0, ERROR))
        .inner_margin(egui::Margin::same(20.0))
        .outer_margin(egui::Margin::symmetric(0.0, 6.0))
}

#[derive(Debug)]
pub enum AppMessage {
    AuthUrlGenerated(String),
    AuthComplete(String),
    RecordingsLoaded(RecordingSearchResponse),
    DownloadProgress(String),
    DownloadComplete(Vec<String>),
    DownloadPaused,
    DownloadResumed,
    DownloadCancelled,
    LogExported(String),
    Error(String),
}

#[derive(Debug, PartialEq)]
pub enum AppScreen {
    Config,     // SC002: 設定画面
    Auth,       // SC003: 認証画面
    Recordings, // SC004: 録画リスト画面
    Progress,   // SC005: ダウンロード進捗画面
    Error,      // SC006: エラー表示画面
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
    is_download_paused: bool,
    download_can_resume: bool,
    access_token: Option<String>,

    // Recordings Data
    recordings: Option<RecordingSearchResponse>,
    selected_recordings: std::collections::HashSet<String>,

    // Progress
    status_message: String,
    download_progress: Vec<String>,
    current_file: String,
    progress_percentage: f32,

    // Error State
    error_message: String,
    error_details: String,

    // Logging
    log_entries: Vec<LogEntry>,

    // Communication
    receiver: mpsc::Receiver<AppMessage>,
    sender: mpsc::Sender<AppMessage>,

    // Services (DI)
    services: AppServices,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: chrono::DateTime<chrono::Local>,
    pub level: LogLevel,
    pub message: String,
    pub details: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Debug,
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

        let mut app = Self {
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
            is_download_paused: false,
            download_can_resume: false,
            access_token: None,
            recordings: None,
            selected_recordings: std::collections::HashSet::new(),
            status_message: "Ready".to_string(),
            download_progress: Vec::new(),
            current_file: String::new(),
            progress_percentage: 0.0,
            error_message: String::new(),
            error_details: String::new(),
            log_entries: Vec::new(),
            receiver,
            sender,
            services: AppServices::default(),
        };

        // 初期ログエントリを追加
        app.add_log_entry(LogLevel::Info, "Application started".to_string(), None);
        app
    }
}

impl ZoomDownloaderApp {
    /// ログエントリを追加する
    ///
    /// # 事前条件
    /// - level は有効なLogLevelである
    /// - message は空でない文字列である
    ///
    /// # 事後条件
    /// - 新しいログエントリがlog_entriesに追加される
    /// - タイムスタンプが自動で設定される
    fn add_log_entry(&mut self, level: LogLevel, message: String, details: Option<String>) {
        let entry = LogEntry {
            timestamp: chrono::Local::now(),
            level,
            message,
            details,
        };
        self.log_entries.push(entry);

        // ログエントリ数制限（最大1000件）
        if self.log_entries.len() > 1000 {
            self.log_entries.remove(0);
        }
    }

    /// ログをファイルにエクスポートする
    ///
    /// # 事前条件
    /// - output_dir が有効なディレクトリパスである
    ///
    /// # 事後条件
    /// - ログファイルが指定ディレクトリに作成される
    /// - 成功時はファイルパスが返される
    fn export_logs(&self) -> Result<String, String> {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("zoom_video_mover_log_{}.txt", timestamp);
        let filepath = std::path::Path::new(&self.output_dir).join(&filename);

        let mut log_content = String::new();
        log_content.push_str("=== Zoom Video Mover Log Export ===\n");
        log_content.push_str(&format!(
            "Export Time: {}\n",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        ));
        log_content.push_str(&format!("Total Entries: {}\n", self.log_entries.len()));
        log_content.push_str("=====================================\n\n");

        for entry in &self.log_entries {
            log_content.push_str(&format!(
                "[{}] {} - {}\n",
                entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
                match entry.level {
                    LogLevel::Info => "INFO ",
                    LogLevel::Warning => "WARN ",
                    LogLevel::Error => "ERROR",
                    LogLevel::Debug => "DEBUG",
                },
                entry.message
            ));

            if let Some(details) = &entry.details {
                log_content.push_str(&format!("  Details: {}\n", details));
            }
            log_content.push('\n');
        }

        // 現在のアプリ状態も追加
        log_content.push_str("\n=== Current Application State ===\n");
        log_content.push_str(&format!("Current Screen: {:?}\n", self.current_screen));
        log_content.push_str(&format!("Config Loaded: {}\n", self.config_loaded));
        log_content.push_str(&format!("Is Authenticating: {}\n", self.is_authenticating));
        log_content.push_str(&format!("Is Downloading: {}\n", self.is_downloading));
        log_content.push_str(&format!(
            "Is Download Paused: {}\n",
            self.is_download_paused
        ));
        log_content.push_str(&format!(
            "Access Token Present: {}\n",
            self.access_token.is_some()
        ));
        log_content.push_str(&format!(
            "Selected Recordings: {}\n",
            self.selected_recordings.len()
        ));
        log_content.push_str(&format!("Progress: {:.1}%\n", self.progress_percentage));

        match std::fs::write(&filepath, log_content) {
            Ok(_) => Ok(filepath.to_string_lossy().to_string()),
            Err(e) => Err(format!("Failed to write log file: {}", e)),
        }
    }

    /// ダウンロードを一時停止する
    fn pause_download(&mut self) {
        if self.is_downloading && !self.is_download_paused {
            self.is_download_paused = true;
            self.download_can_resume = true;
            self.status_message = "Download paused by user".to_string();
            self.add_log_entry(
                LogLevel::Info,
                "Download paused".to_string(),
                Some("User requested pause".to_string()),
            );

            let _ = self.sender.send(AppMessage::DownloadPaused);
        }
    }

    /// ダウンロードを再開する
    fn resume_download(&mut self) {
        if self.is_download_paused && self.download_can_resume {
            self.is_download_paused = false;
            self.status_message = "Download resumed".to_string();
            self.add_log_entry(
                LogLevel::Info,
                "Download resumed".to_string(),
                Some("User requested resume".to_string()),
            );

            let _ = self.sender.send(AppMessage::DownloadResumed);
        }
    }

    /// ダウンロードをキャンセルする
    fn cancel_download(&mut self) {
        if self.is_downloading {
            self.is_downloading = false;
            self.is_download_paused = false;
            self.download_can_resume = false;
            self.progress_percentage = 0.0;
            self.current_screen = AppScreen::Recordings;
            self.status_message = "Download cancelled by user".to_string();
            self.add_log_entry(
                LogLevel::Warning,
                "Download cancelled".to_string(),
                Some("User requested cancellation".to_string()),
            );

            let _ = self.sender.send(AppMessage::DownloadCancelled);
        }
    }

    /// メッセージを処理する
    fn process_messages(&mut self) {
        while let Ok(msg) = self.receiver.try_recv() {
            match msg {
                AppMessage::AuthUrlGenerated(url) => {
                    self.auth_url = Some(url);
                    self.is_authenticating = true;
                    self.status_message =
                        "Auth URL generated. Please complete authentication in browser."
                            .to_string();
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
                    self.download_progress
                        .push(format!("Completed: Downloaded {} files", files.len()));
                }
                AppMessage::DownloadPaused => {
                    self.is_download_paused = true;
                    self.download_can_resume = true;
                    self.add_log_entry(
                        LogLevel::Info,
                        "Download paused by background task".to_string(),
                        None,
                    );
                }
                AppMessage::DownloadResumed => {
                    self.is_download_paused = false;
                    self.add_log_entry(
                        LogLevel::Info,
                        "Download resumed by background task".to_string(),
                        None,
                    );
                }
                AppMessage::DownloadCancelled => {
                    self.is_downloading = false;
                    self.is_download_paused = false;
                    self.download_can_resume = false;
                    self.current_screen = AppScreen::Recordings;
                    self.add_log_entry(
                        LogLevel::Warning,
                        "Download cancelled by background task".to_string(),
                        None,
                    );
                }
                AppMessage::LogExported(filepath) => {
                    self.add_log_entry(
                        LogLevel::Info,
                        format!("Log exported to: {}", filepath),
                        None,
                    );
                    self.status_message = format!("Log exported successfully: {}", filepath);
                }
                AppMessage::Error(err) => {
                    self.is_authenticating = false;
                    self.is_downloading = false;
                    self.error_message = err.clone();
                    self.error_details = format!(
                        "Timestamp: {}",
                        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
                    );
                    self.current_screen = AppScreen::Error;
                    self.status_message = format!("Error: {}", err);
                    self.add_log_entry(
                        LogLevel::Error,
                        err,
                        Some("Application error occurred".to_string()),
                    );
                }
            }
        }
    }

    /// UI更新ロジック（テストからも呼び出し可能）
    pub fn update_ui(&mut self, ctx: &egui::Context) {
        self.process_messages();

        let panel_frame = egui::Frame::none()
            .fill(BG_BASE)
            .inner_margin(egui::Margin::same(24.0));

        egui::CentralPanel::default()
            .frame(panel_frame)
            .show(ctx, |ui| {
                // ヘッダー
                ui.add(egui::Label::new(
                    egui::RichText::new("Zoom Video Mover")
                        .size(28.0)
                        .strong()
                        .color(TEXT_PRIMARY),
                ));
                ui.add_space(4.0);

                // SC001: メイン画面 - タブコンテナ（ピル型）
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;

                    let tabs: Vec<(&str, AppScreen, bool)> = vec![
                        ("設定", AppScreen::Config, true),
                        ("認証", AppScreen::Auth, self.config_loaded),
                        (
                            "録画リスト",
                            AppScreen::Recordings,
                            self.access_token.is_some(),
                        ),
                        ("ダウンロード", AppScreen::Progress, self.is_downloading),
                    ];

                    for (label, screen, visible) in tabs {
                        if !visible {
                            continue;
                        }
                        let is_active = self.current_screen == screen;
                        let btn =
                            egui::Button::new(egui::RichText::new(label).color(if is_active {
                                TEXT_ON_PRIMARY
                            } else {
                                TEXT_PRIMARY
                            }))
                            .rounding(egui::Rounding::same(20.0))
                            .fill(if is_active {
                                PRIMARY
                            } else {
                                Color32::TRANSPARENT
                            });
                        if ui.add_sized([100.0, 36.0], btn).clicked() {
                            self.current_screen = screen;
                        }
                    }
                });

                ui.add_space(8.0);

                // 現在のタブコンテンツ表示エリア
                match self.current_screen {
                    AppScreen::Config => self.render_config(ui),
                    AppScreen::Auth => self.render_auth(ui),
                    AppScreen::Recordings => self.render_recordings(ui),
                    AppScreen::Progress => self.render_progress(ui),
                    AppScreen::Error => {
                        self.render_error(ui);
                    }
                }

                ui.add_space(8.0);

                // MC003: ステータスバー
                egui::Frame::none()
                    .fill(BG_CARD)
                    .rounding(egui::Rounding::same(8.0))
                    .stroke(egui::Stroke::new(1.0, BORDER_DEFAULT))
                    .inner_margin(egui::Margin::same(10.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.add(egui::Label::new(
                                egui::RichText::new("Status:")
                                    .size(14.0)
                                    .color(TEXT_SECONDARY),
                            ));
                            let status_color = if self.error_message.is_empty() {
                                SUCCESS_TEXT
                            } else {
                                ERROR_TEXT
                            };
                            ui.add(egui::Label::new(
                                egui::RichText::new(&self.status_message)
                                    .size(14.0)
                                    .color(status_color),
                            ));
                        });
                    });
            });

        ctx.request_repaint();
    }

    /// SC002: 設定画面をレンダリングする
    fn render_config(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::Label::new(
            egui::RichText::new("設定")
                .size(26.0)
                .strong()
                .color(TEXT_PRIMARY),
        ));
        ui.add(egui::Label::new(
            egui::RichText::new("Zoom API接続の設定を行います")
                .size(14.0)
                .color(TEXT_SECONDARY),
        ));
        ui.add_space(8.0);

        // 設定フォーム（カード内）
        card_frame().show(ui, |ui| {
            let field_width = (ui.available_width() - 180.0 - 20.0).max(300.0);
            egui::Grid::new("config_grid")
                .num_columns(2)
                .spacing([20.0, 12.0])
                .show(ui, |ui| {
                    // CF001: Client ID入力
                    ui.add(egui::Label::new(
                        egui::RichText::new("Client ID").color(TEXT_SECONDARY),
                    ));
                    ui.add_sized(
                        [field_width, 30.0],
                        egui::TextEdit::singleline(&mut self.client_id),
                    );
                    ui.end_row();

                    // CF002: Client Secret入力 (パスワード形式)
                    ui.add(egui::Label::new(
                        egui::RichText::new("Client Secret").color(TEXT_SECONDARY),
                    ));
                    ui.add_sized(
                        [field_width, 30.0],
                        egui::TextEdit::singleline(&mut self.client_secret).password(true),
                    );
                    ui.end_row();

                    // CF003: 出力ディレクトリ入力
                    ui.add(egui::Label::new(
                        egui::RichText::new("Output Directory").color(TEXT_SECONDARY),
                    ));
                    if self.output_dir.is_empty() {
                        self.output_dir = self.get_default_downloads_dir();
                    }
                    ui.add_sized(
                        [field_width, 30.0],
                        egui::TextEdit::singleline(&mut self.output_dir),
                    );
                    ui.end_row();
                });
        });

        ui.add_space(12.0);

        // CF004 & CF005: 設定保存・読込ボタン
        ui.horizontal(|ui| {
            let save_button =
                egui::Button::new(egui::RichText::new("設定を保存").color(TEXT_ON_PRIMARY))
                    .fill(BTN_SAVE)
                    .rounding(egui::Rounding::same(8.0));
            if ui.add_sized([140.0, 40.0], save_button).clicked() {
                self.save_config();
            }

            ui.add_space(12.0);

            let load_button =
                egui::Button::new(egui::RichText::new("設定を読込").color(TEXT_ON_PRIMARY))
                    .fill(BTN_LOAD)
                    .rounding(egui::Rounding::same(8.0));
            if ui.add_sized([140.0, 40.0], load_button).clicked() {
                self.load_config();
            }
        });

        ui.add_space(12.0);

        // 入力検証とバリデーションメッセージ（色付きフレーム）
        if self.client_id.is_empty() {
            egui::Frame::none()
                .fill(WARNING_BG)
                .rounding(egui::Rounding::same(8.0))
                .stroke(egui::Stroke::new(1.0, WARNING_TEXT))
                .inner_margin(egui::Margin::same(10.0))
                .show(ui, |ui| {
                    ui.add(egui::Label::new(
                        egui::RichText::new("Client ID is required").color(WARNING_TEXT),
                    ));
                });
        } else if self.client_secret.is_empty() {
            egui::Frame::none()
                .fill(WARNING_BG)
                .rounding(egui::Rounding::same(8.0))
                .stroke(egui::Stroke::new(1.0, WARNING_TEXT))
                .inner_margin(egui::Margin::same(10.0))
                .show(ui, |ui| {
                    ui.add(egui::Label::new(
                        egui::RichText::new("Client Secret is required").color(WARNING_TEXT),
                    ));
                });
        } else {
            egui::Frame::none()
                .fill(SUCCESS_BG)
                .rounding(egui::Rounding::same(8.0))
                .stroke(egui::Stroke::new(1.0, SUCCESS_TEXT))
                .inner_margin(egui::Margin::same(10.0))
                .show(ui, |ui| {
                    ui.add(egui::Label::new(
                        egui::RichText::new("設定が有効です").color(SUCCESS_TEXT),
                    ));
                });
            self.config_loaded = true;
        }
    }
}

impl eframe::App for ZoomDownloaderApp {
    /// GUI の更新処理を実行する（画面仕様準拠・タブベース）
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_ui(ctx);
    }
}

impl ZoomDownloaderApp {
    /// SC003: 認証画面をレンダリングする
    fn render_auth(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::Label::new(
            egui::RichText::new("認証")
                .size(26.0)
                .strong()
                .color(TEXT_PRIMARY),
        ));
        ui.add(egui::Label::new(
            egui::RichText::new("Zoom OAuthで接続します")
                .size(14.0)
                .color(TEXT_SECONDARY),
        ));
        ui.add_space(8.0);

        // 認証状態バッジ
        card_frame().show(ui, |ui| {
            let (badge_text, badge_bg, badge_fg) = if self.access_token.is_some() {
                ("Authenticated", SUCCESS_BG, SUCCESS_TEXT)
            } else if self.is_authenticating {
                ("Waiting for Code", WARNING_BG, WARNING_TEXT)
            } else if self.auth_url.is_some() {
                ("Auth URL Generated", WARNING_BG, WARNING_TEXT)
            } else {
                ("Ready", Color32::from_gray(240), TEXT_SECONDARY)
            };

            egui::Frame::none()
                .fill(badge_bg)
                .rounding(egui::Rounding::same(16.0))
                .inner_margin(egui::Margin::symmetric(14.0, 6.0))
                .show(ui, |ui| {
                    ui.add(egui::Label::new(
                        egui::RichText::new(badge_text).color(badge_fg).size(14.0),
                    ));
                });

            ui.add_space(12.0);

            if self.access_token.is_none() {
                if !self.is_authenticating {
                    // AU001: 認証開始ボタン
                    let btn =
                        egui::Button::new(egui::RichText::new("認証開始").color(TEXT_ON_PRIMARY))
                            .fill(PRIMARY)
                            .rounding(egui::Rounding::same(8.0));
                    if ui.add_sized([140.0, 40.0], btn).clicked() {
                        self.start_authentication();
                    }
                } else {
                    self.render_auth_in_progress(ui);
                }
            } else {
                ui.add(egui::Label::new(
                    egui::RichText::new("Authenticated").color(SUCCESS_TEXT),
                ));
                let btn =
                    egui::Button::new("Reset Authentication").rounding(egui::Rounding::same(8.0));
                if ui.add(btn).clicked() {
                    self.access_token = None;
                    self.auth_url = None;
                    self.auth_code.clear();
                    self.is_authenticating = false;
                }
            }
        });
    }

    /// SC003: 認証進行中の詳細UIをレンダリングする
    fn render_auth_in_progress(&mut self, ui: &mut egui::Ui) {
        if let Some(url) = &self.auth_url {
            // AU002: Auth URL表示
            ui.add(egui::Label::new(
                egui::RichText::new("Auth URL").color(TEXT_SECONDARY),
            ));
            let mut url_display = url.clone();
            ui.add_sized(
                [ui.available_width(), 60.0],
                egui::TextEdit::multiline(&mut url_display),
            );

            let url_for_open = url.clone();
            ui.horizontal(|ui| {
                // AU003: URLコピーボタン
                let copy_btn = egui::Button::new("コピー").rounding(egui::Rounding::same(8.0));
                if ui.add(copy_btn).clicked() {
                    ui.output_mut(|o| o.copied_text = url_for_open.clone());
                }

                // AU004: ブラウザ起動ボタン（サービス経由）
                let open_btn =
                    egui::Button::new(egui::RichText::new("ブラウザで開く").color(TEXT_ON_PRIMARY))
                        .fill(PRIMARY)
                        .rounding(egui::Rounding::same(8.0));
                if ui.add(open_btn).clicked() {
                    let _ = self.services.browser_launcher.open_url(&url_for_open);
                }
            });

            ui.add_space(12.0);

            // AU005: 認証コード入力
            ui.add(egui::Label::new(
                egui::RichText::new("Authorization Code").color(TEXT_SECONDARY),
            ));
            ui.add_sized(
                [ui.available_width(), 30.0],
                egui::TextEdit::singleline(&mut self.auth_code),
            );

            ui.add_space(10.0);

            // AU006: 認証完了ボタン
            let complete_btn =
                egui::Button::new(egui::RichText::new("認証完了").color(TEXT_ON_PRIMARY))
                    .fill(PRIMARY)
                    .rounding(egui::Rounding::same(8.0));
            if ui
                .add_enabled(!self.auth_code.is_empty(), complete_btn)
                .clicked()
            {
                self.complete_authentication();
            }
        }
    }

    /// SC004: 録画リスト画面をレンダリングする
    fn render_recordings(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::Label::new(
            egui::RichText::new("録画リスト")
                .size(26.0)
                .strong()
                .color(TEXT_PRIMARY),
        ));
        ui.add_space(8.0);

        // 検索期間設定（カード内）
        card_frame().show(ui, |ui| {
            ui.add(egui::Label::new(
                egui::RichText::new("検索期間").color(TEXT_SECONDARY),
            ));
            ui.horizontal(|ui| {
                ui.add(egui::Label::new(
                    egui::RichText::new("From").color(TEXT_SECONDARY),
                ));
                if self.from_date.is_empty() {
                    let today = Local::now().date_naive();
                    let month_start = today.with_day(1).unwrap();
                    self.from_date = month_start.format("%Y-%m-%d").to_string();
                }
                ui.add_sized(
                    [150.0, 30.0],
                    egui::TextEdit::singleline(&mut self.from_date),
                );

                ui.add(egui::Label::new(
                    egui::RichText::new("To").color(TEXT_SECONDARY),
                ));
                if self.to_date.is_empty() {
                    self.to_date = Local::now().date_naive().format("%Y-%m-%d").to_string();
                }
                ui.add_sized([150.0, 30.0], egui::TextEdit::singleline(&mut self.to_date));

                // RL003: 検索実行ボタン
                let search_btn =
                    egui::Button::new(egui::RichText::new("検索実行").color(TEXT_ON_PRIMARY))
                        .fill(PRIMARY)
                        .rounding(egui::Rounding::same(8.0));
                if ui.add_sized([120.0, 36.0], search_btn).clicked() {
                    self.fetch_recordings();
                }
            });
        });

        // 録画リスト表示
        if let Some(recordings) = &self.recordings {
            let meeting_uuids: Vec<String> =
                recordings.meetings.iter().map(|m| m.uuid.clone()).collect();

            card_frame().show(ui, |ui| {
                // ヘッダー行: 選択数バッジ + 全選択/全解除
                ui.horizontal(|ui| {
                    ui.add(egui::Label::new(
                        egui::RichText::new("録画一覧").strong().color(TEXT_PRIMARY),
                    ));

                    // 選択数バッジ
                    egui::Frame::none()
                        .fill(PRIMARY)
                        .rounding(egui::Rounding::same(12.0))
                        .inner_margin(egui::Margin::symmetric(10.0, 3.0))
                        .show(ui, |ui| {
                            ui.add(egui::Label::new(
                                egui::RichText::new(format!("{}", self.selected_recordings.len()))
                                    .color(TEXT_ON_PRIMARY)
                                    .size(13.0),
                            ));
                        });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let deselect_btn =
                            egui::Button::new("全解除").rounding(egui::Rounding::same(8.0));
                        if ui.add(deselect_btn).clicked() {
                            self.selected_recordings.clear();
                        }
                        let select_btn =
                            egui::Button::new("全選択").rounding(egui::Rounding::same(8.0));
                        if ui.add(select_btn).clicked() {
                            for uuid in &meeting_uuids {
                                self.selected_recordings.insert(uuid.clone());
                            }
                        }
                    });
                });

                ui.add_space(6.0);

                // 録画リスト
                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        for meeting in &recordings.meetings {
                            ui.horizontal(|ui| {
                                let mut meeting_selected =
                                    self.selected_recordings.contains(&meeting.uuid);
                                if ui
                                    .checkbox(
                                        &mut meeting_selected,
                                        format!("Meeting - {}", meeting.topic),
                                    )
                                    .changed()
                                {
                                    if meeting_selected {
                                        self.selected_recordings.insert(meeting.uuid.clone());
                                    } else {
                                        self.selected_recordings.remove(&meeting.uuid);
                                    }
                                }
                            });

                            for file in &meeting.recording_files {
                                ui.horizontal(|ui| {
                                    ui.add_space(20.0);
                                    let file_id = format!("{}-{}", meeting.uuid, file.stable_id());
                                    let mut file_selected =
                                        self.selected_recordings.contains(&file_id);
                                    let ext_display = if file.file_extension.is_empty() {
                                        file.file_type.to_string()
                                    } else {
                                        file.file_extension.clone()
                                    };
                                    if ui
                                        .checkbox(
                                            &mut file_selected,
                                            format!(
                                                "{} ({}) - {}MB",
                                                file.file_type,
                                                ext_display,
                                                file.file_size / 1024 / 1024
                                            ),
                                        )
                                        .changed()
                                    {
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
            });

            ui.add_space(8.0);

            // ダウンロードボタン
            ui.horizontal(|ui| {
                ui.add(egui::Label::new(
                    egui::RichText::new(format!(
                        "Selected: {} items",
                        self.selected_recordings.len()
                    ))
                    .color(TEXT_SECONDARY),
                ));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let dl_btn = egui::Button::new(
                        egui::RichText::new("ダウンロード").color(TEXT_ON_PRIMARY),
                    )
                    .fill(PRIMARY)
                    .rounding(egui::Rounding::same(8.0));
                    if ui
                        .add_enabled(
                            !self.selected_recordings.is_empty() && !self.is_downloading,
                            dl_btn,
                        )
                        .clicked()
                    {
                        self.start_download();
                    }
                });
            });
        } else {
            // 空状態
            card_frame().show(ui, |ui| {
                ui.add(egui::Label::new(
                    egui::RichText::new(
                        "録画データを読み込むには検索実行ボタンをクリックしてください。",
                    )
                    .color(TEXT_SECONDARY),
                ));
            });
        }
    }

    /// SC005: ダウンロード進捗画面をレンダリングする
    fn render_progress(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::Label::new(
            egui::RichText::new("ダウンロード進捗")
                .size(26.0)
                .strong()
                .color(TEXT_PRIMARY),
        ));
        ui.add_space(8.0);

        // 進捗カード
        card_frame().show(ui, |ui| {
            // PR001: 全体進捗バー
            ui.add(egui::Label::new(
                egui::RichText::new("Overall Progress").color(TEXT_SECONDARY),
            ));
            ui.add(
                egui::ProgressBar::new(self.progress_percentage)
                    .fill(PROGRESS_FILL)
                    .rounding(egui::Rounding::same(6.0))
                    .desired_height(12.0)
                    .show_percentage(),
            );

            ui.add_space(10.0);

            // PR002: 現在ファイル名
            if !self.current_file.is_empty() {
                ui.add(egui::Label::new(
                    egui::RichText::new(format!("Current: {}", self.current_file))
                        .color(TEXT_PRIMARY),
                ));

                // PR003: ファイル進捗バー
                ui.add(
                    egui::ProgressBar::new(self.progress_percentage)
                        .fill(PROGRESS_FILL)
                        .rounding(egui::Rounding::same(6.0))
                        .desired_height(12.0)
                        .show_percentage(),
                );
            }

            ui.add_space(12.0);

            // PR004 & PR005: 制御ボタン
            ui.horizontal(|ui| {
                if self.is_download_paused {
                    let resume_button =
                        egui::Button::new(egui::RichText::new("再開").color(TEXT_ON_PRIMARY))
                            .fill(BTN_SAVE)
                            .rounding(egui::Rounding::same(8.0));
                    if ui.add_sized([120.0, 40.0], resume_button).clicked() {
                        self.resume_download();
                    }
                } else {
                    let pause_button =
                        egui::Button::new(egui::RichText::new("一時停止").color(TEXT_ON_PRIMARY))
                            .fill(BTN_PAUSE)
                            .rounding(egui::Rounding::same(8.0));
                    if ui.add_sized([120.0, 40.0], pause_button).clicked() {
                        self.pause_download();
                    }
                }

                ui.add_space(8.0);

                let cancel_button =
                    egui::Button::new(egui::RichText::new("キャンセル").color(TEXT_ON_PRIMARY))
                        .fill(BTN_CANCEL)
                        .rounding(egui::Rounding::same(8.0));
                if ui.add_sized([120.0, 40.0], cancel_button).clicked() {
                    self.cancel_download();
                }
            });
        });

        // ログカード
        card_frame().show(ui, |ui| {
            ui.add(egui::Label::new(
                egui::RichText::new("Download Log")
                    .strong()
                    .color(TEXT_PRIMARY),
            ));
            ui.add_space(4.0);
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for msg in &self.download_progress {
                        ui.add(egui::Label::new(
                            egui::RichText::new(msg).size(13.0).color(TEXT_SECONDARY),
                        ));
                    }
                });
        });

        ui.add_space(6.0);

        // PR007: 統計情報
        ui.add(egui::Label::new(
            egui::RichText::new(format!("Status: {}", self.status_message))
                .size(14.0)
                .color(TEXT_SECONDARY),
        ));
    }

    /// SC006: エラー表示画面をレンダリングする
    fn render_error(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::Label::new(
            egui::RichText::new("エラー")
                .size(26.0)
                .strong()
                .color(ERROR_TEXT),
        ));
        ui.add_space(8.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            // エラー種別自動判定
            let error_type = if self.error_message.contains("auth")
                || self.error_message.contains("401")
            {
                "認証エラー"
            } else if self.error_message.contains("network")
                || self.error_message.contains("timeout")
            {
                "ネットワークエラー"
            } else if self.error_message.contains("file") || self.error_message.contains("disk") {
                "ファイルエラー"
            } else {
                "一般エラー"
            };

            // エラー詳細カード（赤枠）
            error_card_frame().show(ui, |ui| {
                // エラー種別バッジ
                egui::Frame::none()
                    .fill(ERROR)
                    .rounding(egui::Rounding::same(12.0))
                    .inner_margin(egui::Margin::symmetric(10.0, 4.0))
                    .show(ui, |ui| {
                        ui.add(egui::Label::new(
                            egui::RichText::new(error_type)
                                .color(TEXT_ON_PRIMARY)
                                .size(13.0),
                        ));
                    });

                ui.add_space(8.0);

                // エラーメッセージ
                ui.add(egui::Label::new(
                    egui::RichText::new("エラーメッセージ")
                        .color(ERROR_TEXT)
                        .size(14.0),
                ));
                let error_width = ui.available_width();
                ui.add_sized(
                    [error_width, 40.0],
                    egui::TextEdit::multiline(&mut self.error_message.clone())
                        .desired_width(f32::INFINITY),
                );

                ui.add_space(6.0);

                // 詳細情報
                ui.add(egui::Label::new(
                    egui::RichText::new("詳細情報").color(ERROR_TEXT).size(14.0),
                ));
                ui.add_sized(
                    [error_width, 50.0],
                    egui::TextEdit::multiline(&mut self.error_details.clone())
                        .desired_width(f32::INFINITY),
                );
            });

            // 推奨アクション（通常カード）
            card_frame().show(ui, |ui| {
                ui.add(egui::Label::new(
                    egui::RichText::new("推奨アクション")
                        .strong()
                        .color(TEXT_PRIMARY),
                ));
                ui.add_space(4.0);
                match error_type {
                    "認証エラー" => {
                        ui.label("・設定画面でClient IDとClient Secretを確認してください");
                        ui.label("・Zoom Developer Appの設定を確認してください");
                    }
                    "ネットワークエラー" => {
                        ui.label("・インターネット接続を確認してください");
                        ui.label("・ファイアウォール設定を確認してください");
                    }
                    "ファイルエラー" => {
                        ui.label("・ディスク容量を確認してください");
                        ui.label("・出力ディレクトリの権限を確認してください");
                    }
                    _ => {
                        ui.label("・設定を確認してからリトライしてください");
                    }
                }
            });

            ui.add_space(8.0);

            // アクションボタン
            ui.horizontal(|ui| {
                let retry_btn =
                    egui::Button::new(egui::RichText::new("リトライ").color(TEXT_ON_PRIMARY))
                        .fill(PRIMARY)
                        .rounding(egui::Rounding::same(8.0));
                if ui.add_sized([120.0, 40.0], retry_btn).clicked() {
                    self.error_message.clear();
                    self.error_details.clear();
                    self.current_screen = AppScreen::Recordings;
                }

                let back_btn = egui::Button::new("設定に戻る")
                    .rounding(egui::Rounding::same(8.0))
                    .stroke(egui::Stroke::new(1.0, PRIMARY));
                if ui.add_sized([120.0, 40.0], back_btn).clicked() {
                    self.error_message.clear();
                    self.error_details.clear();
                    self.current_screen = AppScreen::Config;
                }

                let log_button =
                    egui::Button::new(egui::RichText::new("ログ出力").color(TEXT_ON_PRIMARY))
                        .fill(BTN_LOAD)
                        .rounding(egui::Rounding::same(8.0));
                if ui.add_sized([120.0, 40.0], log_button).clicked() {
                    match self.export_logs() {
                        Ok(filepath) => {
                            let _ = self.sender.send(AppMessage::LogExported(filepath));
                        }
                        Err(error_msg) => {
                            let _ = self.sender.send(AppMessage::Error(format!(
                                "Failed to export log: {}",
                                error_msg
                            )));
                        }
                    }
                }
            });
        });
    }

    /// 録画データを取得する（サービス経由）
    fn fetch_recordings(&mut self) {
        // 日付バリデーション
        let from_parsed = chrono::NaiveDate::parse_from_str(&self.from_date, "%Y-%m-%d");
        let to_parsed = chrono::NaiveDate::parse_from_str(&self.to_date, "%Y-%m-%d");

        match (from_parsed, to_parsed) {
            (Err(_), _) => {
                self.status_message = "エラー: 開始日の形式が不正です (YYYY-MM-DD)".to_string();
                return;
            }
            (_, Err(_)) => {
                self.status_message = "エラー: 終了日の形式が不正です (YYYY-MM-DD)".to_string();
                return;
            }
            (Ok(from), Ok(to)) if from > to => {
                self.status_message = "エラー: 開始日は終了日以前にしてください".to_string();
                return;
            }
            _ => {} // OK
        }

        if let Some(access_token) = &self.access_token {
            let access_token = access_token.clone();
            let from_date = self.from_date.clone();
            let to_date = self.to_date.clone();
            let sender = self.sender.clone();
            let recording_service = Arc::clone(&self.services.recording_service);

            thread::spawn(move || {
                match recording_service.get_recordings(&access_token, "me", &from_date, &to_date) {
                    Ok(recordings) => {
                        let _ = sender.send(AppMessage::RecordingsLoaded(recordings));
                    }
                    Err(e) => {
                        let _ = sender.send(AppMessage::Error(format!(
                            "Failed to fetch recordings: {}",
                            e
                        )));
                    }
                }
            });
        }
    }

    /// 設定ファイルを読み込み、GUI状態を更新する（サービス経由）
    fn load_config(&mut self) {
        match self.services.config_service.load_config("config.toml") {
            Ok(config) => {
                self.client_id = config.client_id;
                self.client_secret = config.client_secret;
                self.config_loaded = true;
                self.status_message = "Configuration loaded".to_string();
            }
            Err(_) => {
                let _ = self
                    .services
                    .config_service
                    .create_sample_config("config.toml");
                self.status_message =
                    "Configuration file not found. Created config.toml.".to_string();
            }
        }
    }

    /// 現在のGUI設定をファイルに保存する（サービス経由）
    fn save_config(&mut self) {
        let config = Config {
            client_id: self.client_id.clone(),
            client_secret: self.client_secret.clone(),
            redirect_uri: Some("http://localhost:8080/callback".to_string()),
        };

        match self
            .services
            .config_service
            .save_config(&config, "config.toml")
        {
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

    /// 認証開始（サービス経由）
    fn start_authentication(&mut self) {
        let client_id = self.client_id.clone();
        let client_secret = self.client_secret.clone();
        let sender = self.sender.clone();
        let auth_service = Arc::clone(&self.services.auth_service);

        thread::spawn(
            move || match auth_service.generate_auth_url(&client_id, &client_secret) {
                Ok(url) => {
                    let _ = sender.send(AppMessage::AuthUrlGenerated(url));
                }
                Err(e) => {
                    let _ = sender.send(AppMessage::Error(format!(
                        "Auth URL generation error: {}",
                        e
                    )));
                }
            },
        );
    }

    /// 認証完了（サービス経由）
    fn complete_authentication(&mut self) {
        let client_id = self.client_id.clone();
        let client_secret = self.client_secret.clone();
        let auth_code = self.auth_code.clone();
        let sender = self.sender.clone();
        let auth_service = Arc::clone(&self.services.auth_service);

        thread::spawn(move || {
            match auth_service.exchange_code_for_token(&client_id, &client_secret, &auth_code) {
                Ok(token) => {
                    let _ = sender.send(AppMessage::AuthComplete(token));
                }
                Err(e) => {
                    let _ =
                        sender.send(AppMessage::Error(format!("Token acquisition error: {}", e)));
                }
            }
        });
    }

    /// ダウンロード開始（サービス経由）
    fn start_download(&mut self) {
        if let (Some(access_token), Some(recordings)) = (&self.access_token, &self.recordings) {
            self.is_downloading = true;
            self.download_progress.clear();

            let access_token = access_token.clone();
            let recordings = recordings.clone();
            let output_dir = self.output_dir.clone();
            let selected: Vec<String> = self.selected_recordings.iter().cloned().collect();
            let sender = self.sender.clone();
            let download_service = Arc::clone(&self.services.download_service);

            thread::spawn(move || {
                match download_service.download_files(
                    &access_token,
                    &recordings,
                    &selected,
                    &output_dir,
                    sender.clone(),
                ) {
                    Ok(_) => {}
                    Err(e) => {
                        let _ = sender.send(AppMessage::Error(format!("Download error: {}", e)));
                    }
                }
            });
        }
    }
}

/// 認証URL生成の非同期実装（services_implから呼ばれる）
pub(crate) async fn generate_auth_url_async(
    client_id: &str,
    client_secret: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    use oauth2::{
        basic::BasicClient, AuthUrl, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
        RedirectUrl, Scope, TokenUrl,
    };

    let oauth_client = BasicClient::new(
        ClientId::new(client_id.to_string()),
        Some(ClientSecret::new(client_secret.to_string())),
        AuthUrl::new("https://zoom.us/oauth/authorize".to_string())?,
        Some(TokenUrl::new("https://zoom.us/oauth/token".to_string())?),
    )
    .set_redirect_uri(RedirectUrl::new(
        "http://localhost:8080/callback".to_string(),
    )?);

    let (pkce_challenge, _pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, _csrf_token) = oauth_client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("recording:read".to_string()))
        .add_scope(Scope::new("user:read".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    Ok(auth_url.to_string())
}

/// トークン交換の非同期実装（services_implから呼ばれる）
pub(crate) async fn exchange_code_for_token_async(
    client_id: &str,
    client_secret: &str,
    auth_code: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    use oauth2::{
        basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, RedirectUrl,
        TokenResponse, TokenUrl,
    };

    let oauth_client = BasicClient::new(
        ClientId::new(client_id.to_string()),
        Some(ClientSecret::new(client_secret.to_string())),
        AuthUrl::new("https://zoom.us/oauth/authorize".to_string())?,
        Some(TokenUrl::new("https://zoom.us/oauth/token".to_string())?),
    )
    .set_redirect_uri(RedirectUrl::new(
        "http://localhost:8080/callback".to_string(),
    )?);

    let (_, pkce_verifier) = oauth2::PkceCodeChallenge::new_random_sha256();

    let token_result = oauth_client
        .exchange_code(AuthorizationCode::new(auth_code.to_string()))
        .set_pkce_verifier(pkce_verifier)
        .request_async(oauth2::reqwest::async_http_client)
        .await?;

    Ok(token_result.access_token().secret().to_string())
}

// テスト用アクセサ・ファクトリ
#[cfg(any(test, feature = "test-support"))]
impl ZoomDownloaderApp {
    pub fn new_with_services(services: AppServices) -> Self {
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
            is_download_paused: false,
            download_can_resume: false,
            access_token: None,
            recordings: None,
            selected_recordings: std::collections::HashSet::new(),
            status_message: "Ready".to_string(),
            download_progress: Vec::new(),
            current_file: String::new(),
            progress_percentage: 0.0,
            error_message: String::new(),
            error_details: String::new(),
            log_entries: Vec::new(),
            receiver,
            sender,
            services,
        }
    }

    pub fn current_screen(&self) -> &AppScreen {
        &self.current_screen
    }

    pub fn sender(&self) -> &mpsc::Sender<AppMessage> {
        &self.sender
    }

    pub fn is_authenticating(&self) -> bool {
        self.is_authenticating
    }

    pub fn is_downloading(&self) -> bool {
        self.is_downloading
    }

    pub fn is_download_paused(&self) -> bool {
        self.is_download_paused
    }

    pub fn access_token(&self) -> &Option<String> {
        &self.access_token
    }

    pub fn config_loaded(&self) -> bool {
        self.config_loaded
    }

    pub fn error_message(&self) -> &str {
        &self.error_message
    }

    pub fn status_message(&self) -> &str {
        &self.status_message
    }

    pub fn recordings(&self) -> &Option<RecordingSearchResponse> {
        &self.recordings
    }

    pub fn download_progress_log(&self) -> &Vec<String> {
        &self.download_progress
    }

    pub fn process_messages_for_test(&mut self) {
        self.process_messages();
    }

    // 状態設定用
    pub fn set_config_loaded(&mut self, v: bool) {
        self.config_loaded = v;
    }

    pub fn set_access_token(&mut self, v: Option<String>) {
        self.access_token = v;
    }

    pub fn set_is_downloading(&mut self, v: bool) {
        self.is_downloading = v;
    }

    pub fn set_is_authenticating(&mut self, v: bool) {
        self.is_authenticating = v;
    }

    pub fn set_current_screen(&mut self, screen: AppScreen) {
        self.current_screen = screen;
    }

    pub fn set_error_message(&mut self, msg: String) {
        self.error_message = msg;
    }
}
