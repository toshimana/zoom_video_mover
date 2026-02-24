use eframe::egui;
use zoom_video_mover_lib::windows_console;

use zoom_video_mover_lib::ZoomDownloaderApp;


/// GUI表示設定（シンプル・安全版）
/// 
/// 事前条件:
/// - ctx は有効なegui::Contextである
/// 
/// 事後条件:
/// - 明確なライトテーマが設定される
/// - フォントサイズと色が読みやすく調整される
/// - UI要素の可視性が向上する
fn setup_gui_appearance(ctx: &egui::Context) {
    windows_console::println_japanese("Setting up GUI appearance...");

    // 日本語フォント（NotoSansJP）を埋め込み設定
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "NotoSansJP".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/NotoSansJP-Regular.ttf")),
    );
    fonts.families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, "NotoSansJP".to_owned());
    fonts.families
        .get_mut(&egui::FontFamily::Monospace)
        .unwrap()
        .insert(1, "NotoSansJP".to_owned());
    ctx.set_fonts(fonts);

    // ライトテーマを明示的に設定
    ctx.set_visuals(egui::Visuals::light());
    
    // フォントとスタイルの設定
    ctx.style_mut(|style| {
        // フォントサイズを大幅に拡大（日本語の可読性向上）
        for (_, font_id) in style.text_styles.iter_mut() {
            font_id.size = (font_id.size * 1.5).max(20.0); // 大きなフォント
        }
        
        // スペーシングの改善
        style.spacing.item_spacing.x = 12.0;
        style.spacing.item_spacing.y = 10.0;
        style.spacing.button_padding.x = 16.0;
        style.spacing.button_padding.y = 12.0;
        
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
        
        // テキストのstroke幅を太くして視認性向上
        style.visuals.widgets.noninteractive.fg_stroke.width = 1.0;
        style.visuals.widgets.inactive.fg_stroke.width = 1.0;
        style.visuals.widgets.hovered.fg_stroke.width = 1.0;
        style.visuals.widgets.active.fg_stroke.width = 1.0;
    });
    
    windows_console::println_japanese("GUI appearance setup completed - using default fonts with enhanced visibility");
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