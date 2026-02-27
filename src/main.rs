use eframe::egui;
use zoom_video_mover_lib::gui::setup_gui_appearance;
use zoom_video_mover_lib::windows_console;

use zoom_video_mover_lib::ZoomDownloaderApp;

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
            .with_inner_size([820.0, 650.0]) // Optimized size for GUI display
            .with_title("Zoom Recording Downloader"),
        ..Default::default()
    };

    eframe::run_native(
        "Zoom Recording Downloader",
        options,
        Box::new(|cc| {
            windows_console::println_japanese("Setting up GUI appearance...");
            // GUI appearance configuration for better visibility
            setup_gui_appearance(&cc.egui_ctx);
            windows_console::println_japanese(
                "GUI appearance setup completed - using default fonts with enhanced visibility",
            );

            Ok(Box::new(ZoomDownloaderApp::default()))
        }),
    )
}
