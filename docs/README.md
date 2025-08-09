# ドキュメント構造

本プロジェクトのドキュメントは、階層的な体系に基づいて整理されています。

## 📂 フォルダ構造

```
/docs/
├── policies/                    # ポリシー層（方針・原則）
│   ├── universal/               # Level 1: 汎用（プロジェクト非依存、技術非依存）
│   ├── technology-specific/     # Level 2: 技術固有（プロジェクト非依存、技術依存）
│   │   ├── rust/               # Rust関連ポリシー
│   │   └── plantuml/           # PlantUML関連ポリシー
│   ├── project-generic/         # Level 3: プロジェクト汎用（プロジェクト依存、技術非依存）
│   │   └── zoom-video-mover/   # Zoom Video Moverビジネスポリシー
│   └── project-specific/        # Level 4: プロジェクト固有（プロジェクト依存、技術依存）
│       └── zoom-video-mover/   # Zoom Video Mover実装固有ポリシー
│
├── rules/                        # ルール層（実施規則）
│   ├── universal/               # Level 1: 汎用ルール
│   ├── technology-specific/     # Level 2: 技術固有ルール
│   │   └── rust/               # Rust実装ルール
│   ├── project-generic/         # Level 3: プロジェクト汎用ルール
│   └── project-specific/        # Level 4: プロジェクト固有ルール
│
├── artifacts/                    # 成果物層（実装結果）
│   ├── requirements/            # 要件定義成果物
│   ├── design/                  # 設計成果物
│   ├── implementation/          # 実装成果物
│   └── testing/                 # テスト成果物
│
├── archives/                    # 履歴層（スナップショット）
│   └── analysis/               # 過去の品質分析レポート
│
└── guides/                      # ガイド（開発手順書）
```

## 📋 ドキュメント階層の説明

### ポリシー（Policies）
**定義**: プロジェクトやシステムの基本方針、原則、哲学を定義  
**特徴**: 抽象度が高く、長期的に安定  
**例**: 品質方針、セキュリティ方針、開発方針

### ルール（Rules）
**定義**: ポリシーを実現するための具体的な規則、手順、ガイドライン  
**特徴**: 実行可能で検証可能な規則  
**例**: コーディング規約、命名規則、レビュー手順

### 成果物（Artifacts）
**定義**: ポリシーとルールに基づいて作成された実際の文書、設計書  
**特徴**: プロジェクト固有で具体的  
**例**: 要件定義書、設計書、テストレポート

### 履歴（Archives）
**定義**: 過去のバージョンや分析スナップショット等の履歴情報  
**特徴**: 時系列的に管理され、監査・トレンド分析に使用  
**例**: 過去の品質分析レポート、プロジェクトマイルストーンのスナップショット

## 🔢 依存性レベル

### Level 1: 汎用（Universal）
- **特徴**: あらゆるプロジェクト、技術で適用可能
- **再利用性**: ★★★★★
- **例**: V字モデル開発プロセス、品質管理ポリシー

### Level 2: 技術固有（Technology-Specific）
- **特徴**: 特定の技術・言語に依存、プロジェクトには非依存
- **再利用性**: ★★★★☆
- **例**: Rustコーディング規約、PlantUML検証ポリシー

### Level 3: プロジェクト汎用（Project-Generic）
- **特徴**: 特定プロジェクトの方針、技術には非依存
- **再利用性**: ★★★☆☆
- **例**: Zoom Video Mover用語集、ビジネスルール

### Level 4: プロジェクト固有（Project-Specific）
- **特徴**: 特定プロジェクト、特定技術に完全依存
- **再利用性**: ★★☆☆☆
- **例**: Zoom OAuth統合仕様、Windows固有処理

## 📚 主要ドキュメント

### 基本フレームワーク
- 📄 [ドキュメント体系化フレームワーク](policies/universal/document_hierarchy_framework.md)
- 📄 [V字モデル開発プロセス](policies/universal/v_model_development_process.md)
- 📄 [ポリシー・ルール階層管理ガイドライン](policies/universal/policy_rule_hierarchy_guidelines.md)

### 開発ポリシー
- 📄 [要件定義方針](policies/universal/requirements_policy.md)
- 📄 [設計方針](policies/universal/design_policy.md)
- 📄 [テスト方針](policies/universal/testing_policy.md)
- 📄 [品質管理方針](policies/universal/project_quality_management_policy.md)

### 技術固有ポリシー（Rust）
- 📄 [Rustコーディング規約](policies/technology-specific/rust/rust_coding_standards.md)
- 📄 [Rust実装ポリシー](policies/technology-specific/rust/rust_implementation_policy.md)
- 📄 [Tokio非同期処理](policies/technology-specific/rust/tokio_async_policy.md)

## 🔄 文書管理プロセス

1. **新規作成時**: 依存性レベルを判定し、適切なフォルダに配置
2. **更新時**: 影響分析を実施し、関連文書も更新
3. **レビュー**: レベルに応じた承認者がレビュー
4. **廃止**: 6ヶ月未使用の文書はアーカイブへ移動

## 📊 メリット

- **再利用性の向上**: Level 1, 2の文書は複数プロジェクトで活用可能
- **保守性の向上**: 依存関係が明確で影響分析が容易
- **品質の向上**: 上位レベルの品質基準が自動的に適用
- **知識管理**: 組織的なノウハウの体系化

## 🚀 使い方

1. **新規プロジェクト開始時**
   - Level 1の汎用ポリシーを確認
   - 使用技術に応じてLevel 2を選定
   - プロジェクト固有のLevel 3, 4を作成

2. **開発中**
   - ポリシー → ルール → 成果物の順で参照
   - 上位レベルの方針に従って詳細化

3. **技術変更時**
   - Level 2の文書を変更
   - Level 4の実装固有文書を更新
   - Level 1, 3は変更不要