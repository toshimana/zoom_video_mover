# eGUI GUIポリシー - Zoom Video Mover

**技術要素**: egui 0.22+, eframe  
**適用範囲**: 全GUI実装（UI設計、状態管理、イベント処理）

## eGUI設計原則

### UI設計原則
- **即時モード**: 毎フレーム再描画による柔軟なUI
- **宣言的UI**: 状態に基づくUI記述
- **レスポンシブ**: ウィンドウサイズ変更への対応
- **アクセシブル**: キーボード操作・スクリーンリーダー対応
- **一貫性**: 統一されたルック&フィール

### パフォーマンス原則
- **効率的描画**: 不要な再描画の回避
- **メモリ効率**: UI状態の最適化
- **60FPS維持**: スムーズなユーザー体験
- **適応的品質**: デバイス性能に応じた最適化

## アプリケーション構造

### メインアプリケーション設計
```rust
use eframe::egui;

/// メインアプリケーション構造体
/// 
/// # 不変条件
/// - current_tab は常に有効なタブ
/// - auth_state と ui_state の整合性が保たれる
/// - download_state は null でない
#[derive(Debug)]
pub struct ZoomDownloaderApp {
    // UI状態
    current_tab: TabType,
    window_size: egui::Vec2,
    
    // アプリケーション状態
    auth_state: AuthenticationState,
    download_state: DownloadState,
    config_state: ConfigurationState,
    
    // 非同期通信
    runtime_handle: tokio::runtime::Handle,
    event_receiver: Option<mpsc::Receiver<AppEvent>>,
    command_sender: mpsc::Sender<AppCommand>,
    
    // UI状態
    show_error_dialog: bool,
    error_message: Option<String>,
    progress_display: ProgressDisplay,
}

impl ZoomDownloaderApp {
    /// 新しいアプリケーションインスタンス作成
    /// 
    /// # 事前条件
    /// - runtime_handle が有効
    /// 
    /// # 事後条件
    /// - 全状態が初期化される
    /// - 非同期通信チャンネルが設定される
    pub fn new(cc: &eframe::CreationContext<'_>, runtime_handle: tokio::runtime::Handle) -> Self {
        // UI設定
        configure_ui_style(&cc.egui_ctx);
        
        // 非同期通信セットアップ
        let (command_sender, command_receiver) = mpsc::channel(100);
        let (event_sender, event_receiver) = mpsc::channel(100);
        
        // バックグラウンドタスク起動
        runtime_handle.spawn(background_task_handler(command_receiver, event_sender));
        
        Self {
            current_tab: TabType::Config,
            window_size: egui::Vec2::new(800.0, 600.0),
            auth_state: AuthenticationState::Unauthenticated,
            download_state: DownloadState::new(),
            config_state: ConfigurationState::load_or_default(),
            runtime_handle,
            event_receiver: Some(event_receiver),
            command_sender,
            show_error_dialog: false,
            error_message: None,
            progress_display: ProgressDisplay::new(),
        }
    }
}

impl eframe::App for ZoomDownloaderApp {
    /// フレーム更新処理
    /// 
    /// # 副作用
    /// - UI の再描画
    /// - 非同期イベントの処理
    /// - 状態の更新
    /// 
    /// # 不変条件
    /// - フレーム処理中に状態の整合性が保たれる
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // 非同期イベント処理
        self.process_async_events();
        
        // レスポンシブ対応
        self.update_window_size(ctx);
        
        // メインUI描画
        self.render_main_ui(ctx, frame);
        
        // エラーダイアログ
        self.render_error_dialog(ctx);
        
        // 次フレームリクエスト（アニメーションや進捗表示時）
        if self.should_request_repaint() {
            ctx.request_repaint();
        }
    }
}
```

