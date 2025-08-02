use eframe::egui;
use zoom_video_mover_lib::windows_console;

use zoom_video_mover_lib::ZoomDownloaderApp;


/// GUI表示設定（フォント・テーマ・可視性）
/// 
/// 事前条件:
/// - ctx は有効なegui::Contextである
/// - windows_console::setup_console_encoding() が事前に呼び出されている
/// 
/// 事後条件:
/// - 明確なライトテーマが設定される
/// - フォントサイズと色が読みやすく調整される
/// - UI要素の可視性が向上する
fn setup_gui_appearance(ctx: &egui::Context) {
    windows_console::println_japanese("Setting up GUI appearance...");
    
    // ライトテーマを明示的に設定
    ctx.set_visuals(egui::Visuals::light());
    
    // フォントとスタイルの設定
    ctx.style_mut(|style| {
        // フォントサイズを読みやすく調整
        for (_, font_id) in style.text_styles.iter_mut() {
            font_id.size = (font_id.size * 1.2).max(16.0); // より大きなフォント
        }
        
        // スペーシングの改善
        style.spacing.item_spacing.x = 8.0;
        style.spacing.item_spacing.y = 6.0;
        style.spacing.button_padding.x = 12.0;
        style.spacing.button_padding.y = 8.0;
        
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
    
    windows_console::println_japanese("GUI appearance setup completed");
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