# CLAUDE.md - Zoom Video Mover プロジェクト

## プロジェクト概要
ZoomクラウドレコーディングをローカルにダウンロードするRust GUIアプリケーション

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

## 📋 開発ポリシー・規約

詳細な開発ポリシーは以下の専用ファイルを参照：

### コーディング規約
**参照**: `docs/policies/rust_coding_standards.md`
- 関数コメント必須要素（事前条件・事後条件・不変条件・副作用）
- アサーション使用規約
- 品質チェックコマンド

### テスト戦略
**参照**: `docs/policies/testing_strategy_policy.md`
- Property-basedテスト基盤戦略（1000ケース以上）
- 日時・日付検証規約
- 品質保証指標

### Git ワークフロー
**参照**: `docs/policies/git_workflow_policy.md`
- 必須コミット規則（やりとり完了時の自動コミット必須）
- コミットメッセージテンプレート
- 品質チェック基準

### トレーサビリティ管理
**参照**: `docs/policies/traceability_management_policy.md`
- 二層トレーサビリティ管理体制
- 要件プロセス内・プロセス間トレーサビリティ
- 変更管理プロセス

### 人の判断ガイドライン
**参照**: `docs/policies/human_judgment_guidelines.md`
- Claude Code Assistant支援時の判断基準
- 自動判断可能事項・人間判断必須事項の区分
- 効果的な指示のコツ

### PlantUML構文チェック
**参照**: `docs/policies/plantuml_validation_policy.md`
- 構文チェック環境・コマンド
- 一般的なエラーと対策
- 品質保証基準

### プロジェクト品質管理
**参照**: `docs/policies/project_quality_management_policy.md`
- 矛盾・不整合の自動検出・報告プロセス
- 品質保証基盤（Property-basedテスト戦略）
- 継続的品質改善

## 要件と設計のトレーサビリティ

### 二層トレーサビリティ管理体制

#### 1. 要件プロセス内トレーサビリティ (`requirements_traceability_matrix.md`)
要件定義Phase0-6内での成果物間関係性を追跡：

**対象範囲**:
- Phase間の成果物関連付け: 前フェーズ成果物→次フェーズ成果物  
- Phase内の要件間整合性: ステークホルダー要求→システム要件→非機能要件
- 要件変更の要件プロセス内影響範囲分析

#### 2. 全体プロセス間トレーサビリティ (`overall_traceability_matrix.md`)  
要件→設計→実装→テスト間の一貫性を追跡：

**対象範囲**:
- 要件プロセス→設計プロセス: システム要件→アーキテクチャ設計
- 設計プロセス→実装プロセス: 詳細設計→Rustコード実装  
- 実装プロセス→テストプロセス: コード→単体・統合・E2Eテスト
- プロセス間変更影響の全体分析

#### トレーサビリティ管理ルール
**要件プロセス内**:
- Phase間成果物関連付けの維持
- 要件変更時の要件プロセス内影響分析  
- Phase完了時の要件整合性監査

**プロセス間**:
- 要件→設計→実装→テストの完全トレース
- プロセス間変更影響の全体分析
- 品質ゲート通過時の整合性確認

#### 品質指標
- **要件プロセス内完全性**: Phase間関連付け率 = 100%
- **プロセス間完全性**: 要件→テスト追跡可能率 = 100%  
- **更新完全性**: 変更時のマトリックス更新率 = 100%

### トレーサビリティマトリックス参照

#### 詳細トレーサビリティ情報
詳細なトレーサビリティ情報は以下の専用ファイルを参照：

**要件プロセス内トレーサビリティ**:
- `docs/requirements/crosscutting/requirements_traceability_matrix.md`
- Phase0-6内の成果物間関係性
- 要件変更の要件プロセス内影響分析

**プロセス間トレーサビリティ**:
- `docs/requirements/crosscutting/overall_traceability_matrix.md`
- 要件→設計→実装→テストの完全トレース
- プロセス間変更影響の全体分析

#### 変更管理プロセス

1. **要件変更時**: 要件プロセス内影響分析 → プロセス間影響分析 → 実装・テスト更新
2. **設計変更時**: プロセス間影響分析 → 実装・テスト更新  
3. **実装変更時**: 事前条件・事後条件・不変条件の維持 → テスト更新

詳細な変更管理手順は以下を参照：
- `docs/requirements/crosscutting/change_management.md`

## デバッグ・ログ
- `env_logger`を使用
- `RUST_LOG=debug cargo run`でデバッグログ出力

## トラブルシューティング参考
- README.mdの詳細なトラブルシューティングセクション参照
- 特にZoom OAuth設定とWindows環境の問題