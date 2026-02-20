# CLAUDE.md - Zoom Video Mover プロジェクト

## プロジェクト概要
ZoomクラウドレコーディングをローカルにダウンロードするGUIアプリケーション

## プロジェクト機能
主要機能: OAuth認証、録画ダウンロード（動画・音声・チャット・トランスクリプト）、AI要約、並列処理
詳細: [README.md](README.md)、[project_features_detailed.md](docs/requirements/project_features_detailed.md)

## Rust開発環境
**参照**: [docs/policies/rust_development.md](docs/policies/rust_development.md)
- プロジェクト構造・依存関係
- ビルド・テスト・品質チェックコマンド
- コーディング規約（関数コメント・アサーション）
- デバッグ・ログ設定

## 開発ポリシー・規約

| ポリシー | ファイル |
|---------|---------|
| Rust開発ガイド | [docs/policies/rust_development.md](docs/policies/rust_development.md) |
| テスト戦略 | [docs/policies/testing_strategy.md](docs/policies/testing_strategy.md) |
| Gitワークフロー | [docs/policies/git_workflow.md](docs/policies/git_workflow.md) |
| 人の判断ガイドライン | [docs/policies/human_judgment_guidelines.md](docs/policies/human_judgment_guidelines.md) |
| 開発チェックリスト | [docs/policies/development_checklist.md](docs/policies/development_checklist.md) |

## 要件定義（RDRA）

要件定義ドキュメント: `docs/requirements/` 配下
- Phase 0-6の要件定義（各Phaseが1ファイル）
- システム要件・機能仕様詳細・RDRAモデル分析
- トレーサビリティ: [docs/requirements/traceability_matrix.md](docs/requirements/traceability_matrix.md)
- 変更管理: [docs/requirements/change_management.md](docs/requirements/change_management.md)

## トラブルシューティング
- **詳細ガイド**: README.md のトラブルシューティングセクション
- **技術実装**: [docs/policies/rust_development.md](docs/policies/rust_development.md)