### UI状態管理
```rust
/// UI状態の中央管理
#[derive(Debug, Clone)]
pub struct UiState {
    // タブ状態
    pub current_tab: TabType,
    pub tab_history: Vec<TabType>,
    
    // 表示状態
    pub show_progress: bool,
    pub show_settings: bool,
    pub show_about: bool,
    
    // 入力状態
    pub search_query: String,
    pub date_range: DateRange,
    pub selected_recordings: HashSet<RecordingId>,
    
    // 一時的な状態
    pub last_error: Option<AppError>,
    pub notification_queue: VecDeque<Notification>,
}

impl UiState {
    /// 状態遷移の検証
    /// 
    /// # 事前条件
    /// - new_tab が有効なタブ種別
    /// 
    /// # 事後条件
    /// - 状態遷移が完了
    /// - 履歴が更新される
    /// 
    /// # 不変条件
    /// - タブ履歴の一貫性が保たれる
    pub fn transition_to_tab(&mut self, new_tab: TabType) -> Result<(), UiError> {
        // 遷移可能性チェック
        if !self.can_transition_to(&new_tab) {
            return Err(UiError::InvalidTransition {
                from: self.current_tab,
                to: new_tab,
            });
        }
        
        // 履歴更新
        if self.current_tab != new_tab {
            self.tab_history.push(self.current_tab);
            if self.tab_history.len() > 10 {
                self.tab_history.remove(0);
            }
        }
        
        self.current_tab = new_tab;
        Ok(())
    }
    
    /// タブ遷移可能性チェック
    fn can_transition_to(&self, target: &TabType) -> bool {
        match (self.current_tab, target) {
            // 認証が必要なタブ
            (_, TabType::Recordings) if !self.is_authenticated() => false,
            (_, TabType::Downloads) if !self.is_authenticated() => false,
            _ => true,
        }
    }
}
```

## コンポーネント設計

### 再利用可能コンポーネント
```rust
/// 進捗表示コンポーネント
/// 
/// # 事前条件
/// - progress は 0.0 から 1.0 の範囲
/// - ui が有効なコンテキスト
/// 
/// # 事後条件
/// - 進捗バーが描画される
/// - テキスト情報が表示される
pub fn render_progress_bar(
    ui: &mut egui::Ui,
    progress: f32,
    message: &str,
    estimated_time: Option<Duration>,
) -> egui::Response {
    debug_assert!((0.0..=1.0).contains(&progress), "progress must be between 0.0 and 1.0");
    
    ui.vertical(|ui| {
        // 進捗バー
        let progress_bar = egui::ProgressBar::new(progress)
            .desired_width(ui.available_width() - 20.0)
            .text(format!("{:.1}%", progress * 100.0));
        
        ui.add(progress_bar);
        
        // メッセージ表示
        ui.horizontal(|ui| {
            ui.label(message);
            
            if let Some(eta) = estimated_time {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("残り: {}", format_duration(eta)));
                });
            }
        });
    }).response
}

/// ファイル選択コンポーネント
/// 
/// # 副作用
/// - ファイルダイアログの表示
/// - ファイルパスの更新
pub fn render_file_selector(
    ui: &mut egui::Ui,
    label: &str,
    current_path: &mut String,
    file_filter: FileFilter,
) -> egui::Response {
    ui.horizontal(|ui| {
        ui.label(label);
        
        // パス表示・編集
        let text_edit = egui::TextEdit::singleline(current_path)
            .desired_width(ui.available_width() - 80.0);
        ui.add(text_edit);
        
        // 参照ボタン
        if ui.button("参照...").clicked() {
            if let Some(path) = open_file_dialog(file_filter) {
                *current_path = path.to_string_lossy().to_string();
            }
        }
    }).response
}

/// データテーブルコンポーネント
pub fn render_recording_table(
    ui: &mut egui::Ui,
    recordings: &[Recording],
    selected: &mut HashSet<RecordingId>,
) -> egui::Response {
    use egui_extras::{Column, TableBuilder};
    
    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::auto().resizable(false)) // チェックボックス
        .column(Column::remainder().at_least(200.0)) // トピック
        .column(Column::auto().at_least(100.0)) // 日時
        .column(Column::auto().at_least(80.0)) // 長さ
        .column(Column::auto().at_least(100.0)) // サイズ
        .header(20.0, |mut header| {
            header.col(|ui| { ui.label(""); });
            header.col(|ui| { ui.label("トピック"); });
            header.col(|ui| { ui.label("日時"); });
            header.col(|ui| { ui.label("長さ"); });
            header.col(|ui| { ui.label("サイズ"); });
        })
        .body(|mut body| {
            for recording in recordings {
                body.row(20.0, |mut row| {
                    // チェックボックス
                    row.col(|ui| {
                        let mut is_selected = selected.contains(&recording.id);
                        ui.checkbox(&mut is_selected, "");
                        
                        if is_selected {
                            selected.insert(recording.id.clone());
                        } else {
                            selected.remove(&recording.id);
                        }
                    });
                    
                    // データ列
                    row.col(|ui| { ui.label(&recording.topic); });
                    row.col(|ui| { ui.label(format_date(&recording.start_time)); });
                    row.col(|ui| { ui.label(format_duration(recording.duration)); });
                    row.col(|ui| { ui.label(format_file_size(recording.total_size)); });
                });
            }
        })
        .response
}
```

