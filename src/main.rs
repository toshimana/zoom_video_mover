use eframe::egui;
use zoom_video_mover_lib::windows_console;

use zoom_video_mover_lib::ZoomDownloaderApp;


/// 日本語文字表示のためのフォント設定
/// 
/// 事前条件:
/// - ctx は有効なegui::Contextである
/// - windows_console::setup_console_encoding() が事前に呼び出されている
/// 
/// 事後条件:
/// - GUIのフォントサイズとスペーシングが読みやすく調整される
/// - フォントサイズが最小14ptに設定される
/// - 既存のフォント設定を保持し、サイズのみ調整される
fn setup_gui_fonts(ctx: &egui::Context) {
    windows_console::println_japanese("GUI font setup starting...");
    
    // Use eGUI's default font settings
    // Avoid custom font name specifications to prevent errors
    
    // Optimize font size and display settings only
    ctx.style_mut(|style| {
        // Adjust font size for better readability
        for (_, font_id) in style.text_styles.iter_mut() {
            font_id.size = (font_id.size * 1.1).max(14.0); // 10% larger, minimum 14pt
        }
        
        // Adjust spacing for better readability
        style.spacing.item_spacing.x *= 1.2;
        style.spacing.item_spacing.y *= 1.1;
        
        // Adjust button padding
        style.spacing.button_padding.x *= 1.3;
        style.spacing.button_padding.y *= 1.2;
    });
    
    windows_console::println_japanese("GUI display optimization completed");
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
            // GUI font configuration for Windows environment
            setup_gui_fonts(&cc.egui_ctx);
            
            Ok(Box::new(ZoomDownloaderApp::default()))
        }),
    )
}