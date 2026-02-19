# フォルダ再編計画

## 現在のポリシー文書の分類

### Level 1: 汎用（プロジェクト非依存、技術非依存）
- document_hierarchy_framework.md - ドキュメント体系化フレームワーク
- v_model_development_process.md - V字モデル開発プロセス
- policy_rule_hierarchy_guidelines.md - ポリシー・ルール階層管理
- requirements_policy.md - 要件定義方針
- design_policy.md - 設計方針
- testing_policy.md - テスト方針
- testing_strategy_policy.md - テスト戦略
- git_workflow_policy.md - Gitワークフロー
- project_quality_management_policy.md - 品質管理
- traceability_management_policy.md - トレーサビリティ管理
- component_based_development_process.md - コンポーネントベース開発
- component_management_policy.md - コンポーネント管理
- human_judgment_guidelines.md - 人的判断ガイドライン
- rdra_methodology.md - RDRA方法論

### Level 2: 技術固有（プロジェクト非依存、技術依存）
- rust_coding_standards.md - Rustコーディング規約
- plantuml_validation_policy.md - PlantUML検証
- technology-specific/rust_coding_policy.md
- technology-specific/rust_implementation_policy.md
- technology-specific/rust_proptest_testing_policy.md
- technology-specific/rust_tokio_egui_design_policy.md
- technology-specific/cargo_clippy_policy.md
- technology-specific/egui_gui_policy.md
- technology-specific/thiserror_error_policy.md
- technology-specific/tokio_async_policy.md
- technology-specific/windows_specific_policy.md

### Level 3: プロジェクト汎用（プロジェクト依存、技術非依存）
- terminology_glossary.md - Zoom Video Mover用語集

### Level 4: プロジェクト固有（プロジェクト依存、技術依存）
- technology-specific/zoom_oauth_functional_requirements.md

## 新しいフォルダ構造

```
/docs/
├── policies/                    # ポリシー層
│   ├── universal/               # Level 1: 汎用
│   ├── technology-specific/     # Level 2: 技術固有
│   ├── project-generic/         # Level 3: プロジェクト汎用
│   └── project-specific/        # Level 4: プロジェクト固有
│
├── rules/                        # ルール層
│   ├── universal/               
│   ├── technology-specific/     
│   ├── project-generic/         
│   └── project-specific/        
│
└── artifacts/                    # 成果物層
    ├── requirements/            
    ├── design/                  
    ├── implementation/          
    └── testing/                 
```

## 移動計画

1. 新フォルダ構造の作成
2. Level 1文書を /docs/policies/universal/ へ移動
3. Level 2文書を /docs/policies/technology-specific/ へ再編
4. Level 3文書を /docs/policies/project-generic/ へ移動
5. Level 4文書を /docs/policies/project-specific/ へ移動
6. 成果物を /docs/artifacts/ へ整理
7. 旧フォルダのクリーンアップ