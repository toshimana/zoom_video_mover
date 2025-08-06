# 📚 ドキュメント索引 - Zoom Video Mover

## 🎯 開発者向け必須文書

### 🔴 必読（開発開始前）
- **[クイックスタートガイド](../QUICKSTART.md)** - 30分で開発環境構築
- **[開発チェックリスト](../DEVELOPMENT_CHECKLIST.md)** - 品質チェック手順
- **[開発ポリシー一覧](policies/README.md)** - 優先度付きポリシー一覧
- **[Gitワークフロー](policies/git_workflow_policy.md)** - コミット・PR規則
- **[Rustコーディング標準](policies/rust_coding_standards.md)** - コーディング規約

### 🟡 機能開発時に参照
- **[プロジェクト機能仕様](../PROJECT_FEATURES.md)** - 全機能の詳細仕様
- **[Rust開発環境](../RUST_DEVELOPMENT.md)** - Rust固有の開発ガイド
- **[システムアーキテクチャ](design/system_architecture.md)** - 全体設計
- **[テスト戦略](policies/testing_strategy_policy.md)** - Property-basedテスト

### 🟢 必要に応じて参照
- **[要件定義サマリ](requirements/system_requirements_summary.md)** - 要件全体像
- **[トレーサビリティ管理](policies/traceability_management_policy.md)** - 変更影響分析

## 📁 カテゴリ別ドキュメント

### 📋 要件定義 (27文書)
**メインディレクトリ**: `01-requirements/`

#### フェーズ別要件
- **[Phase 0: プロジェクト準備](01-requirements/phase0_project_preparation/)** - プロジェクト範囲・計画
- **[Phase 1: システム価値](01-requirements/phase1_system_value/)** - ビジネス価値・コンテキスト
- **[Phase 2: 外部環境](01-requirements/phase2_external_environment/)** - ビジネスフロー・概念モデル
- **[Phase 3: システム境界](01-requirements/phase3_system_boundary/)** - API・UI・ユースケース仕様
- **[Phase 4: システム内部](01-requirements/phase4_system_internal/)** - データモデル・処理アルゴリズム
- **[Phase 5: 非機能要件](01-requirements/phase5_non_functional/)** - 性能・信頼性・セキュリティ
- **[Phase 6: 統合検証](01-requirements/phase6_integration/)** - 要件統合

#### 横断的要件
- **[変更管理](01-requirements/crosscutting/change_management.md)** - 変更プロセス
- **[トレーサビリティマトリックス](01-requirements/crosscutting/overall_traceability_matrix.md)** - 要件追跡
- **[リスク管理](01-requirements/crosscutting/risk_management.md)** - リスク対応

### 🏗️ 設計 (18文書)
**メインディレクトリ**: `02-design/`

#### アーキテクチャ設計
- **[システムアーキテクチャ](02-design/system_architecture.md)** - 全体構成
- **[データモデル設計](02-design/data_model_design.md)** - データ構造
- **[インターフェース設計](02-design/interface_design.md)** - API・UI設計
- **[セキュリティ設計](02-design/security_design.md)** - セキュリティ対策

#### コンポーネント設計
- **[認証コンポーネント](02-design/components/auth_component_design.md)** - OAuth認証
- **[ダウンロードコンポーネント](02-design/components/download_component_design.md)** - ファイルダウンロード
- **[UIコンポーネント](02-design/components/ui_component_design.md)** - GUI実装
- **[APIコンポーネント](02-design/components/api_component_design.md)** - Zoom API連携

#### UML図・図表
- **[Phase 1: 概念設計](02-design/diagrams/phase1/)** - ユースケース図、概念クラス図
- **[Phase 2: 詳細設計](02-design/diagrams/phase2/)** - 詳細クラス図、シーケンス図
- **[Phase 3: 実装設計](02-design/diagrams/phase3/)** - Rust実装クラス図、デプロイ図

### 💻 開発 (24文書)
**メインディレクトリ**: `03-development/`

#### 🔴 必須ポリシー
- **[Gitワークフロー](03-development/policies/git_workflow_policy.md)** - Git運用規則
- **[Rustコーディング標準](03-development/policies/rust_coding_standards.md)** - コード品質基準
- **[テスト戦略](03-development/policies/testing_strategy_policy.md)** - Property-basedテスト