### レイアウト管理
```rust
/// レスポンシブレイアウト管理
pub struct ResponsiveLayout {
    breakpoints: LayoutBreakpoints,
    current_size: egui::Vec2,
}

impl ResponsiveLayout {
    /// ウィンドウサイズに基づくレイアウト決定
    /// 
    /// # 事前条件
    /// - size の width, height > 0
    /// 
    /// # 事後条件
    /// - 適切なレイアウトタイプを返す
    pub fn determine_layout(&self, size: egui::Vec2) -> LayoutType {
        assert!(size.x > 0.0 && size.y > 0.0, "size must be positive");
        
        if size.x < self.breakpoints.mobile {
            LayoutType::Mobile
        } else if size.x < self.breakpoints.tablet {
            LayoutType::Tablet
        } else {
            LayoutType::Desktop
        }
    }
    
    /// レイアウトに応じた列数決定
    pub fn column_count(&self, layout: LayoutType) -> usize {
        match layout {
            LayoutType::Mobile => 1,
            LayoutType::Tablet => 2,
            LayoutType::Desktop => 3,
        }
    }
}

/// メインレイアウト描画
pub fn render_main_layout(
    ctx: &egui::Context,
    app: &mut ZoomDownloaderApp,
) {
    let layout_type = app.responsive_layout.determine_layout(app.window_size);
    
    match layout_type {
        LayoutType::Mobile => render_mobile_layout(ctx, app),
        LayoutType::Tablet | LayoutType::Desktop => render_desktop_layout(ctx, app),
    }
}

/// デスクトップ版レイアウト
fn render_desktop_layout(ctx: &egui::Context, app: &mut ZoomDownloaderApp) {
    // トップパネル
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        render_top_menu_bar(ui, app);
    });
    
    // サイドパネル
    egui::SidePanel::left("side_panel")
        .default_width(200.0)
        .resizable(true)
        .show(ctx, |ui| {
            render_navigation_panel(ui, app);
        });
    
    // メインコンテンツ
    egui::CentralPanel::default().show(ctx, |ui| {
        render_main_content(ui, app);
    });
    
    // ステータスバー
    egui::TopBottomPanel::bottom("status_panel").show(ctx, |ui| {
        render_status_bar(ui, app);
    });
}
```

## イベント処理・状態管理

### 非同期イベント統合
```rust
/// アプリケーションコマンド
#[derive(Debug, Clone)]
pub enum AppCommand {
    // 認証関連
    StartAuthentication { client_id: String, client_secret: String },
    RefreshToken,
    Logout,
    
    // 録画関連
    SearchRecordings { date_range: DateRange, query: Option<String> },
    LoadRecordingDetails { recording_id: RecordingId },
    
    // ダウンロード関連
    StartDownload { recording_ids: Vec<RecordingId> },
    PauseDownload { session_id: SessionId },
    CancelDownload { session_id: SessionId },
    
    // 設定関連
    SaveConfiguration { config: AppConfig },
    LoadConfiguration,
}

/// アプリケーションイベント
#[derive(Debug, Clone)]
pub enum AppEvent {
    // 認証イベント
    AuthenticationCompleted { user_info: UserInfo },
    AuthenticationFailed { error: AuthError },
    TokenRefreshed { expires_at: DateTime<Utc> },
    
    // データイベント
    RecordingsLoaded { recordings: Vec<Recording> },
    RecordingDetailsLoaded { recording_id: RecordingId, details: RecordingDetails },
    
    // ダウンロードイベント
    DownloadStarted { session_id: SessionId },
    DownloadProgress { session_id: SessionId, progress: DownloadProgress },
    DownloadCompleted { session_id: SessionId, results: DownloadResults },
    DownloadFailed { session_id: SessionId, error: DownloadError },
    
    // エラーイベント
    ErrorOccurred { error: AppError, context: String },
}

impl ZoomDownloaderApp {
    /// 非同期イベント処理
    /// 
    /// # 副作用
    /// - アプリケーション状態の更新
    /// - UI状態の変更
    /// - エラー表示の更新
    /// 
    /// # 不変条件
    /// - イベント処理中の状態整合性
    fn process_async_events(&mut self) {
        if let Some(receiver) = &mut self.event_receiver {
            while let Ok(event) = receiver.try_recv() {
                match event {
                    AppEvent::AuthenticationCompleted { user_info } => {
                        self.auth_state = AuthenticationState::Authenticated { user_info };
                        self.current_tab = TabType::Recordings;
                        self.clear_error();
                    }
                    
                    AppEvent::AuthenticationFailed { error } => {
                        self.auth_state = AuthenticationState::Failed { error: error.clone() };
                        self.show_error(format!("認証に失敗しました: {}", error));
                    }
                    
                    AppEvent::RecordingsLoaded { recordings } => {
                        self.download_state.available_recordings = recordings;
                    }
                    
                    AppEvent::DownloadProgress { session_id, progress } => {
                        self.progress_display.update_progress(session_id, progress);
                    }
                    
                    AppEvent::DownloadCompleted { session_id, results } => {
                        self.progress_display.complete_session(session_id);
                        self.show_notification(format!("ダウンロード完了: {}ファイル", results.successful_count));
                    }
                    
                    AppEvent::ErrorOccurred { error, context } => {
                        self.show_error(format!("{}: {}", context, error));
                    }
                    
                    _ => {} // その他のイベント処理
                }
            }
        }
    }
    
    /// コマンド送信（非同期処理への橋渡し）
    fn send_command(&self, command: AppCommand) {
        if let Err(e) = self.command_sender.try_send(command) {
            log::error!("Failed to send command: {}", e);
        }
    }
}
```

