# UI制御コンポーネント詳細設計書 - Zoom Video Mover

## 文書概要
**文書ID**: DES-GUI-001  
**コンポーネント名**: UI制御コンポーネント（UI Control Component）  
**作成日**: 2025-08-03  
  
**バージョン**: 1.0  

## コンポーネント概要

### 責任・役割
- **統合GUI制御**: egui/eframeベースのウィンドウ・レイアウト・イベント管理
- **状態管理**: アプリケーション全体のUI状態と全コンポーネント間の状態同期
- **進捗表示**: リアルタイム進捗バー・通知・ステータス表示システム
- **Windows最適化**: 日本語文字化け対策・DPI対応・アクセシビリティ機能

### アーキテクチャ位置
```
┌─────────────────────────────────────────────────────────────────┐
│                   Presentation Layer                            │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │              UI Control Component                            │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │ │
│  │  │   Window    │  │   State     │  │     Event           │ │ │
│  │  │  Manager    │  │  Manager    │  │     Handler         │ │ │
│  │  │             │  │             │  │                     │ │ │
│  │  └─────────────┘  └─────────────┘  └─────────────────────┘ │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │ │
│  │  │  Notification│  │  Progress   │  │     Layout          │ │ │
│  │  │   System    │  │  Monitor    │  │     Engine          │ │ │
│  │  │             │  │             │  │                     │ │ │
│  │  └─────────────┘  └─────────────┘  └─────────────────────┘ │ │
│  └─────────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                 Infrastructure Layer                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │   egui      │  │  Windows    │  │    Font                 │  │
│  │  Renderer   │  │    API      │  │   Manager               │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## モジュール構造設計

### 内部モジュール構成
```rust
pub mod ui {
    /// メインアプリケーション
    pub mod main_app;
    
    /// ウィンドウ管理
    pub mod window_manager;
    
    /// 状態管理
    pub mod state_manager;
    
    /// イベントハンドラー
    pub mod event_handler;
    
    /// 画面・ビュー
    pub mod views {
        pub mod main_view;
        pub mod settings_view;
        pub mod progress_view;
        pub mod file_selection_view;
        pub mod error_dialog;
    }
    
    /// ウィジェット
    pub mod widgets {
        pub mod progress_bar;
        pub mod file_tree;
        pub mod filter_panel;
        pub mod status_bar;
        pub mod notification_toast;
    }
    
    /// 通知システム
    pub mod notification_system;
    
    /// 進捗監視
    pub mod progress_monitor;
    
    /// レイアウトエンジン
    pub mod layout_engine;
    
    /// テーマ・スタイル
    pub mod styling;
    
    /// 国際化
    pub mod i18n;
    
    /// エラー定義
    pub mod error;
    
    /// 設定・定数
    pub mod config;
}
```

### モジュール依存関係
```
main_app
    ├── → window_manager
    ├── → state_manager
    ├── → event_handler
    ├── → views::*
    └── → error

window_manager
    ├── → layout_engine
    ├── → styling
    └── → error

state_manager
    ├── → notification_system
    └── → error

event_handler
    ├── → state_manager
    └── → error

views::*
    ├── → widgets::*
    ├── → state_manager
    ├── → styling
    └── → error

widgets::*
    ├── → styling
    ├── → i18n
    └── → error

notification_system
    └── → error

progress_monitor
    ├── → notification_system
    └── → error

layout_engine
    └── → styling

styling
    └── → error

i18n
    └── → error
```

## データ構造設計

### コアデータ構造

#### 1. アプリケーション状態
```rust
/// アプリケーション全体の状態
#[derive(Debug, Clone)]
pub struct AppState {
    /// 現在のビュー
    pub current_view: AppView,
    
    /// 認証状態
    pub auth_state: AuthenticationState,
    
    /// 録画データ状態
    pub recording_state: RecordingDataState,
    
    /// ダウンロード状態
    pub download_state: DownloadState,
    
    /// 設定状態
    pub config_state: ConfigurationState,
    
    /// UI状態
    pub ui_state: UiState,
    
    /// エラー状態
    pub error_state: ErrorState,
    
    /// 通知状態
    pub notification_state: NotificationState,
    
    /// 最終更新時刻
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// アプリケーションビュー
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppView {
    /// 初期化画面
    Loading,
    
    /// 設定画面
    Settings,
    
    /// メイン操作画面
    Main {
        selected_tab: MainTabType,
    },
    
    /// 進捗表示画面
    Progress {
        download_session_id: String,
    },
    
    /// エラー表示画面
    Error {
        error_id: String,
        recoverable: bool,
    },
}

/// メインタブ種別
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MainTabType {
    /// 録画検索タブ
    Search,
    
    /// ファイル選択タブ
    FileSelection,
    
    /// ダウンロード履歴タブ
    History,
    
    /// 設定タブ
    Settings,
    
    /// ヘルプタブ
    Help,
}

/// UI状態
#[derive(Debug, Clone)]
pub struct UiState {
    /// ウィンドウ状態
    pub window: WindowState,
    
    /// テーマ設定
    pub theme: ThemeState,
    
    /// フォント設定
    pub font: FontState,
    
    /// レイアウト状態
    pub layout: LayoutState,
    
    /// 入力状態
    pub input: InputState,
    
    /// アニメーション状態
    pub animations: AnimationState,
    
    /// アクセシビリティ状態
    pub accessibility: AccessibilityState,
}

/// ウィンドウ状態
#[derive(Debug, Clone)]
pub struct WindowState {
    /// ウィンドウサイズ
    pub size: (f32, f32),
    
    /// ウィンドウ位置
    pub position: Option<(f32, f32)>,
    
    /// 最大化状態
    pub maximized: bool,
    
    /// 最小化状態
    pub minimized: bool,
    
    /// フルスクリーン状態
    pub fullscreen: bool,
    
    /// DPI スケール
    pub dpi_scale: f32,
    
    /// ウィンドウタイトル
    pub title: String,
    
    /// ウィンドウアイコン
    pub icon: Option<IconData>,
}
```

#### 2. UI イベント・メッセージ
```rust
/// UIイベント（ユーザー操作）
#[derive(Debug, Clone)]
pub enum UiEvent {
    /// ウィンドウイベント
    Window(WindowEvent),
    
    /// ユーザー入力イベント
    Input(InputEvent),
    
    /// ナビゲーションイベント
    Navigation(NavigationEvent),
    
    /// ファイル選択イベント
    FileSelection(FileSelectionEvent),
    
    /// 設定変更イベント
    Configuration(ConfigurationEvent),
    
    /// システムイベント
    System(SystemEvent),
}

/// ナビゲーションイベント
#[derive(Debug, Clone)]
pub enum NavigationEvent {
    /// タブ切り替え
    TabChanged {
        from_tab: MainTabType,
        to_tab: MainTabType,
    },
    
    /// ビュー切り替え
    ViewChanged {
        from_view: AppView,
        to_view: AppView,
    },
    
    /// 戻る操作
    GoBack,
    
    /// 進む操作
    GoForward,
    
    /// ホームに戻る
    GoHome,
}

/// ファイル選択イベント
#[derive(Debug, Clone)]
pub enum FileSelectionEvent {
    /// ファイル選択変更
    SelectionChanged {
        selected_files: Vec<String>,
        total_size: u64,
    },
    
    /// 全選択
    SelectAll,
    
