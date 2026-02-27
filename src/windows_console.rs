//! Windows環境でのコンソールエンコーディングを設定する
//!
//! 事前条件:
//! - Windows環境で実行されている
//! - Windows API が利用可能である
//!
//! 事後条件:
//! - 成功時: コンソールのコードページがUTF-8(65001)に設定される
//! - 失敗時: 何もしない（エラーは無視される）
//!
//! 不変条件:
//! - この関数はWindows以外の環境では何もしない

/// 出力操作を表現する純粋なデータ構造
#[derive(Debug, Clone)]
pub struct OutputOperation {
    pub text: String,
    pub with_newline: bool,
}

impl OutputOperation {
    /// 新しい出力操作を作成する（副作用なし）
    ///
    /// 事前条件:
    /// - text は有効なUTF-8文字列である
    ///
    /// 事後条件:
    /// - 新しいOutputOperationインスタンスを返す
    /// - 副作用なし
    pub fn new(text: String, with_newline: bool) -> Self {
        Self { text, with_newline }
    }

    /// 出力操作を実行する（副作用あり）
    ///
    /// 事前条件:
    /// - setup_console_encoding() が事前に呼び出されている（推奨）
    ///
    /// 事後条件:
    /// - テキストがコンソールに出力される
    /// - Windows環境では標準出力バッファがフラッシュされる
    pub fn execute(&self) {
        if self.with_newline {
            println_japanese(&self.text);
        } else {
            print_japanese(&self.text);
        }
    }
}

/// 複数の出力操作をまとめて実行する（副作用あり）
///
/// 事前条件:
/// - operations は有効なOutputOperationのベクターである
///
/// 事後条件:
/// - 全ての出力操作が順次実行される
pub fn execute_output_operations(operations: Vec<OutputOperation>) {
    for operation in operations {
        operation.execute();
    }
}

#[cfg(windows)]
pub fn setup_console_encoding() {
    use windows::Win32::Foundation::INVALID_HANDLE_VALUE;
    use windows::Win32::System::Console::{
        GetStdHandle, SetConsoleCP, SetConsoleOutputCP, STD_OUTPUT_HANDLE,
    };

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

/// Windows以外の環境でのコンソールエンコーディング設定（何もしない）
///
/// 事前条件:
/// - Windows以外の環境で実行されている
///
/// 事後条件:
/// - 何もしない（通常、Unix系システムはUTF-8がデフォルト）
#[cfg(not(windows))]
pub fn setup_console_encoding() {
    // Windows以外では何もしない
}

/// Windows環境での日本語コンソール出力をサポートする関数
///
/// 事前条件:
/// - text は有効なUTF-8文字列である
/// - setup_console_encoding() が事前に呼び出されている（推奨）
///
/// 事後条件:
/// - 成功時: テキストが適切にコンソールに出力される
/// - Windows環境では標準出力バッファがフラッシュされる
/// - 失敗時: 出力は部分的に行われる可能性がある
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

/// 純粋関数版: Windows環境での日本語テキスト出力準備
///
/// 事前条件:
/// - text は有効なUTF-8文字列である
///
/// 事後条件:
/// - 副作用なし: 出力すべきテキストを返すのみ
/// - Windows環境での適切なエンコーディングが考慮される
///
/// 不変条件:
/// - 入力テキストを変更しない
/// - グローバル状態を変更しない
pub fn prepare_japanese_text(text: &str) -> String {
    // 純粋関数として、テキストの準備のみ行う
    // 実際の出力は呼び出し側で行う
    text.to_string()
}

/// Windows環境での日本語コンソール出力をサポートする関数（改行付き）
///
/// 事前条件:
/// - text は有効なUTF-8文字列である
/// - setup_console_encoding() が事前に呼び出されている（推奨）
///
/// 事後条件:
/// - 成功時: テキストが改行付きで適切にコンソールに出力される
/// - Windows環境では標準出力バッファがフラッシュされる
/// - 失敗時: 出力は部分的に行われる可能性がある
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

/// 純粋関数版: Windows環境での日本語テキスト出力準備（改行付き）
///
/// 事前条件:
/// - text は有効なUTF-8文字列である
///
/// 事後条件:
/// - 副作用なし: 改行付きテキストを返すのみ
/// - Windows環境での適切なエンコーディングが考慮される
///
/// 不変条件:
/// - 入力テキストを変更しない
/// - グローバル状態を変更しない
pub fn prepare_japanese_text_with_newline(text: &str) -> String {
    // 純粋関数として、改行付きテキストの準備のみ行う
    // 実際の出力は呼び出し側で行う
    format!("{}\n", text)
}

/// Windows環境の検出とUTF-8サポート確認
///
/// 事前条件:
/// - Windows環境で実行されている
/// - Windows API が利用可能である
///
/// 事後条件:
/// - 成功時: 現在のコンソールがUTF-8をサポートしている場合は true を返す
/// - 失敗時: false を返す
///
/// 不変条件:
/// - UTF-8コードページ(65001)の場合のみ true を返す
#[cfg(windows)]
pub fn is_utf8_supported() -> bool {
    use windows::Win32::System::Console::GetConsoleOutputCP;

    unsafe {
        // 現在のコードページを確認
        match GetConsoleOutputCP() {
            65001 => true, // UTF-8
            _ => false,
        }
    }
}

/// Windows以外の環境でのUTF-8サポート確認
///
/// 事前条件:
/// - Windows以外の環境で実行されている
///
/// 事後条件:
/// - 常に true を返す（Unix系システムは通常UTF-8対応）
#[cfg(not(windows))]
pub fn is_utf8_supported() -> bool {
    true // Windows以外は通常UTF-8対応
}