#### 🟡 推奨ポリシー
- **[要件定義方針](03-development/policies/requirements_policy.md)** - RDRA手法
- **[設計方針](03-development/policies/design_policy.md)** - アーキテクチャ原則
- **[コンポーネント管理](03-development/policies/component_management_policy.md)** - コンポーネント戦略

#### 技術固有ポリシー
- **[Rust実装方針](03-development/policies/technology-specific/rust_implementation_policy.md)**
- **[Tokio非同期処理](03-development/policies/technology-specific/tokio_async_policy.md)**
- **[eGUI GUI実装](03-development/policies/technology-specific/egui_gui_policy.md)**
- **[Zoom OAuth機能要件](03-development/policies/technology-specific/zoom_oauth_functional_requirements.md)**

### 📊 品質分析 (12文書)
**メインディレクトリ**: `04-quality-analysis/`

#### 最新分析結果
- **[現在の課題](04-quality-analysis/policy_consistency_issues.md)** - 対応必要な課題
- **[優先対応項目](04-quality-analysis/priority_action_items.md)** - アクションプラン

#### 一貫性分析
- コンポーネント分析レポート
- 設計実装整合性分析
- ポリシー一貫性検証

### 📝 実装 (準備中)
**メインディレクトリ**: `05-implementation/`
- 実装詳細文書（将来追加予定）
- 運用・デプロイメント手順
- 実装レビュー結果

## 🔍 ドキュメント検索

### よく使用される検索キーワード
- **OAuth認証**: `02-design/components/auth_component_design.md`, `03-development/policies/technology-specific/zoom_oauth_functional_requirements.md`
- **ダウンロード機能**: `02-design/components/download_component_design.md`, `02-design/components/recording_component_design.md`
- **テスト**: `03-development/policies/testing_strategy_policy.md`, `03-development/policies/technology-specific/rust_proptest_testing_policy.md`
- **エラーハンドリング**: `02-design/error_handling_design.md`, `03-development/policies/technology-specific/thiserror_error_policy.md`
- **Windows対応**: `03-development/policies/technology-specific/windows_specific_policy.md`

### ファイル名で検索
```bash
# 特定のキーワードを含むファイルを検索
find docs -name "*oauth*" -type f
find docs -name "*test*" -type f
find docs -name "*design*" -type f
```

## 📈 ドキュメント統計

| カテゴリ | ディレクトリ | ファイル数 | 主な用途 |
|---------|------------|-----------|----------|
| 要件定義 | 01-requirements | 27 | 機能仕様・制約条件の確認 |
| 設計 | 02-design | 18 | アーキテクチャ・実装方針 |
| 開発 | 03-development | 24 | 開発規約・品質基準 |
| 品質分析 | 04-quality-analysis | 12 | 品質確認・課題管理 |
| 実装 | 05-implementation | 準備中 | 実装詳細・運用手順 |
| アーカイブ | 99-archive | 8 | 過去レポート・履歴 |
| **合計** | **6カテゴリ** | **89+** | - |

## 🚀 新規開発者向けの学習パス

### Week 1: 基礎理解
1. [QUICKSTART.md](../QUICKSTART.md) で環境構築
2. [PROJECT_FEATURES.md](../PROJECT_FEATURES.md) で機能理解
3. [03-development/policies/README.md](03-development/policies/README.md) で必須ポリシー確認

### Week 2: 詳細理解
1. [システムアーキテクチャ](02-design/system_architecture.md) で全体設計理解
2. [02-design/components/](02-design/components/) で担当領域の詳細確認
3. [DEVELOPMENT_CHECKLIST.md](../DEVELOPMENT_CHECKLIST.md) で開発フロー習得

### Week 3以降: 実践
1. 小さな機能追加・バグ修正から開始
2. 必要に応じて関連ドキュメントを参照
3. PR作成・レビューを通じて品質基準を学習

---
**最終更新**: 2025-08-06  
**総ファイル数**: 94文書  
**メンテナ**: 開発チーム