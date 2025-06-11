# CLAUDE.md - Zoom Video Mover プロジェクト

## プロジェクト概要
ZoomクラウドレコーディングをローカルにダウンロードするRustアプリケーション（CLI・GUI両対応）

## プロジェクト構造
```
src/
├── main.rs          # CLIアプリケーションのエントリーポイント
├── main_gui.rs      # GUIアプリケーションのエントリーポイント
├── lib.rs           # コアライブラリ
├── gui.rs           # GUI実装
└── windows_console.rs # Windows固有のコンソール処理
```

## 主要依存関係
- **HTTP/OAuth**: `reqwest`, `oauth2`, `url`
- **GUI**: `eframe`, `egui`
- **非同期処理**: `tokio`
- **シリアライゼーション**: `serde`, `serde_json`
- **Windows固有**: `windows` crate

## ビルド・実行コマンド

### 開発・テスト
```bash
# CLIアプリケーション
cargo run

# GUIアプリケーション  
cargo run --bin zoom_video_mover_gui

# リリースビルド
cargo build --release
```

### テスト
```bash
cargo test
```

### リント・フォーマット・型チェック
```bash
cargo fmt
cargo clippy
cargo check  # 型チェック・コンパイルエラー確認
```

## コード変更時の品質チェック
ビルドに影響するファイル（`*.rs`, `Cargo.toml`, `build.rs`）を変更した際は、以下のコマンドを実行して型安全性を確認する：

```bash
# 型チェック（コンパイルせずに型エラーを確認）
cargo check

# より詳細な静的解析
cargo clippy

# すべてのターゲットとテストの型チェック
cargo check --all-targets
```

## 設定ファイル
- `config.toml`: Zoom OAuth設定（client_id, client_secret, redirect_uri）
- 初回実行時に自動生成される

## 主要機能
1. **OAuth認証**: Zoom APIアクセス用の認証処理
2. **録画ダウンロード**: 動画・音声・チャット・トランスクリプト
3. **AI要約**: Zoom AI Companion生成の会議要約
4. **Windows対応**: 日本語文字化け対策、パス処理
5. **GUI**: 直感的なユーザーインターface

## API権限（Scopes）
- `recording:read`: 録画ファイルアクセス
- `user:read`: ユーザー情報（必須）
- `meeting:read`: AI要約アクセス

## 開発時の注意点
- Windows環境での文字エンコーディング処理
- Zoom API rate limit対応
- OAuth flow の適切な実装
- エラーハンドリングの重要性

## デバッグ・ログ
- `env_logger`を使用
- `RUST_LOG=debug cargo run`でデバッグログ出力

## トラブルシューティング参考
- README.mdの詳細なトラブルシューティングセクション参照
- 特にZoom OAuth設定とWindows環境の問題