    /// 全選択解除
    DeselectAll,
    
    /// 選択反転
    InvertSelection,
    
    /// フィルタ適用
    FilterApplied {
        filter_criteria: FilterCriteria,
        result_count: usize,
    },
    
    /// ソート変更
    SortChanged {
        sort_field: SortField,
        sort_order: SortOrder,
    },
}

/// UI アクション（内部処理要求）
#[derive(Debug, Clone)]
pub enum UiAction {
    /// 認証開始
    StartAuthentication,
    
    /// 録画検索開始
    StartRecordingSearch {
        criteria: FilterCriteria,
    },
    
    /// ダウンロード開始
    StartDownload {
        selected_files: Vec<String>,
        output_directory: PathBuf,
    },
    
    /// ダウンロードキャンセル
    CancelDownload {
        download_id: String,
    },
    
    /// 設定保存
    SaveConfiguration {
        config: AppConfig,
    },
    
    /// エラー復旧試行
    RetryOperation {
        operation_id: String,
    },
    
    /// 通知クリア
    ClearNotifications,
    
    /// 終了
    Quit,
}
```

#### 3. 進捗・通知データ
```rust
/// 進捗表示データ
#[derive(Debug, Clone)]
pub struct ProgressDisplayData {
    /// 進捗ID
    pub progress_id: String,
    
    /// 進捗タイプ
    pub progress_type: ProgressType,
    
    /// 全体進捗
    pub overall_progress: OverallProgress,
    
    /// ファイル別進捗
    pub file_progress: HashMap<String, FileProgress>,
    
    /// 現在のアクション
    pub current_action: String,
    
    /// 推定残り時間
    pub estimated_time_remaining: Option<Duration>,
    
    /// エラー情報
    pub errors: Vec<ProgressError>,
    
    /// 開始時刻
    pub started_at: chrono::DateTime<chrono::Utc>,
    
    /// 最終更新時刻
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// 進捗タイプ
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProgressType {
    /// 認証進捗
    Authentication,
    
    /// 録画検索進捗
    RecordingSearch,
    
    /// ダウンロード進捗
    Download,
    
    /// 設定保存進捗
    ConfigurationSave,
    
    /// 一般的な処理進捗
    General {
        description: String,
    },
}

/// 通知データ
#[derive(Debug, Clone)]
pub struct NotificationData {
    /// 通知ID
    pub id: String,
    
    /// 通知タイプ
    pub notification_type: NotificationType,
    
    /// タイトル
    pub title: String,
    
    /// メッセージ
    pub message: String,
    
    /// 詳細情報
    pub details: Option<String>,
    
    /// 重要度
    pub severity: NotificationSeverity,
    
    /// アクション
    pub actions: Vec<NotificationAction>,
    
    /// 自動削除時間
    pub auto_dismiss_after: Option<Duration>,
    
    /// 作成時刻
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// 表示状態
    pub display_state: NotificationDisplayState,
}

/// 通知タイプ
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotificationType {
    /// 情報通知
    Info,
    
    /// 成功通知
    Success,
    
    /// 警告通知
    Warning,
    
    /// エラー通知
    Error,
    
    /// 進捗通知
    Progress,
    
    /// システム通知
    System,
}

/// 通知重要度
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum NotificationSeverity {
    /// 低重要度
    Low,
    
    /// 中重要度
    Medium,
    
    /// 高重要度
    High,
    
    /// 緊急
    Critical,
}

/// 通知アクション
#[derive(Debug, Clone)]
pub struct NotificationAction {
    /// アクションID
    pub id: String,
    
    /// アクションラベル
    pub label: String,
    
    /// アクション種別
    pub action_type: NotificationActionType,
    
    /// プライマリアクションフラグ
    pub is_primary: bool,
}

/// 通知アクション種別
#[derive(Debug, Clone)]
pub enum NotificationActionType {
    /// 単純な確認
    Acknowledge,
    
    /// リトライ
    Retry {
        operation_id: String,
    },
    
    /// キャンセル
    Cancel {
        operation_id: String,
    },
    
    /// 詳細表示
    ShowDetails {
        details_url: Option<String>,
    },
    
    /// 設定開く
    OpenSettings {
        settings_tab: Option<String>,
    },
    
    /// カスタムアクション
    Custom {
        action_name: String,
        parameters: HashMap<String, serde_json::Value>,
    },
}
```

## インターフェース設計

### 公開API

#### 1. UI制御マネージャー
```rust
/// UI制御マネージャー - コンポーネントのメインインターフェース
#[async_trait]
pub trait UiController: Send + Sync {
    /// アプリケーション実行
    async fn run_application(&self) -> Result<(), UiError>;
    
    /// UI状態更新
    async fn update_state(&self, new_state: AppState) -> Result<(), UiError>;
    
    /// イベント送信
    async fn send_event(&self, event: UiEvent) -> Result<(), UiError>;
    
    /// アクション実行
    async fn execute_action(&self, action: UiAction) -> Result<(), UiError>;
    
    /// 通知表示
    async fn show_notification(&self, notification: NotificationData) -> Result<(), UiError>;
    
    /// 進捗更新
    async fn update_progress(&self, progress: ProgressDisplayData) -> Result<(), UiError>;
    
    /// ダイアログ表示
    async fn show_dialog(&self, dialog: DialogData) -> Result<DialogResult, UiError>;
    
    /// ビュー切り替え
    async fn navigate_to_view(&self, view: AppView) -> Result<(), UiError>;
    
    /// UI イベント監視
    fn subscribe_events(&self) -> broadcast::Receiver<UiEvent>;
    
    /// アプリケーション終了
    async fn shutdown(&self) -> Result<(), UiError>;
}
```

#### 2. 実装クラス
```rust
/// egui ベース UI制御マネージャー実装
pub struct EguiUiController {
    /// メインアプリケーション
    app: Arc<Mutex<ZoomVideoMoverApp>>,
    
    /// 状態マネージャー
    state_manager: Arc<UiStateManager>,
    
    /// イベントハンドラー
    event_handler: Arc<UiEventHandler>,
    
    /// 通知システム
    notification_system: Arc<NotificationSystem>,
    
    /// 進捗監視システム
    progress_monitor: Arc<UiProgressMonitor>,
    
    /// ウィンドウマネージャー
    window_manager: Arc<WindowManager>,
    
    /// テーママネージャー
    theme_manager: Arc<ThemeManager>,
    
    /// 設定
    ui_config: UiConfiguration,
    
    /// イベント通知チャンネル
    event_tx: broadcast::Sender<UiEvent>,
    
