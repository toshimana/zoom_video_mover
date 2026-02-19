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
- **[システムアーキテクチャ](artifacts/architecture/system_architecture.md)** - 全体設計
- **[テスト戦略](policies/testing_strategy_policy.md)** - Property-basedテスト

### 🟢 必要に応じて参照
- **[要件定義サマリ](requirements/system_requirements_summary.md)** - 要件全体像
- **[トレーサビリティ管理](policies/traceability_management_policy.md)** - 変更影響分析

## 📁 カテゴリ別ドキュメント

### 📋 要件定義 (27文書)
**メインディレクトリ**: `requirements/`

#### フェーズ別要件
- **[Phase 0: プロジェクト準備](requirements/phase0_project_preparation/)** - プロジェクト範囲・計画
- **[Phase 1: システム価値](requirements/phase1_system_value/)** - ビジネス価値・コンテキスト
- **[Phase 2: 外部環境](requirements/phase2_external_environment/)** - ビジネスフロー・概念モデル
- **[Phase 3: システム境界](requirements/phase3_system_boundary/)** - API・UI・ユースケース仕様
- **[Phase 4: システム内部](requirements/phase4_system_internal/)** - データモデル・処理アルゴリズム
- **[Phase 5: 非機能要件](requirements/phase5_non_functional/)** - 性能・信頼性・セキュリティ
- **[Phase 6: 統合検証](requirements/phase6_integration/)** - 要件統合

#### 横断的要件
- **[変更管理](requirements/crosscutting/change_management.md)** - 変更プロセス
- **[トレーサビリティマトリックス](requirements/crosscutting/overall_traceability_matrix.md)** - 要件追跡
- **[リスク管理](requirements/crosscutting/risk_management.md)** - リスク対応

### 🏗️ 設計 (18文書)
**メインディレクトリ**: `artifacts/` (アーキテクチャ・コンポーネント・実装別に分割)

#### アーキテクチャ設計 (`artifacts/architecture/`)
- **[システムアーキテクチャ](artifacts/architecture/system_architecture.md)** - 全体構成
- **[データモデル設計](artifacts/architecture/data_model_design.md)** - データ構造
- **[インターフェース設計](artifacts/architecture/interface_design.md)** - API・UI設計
- **[セキュリティ設計](artifacts/architecture/security_design.md)** - セキュリティ対策
- **[パフォーマンス設計](artifacts/architecture/performance_design.md)** - 性能設計
- **[API仕様書](artifacts/architecture/api_specifications.md)** - API詳細仕様

#### コンポーネント設計 (`artifacts/components/`)
- **[認証コンポーネント](artifacts/components/design/auth_component_design.md)** - OAuth認証
- **[ダウンロードコンポーネント](artifacts/components/design/download_component_design.md)** - ファイルダウンロード
- **[UIコンポーネント](artifacts/components/design/ui_component_design.md)** - GUI実装
- **[APIコンポーネント](artifacts/components/design/api_component_design.md)** - Zoom API連携
- **[エラーハンドリング設計](artifacts/components/error_handling_design.md)** - エラー処理

#### UML図・図表
- **[アーキテクチャ図面](artifacts/architecture/diagrams/)** - システムコンテキスト、概念設計
- **[コンポーネント詳細図面](artifacts/components/diagrams/)** - 詳細クラス図、シーケンス図
- **[実装設計図面](artifacts/implementation/diagrams/)** - Rust実装クラス図、デプロイ図

### 💻 開発 (24文書)
**メインディレクトリ**: `policies/`

#### 🔴 必須ポリシー
- **[Gitワークフロー](policies/git_workflow_policy.md)** - Git運用規則
- **[テスト戦略](policies/testing_strategy_policy.md)** - Property-basedテスト
- **[セキュリティポリシー](policies/universal/security_policy.md)** - セキュリティ基準

#### 🟡 推奨ポリシー
- **[設計方針](policies/universal/design_policy.md)** - アーキテクチャ原則
- **[文書管理](policies/universal/documentation_management_policy.md)** - 文書品質管理
- **[パフォーマンス監視](policies/universal/performance_monitoring_policy.md)** - 性能管理

#### 技術固有ポリシー
- **[Rust実装方針](policies/technology-specific/rust_implementation_policy.md)**
- **[OAuth機能要件](policies/technology-specific/zoom_oauth_functional_requirements.md)**
- **[GUI実装](policies/technology-specific/egui_gui_policy.md)**

### 📊 品質分析 (12文書)
**メインディレクトリ**: `testing/`

#### 最新分析結果
- **[一貫性分析レポート](testing/policy_consistency_analysis.md)** - ポリシー整合性
- **[品質改善項目](testing/priority_action_items.md)** - アクションプラン

#### 品質検証
- **[設計実装整合性](testing/design_implementation_consistency_report.md)** - 設計と実装の一致性
- **[コンポーネント整合性](testing/component_consistency_analysis.md)** - コンポーネント間整合性
- **[文書一貫性](testing/document_consistency_check_report.md)** - 文書品質検証

### 📝 実装
**メインディレクトリ**: `artifacts/implementation/`
- **[実装進捗](artifacts/implementation/progress_tracking.csv)** - 実装状況追跡
- **[実装図面](artifacts/implementation/diagrams/)** - 実装レベル設計図

## 🔍 ドキュメント検索

### よく使用される検索キーワード
- **OAuth認証**: `artifacts/components/design/auth_component_design.md`, `policies/technology-specific/zoom_oauth_functional_requirements.md`
- **ダウンロード機能**: `artifacts/components/design/download_component_design.md`, `artifacts/components/design/recording_component_design.md`
- **テスト**: `policies/testing_strategy_policy.md`, `policies/technology-specific/rust_proptest_testing_policy.md`
- **エラーハンドリング**: `artifacts/components/error_handling_design.md`, `policies/technology-specific/thiserror_error_policy.md`
- **Windows対応**: `policies/technology-specific/windows_specific_policy.md`

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
| 要件定義 | requirements | 27 | 機能仕様・制約条件の確認 |
| 設計 | artifacts | 18+ | アーキテクチャ・実装方針 |
| 開発 | policies | 24 | 開発規約・品質基準 |
| 品質分析 | testing | 12 | 品質確認・課題管理 |
| ガイド | guides | 8 | 開発ガイド・手順書 |
| アーカイブ | archives | 8 | 過去レポート・履歴 |
| **合計** | **6カテゴリ** | **97+** | - |

## 🚀 新規開発者向けの学習パス

### Week 1: 基礎理解
1. [QUICKSTART.md](../QUICKSTART.md) で環境構築
2. [PROJECT_FEATURES.md](../PROJECT_FEATURES.md) で機能理解
3. [policies/README.md](policies/README.md) で必須ポリシー確認

### Week 2: 詳細理解
1. [システムアーキテクチャ](artifacts/architecture/system_architecture.md) で全体設計理解
2. [artifacts/components/design/](artifacts/components/design/) で担当領域の詳細確認
3. [DEVELOPMENT_CHECKLIST.md](../DEVELOPMENT_CHECKLIST.md) で開発フロー習得

### Week 3以降: 実践
1. 小さな機能追加・バグ修正から開始
2. 必要に応じて関連ドキュメントを参照
3. PR作成・レビューを通じて品質基準を学習

---
**最終更新**: 2025-08-06  
**総ファイル数**: 94文書  
**メンテナ**: 開発チーム