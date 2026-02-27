# Rust開発ガイド - Zoom Video Mover

## プロジェクト構造
```
src/
├── main.rs            # GUIアプリケーションのエントリーポイント
├── lib.rs             # コアライブラリ
├── gui.rs             # GUI実装
├── errors.rs          # エラー型定義
├── services.rs        # サービスインターフェース（トレイト定義）
├── services_impl.rs   # サービス実装
├── windows_console.rs # Windows固有のコンソール処理
└── components/        # 機能別コンポーネント
    ├── mod.rs         # モジュール定義
    ├── api.rs         # API通信
    ├── auth.rs        # 認証処理
    ├── config.rs      # 設定管理
    ├── crypto.rs      # 暗号化処理
    ├── download.rs    # ダウンロード処理
    ├── integration.rs # 統合処理
    ├── recording.rs   # 録画管理
    └── ui.rs          # UIコンポーネント
```

## 主要依存関係
- **HTTP/OAuth**: `reqwest`, `oauth2`, `url`
- **GUI**: `eframe`, `egui`
- **非同期処理**: `tokio`
- **シリアライゼーション**: `serde`, `serde_json`
- **Windows固有**: `windows` crate

## ビルド・実行コマンド

```bash
cargo run              # GUIアプリケーション実行
cargo build --release  # リリースビルド
```

## テスト

```bash
cargo test                    # 全テスト実行
cargo test property_tests     # Property-basedテスト
cargo test tests              # 通常テスト
```

## リント・フォーマット・型チェック

```bash
cargo fmt                     # コードフォーマット
cargo clippy                  # 静的解析
cargo check                   # 型チェック
cargo check --all-targets     # 全ターゲットの型チェック
```

## コード変更時の品質チェック

ビルドに影響するファイル（`*.rs`, `Cargo.toml`, `build.rs`）を変更した際は必ず実行：

```bash
cargo check && cargo clippy && cargo fmt --check
```

## コーディング基本原則

- **型安全性**: Rustの型システムを最大限活用
- **メモリ安全性**: 所有権システムによる安全なメモリ管理
- **エラー安全性**: Result型による明示的エラーハンドリング
- **関数の長さ**: 50行以内（推奨25行以内）
- **循環的複雑度**: 関数あたり10以下
- **ネストレベル**: 4レベル以内

## 関数コメント規約

すべての**public関数**に以下を含むコメントを記載：

1. **事前条件**: 正常動作に必要な条件
2. **事後条件**: 実行後に保証される条件
3. **不変条件**: 実行中に維持される条件
4. **副作用**: 外部への影響（ファイル操作、ネットワーク通信等）

```rust
/// ユーザー認証を行い、トークンをファイルに保存する
///
/// # 副作用
/// - `config.toml`ファイルへの書き込み
/// - HTTPリクエストの送信
///
/// # 事前条件
/// - `client_id`と`client_secret`が空でない
///
/// # 事後条件
/// - 成功時：有効なAuthTokenが返される
///
/// # 不変条件
/// - 認証プロセス中にclient_idとclient_secretは変更されない
async fn authenticate_user(client_id: &str, client_secret: &str) -> Result<AuthToken, Error> {
    assert!(!client_id.is_empty(), "client_id must not be empty");
    // ...
}
```

## アサーション使い分け

- **`assert!`**: 常にチェックが必要な重要な条件（本番環境でも実行）
- **`debug_assert!`**: デバッグ時のみチェック（本番環境では無効化）
- **`unreachable!`**: 到達不可能なコードパス

## エラー型設計

```rust
#[derive(Debug, thiserror::Error)]
pub enum ZoomVideoMoverError {
    #[error("認証エラー: {message}")]
    Authentication { message: String },

    #[error("ネットワークエラー: {source}")]
    Network { #[from] source: reqwest::Error },

    #[error("ファイルシステムエラー: {operation} failed for {path}: {source}")]
    FileSystem { operation: String, path: PathBuf, #[source] source: std::io::Error },
}
```

## 品質基準

- **clippy警告**: 0件
- **関数コメント**: public関数100%準拠
- **assertion**: 事前・事後条件の適切な配置

## デバッグ・ログ

- `env_logger`を使用
- `RUST_LOG=debug cargo run`でデバッグログ出力