    /// アプリケーション実行フラグ
    running: Arc<AtomicBool>,
}

impl EguiUiController {
    /// 新しいUI制御マネージャーを作成
    pub fn new(ui_config: UiConfiguration) -> Result<Self, UiError> {
        let app = Arc::new(Mutex::new(ZoomVideoMoverApp::new(&ui_config)?));
        let state_manager = Arc::new(UiStateManager::new());
        let event_handler = Arc::new(UiEventHandler::new());
        let notification_system = Arc::new(NotificationSystem::new(&ui_config)?);
        let progress_monitor = Arc::new(UiProgressMonitor::new());
        let window_manager = Arc::new(WindowManager::new(&ui_config)?);
        let theme_manager = Arc::new(ThemeManager::new(&ui_config)?);
        let (event_tx, _) = broadcast::channel(1000);
        let running = Arc::new(AtomicBool::new(false));
        
        Ok(Self {
            app,
            state_manager,
            event_handler,
            notification_system,
            progress_monitor,
            window_manager,
            theme_manager,
            ui_config,
            event_tx,
            running,
        })
    }
}

#[async_trait]
impl UiController for EguiUiController {
    async fn run_application(&self) -> Result<(), UiError> {
        // 1. アプリケーション実行開始
        self.running.store(true, Ordering::SeqCst);
        
        // 2. ネイティブオプション設定
        let native_options = self.create_native_options()?;
        
        // 3. eframe アプリケーション実行
        let app_clone = self.app.clone();
        let event_tx_clone = self.event_tx.clone();
        
        eframe::run_native(
            &self.ui_config.window.title,
            native_options,
            Box::new(move |cc| {
                // egui コンテキスト設定
                Self::setup_egui_context(&cc.egui_ctx);
                
                // アプリケーションインスタンス取得
                let mut app = app_clone.lock().unwrap();
                app.set_creation_context(cc);
                app.set_event_sender(event_tx_clone);
                
                Box::new(app.clone())
            }),
        );
        
        self.running.store(false, Ordering::SeqCst);
        Ok(())
    }
    
    async fn update_state(&self, new_state: AppState) -> Result<(), UiError> {
        // 1. 状態マネージャー更新
        self.state_manager.update_state(new_state.clone()).await?;
        
        // 2. アプリケーション状態同期
        {
            let mut app = self.app.lock().unwrap();
            app.update_state(new_state)?;
        }
        
        // 3. UI再描画要求
        self.request_repaint().await?;
        
        Ok(())
    }
    
    async fn send_event(&self, event: UiEvent) -> Result<(), UiError> {
        // 1. イベントハンドラーで処理
        let action = self.event_handler.handle_event(event.clone()).await?;
        
        // 2. アクションが生成された場合は実行
        if let Some(action) = action {
            self.execute_action(action).await?;
        }
        
        // 3. イベント通知
        self.event_tx.send(event).ok();
        
        Ok(())
    }
    
    async fn execute_action(&self, action: UiAction) -> Result<(), UiError> {
        match action {
            UiAction::StartAuthentication => {
                // 認証開始: 認証コンポーネントに委譲
                self.delegate_to_auth_component().await?;
            },
            
            UiAction::StartRecordingSearch { criteria } => {
                // 録画検索開始: 録画管理コンポーネントに委譲
                self.delegate_to_recording_component(criteria).await?;
            },
            
            UiAction::StartDownload { selected_files, output_directory } => {
                // ダウンロード開始: ダウンロード実行コンポーネントに委譲
                self.delegate_to_download_component(selected_files, output_directory).await?;
            },
            
            UiAction::SaveConfiguration { config } => {
                // 設定保存: 設定管理コンポーネントに委譲
                self.delegate_to_config_component(config).await?;
            },
            
            UiAction::CancelDownload { download_id } => {
                // ダウンロードキャンセル
                self.cancel_download_operation(&download_id).await?;
            },
            
            UiAction::RetryOperation { operation_id } => {
                // 操作リトライ
                self.retry_failed_operation(&operation_id).await?;
            },
            
            UiAction::ClearNotifications => {
                // 通知クリア
                self.notification_system.clear_all_notifications().await?;
            },
            
            UiAction::Quit => {
                // アプリケーション終了
                self.shutdown().await?;
            },
        }
        
        Ok(())
    }
    
    async fn show_notification(&self, notification: NotificationData) -> Result<(), UiError> {
        // 1. 通知システムに追加
        self.notification_system.add_notification(notification).await?;
        
        // 2. UI更新
        self.request_repaint().await?;
        
        Ok(())
    }
    
    async fn update_progress(&self, progress: ProgressDisplayData) -> Result<(), UiError> {
        // 1. 進捗監視システム更新
        self.progress_monitor.update_progress(progress).await?;
        
        // 2. 必要に応じて進捗ビューに自動切り替え
        if self.should_switch_to_progress_view(&progress).await {
            self.navigate_to_view(AppView::Progress {
                download_session_id: progress.progress_id,
            }).await?;
        }
        
        Ok(())
    }
    
    async fn navigate_to_view(&self, view: AppView) -> Result<(), UiError> {
        // 1. 現在の状態取得
        let current_state = self.state_manager.get_current_state().await;
        
        // 2. ビュー遷移イベント生成
        let navigation_event = UiEvent::Navigation(NavigationEvent::ViewChanged {
            from_view: current_state.current_view,
            to_view: view.clone(),
        });
        
        // 3. 状態更新
        let mut new_state = current_state;
        new_state.current_view = view;
        new_state.last_updated = chrono::Utc::now();
        
        self.update_state(new_state).await?;
        
        // 4. ナビゲーションイベント送信
        self.send_event(navigation_event).await?;
        
        Ok(())
    }
}

/// メインアプリケーション（egui App トレイト実装）
pub struct ZoomVideoMoverApp {
    /// アプリケーション状態
    state: AppState,
    
    /// ビューマネージャー
    view_manager: ViewManager,
    
    /// イベント送信チャンネル
    event_sender: Option<broadcast::Sender<UiEvent>>,
    
    /// 作成コンテキスト
    creation_context: Option<eframe::CreationContext>,
    
    /// 最後の更新時刻
    last_frame_time: Instant,
    
    /// フレームレート監視
    frame_rate_monitor: FrameRateMonitor,
}

impl ZoomVideoMoverApp {
    pub fn new(ui_config: &UiConfiguration) -> Result<Self, UiError> {
        let state = AppState::default();
        let view_manager = ViewManager::new(ui_config)?;
        let event_sender = None;
        let creation_context = None;
        let last_frame_time = Instant::now();
        let frame_rate_monitor = FrameRateMonitor::new();
        
        Ok(Self {
            state,
            view_manager,
            event_sender,
            creation_context,
            last_frame_time,
            frame_rate_monitor,
        })
    }
}

impl eframe::App for ZoomVideoMoverApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // 1. フレーム時間監視
        let current_time = Instant::now();
        let frame_time = current_time.duration_since(self.last_frame_time);
        self.frame_rate_monitor.record_frame_time(frame_time);
        self.last_frame_time = current_time;
        
        // 2. 入力イベント処理
        self.process_input_events(ctx);
        
        // 3. 現在のビューに応じたUI描画
        match &self.state.current_view {
            AppView::Loading => {
                self.view_manager.render_loading_view(ctx, &self.state);
            },
            
            AppView::Settings => {
                self.view_manager.render_settings_view(ctx, &mut self.state);
            },
            
            AppView::Main { selected_tab } => {
                self.view_manager.render_main_view(ctx, &mut self.state, selected_tab);
            },
            
            AppView::Progress { download_session_id } => {
                self.view_manager.render_progress_view(ctx, &self.state, download_session_id);
            },
            
            AppView::Error { error_id, recoverable } => {
                self.view_manager.render_error_view(ctx, &self.state, error_id, *recoverable);
            },
        }
        
        // 4. 通知・トースト表示
        self.render_notifications(ctx);
        
        // 5. ステータスバー表示
        self.render_status_bar(ctx, frame);
        
        // 6. デバッグ情報表示（デバッグビルド時のみ）
        #[cfg(debug_assertions)]
        self.render_debug_info(ctx);
        