## スタイリング・テーマ

### UI設定・カスタマイゼーション
```rust
/// UI スタイル設定
/// 
/// # 事前条件
/// - ctx が有効なコンテキスト
/// 
/// # 事後条件
/// - アプリケーション固有スタイルが適用
pub fn configure_ui_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    
    // フォント設定
    configure_fonts(ctx);
    
    // カラー設定
    style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(248, 249, 250);
    style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(233, 236, 239);
    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(222, 226, 230);
    style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(13, 110, 253);
    
    // スペーシング設定
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(12.0, 8.0);
    style.spacing.window_margin = egui::Margin::same(12.0);
    
    // アニメーション設定
    style.animation_time = 0.2;
    
    ctx.set_style(style);
}

/// フォント設定
fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    
    // 日本語フォント追加
    fonts.font_data.insert(
        "NotoSansJP".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/NotoSansJP-Regular.ttf")),
    );
    
    // フォントファミリー設定
    fonts.families.entry(egui::FontFamily::Proportional).or_default()
        .insert(0, "NotoSansJP".to_owned());
    
    fonts.families.entry(egui::FontFamily::Monospace).or_default()
        .insert(0, "NotoSansJP".to_owned());
    
    ctx.set_fonts(fonts);
}

/// ダークモード対応
pub fn apply_theme(ctx: &egui::Context, theme: Theme) {
    let mut visuals = match theme {
        Theme::Light => egui::Visuals::light(),
        Theme::Dark => egui::Visuals::dark(),
        Theme::Auto => {
            if is_system_dark_mode() {
                egui::Visuals::dark()
            } else {
                egui::Visuals::light()
            }
        }
    };
    
    // カスタムカラー適用
    customize_theme_colors(&mut visuals, theme);
    
    ctx.set_visuals(visuals);
}
```

## アクセシビリティ

### キーボード・スクリーンリーダー対応
```rust
/// アクセシビリティ対応
impl ZoomDownloaderApp {
    /// キーボードショートカット処理
    /// 
    /// # 事前条件
    /// - ctx が有効なコンテキスト
    /// 
    /// # 事後条件
    /// - ショートカットに応じた動作実行
    fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::N)) {
            // Ctrl+N: 新しい検索
            self.current_tab = TabType::Recordings;
            self.download_state.search_query.clear();
        }
        
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::D)) {
            // Ctrl+D: ダウンロード開始
            if !self.download_state.selected_recordings.is_empty() {
                self.start_selected_downloads();
            }
        }
        
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Comma)) {
            // Ctrl+,: 設定画面
            self.current_tab = TabType::Config;
        }
        
        if ctx.input_mut(|i| i.consume_key(egui::Key::Escape)) {
            // ESC: モーダル閉じる
            self.close_modals();
        }
    }
    
    /// スクリーンリーダー対応ラベル設定
    fn set_accessibility_labels(&self, ui: &mut egui::Ui) {
        // アクセシビリティ情報の追加
        ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover())
            .on_hover_text("メインアプリケーション画面");
    }
}

/// フォーカス管理
pub struct FocusManager {
    focus_chain: Vec<egui::Id>,
    current_focus: Option<egui::Id>,
}

impl FocusManager {
    /// フォーカスチェーン設定
    pub fn set_focus_chain(&mut self, chain: Vec<egui::Id>) {
        self.focus_chain = chain;
    }
    
    /// 次の要素にフォーカス移動
    pub fn focus_next(&mut self, ctx: &egui::Context) {
        if let Some(current) = self.current_focus {
            if let Some(index) = self.focus_chain.iter().position(|&id| id == current) {
                let next_index = (index + 1) % self.focus_chain.len();
                self.current_focus = Some(self.focus_chain[next_index]);
                ctx.memory_mut(|mem| mem.request_focus(self.focus_chain[next_index]));
            }
        }
    }
}
```

