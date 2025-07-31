// src/main.rs
// CLI アプリケーションのメインファイル

use zoom_video_mover_lib::windows_console;

fn main() {
    windows_console::setup_console_encoding();
    windows_console::println_japanese("Zoom Video Mover CLI");
    println!("Use --help for usage information");
}