        // 7. アクセシビリティサポート
        self.update_accessibility_info(ctx);
        
        // 8. 再描画の必要性判定
        if self.should_request_repaint() {
            ctx.request_repaint();
        }
    }
    
    fn on_close_event(&mut self) -> bool {
        // アプリケーション終了前の処理
        if let Some(sender) = &self.event_sender {
            sender.send(UiEvent::System(SystemEvent::AppClosing)).ok();
        }
        
        // 未保存の変更確認
        self.check_unsaved_changes()
    }
    
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // アプリケーション終了時のクリーンアップ
        if let Some(sender) = &self.event_sender {
            sender.send(UiEvent::System(SystemEvent::AppExiting)).ok();
        }
    }
}
```

### 内部インターフェース

#### 1. ビューマネージャー
```rust
/// ビュー管理インターフェース
pub trait ViewManager: Send + Sync {
    /// ローディングビュー描画
    fn render_loading_view(&self, ctx: &egui::Context, state: &AppState);
    
    /// 設定ビュー描画
    fn render_settings_view(&self, ctx: &egui::Context, state: &mut AppState);
    
    /// メインビュー描画
    fn render_main_view(&self, ctx: &egui::Context, state: &mut AppState, selected_tab: &MainTabType);
    
    /// 進捗ビュー描画
    fn render_progress_view(&self, ctx: &egui::Context, state: &AppState, session_id: &str);
    
    /// エラービュー描画
    fn render_error_view(&self, ctx: &egui::Context, state: &AppState, error_id: &str, recoverable: bool);
    
    /// ビュー遷移処理
    fn handle_view_transition(&self, from: &AppView, to: &AppView) -> Result<(), UiError>;
}

/// egui ベースビューマネージャー実装
pub struct EguiViewManager {
    /// 各ビューの実装
    loading_view: LoadingView,
    settings_view: SettingsView,
    main_view: MainView,
    progress_view: ProgressView,
    error_view: ErrorView,
    
    /// テーママネージャー
    theme_manager: Arc<ThemeManager>,
    
    /// レイアウトエンジン
    layout_engine: Arc<LayoutEngine>,
    
    /// 国際化マネージャー
    i18n_manager: Arc<I18nManager>,
}

impl EguiViewManager {
    pub fn new(ui_config: &UiConfiguration) -> Result<Self, UiError> {
        let theme_manager = Arc::new(ThemeManager::new(ui_config)?);
        let layout_engine = Arc::new(LayoutEngine::new(ui_config)?);
        let i18n_manager = Arc::new(I18nManager::new(ui_config)?);
        
        let loading_view = LoadingView::new(theme_manager.clone());
        let settings_view = SettingsView::new(theme_manager.clone(), i18n_manager.clone());
        let main_view = MainView::new(theme_manager.clone(), layout_engine.clone());
        let progress_view = ProgressView::new(theme_manager.clone());
        let error_view = ErrorView::new(theme_manager.clone(), i18n_manager.clone());
        
        Ok(Self {
            loading_view,
            settings_view,
            main_view,
            progress_view,
            error_view,
            theme_manager,
            layout_engine,
            i18n_manager,
        })
    }
}

impl ViewManager for EguiViewManager {
    fn render_main_view(&self, ctx: &egui::Context, state: &mut AppState, selected_tab: &MainTabType) {
        // 1. メインレイアウト設定
        egui::CentralPanel::default().show(ctx, |ui| {
            // 2. タブバー描画
            ui.horizontal(|ui| {
                self.render_tab_bar(ui, state, selected_tab);
            });
            
            ui.separator();
            
            // 3. 選択されたタブの内容描画
            match selected_tab {
                MainTabType::Search => {
                    self.render_search_tab(ui, state);
                },
                MainTabType::FileSelection => {
                    self.render_file_selection_tab(ui, state);
                },
                MainTabType::History => {
                    self.render_history_tab(ui, state);
                },
                MainTabType::Settings => {
                    self.render_settings_tab(ui, state);
                },
                MainTabType::Help => {
                    self.render_help_tab(ui, state);
                },
            }
        });
        
        // 4. サイドパネル（必要に応じて）
        if self.should_show_side_panel(state) {
            self.render_side_panel(ctx, state);
        }
    }
    
    /// 検索タブ描画
    fn render_search_tab(&self, ui: &mut egui::Ui, state: &mut AppState) {
        ui.vertical(|ui| {
            // 1. フィルタパネル
            ui.group(|ui| {
                ui.label(self.i18n_manager.get_text("search.filters.title"));
                
                // 日付範囲フィルタ
                ui.horizontal(|ui| {
                    ui.label(self.i18n_manager.get_text("search.filters.date_range"));
                    
                    // カスタム日付ピッカーウィジェット使用
                    if let Some(date_range) = &mut state.recording_state.filter_criteria.date_range {
                        self.render_date_range_picker(ui, date_range);
                    }
                });
                
                // ファイル種別フィルタ
                ui.horizontal(|ui| {
                    ui.label(self.i18n_manager.get_text("search.filters.file_types"));
                    
                    if let Some(file_types) = &mut state.recording_state.filter_criteria.file_types {
                        self.render_file_type_checkboxes(ui, file_types);
                    }
                });
                
                // テキスト検索
                ui.horizontal(|ui| {
                    ui.label(self.i18n_manager.get_text("search.filters.text_search"));
                    
                    if let Some(text_search) = &mut state.recording_state.filter_criteria.text_search {
                        ui.text_edit_singleline(&mut text_search.query);
                    }
                });
                
                // 検索ボタン
                if ui.button(self.i18n_manager.get_text("search.buttons.search")).clicked() {
                    // 検索開始イベント送信
                    self.send_search_event(state);
                }
            });
            
            ui.separator();
            
            // 2. 検索結果表示
            ui.group(|ui| {
                ui.label(self.i18n_manager.get_text("search.results.title"));
                
                self.render_recording_list(ui, &state.recording_state.search_results);
            });
        });
    }
    
    /// ファイル選択タブ描画
    fn render_file_selection_tab(&self, ui: &mut egui::Ui, state: &mut AppState) {
        ui.vertical(|ui| {
            // 1. 選択統計表示
            ui.horizontal(|ui| {
                let selection_count = state.recording_state.selected_files.len();
                let total_size = self.calculate_total_selected_size(state);
                
                ui.label(format!(
                    "{}: {} files, {}",
                    self.i18n_manager.get_text("file_selection.stats.selected"),
                    selection_count,
                    self.format_file_size(total_size)
                ));
                
                ui.separator();
                
                // 全選択・全解除ボタン
                if ui.button(self.i18n_manager.get_text("file_selection.buttons.select_all")).clicked() {
                    self.send_select_all_event(state);
                }
                
                if ui.button(self.i18n_manager.get_text("file_selection.buttons.deselect_all")).clicked() {
                    self.send_deselect_all_event(state);
                }
            });
            
            ui.separator();
            
            // 2. ファイルツリー表示（階層構造）
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    self.render_file_tree(ui, state);
                });
            
            ui.separator();
            
            // 3. ダウンロード設定・開始
            ui.horizontal(|ui| {
                ui.label(self.i18n_manager.get_text("file_selection.download.output_directory"));
                
                ui.text_edit_singleline(&mut state.download_state.output_directory_display);
                
                if ui.button(self.i18n_manager.get_text("file_selection.buttons.browse")).clicked() {
                    // フォルダ選択ダイアログ表示
                    self.show_folder_selection_dialog(state);
                }
            });
            
            // ダウンロード開始ボタン
            let download_enabled = !state.recording_state.selected_files.is_empty() 
                && !state.download_state.output_directory_display.is_empty();
                
            ui.add_enabled_ui(download_enabled, |ui| {
                if ui.button(self.i18n_manager.get_text("file_selection.buttons.start_download")).clicked() {
                    self.send_download_start_event(state);
                }
            });
        });
    }
}
```

#### 2. 通知システム
```rust
/// 通知システムインターフェース
#[async_trait]
pub trait NotificationSystem: Send + Sync {
    /// 通知追加
    async fn add_notification(&self, notification: NotificationData) -> Result<(), UiError>;
    
