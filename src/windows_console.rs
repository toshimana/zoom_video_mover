#[cfg(windows)]
pub fn setup_console_encoding() {
    use windows::Win32::System::Console::{
        GetStdHandle, SetConsoleOutputCP, SetConsoleCP, STD_OUTPUT_HANDLE,
    };
    use windows::Win32::Foundation::INVALID_HANDLE_VALUE;
    
    unsafe {
        // UTF-8 code page (65001) を設定
        let _ = SetConsoleOutputCP(65001);
        let _ = SetConsoleCP(65001);
        
        // コンソールハンドルを取得してUTF-8モードを有効化
        match GetStdHandle(STD_OUTPUT_HANDLE) {
            Ok(handle) => {
                if handle != INVALID_HANDLE_VALUE {
                    // Windows 10 version 1903以降でサポートされているUTF-8モード
                    // コンソールの出力をUTF-8として処理するように設定
                    // 現在は基本的なコードページ設定のみ実装
                }
            }
            Err(_) => {
                // ハンドル取得に失敗した場合は何もしない
                // コードページ設定は既に実行済み
            }
        }
    }
}

#[cfg(not(windows))]
pub fn setup_console_encoding() {
    // Windows以外では何もしない
}

/// Windows環境での日本語コンソール出力をサポートする関数
pub fn print_japanese(text: &str) {
    #[cfg(windows)]
    {
        // Windows環境では、UTF-8エンコーディングで出力
        // flush()を追加してバッファリング問題を回避
        use std::io::{self, Write};
        print!("{}", text);
        let _ = io::stdout().flush();
    }
    
    #[cfg(not(windows))]
    {
        // その他の環境では通常通り出力
        print!("{}", text);
    }
}

/// Windows環境での日本語コンソール出力をサポートする関数（改行付き）
pub fn println_japanese(text: &str) {
    #[cfg(windows)]
    {
        // Windows環境では、UTF-8エンコーディングで出力
        // flush()を追加してバッファリング問題を回避
        use std::io::{self, Write};
        println!("{}", text);
        let _ = io::stdout().flush();
    }
    
    #[cfg(not(windows))]
    {
        // その他の環境では通常通り出力
        println!("{}", text);
    }
}

/// Windows環境の検出とUTF-8サポート確認
#[cfg(windows)]
pub fn is_utf8_supported() -> bool {
    use windows::Win32::System::Console::GetConsoleOutputCP;
    
    unsafe {
        // 現在のコードページを確認
        match GetConsoleOutputCP() {
            65001 => true,  // UTF-8
            _ => false,
        }
    }
}

#[cfg(not(windows))]
pub fn is_utf8_supported() -> bool {
    true // Windows以外は通常UTF-8対応
}