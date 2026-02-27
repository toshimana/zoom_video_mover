# Zoom Video Mover - プロジェクト要件仕様書

## 1. プロジェクト概要

### 目的
ZoomクラウドレコーディングをローカルにダウンロードするRust GUIアプリケーション

### 対象環境
- Windows（主要対象）
- 日本語環境対応
- OAuth 2.0認証

## 2. 機能要件

### 2.1 認証機能
- **Zoom OAuth 2.0認証**
  - Client ID、Client Secret、Redirect URI設定
  - アクセストークンの取得・更新
  - 認証情報の永続化（config.toml）

### 2.2 API権限（Scopes）
- `recording:read`: 録画ファイルアクセス
- `user:read`: ユーザー情報取得（必須）
- `meeting:read`: AI要約アクセス

### 2.3 ダウンロード機能
- **対象ファイル**
  - 動画ファイル（MP4等）
  - 音声ファイル（MP3等）
  - チャットログ
  - トランスクリプト
  - AI要約（Zoom AI Companion生成）

- **ダウンロード機能**
  - 複数ファイル同時ダウンロード
  - 進捗表示
  - ダウンロード先フォルダ指定
  - ファイル名の自動生成

### 2.4 ユーザーインターフェース
- **GUI（egui/eframe）**
  - 直感的な操作画面
  - リアルタイムダウンロード進捗表示
  - ファイル選択・フォルダ選択
  - 設定画面
  - 日本語フォント対応

## 3. 非機能要件

### 3.1 パフォーマンス
- 非同期処理によるダウンロード
- tokioランタイム使用
- Zoom API rate limit対応

### 3.2 信頼性
- エラーハンドリング
- ログ出力（env_logger）
- 設定ファイルの検証

### 3.3 セキュリティ
- OAuth認証情報の安全な保存
- APIキーの適切な管理
- HTTPSによる通信

### 3.4 国際化・ローカライゼーション
- Windows日本語環境対応
- 文字エンコーディング処理
- パス処理（日本語ファイル名対応）

## 4. 技術要件

### 4.1 プログラミング言語・フレームワーク
- **言語**: Rust
- **非同期ランタイム**: tokio
- **HTTP クライアント**: reqwest
- **OAuth**: oauth2 crate
- **GUI**: eframe/egui
- **シリアライゼーション**: serde/serde_json

### 4.2 依存関係
```toml
[dependencies]
reqwest = { version = "0.11", features = ["json", "rustls-tls", "stream"], default-features = false }
oauth2 = "4.4"
url = "2.4"
eframe = "0.28"
egui = "0.28"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
env_logger = "0.10"
log = "0.4"

[target.'cfg(windows)'.dependencies]
windows = { version = "0.58", features = ["Win32_System_Console", "Win32_Foundation"] }
```

### 4.3 プロジェクト構造
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

## 5. 品質要件

### 5.1 コーディング規約
- **関数設計**: 原則として副作用のない純粋関数
- **関数コメント**: 事前条件・事後条件・不変条件・副作用を明記
- **アサーション**: 実行時の条件チェック（assert!/debug_assert!）

### 5.2 テスト要件
- **Property-Based Testing**: proptest使用
- **関数単位テスト**: 事前条件・事後条件・不変条件の検証
- **日時検証**: 実際に存在する日付のみを使用
- **ラウンドトリップテスト**: シリアライゼーション可逆性

### 5.3 品質チェック
- **型安全性**: `cargo check`
- **静的解析**: `cargo clippy`
- **フォーマット**: `cargo fmt`
- **テスト**: `cargo test`

## 6. 設定・運用要件

### 6.1 設定ファイル
- **config.toml**: OAuth設定
  - client_id
  - client_secret
  - redirect_uri
  - 初回実行時自動生成

### 6.2 ビルド・実行
```bash
# GUIアプリケーション実行
cargo run

# リリースビルド（実行）
cargo run --release

# リリースビルド（ビルドのみ）
cargo build --release
```

### 6.3 ログ・デバッグ
- env_logger使用
- `RUST_LOG=debug cargo run`でデバッグ出力

## 7. 制約事項

### 7.1 API制限
- Zoom API rate limit
- OAuth認証フローの実装必須
- 権限スコープの適切な設定

### 7.2 環境制約
- Windows環境での文字エンコーディング
- ファイルパス処理（日本語対応）
- インターネット接続必須（OAuth・ダウンロード）

### 7.3 セキュリティ制約
- OAuth認証情報の適切な管理
- APIキーの漏洩防止
- HTTPS通信の強制

## 8. 将来拡張

### 8.1 機能拡張候補
- 他の会議プラットフォーム対応（Teams、Webex等）
- スケジュール機能（定期ダウンロード）
- クラウドストレージ連携

### 8.2 技術的改善
- パフォーマンス最適化
- UIの改善
- 多言語対応

## 9. 成功基準

### 9.1 機能面
- ✅ Zoom OAuth認証が正常に動作する
- ✅ 録画ファイルが確実にダウンロードできる
- ✅ GUIアプリケーションが安定動作する
- ✅ Windows日本語環境で文字化けしない

### 9.2 品質面
- ✅ 全テストが通る（unit test + property-based test）
- ✅ cargo clippy でwarningが出ない
- ✅ 型安全性が保たれている
- ✅ 適切なエラーハンドリングがされている

### 9.3 運用面
- ✅ ビルド・実行手順が明確
- ✅ トラブルシューティング情報が整備されている
- ✅ 設定方法が分かりやすい