    /// 通知削除
    async fn remove_notification(&self, notification_id: &str) -> Result<(), UiError>;
    
    /// 全通知クリア
    async fn clear_all_notifications(&self) -> Result<(), UiError>;
    
    /// アクティブ通知取得
    async fn get_active_notifications(&self) -> Vec<NotificationData>;
    
    /// 通知描画
    fn render_notifications(&self, ctx: &egui::Context) -> Result<(), UiError>;
    
    /// 通知アクション実行
    async fn execute_notification_action(&self, notification_id: &str, action_id: &str) -> Result<(), UiError>;
}

/// トースト通知システム実装
pub struct ToastNotificationSystem {
    /// アクティブ通知
    active_notifications: Arc<RwLock<HashMap<String, NotificationData>>>,
    
    /// 通知表示設定
    display_config: NotificationDisplayConfig,
    
    /// アニメーションエンジン
    animation_engine: Arc<AnimationEngine>,
    
    /// 通知レイアウト
    layout_manager: Arc<NotificationLayoutManager>,
    
    /// 自動削除タイマー
    auto_dismiss_timers: Arc<RwLock<HashMap<String, tokio::time::Instant>>>,
}

impl ToastNotificationSystem {
    pub fn new(ui_config: &UiConfiguration) -> Result<Self, UiError> {
        let active_notifications = Arc::new(RwLock::new(HashMap::new()));
        let display_config = NotificationDisplayConfig::from_ui_config(ui_config);
        let animation_engine = Arc::new(AnimationEngine::new(&display_config)?);
        let layout_manager = Arc::new(NotificationLayoutManager::new(&display_config));
        let auto_dismiss_timers = Arc::new(RwLock::new(HashMap::new()));
        
        Ok(Self {
            active_notifications,
            display_config,
            animation_engine,
            layout_manager,
            auto_dismiss_timers,
        })
    }
}

#[async_trait]
impl NotificationSystem for ToastNotificationSystem {
    async fn add_notification(&self, mut notification: NotificationData) -> Result<(), UiError> {
        // 1. 通知の表示状態設定
        notification.display_state = NotificationDisplayState::Entering;
        
        // 2. アクティブ通知に追加
        {
            let mut notifications = self.active_notifications.write().await;
            notifications.insert(notification.id.clone(), notification.clone());
        }
        
        // 3. 自動削除タイマー設定
        if let Some(auto_dismiss_after) = notification.auto_dismiss_after {
            let dismiss_time = tokio::time::Instant::now() + auto_dismiss_after;
            {
                let mut timers = self.auto_dismiss_timers.write().await;
                timers.insert(notification.id.clone(), dismiss_time);
            }
            
            // 自動削除タスク開始
            let notification_id = notification.id.clone();
            let system_ref = Arc::new(self);
            tokio::spawn(async move {
                tokio::time::sleep(auto_dismiss_after).await;
                system_ref.remove_notification(&notification_id).await.ok();
            });
        }
        
        // 4. 入場アニメーション開始
        self.animation_engine.start_enter_animation(&notification.id).await?;
        
        Ok(())
    }
    
    fn render_notifications(&self, ctx: &egui::Context) -> Result<(), UiError> {
        let notifications = self.active_notifications.blocking_read();
        
        if notifications.is_empty() {
            return Ok(());
        }
        
        // 1. 通知表示領域計算
        let screen_rect = ctx.screen_rect();
        let notification_area = self.layout_manager.calculate_notification_area(&screen_rect);
        
        // 2. 通知を重要度順・作成時刻順でソート
        let mut sorted_notifications: Vec<_> = notifications.values().collect();
        sorted_notifications.sort_by(|a, b| {
            b.severity.cmp(&a.severity)
                .then_with(|| b.created_at.cmp(&a.created_at))
        });
        
        // 3. 最大表示数制限適用
        let max_visible = self.display_config.max_visible_notifications;
        let visible_notifications = &sorted_notifications[..sorted_notifications.len().min(max_visible)];
        
        // 4. 各通知を描画
        for (index, notification) in visible_notifications.iter().enumerate() {
            let notification_rect = self.layout_manager.calculate_notification_rect(
                &notification_area,
                index,
                visible_notifications.len(),
            );
            
            // アニメーション状態取得
            let animation_state = self.animation_engine.get_animation_state(&notification.id);
            let adjusted_rect = self.apply_animation_transform(notification_rect, &animation_state);
            
            // 通知ウィンドウ描画
            egui::Window::new(&notification.title)
                .id(egui::Id::new(format!("notification_{}", notification.id)))
                .fixed_rect(adjusted_rect)
                .collapsible(false)
                .resizable(false)
                .title_bar(false)
                .frame(self.get_notification_frame_style(&notification.notification_type))
                .show(ctx, |ui| {
                    self.render_notification_content(ui, notification);
                });
        }
        
        Ok(())
    }
    
    /// 通知内容描画
    fn render_notification_content(&self, ui: &mut egui::Ui, notification: &NotificationData) {
        ui.vertical(|ui| {
            // 1. アイコン・タイトル行
            ui.horizontal(|ui| {
                // 通知タイプアイコン
                let icon = self.get_notification_icon(&notification.notification_type);
                ui.label(egui::RichText::new(icon).size(16.0));
                
                // タイトル
                ui.label(egui::RichText::new(&notification.title).strong());
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // 閉じるボタン
                    if ui.small_button("✕").clicked() {
                        // 通知削除イベント送信
                        self.send_dismiss_event(&notification.id);
                    }
                });
            });
            
            // 2. メッセージ
            ui.label(&notification.message);
            
            // 3. 詳細情報（展開可能）
            if let Some(details) = &notification.details {
                ui.collapsing(self.i18n_manager.get_text("notification.show_details"), |ui| {
                    ui.label(details);
                });
            }
            
            // 4. アクションボタン
            if !notification.actions.is_empty() {
                ui.separator();
                ui.horizontal(|ui| {
                    for action in &notification.actions {
                        let button_style = if action.is_primary {
                            ui.style().visuals.widgets.active
                        } else {
                            ui.style().visuals.widgets.inactive
                        };
                        
                        if ui.button(&action.label).clicked() {
                            // アクション実行イベント送信
                            self.send_action_event(&notification.id, &action.id);
                        }
                    }
                });
            }
        });
    }
}
```

## アルゴリズム設計

### レスポンシブレイアウトアルゴリズム

#### 動的レイアウト調整
```rust
impl LayoutEngine {
    /// レスポンシブレイアウト計算
    pub fn calculate_responsive_layout(
        &self,
        available_rect: egui::Rect,
        content_requirements: &ContentRequirements,
    ) -> LayoutResult {
        // 1. 基準解像度と現在解像度の比率計算
        let base_resolution = self.config.base_resolution;
        let current_resolution = (available_rect.width(), available_rect.height());
        
        let scale_x = current_resolution.0 / base_resolution.0;
        let scale_y = current_resolution.1 / base_resolution.1;
        let scale_factor = scale_x.min(scale_y); // アスペクト比維持
        
        // 2. ブレークポイント判定
        let breakpoint = self.determine_breakpoint(current_resolution);
        
        // 3. ブレークポイント別レイアウト計算
        match breakpoint {
            LayoutBreakpoint::Small => {
                self.calculate_compact_layout(available_rect, content_requirements, scale_factor)
            },
            LayoutBreakpoint::Medium => {
                self.calculate_standard_layout(available_rect, content_requirements, scale_factor)
            },
            LayoutBreakpoint::Large => {
                self.calculate_expanded_layout(available_rect, content_requirements, scale_factor)
            },
        }
    }
    