## パフォーマンス最適化

### 効率的な描画
```rust
/// 効率的なUI更新
impl ZoomDownloaderApp {
    /// 条件付き再描画
    /// 
    /// # 事後条件
    /// - 必要な場合のみ再描画リクエスト
    fn should_request_repaint(&self) -> bool {
        // アニメーション中
        if self.progress_display.has_active_animations() {
            return true;
        }
        
        // ダウンロード進行中
        if self.download_state.has_active_downloads() {
            return true;
        }
        
        // 通知表示中
        if !self.ui_state.notification_queue.is_empty() {
            return true;
        }
        
        false
    }
    
    /// 大量データの仮想化表示
    fn render_virtualized_list(
        &self,
        ui: &mut egui::Ui,
        items: &[Recording],
        item_height: f32,
    ) {
        let visible_range = calculate_visible_range(
            ui.available_rect_before_wrap(),
            item_height,
            items.len(),
        );
        
        // 表示範囲のアイテムのみ描画
        for i in visible_range {
            if let Some(item) = items.get(i) {
                render_recording_item(ui, item, i);
            }
        }
    }
}

/// メモリ効率的な状態管理
#[derive(Debug)]
pub struct OptimizedState {
    // 大量データのキャッシュ
    recording_cache: LruCache<RecordingId, Recording>,
    // UI状態の最小化
    dirty_flags: DirtyFlags,
}

impl OptimizedState {
    /// 変更検出による効率的更新
    pub fn mark_dirty(&mut self, flag: DirtyFlag) {
        self.dirty_flags.set(flag);
    }
    
    /// 必要な部分のみ更新
    pub fn update_if_dirty(&mut self, flag: DirtyFlag, update_fn: impl FnOnce()) {
        if self.dirty_flags.is_set(flag) {
            update_fn();
            self.dirty_flags.clear(flag);
        }
    }
}
```

## エラー処理・デバッグ

### UIエラー表示
```rust
/// エラー表示統合
impl ZoomDownloaderApp {
    /// ユーザーフレンドリーなエラー表示
    /// 
    /// # 副作用
    /// - エラーダイアログの表示
    /// - ログ記録
    /// 
    /// # 事前条件
    /// - error_message が空でない
    pub fn show_error(&mut self, error_message: String) {
        assert!(!error_message.is_empty(), "error_message must not be empty");
        
        log::error!("UI Error: {}", error_message);
        
        self.error_message = Some(error_message);
        self.show_error_dialog = true;
    }
    
    /// エラーダイアログ描画
    fn render_error_dialog(&mut self, ctx: &egui::Context) {
        if self.show_error_dialog {
            egui::Window::new("エラー")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    if let Some(ref message) = self.error_message {
                        ui.label(message);
                        ui.separator();
                        
                        ui.horizontal(|ui| {
                            if ui.button("OK").clicked() {
                                self.close_error_dialog();
                            }
                            
                            if ui.button("詳細").clicked() {
                                self.show_error_details();
                            }
                        });
                    }
                });
        }
    }
}
```

## 品質目標

- **フレームレート**: 60FPS維持
- **メモリ使用量**: 100MB以内（GUI部分）
- **起動時間**: 3秒以内
- **操作応答性**: 16ms以内
- **アクセシビリティ**: WCAG 2.1 AA準拠
- **多言語対応**: 日本語・英語完全サポート

直感的で高性能なGUIにより、優れたユーザー体験を提供します。