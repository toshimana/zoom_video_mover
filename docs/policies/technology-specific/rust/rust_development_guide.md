# Rust開発ガイド - Zoom Video Mover

## プロジェクト構造
```
src/
├── main.rs          # GUIアプリケーションのエントリーポイント
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
# GUIアプリケーション
cargo run

# リリースビルド
cargo build --release
```

### テスト
```bash
# 全テスト実行
cargo test

# Property-basedテスト実行
cargo test property_tests

# 通常のテスト実行
cargo test tests
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

## Rustコーディング規約
**参照**: `docs/policies/rust_coding_standards.md`
- 関数コメント必須要素（事前条件・事後条件・不変条件・副作用）
- アサーション使用規約
- 品質チェックコマンド

## Rust技術固有ポリシー
詳細な技術実装方針は以下を参照：

### 実装ポリシー
- `docs/policies/technology-specific/rust_implementation_policy.md` - Rust実装方針
- `docs/policies/technology-specific/rust_coding_policy.md` - Rust詳細コーディング規約

### フレームワーク固有
- `docs/policies/technology-specific/rust_tokio_egui_design_policy.md` - Rust/tokio/eGUI技術設計
- `docs/policies/technology-specific/tokio_async_policy.md` - Tokio非同期処理ポリシー
- `docs/policies/technology-specific/egui_gui_policy.md` - eGUI GUI実装ポリシー

### テスト・品質
- `docs/policies/technology-specific/rust_proptest_testing_policy.md` - Rust/proptest/cargoテスト実装
- `docs/policies/technology-specific/cargo_clippy_policy.md` - Cargo/Clippy静的解析ポリシー

### プラットフォーム固有
- `docs/policies/technology-specific/windows_specific_policy.md` - Windows固有処理ポリシー

### エラー処理
- `docs/policies/technology-specific/thiserror_error_policy.md` - thiserrorエラー処理ポリシー

## デバッグ・ログ
- `env_logger`を使用
- `RUST_LOG=debug cargo run`でデバッグログ出力

## 実装変更時の注意点
実装変更時は、事前条件・事後条件・不変条件の維持とテスト更新が必要です。
詳細は `docs/policies/rust_coding_standards.md` を参照してください。