    /// コンパクトレイアウト（小画面向け）
    fn calculate_compact_layout(
        &self,
        available_rect: egui::Rect,
        content_requirements: &ContentRequirements,
        scale_factor: f32,
    ) -> LayoutResult {
        let mut layout = LayoutResult::new();
        
        // 1. 垂直レイアウト重視
        let content_width = available_rect.width() - self.config.margins.horizontal * 2.0;
        let mut current_y = available_rect.top + self.config.margins.vertical;
        
        // 2. ヘッダー領域
        let header_height = self.config.header_height * scale_factor;
        layout.header_rect = egui::Rect::from_min_size(
            egui::pos2(available_rect.left + self.config.margins.horizontal, current_y),
            egui::vec2(content_width, header_height)
        );
        current_y += header_height + self.config.spacing.vertical;
        
        // 3. メインコンテンツ領域（全幅使用）
        let remaining_height = available_rect.bottom - current_y - self.config.margins.vertical;
        layout.main_content_rect = egui::Rect::from_min_size(
            egui::pos2(available_rect.left + self.config.margins.horizontal, current_y),
            egui::vec2(content_width, remaining_height * 0.9)
        );
        
        // 4. サイドパネルは折りたたみ・オーバーレイ表示
        layout.side_panel_rect = None;
        layout.side_panel_overlay = Some(self.calculate_overlay_panel_rect(available_rect));
        
        // 5. ステータスバー
        layout.status_bar_rect = egui::Rect::from_min_size(
            egui::pos2(available_rect.left, available_rect.bottom - self.config.status_bar_height),
            egui::vec2(available_rect.width(), self.config.status_bar_height)
        );
        
        layout
    }
    
    /// 標準レイアウト（中画面向け）
    fn calculate_standard_layout(
        &self,
        available_rect: egui::Rect,
        content_requirements: &ContentRequirements,
        scale_factor: f32,
    ) -> LayoutResult {
        let mut layout = LayoutResult::new();
        
        // 1. 2カラムレイアウト
        let side_panel_width = (self.config.side_panel_width * scale_factor).min(available_rect.width() * 0.3);
        let main_content_width = available_rect.width() - side_panel_width - self.config.margins.horizontal;
        
        let mut current_y = available_rect.top + self.config.margins.vertical;
        
        // 2. ヘッダー領域（全幅）
        let header_height = self.config.header_height * scale_factor;
        layout.header_rect = egui::Rect::from_min_size(
            egui::pos2(available_rect.left, current_y),
            egui::vec2(available_rect.width(), header_height)
        );
        current_y += header_height;
        
        // 3. サイドパネル
        let content_height = available_rect.bottom - current_y - self.config.status_bar_height;
        layout.side_panel_rect = Some(egui::Rect::from_min_size(
            egui::pos2(available_rect.left, current_y),
            egui::vec2(side_panel_width, content_height)
        ));
        
        // 4. メインコンテンツ領域
        layout.main_content_rect = egui::Rect::from_min_size(
            egui::pos2(available_rect.left + side_panel_width, current_y),
            egui::vec2(main_content_width, content_height)
        );
        
        // 5. ステータスバー
        layout.status_bar_rect = egui::Rect::from_min_size(
            egui::pos2(available_rect.left, available_rect.bottom - self.config.status_bar_height),
            egui::vec2(available_rect.width(), self.config.status_bar_height)
        );
        
        layout
    }
    
    /// ブレークポイント判定
    fn determine_breakpoint(&self, resolution: (f32, f32)) -> LayoutBreakpoint {
        let (width, height) = resolution;
        
        if width < self.config.breakpoints.small_max_width || height < self.config.breakpoints.small_max_height {
            LayoutBreakpoint::Small
        } else if width < self.config.breakpoints.medium_max_width || height < self.config.breakpoints.medium_max_height {
            LayoutBreakpoint::Medium
        } else {
            LayoutBreakpoint::Large
        }
    }
}
```

### アニメーションエンジン

#### スムーズアニメーション実装
```rust
impl AnimationEngine {
    /// 通知入場アニメーション
    pub async fn start_enter_animation(&self, notification_id: &str) -> Result<(), UiError> {
        let animation = Animation {
            id: notification_id.to_string(),
            animation_type: AnimationType::SlideIn {
                direction: SlideDirection::Right,
                duration: Duration::from_millis(300),
            },
            easing_function: EasingFunction::EaseOutCubic,
            start_time: Instant::now(),
            current_state: AnimationState::Running,
        };
        
        // アニメーション登録
        {
            let mut animations = self.active_animations.write().await;
            animations.insert(notification_id.to_string(), animation);
        }
        
        // アニメーション更新タスク開始
        self.start_animation_update_task(notification_id).await;
        
        Ok(())
    }
    
    /// アニメーション更新タスク
    async fn start_animation_update_task(&self, animation_id: &str) {
        let animation_id = animation_id.to_string();
        let animations_ref = self.active_animations.clone();
        
        tokio::spawn(async move {
            let update_interval = Duration::from_millis(16); // 60 FPS
            let mut interval = tokio::time::interval(update_interval);
            
            loop {
                interval.tick().await;
                
                let should_continue = {
                    let mut animations = animations_ref.write().await;
                    
                    if let Some(animation) = animations.get_mut(&animation_id) {
                        // アニメーション進行度計算
                        let elapsed = animation.start_time.elapsed();
                        let progress = Self::calculate_animation_progress(animation, elapsed);
                        
                        if progress >= 1.0 {
                            // アニメーション完了
                            animation.current_state = AnimationState::Completed;
                            false
                        } else {
                            // アニメーション継続
                            true
                        }
                    } else {
                        false
                    }
                };
                
                if !should_continue {
                    break;
                }
            }
            
            // 完了したアニメーションを削除
            {
                let mut animations = animations_ref.write().await;
                animations.remove(&animation_id);
            }
        });
    }
    
    /// アニメーション進行度計算
    fn calculate_animation_progress(animation: &Animation, elapsed: Duration) -> f32 {
        let duration = match &animation.animation_type {
            AnimationType::SlideIn { duration, .. } => *duration,
            AnimationType::SlideOut { duration, .. } => *duration,
            AnimationType::FadeIn { duration } => *duration,
            AnimationType::FadeOut { duration } => *duration,
            AnimationType::Scale { duration, .. } => *duration,
        };
        
        let raw_progress = elapsed.as_secs_f32() / duration.as_secs_f32();
        let clamped_progress = raw_progress.clamp(0.0, 1.0);
        
        // イージング関数適用
        animation.easing_function.apply(clamped_progress)
    }
    
