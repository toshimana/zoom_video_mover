# 🚀 クイックスタートガイド - Zoom Video Mover

このガイドでは、30分以内に開発環境をセットアップし、最初のビルドを実行できます。

## 📋 前提条件

- **Rust**: 1.70以上（[rustup.rs](https://rustup.rs/)からインストール）
- **Git**: 最新版
- **エディタ**: VSCode推奨（rust-analyzer拡張機能付き）

## 🎯 5分で開発開始

### 1. リポジトリをクローン
```bash
git clone <repository-url>
cd zoom_video_mover
```

### 2. 依存関係をインストール
```bash
cargo build
```

### 3. アプリケーションを実行
```bash
cargo run
```

初回実行時に`config.toml`が自動生成されます。

## ⚙️ Zoom OAuth設定（必須）

### 1. Zoom App Marketplaceでアプリ作成
1. [Zoom App Marketplace](https://marketplace.zoom.us/)にアクセス
2. "Develop" → "Build App" → "OAuth"を選択
3. 以下のスコープを追加：
   - `recording:read`
   - `user:read`
   - `meeting:read`（AI要約使用時）

### 2. config.tomlを編集
```toml
client_id = "YOUR_CLIENT_ID"
client_secret = "YOUR_CLIENT_SECRET"
redirect_uri = "http://localhost:8080/callback"
```

## 🔧 必須コマンド チートシート

| コマンド | 説明 | 使用タイミング |
|---------|------|--------------|
| `cargo run` | アプリケーション実行 | 開発・デバッグ時 |
| `cargo build --release` | リリースビルド | 配布用バイナリ作成時 |
| `cargo test` | 基本テスト実行（lib + integration） | コミット前 |
| `cargo test --features test-support` | 全テスト実行（GUI含む） | コミット前 |
| `cargo test --test gui_tests --features test-support` | GUIテストのみ | GUI変更時 |
| `cargo fmt` | コードフォーマット | コード変更後 |
| `cargo clippy` | 静的解析 | コミット前 |
| `cargo check` | 型チェック | コンパイルエラー確認時 |

## 🐛 よくあるエラーと対処法

### Windows環境

**エラー**: `error: linker 'link.exe' not found`
```bash
# 解決方法: Visual Studio Build Toolsをインストール
# https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022
```

**エラー**: 日本語文字化け
```bash
# 解決方法: 環境変数を設定
set RUST_LOG=debug
```

### Mac/Linux環境

**エラー**: OpenSSL関連のビルドエラー
```bash
# Mac
brew install openssl pkg-config

# Ubuntu/Debian
sudo apt-get install libssl-dev pkg-config
```

### 共通

**エラー**: `cargo test`で大量のテストケース
```bash
# 解決方法: テストケース数を制限
PROPTEST_CASES=10 cargo test
```

**エラー**: OAuth認証失敗
- `config.toml`の設定を確認
- Zoom App Marketplaceでredirect_uriが一致しているか確認
- クライアントID/シークレットが正しいか確認

## 📚 詳細ドキュメント

### 必須（初日に読む）
- 🔴 **[development_checklist.md](docs/policies/development_checklist.md)** - 開発フローチェックリスト
- 🔴 **[git_workflow.md](docs/policies/git_workflow.md)** - Gitワークフロー
- 🔴 **[rust_development.md](docs/policies/rust_development.md)** - Rust開発ガイド・コーディング規約

### 機能開発時
- 🟡 **[rust_development.md](docs/policies/rust_development.md)** - Rust開発環境詳細
- 🟡 **[testing_strategy.md](docs/policies/testing_strategy.md)** - テスト戦略

### 必要に応じて参照
- 🟢 **[CLAUDE.md](CLAUDE.md)** - プロジェクト全体の構成
- 🟢 **[docs/policies/](docs/policies/)** - 各種ポリシー文書
- 🟢 **[README.md](README.md)** - 詳細なトラブルシューティング

## 💡 開発のヒント

### デバッグモード
```bash
# 詳細ログを出力
RUST_LOG=debug cargo run

# 特定モジュールのみデバッグ
RUST_LOG=zoom_video_mover::gui=debug cargo run
```

### 高速ビルド
```bash
# インクリメンタルビルドを有効化
export CARGO_INCREMENTAL=1

# 並列ビルド数を指定
cargo build -j 4
```

### テスト高速化
```bash
# 単体テストのみ実行
cargo test --lib

# 特定テストのみ実行
cargo test test_oauth_flow
```

## 🆘 ヘルプ・サポート

### 問題が解決しない場合

1. **[README.md](README.md)** のトラブルシューティングセクションを確認
2. `RUST_LOG=trace cargo run` で詳細ログを確認
3. エラーメッセージでGitHubのIssuesを検索
4. それでも解決しない場合は新規Issueを作成

## ✅ 次のステップ

開発環境のセットアップが完了したら：

1. **[development_checklist.md](docs/policies/development_checklist.md)** で開発フローを確認
2. 簡単な機能追加やバグ修正から始める
3. PRを作成して他の開発者からフィードバックを得る

---
**最終更新**: 2025-08-06  
**所要時間**: 約30分（Zoom OAuth設定含む）