# CLAUDE.md - Zoom Video Mover プロジェクト

## プロジェクト概要
ZoomクラウドレコーディングをローカルにダウンロードするGUIアプリケーション

## プロジェクト機能
プロジェクトの詳細な機能仕様については以下を参照：
**📄 [PROJECT_FEATURES.md](PROJECT_FEATURES.md)**

含まれる内容：
- 主要機能（OAuth認証、録画ダウンロード、AI要約）
- プラットフォーム対応（Windows/Mac/Linux）
- API権限要件（OAuth Scopes）
- 動作要件・制限事項
- セキュリティ考慮事項

## Rust開発環境
Rust固有の開発環境、ビルド手順、コーディング規約については以下を参照：
**📘 [rust_development_guide.md](docs/policies/technology-specific/rust/rust_development_guide.md)**

含まれる内容：
- プロジェクト構造
- 依存関係（cargo, tokio, egui等）
- ビルド・テスト・品質チェックコマンド
- Rustコーディング規約
- デバッグ・ログ設定


## 📋 開発ポリシー・規約

詳細な開発ポリシーは以下の専用ファイルを参照：

### 開発規約
**Rust開発**: [rust_development_guide.md](docs/policies/technology-specific/rust/rust_development_guide.md) - Rust固有の開発ガイド
**汎用規約**: `docs/policies/` フォルダ内の各ポリシー文書を参照

### テスト戦略
**参照**: `docs/policies/universal/testing_strategy_policy.md`
- Property-basedテスト基盤戦略（1000ケース以上）
- 日時・日付検証規約
- 品質保証指標

### Git ワークフロー
**参照**: `docs/policies/universal/git_workflow_policy.md`
- 必須コミット規則（やりとり完了時の自動コミット必須）
- コミットメッセージテンプレート
- 品質チェック基準

### トレーサビリティ管理
**参照**: `docs/policies/universal/traceability_management_policy.md`
- 二層トレーサビリティ管理体制
- 要件プロセス内・プロセス間トレーサビリティ
- 変更管理プロセス

### 人の判断ガイドライン
**参照**: `docs/policies/universal/human_judgment_guidelines.md`
- Claude Code Assistant支援時の判断基準
- 自動判断可能事項・人間判断必須事項の区分
- 効果的な指示のコツ

### PlantUML構文チェック
**参照**: `docs/policies/technology-specific/plantuml/plantuml_validation_policy.md`
- 構文チェック環境・コマンド
- 一般的なエラーと対策
- 品質保証基準

### プロジェクト品質管理
**参照**: `docs/policies/universal/project_quality_management_policy.md`
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
- 設計プロセス→実装プロセス: 詳細設計→コード実装  
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
- `docs/artifacts/requirements/crosscutting/requirements_traceability_matrix.md`
- Phase0-6内の成果物間関係性
- 要件変更の要件プロセス内影響分析

**プロセス間トレーサビリティ**:
- `docs/artifacts/requirements/crosscutting/overall_traceability_matrix.md`
- 要件→設計→実装→テストの完全トレース
- プロセス間変更影響の全体分析

#### 変更管理プロセス

1. **要件変更時**: 要件プロセス内影響分析 → プロセス間影響分析 → 実装・テスト更新
2. **設計変更時**: プロセス間影響分析 → 実装・テスト更新  
3. **実装変更時**: 事前条件・事後条件・不変条件の維持 → テスト更新

詳細な変更管理手順は以下を参照：
- `docs/artifacts/requirements/crosscutting/change_management.md`

## トラブルシューティング
- **詳細ガイド**: README.md のトラブルシューティングセクション
- **機能仕様**: [PROJECT_FEATURES.md](PROJECT_FEATURES.md)
- **技術実装**: [rust_development_guide.md](docs/policies/technology-specific/rust/rust_development_guide.md)