    /// 変換行列適用
    pub fn apply_animation_transform(&self, base_rect: egui::Rect, animation_state: &AnimationState) -> egui::Rect {
        match animation_state {
            AnimationState::NotAnimated => base_rect,
            AnimationState::Running => {
                // アニメーション進行中の変換計算
                self.calculate_animated_rect(base_rect, animation_state)
            },
            AnimationState::Completed => base_rect,
        }
    }
    
    /// アニメーション中の矩形計算
    fn calculate_animated_rect(&self, base_rect: egui::Rect, animation_state: &AnimationState) -> egui::Rect {
        // アニメーション種別に応じた変換
        // 実装はアニメーションタイプとイージング関数によって決定
        base_rect // 簡略化
    }
}

/// イージング関数実装
impl EasingFunction {
    pub fn apply(&self, t: f32) -> f32 {
        match self {
            EasingFunction::Linear => t,
            EasingFunction::EaseInCubic => t * t * t,
            EasingFunction::EaseOutCubic => 1.0 - (1.0 - t).powi(3),
            EasingFunction::EaseInOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            },
            EasingFunction::EaseInQuart => t * t * t * t,
            EasingFunction::EaseOutQuart => 1.0 - (1.0 - t).powi(4),
            EasingFunction::Bounce => {
                if t < 1.0 / 2.75 {
                    7.5625 * t * t
                } else if t < 2.0 / 2.75 {
                    let t = t - 1.5 / 2.75;
                    7.5625 * t * t + 0.75
                } else if t < 2.5 / 2.75 {
                    let t = t - 2.25 / 2.75;
                    7.5625 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / 2.75;
                    7.5625 * t * t + 0.984375
                }
            },
        }
    }
}
```

## エラー処理設計

### エラー階層構造
```rust
/// UI制御エラー定義
#[derive(Debug, thiserror::Error)]
pub enum UiError {
    /// eframe初期化エラー
    #[error("eframe initialization failed: {reason}")]
    EframeInitializationError {
        reason: String,
    },
    
    /// ウィンドウ作成エラー
    #[error("Window creation failed: {source}")]
    WindowCreationError {
        #[from]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    /// 状態管理エラー
    #[error("State management error: {operation} - {details}")]
    StateManagementError {
        operation: String,
        details: String,
    },
    
    /// イベント処理エラー
    #[error("Event processing error: {event_type} - {source}")]
    EventProcessingError {
        event_type: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    /// レンダリングエラー
    #[error("Rendering error: {component} - {details}")]
    RenderingError {
        component: String,
        details: String,
    },
    
    /// アニメーションエラー
    #[error("Animation error: {animation_id} - {reason}")]
    AnimationError {
        animation_id: String,
        reason: String,
    },
    
    /// 通知システムエラー
    #[error("Notification system error: {operation}")]
    NotificationSystemError {
        operation: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    /// レイアウトエラー
    #[error("Layout calculation error: {layout_type} - {reason}")]
    LayoutError {
        layout_type: String,
        reason: String,
    },
    
    /// リソース読み込みエラー
    #[error("Resource loading error: {resource_type} - {path}")]
    ResourceLoadingError {
        resource_type: String,
        path: String,
        source: std::io::Error,
    },
    
    /// フォント読み込みエラー
    #[error("Font loading error: {font_name} - {source}")]
    FontLoadingError {
        font_name: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    /// DPI スケーリングエラー
    #[error("DPI scaling error: {current_dpi} - {reason}")]
    DpiScalingError {
        current_dpi: f32,
        reason: String,
    },
}

/// UI エラー回復戦略
pub struct UiErrorRecoveryStrategy {
    /// デフォルト状態プロバイダー
    default_state_provider: Arc<DefaultStateProvider>,
    
    /// フォールバック UI
    fallback_ui: Arc<FallbackUi>,
    
    /// エラー履歴
    error_history: Arc<Mutex<Vec<UiErrorRecord>>>,
}

impl UiErrorRecoveryStrategy {
    /// エラー回復試行
    pub async fn attempt_recovery(&self, error: &UiError, context: &UiContext) -> UiRecoveryResult {
        match error {
            UiError::EframeInitializationError { reason } => {
                // eframe初期化失敗: 設定リセット・再初期化試行
                if reason.contains("OpenGL") {
                    UiRecoveryResult::RetryWithFallback {
                        fallback_config: self.create_software_rendering_config(),
                        retry_count: 1,
                    }
                } else {
                    UiRecoveryResult::RestartRequired {
                        reason: "eframe initialization requires application restart".to_string(),
                    }
                }
            },
            
            UiError::StateManagementError { operation, .. } => {
                // 状態管理エラー: 状態リセット・デフォルト復元
                match operation.as_str() {
                    "state_corruption" => {
                        UiRecoveryResult::ResetToDefault {
                            component: "state_manager".to_string(),
                            backup_created: true,
                        }
                    },
                    "state_lock_timeout" => {
                        UiRecoveryResult::RetryAfterDelay {
                            delay: Duration::from_millis(100),
                            max_retries: 3,
                        }
                    },
                    _ => UiRecoveryResult::RequiresUserIntervention,
                }
            },
            
            UiError::RenderingError { component, .. } => {
                // レンダリングエラー: コンポーネント別対応
                match component.as_str() {
                    "notification_system" => {
                        UiRecoveryResult::DisableComponent {
                            component_name: component.clone(),
                            temporary: true,
                            duration: Some(Duration::from_secs(30)),
                        }
                    },
                    "main_view" => {
                        UiRecoveryResult::SwitchToFallbackUi {
                            fallback_ui_type: "minimal".to_string(),
                        }
                    },
                    _ => UiRecoveryResult::RequiresUserIntervention,
                }
            },
            
            UiError::FontLoadingError { font_name, .. } => {
                // フォント読み込みエラー: システムフォントフォールバック
                UiRecoveryResult::UseSystemFallback {
                    original_resource: font_name.clone(),
                    fallback_resource: self.get_system_fallback_font(),
                }
            },
            
            UiError::DpiScalingError { current_dpi, .. } => {
                // DPI エラー: スケーリング無効化・手動設定
                UiRecoveryResult::AdjustSettings {
                    setting_name: "dpi_scaling".to_string(),
                    new_value: serde_json::Value::Bool(false),
                    reason: format!("DPI scaling failed at {}x", current_dpi),
                }
            },
            
            _ => UiRecoveryResult::RequiresUserIntervention,
        }
    }
}
```

## テスト設計

### Property-basedテスト
```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        /// UI状態遷移の一貫性検証
        #[test]
        fn test_ui_state_transition_consistency(
            initial_state in arb_app_state(),
            state_changes in prop::collection::vec(arb_state_change(), 1..10)
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let state_manager = UiStateManager::new();
                
                // 初期状態設定
                state_manager.update_state(initial_state.clone()).await.unwrap();
                
                let mut current_state = initial_state;
                
                // 状態変更を順次適用
                for change in state_changes {
                    let new_state = apply_state_change(&current_state, &change);
                    state_manager.update_state(new_state.clone()).await.unwrap();
                    
                    let retrieved_state = state_manager.get_current_state().await;
                    
                    // Property: 設定した状態が正確に取得される
                    prop_assert_eq!(new_state, retrieved_state);
                    
                    current_state = new_state;
                }
                
                // Property: 最終状態が一貫している
                let final_state = state_manager.get_current_state().await;
                prop_assert_eq!(current_state, final_state);
            });
        }
        
        /// レイアウト計算の正確性検証
        #[test]
        fn test_layout_calculation_accuracy(
            window_size in (100.0f32..2000.0f32, 100.0f32..1500.0f32),
            content_requirements in arb_content_requirements()
        ) {
            let layout_engine = LayoutEngine::new(&create_test_ui_config()).unwrap();
            let available_rect = egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(window_size.0, window_size.1)
            );
            
            let layout = layout_engine.calculate_responsive_layout(&available_rect, &content_requirements);
            
            // Property: レイアウト矩形が利用可能領域内に収まる
            prop_assert!(layout.header_rect.min.x >= available_rect.min.x);
            prop_assert!(layout.header_rect.max.x <= available_rect.max.x);
            prop_assert!(layout.header_rect.min.y >= available_rect.min.y);
            prop_assert!(layout.header_rect.max.y <= available_rect.max.y);
            
            prop_assert!(layout.main_content_rect.min.x >= available_rect.min.x);
            prop_assert!(layout.main_content_rect.max.x <= available_rect.max.x);
            prop_assert!(layout.main_content_rect.min.y >= available_rect.min.y);
            prop_assert!(layout.main_content_rect.max.y <= available_rect.max.y);
            
            // Property: 矩形が重複しない
            prop_assert!(!layout.header_rect.intersects(layout.main_content_rect));
            
            if let Some(side_panel) = layout.side_panel_rect {
                prop_assert!(!side_panel.intersects(layout.main_content_rect));
            }
            
            // Property: レイアウトが利用可能空間を有効活用している
            let total_used_area = layout.header_rect.area() + layout.main_content_rect.area();
            let available_area = available_rect.area();
            prop_assert!(total_used_area >= available_area * 0.6); // 60%以上使用
        }
        
