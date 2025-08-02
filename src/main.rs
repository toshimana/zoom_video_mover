use eframe::egui;
use zoom_video_mover_lib::windows_console;

use zoom_video_mover_lib::ZoomDownloaderApp;


/// GUI表示設定（日本語フォント・テーマ・可視性）
/// 
/// 事前条件:
/// - ctx は有効なegui::Contextである
/// - windows_console::setup_console_encoding() が事前に呼び出されている
/// 
/// 事後条件:
/// - 日本語フォントが適切に設定される
/// - 明確なライトテーマが設定される
/// - フォントサイズと色が読みやすく調整される
/// - UI要素の可視性が向上する
fn setup_gui_appearance(ctx: &egui::Context) {
    windows_console::println_japanese("Setting up GUI appearance and Japanese fonts...");
    
    // 日本語フォントの設定
    let mut fonts = egui::FontDefinitions::default();
    
    // 埋め込み日本語フォント（オプショナル）
    #[cfg(feature = "embedded-fonts")]
    {
        let noto_sans_jp = include_bytes!("../assets/NotoSansJP-Regular.ttf");
        fonts.font_data.insert(
            "NotoSansJP".to_owned(),
            egui::FontData::from_static(noto_sans_jp),
        );
    }
    
    // 既存のシステムフォントも試行
    let mut font_loaded = false;
    #[cfg(windows)]
    {
        if let Some(font_data) = load_system_font("Yu Gothic UI") {
            fonts.font_data.insert("YuGothic".to_owned(), font_data);
            font_loaded = true;
            windows_console::println_japanese("Yu Gothic UI font loaded successfully");
        } else if let Some(font_data) = load_system_font("Meiryo") {
            fonts.font_data.insert("Meiryo".to_owned(), font_data);
            font_loaded = true;
            windows_console::println_japanese("Meiryo font loaded successfully");
        } else if let Some(font_data) = load_system_font("MS Gothic") {
            fonts.font_data.insert("MSGothic".to_owned(), font_data);
            font_loaded = true;
            windows_console::println_japanese("MS Gothic font loaded successfully");
        }
    }
    
    if !font_loaded {
        windows_console::println_japanese("Warning: No Japanese system fonts found, using default fonts");
    }
    
    // デフォルトフォントファミリーに日本語フォントを追加
    for (_, font_family) in fonts.families.iter_mut() {
        #[cfg(feature = "embedded-fonts")]
        font_family.insert(0, "NotoSansJP".to_owned());
        font_family.insert(0, "YuGothic".to_owned());
        font_family.insert(1, "Meiryo".to_owned());
    }
    
    // フォント設定を適用
    ctx.set_fonts(fonts);
    
    // ライトテーマを明示的に設定
    ctx.set_visuals(egui::Visuals::light());
    
    // フォントとスタイルの設定
    ctx.style_mut(|style| {
        // フォントサイズを読みやすく調整
        for (_, font_id) in style.text_styles.iter_mut() {
            font_id.size = (font_id.size * 1.3).max(18.0); // さらに大きなフォント
        }
        
        // スペーシングの改善
        style.spacing.item_spacing.x = 10.0;
        style.spacing.item_spacing.y = 8.0;
        style.spacing.button_padding.x = 14.0;
        style.spacing.button_padding.y = 10.0;
        
        // より明確な色設定
        style.visuals.widgets.noninteractive.bg_fill = egui::Color32::WHITE;
        style.visuals.widgets.inactive.bg_fill = egui::Color32::from_gray(245);
        style.visuals.widgets.hovered.bg_fill = egui::Color32::from_gray(230);
        style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(100, 149, 237);
        
        // テキスト色を明確に
        style.visuals.widgets.noninteractive.fg_stroke.color = egui::Color32::BLACK;
        style.visuals.widgets.inactive.fg_stroke.color = egui::Color32::BLACK;
        style.visuals.widgets.hovered.fg_stroke.color = egui::Color32::BLACK;
        style.visuals.widgets.active.fg_stroke.color = egui::Color32::WHITE;
        
        // 背景色を明確に
        style.visuals.panel_fill = egui::Color32::from_gray(248);
        style.visuals.window_fill = egui::Color32::WHITE;
    });
    
    windows_console::println_japanese("GUI appearance and Japanese font setup completed");
}

/// システムフォントを読み込む
/// 
/// 事前条件:
/// - font_name は有効なフォント名である
/// 
/// 事後条件:
/// - 成功時: フォントデータが返される
/// - 失敗時: None が返される
#[cfg(windows)]
fn load_system_font(font_name: &str) -> Option<egui::FontData> {
    
    // Windowsのフォントディレクトリを確認
    let font_paths = [
        // Yu Gothic系
        "C:\\Windows\\Fonts\\YuGothR.ttc".to_string(),
        "C:\\Windows\\Fonts\\YuGothM.ttc".to_string(),
        "C:\\Windows\\Fonts\\YuGothB.ttc".to_string(),
        "C:\\Windows\\Fonts\\yugothic.ttf".to_string(),
        "C:\\Windows\\Fonts\\yugothm.ttc".to_string(),
        // Meiryo系
        "C:\\Windows\\Fonts\\meiryo.ttc".to_string(),
        "C:\\Windows\\Fonts\\meiryob.ttc".to_string(),
        // MS Gothic系
        "C:\\Windows\\Fonts\\msgothic.ttc".to_string(),
        "C:\\Windows\\Fonts\\MS_Gothic.ttf".to_string(),
        // Fallback
        format!("C:\\Windows\\Fonts\\{}.ttf", font_name.replace(" ", "")),
        format!("C:\\Windows\\Fonts\\{}.ttc", font_name.replace(" ", "")),
    ];
    
    for path in &font_paths {
        if let Ok(font_data) = std::fs::read(path) {
            return Some(egui::FontData::from_owned(font_data));
        }
    }
    
    None
}

#[cfg(not(windows))]
fn load_system_font(_font_name: &str) -> Option<egui::FontData> {
    None
}

/// GUIアプリケーションのエントリーポイント
/// 
/// 事前条件:
/// - eframe ライブラリが正常にリンクされている
/// - GUI環境が利用可能である
/// 
/// 事後条件:
/// - 成功時: Zoom録画ダウンローダーGUIアプリケーションが起動される
/// - Windows環境では日本語コンソール出力が適切に設定される
/// - 失敗時: 適切なエラーを返す
fn main() -> Result<(), eframe::Error> {
    // Windows環境でのコンソール文字化け対策
    windows_console::setup_console_encoding();
    
    // Application startup notification
    #[cfg(windows)]
    {
        windows_console::println_japanese("Starting GUI application...");
    }
    
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([820.0, 650.0])  // Optimized size for GUI display
            .with_title("Zoom Recording Downloader"),
        ..Default::default()
    };

    eframe::run_native(
        "Zoom Recording Downloader",
        options,
        Box::new(|cc| {
            // GUI appearance configuration for better visibility
            setup_gui_appearance(&cc.egui_ctx);
            
            Ok(Box::new(ZoomDownloaderApp::default()))
        }),
    )
}