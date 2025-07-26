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

## コーディング規約

### 関数の副作用について
- **原則**: 関数は副作用がない（純粋関数）ように設計する
- **例外**: 要件達成のために副作用が必要な場合は、関数コメントに明記する
- **副作用の例**: ファイル操作、ネットワーク通信、グローバル状態の変更、標準出力への書き込みなど

### 関数コメントの必須要素
すべての関数には以下の要素を含むコメントを記載する：

#### 1. 事前条件（Preconditions）
- 関数が正常に動作するために満たすべき条件
- 引数の有効性、システム状態、依存関係など

#### 2. 事後条件（Postconditions）
- 関数実行後に保証される条件
- 戻り値の性質、システム状態の変化など

#### 3. 不変条件（Invariants）
- 関数実行中に常に維持される条件
- データ構造の整合性、状態の一貫性など

#### 4. 副作用（Side Effects）
- 関数が引き起こす外部への影響

#### 完全な関数コメント例
```rust
/// ユーザー認証を行い、トークンをファイルに保存する
/// 
/// # 副作用
/// - `config.toml`ファイルへの書き込み
/// - HTTPリクエストの送信
/// - 標準出力へのメッセージ表示
/// 
/// # 事前条件
/// - `client_id`と`client_secret`が空でない
/// - インターネット接続が利用可能
/// - `config.toml`への書き込み権限がある
/// 
/// # 事後条件
/// - 成功時：有効なAuthTokenが返される
/// - 失敗時：適切なエラーメッセージと共にErrorが返される
/// - 認証情報がファイルに保存される
/// 
/// # 不変条件
/// - 認証プロセス中にclient_idとclient_secretは変更されない
/// - ファイルシステムの整合性が保たれる
async fn authenticate_user(client_id: &str, client_secret: &str) -> Result<AuthToken, Error> {
    // 実装
}
```

## デバッグ・ログ
- `env_logger`を使用
- `RUST_LOG=debug cargo run`でデバッグログ出力

## 📝 必須：やりとり完了時の自動コミット

**CLAUDE は以下の指示に絶対従う必要がある：**

### 🚨 重要：毎回のコミット実行
- **すべてのやりとり完了時に必ずgitコミットを実行する**
- **例外は一切認めない**
- **忘れた場合は即座に実行する**

### 📋 コミット手順（必須実行）
```bash
# 1. 変更されたファイルをすべてステージング
git add .

# 2. コミットメッセージとして対話内容の要約を記載
git commit -m "[対話内容の要約] 🤖 Generated with Claude Code

Co-Authored-By: Claude <noreply@anthropic.com>"
```

### ⚠️ 強制実行規則
1. **どんな小さな変更でもコミットする**
2. **やりとりが完了したら即座にコミット**
3. **他のタスクより優先してコミット**
4. **ユーザーが明示的に禁止しない限り必ずコミット**
5. **コミットを忘れた場合は次の対話開始時に前回分をコミット**

### 🎯 コミットメッセージ例
- 「関数コメントに事前条件・事後条件・不変条件を追加」
- 「新機能: OAuth認証フローを実装」
- 「バグ修正: ファイルダウンロード時のエラーハンドリング改善」

## トラブルシューティング参考
- README.mdの詳細なトラブルシューティングセクション参照
- 特にZoom OAuth設定とWindows環境の問題