        /// 通知システムの整合性検証
        #[test]
        fn test_notification_system_consistency(
            notifications in prop::collection::vec(arb_notification_data(), 1..20)
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let notification_system = ToastNotificationSystem::new(&create_test_ui_config()).unwrap();
                
                // 通知を順次追加
                for notification in &notifications {
                    notification_system.add_notification(notification.clone()).await.unwrap();
                }
                
                let active_notifications = notification_system.get_active_notifications().await;
                
                // Property: 追加した通知がすべてアクティブ
                prop_assert_eq!(active_notifications.len(), notifications.len());
                
                // Property: 通知IDが一意
                let notification_ids: std::collections::HashSet<_> = active_notifications.iter()
                    .map(|n| &n.id)
                    .collect();
                prop_assert_eq!(notification_ids.len(), active_notifications.len());
                
                // Property: 重要度順にソートされている
                for window in active_notifications.windows(2) {
                    prop_assert!(window[0].severity >= window[1].severity);
                }
                
                // 通知削除テスト
                if let Some(first_notification) = notifications.first() {
                    notification_system.remove_notification(&first_notification.id).await.unwrap();
                    
                    let updated_notifications = notification_system.get_active_notifications().await;
                    prop_assert_eq!(updated_notifications.len(), notifications.len() - 1);
                    
                    // Property: 削除された通知が含まれていない
                    prop_assert!(!updated_notifications.iter().any(|n| n.id == first_notification.id));
                }
            });
        }
        
        /// アニメーション計算の連続性検証
        #[test]
        fn test_animation_calculation_continuity(
            animation_duration_ms in 100u64..5000u64,
            sample_count in 10usize..100usize
        ) {
            let animation_engine = AnimationEngine::new(&create_test_animation_config()).unwrap();
            
            let animation = Animation {
                id: "test_animation".to_string(),
                animation_type: AnimationType::SlideIn {
                    direction: SlideDirection::Right,
                    duration: Duration::from_millis(animation_duration_ms),
                },
                easing_function: EasingFunction::EaseOutCubic,
                start_time: Instant::now(),
                current_state: AnimationState::Running,
            };
            
            // アニメーション進行度を等間隔でサンプリング
            let mut progress_values = Vec::new();
            for i in 0..sample_count {
                let elapsed_ratio = i as f64 / (sample_count - 1) as f64;
                let elapsed = Duration::from_millis((animation_duration_ms as f64 * elapsed_ratio) as u64);
                
                let progress = animation_engine.calculate_animation_progress(&animation, elapsed);
                progress_values.push(progress);
            }
            
            // Property: 進行度は単調増加
            for window in progress_values.windows(2) {
                prop_assert!(window[1] >= window[0]);
            }
            
            // Property: 進行度は0.0から1.0の範囲内
            for &progress in &progress_values {
                prop_assert!(progress >= 0.0 && progress <= 1.0);
            }
            
            // Property: 開始時は0.0、終了時は1.0
            prop_assert_eq!(progress_values[0], 0.0);
            prop_assert_eq!(*progress_values.last().unwrap(), 1.0);
        }
    }
    
    /// 任意のアプリケーション状態生成
    fn arb_app_state() -> impl Strategy<Value = AppState> {
        (
            arb_app_view(),
            arb_auth_state(),
            arb_ui_state(),
        ).prop_map(|(current_view, auth_state, ui_state)| {
            AppState {
                current_view,
                auth_state,
                recording_state: RecordingDataState::default(),
                download_state: DownloadState::default(),
                config_state: ConfigurationState::default(),
                ui_state,
                error_state: ErrorState::default(),
                notification_state: NotificationState::default(),
                last_updated: chrono::Utc::now(),
            }
        })
    }
    
    /// 任意の通知データ生成
    fn arb_notification_data() -> impl Strategy<Value = NotificationData> {
        (
            "[a-zA-Z0-9]{10,20}",  // id
            arb_notification_type(),
            "[\\w\\s]{5,50}",      // title
            "[\\w\\s]{10,200}",    // message
            arb_notification_severity(),
        ).prop_map(|(id, notification_type, title, message, severity)| {
            NotificationData {
                id,
                notification_type,
                title,
                message,
                details: None,
                severity,
                actions: Vec::new(),
                auto_dismiss_after: Some(Duration::from_secs(5)),
                created_at: chrono::Utc::now(),
                display_state: NotificationDisplayState::Visible,
            }
        })
    }
}
```

## 性能・セキュリティ考慮事項

### 性能最適化
1. **レンダリング最適化**: 必要な部分のみの再描画・差分更新
2. **状態管理効率**: RwLock・Atomic操作による高効率状態アクセス
3. **メモリ効率**: ウィジェットプール・テクスチャキャッシュ管理
4. **アニメーション性能**: GPU アクセラレーション・60FPS 維持

### セキュリティ強化
1. **入力検証**: 全ユーザー入力の検証・サニタイゼーション
2. **状態保護**: 機密状態の適切な管理・メモリクリア
3. **リソース制限**: UI リソース使用量の制限・DoS 防止
4. **ログ保護**: UI 操作ログの機密情報除去

---

**承認**:  
**品質基準適合**: [ ] 確認済  
**ポリシー準拠**: [ ] 確認済  
**承認日**: ___________