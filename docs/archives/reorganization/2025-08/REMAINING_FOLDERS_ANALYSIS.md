# 残存フォルダの分析と処理方針

## 現在ルートに残っている「該当しない」フォルダ

### 🗂️ 整理が必要なフォルダ

#### 1. `3rdparty/`
**現状**: PlantUML JARファイルのみ
**意図**: サードパーティツール格納
**問題**: ツール用途なのにルートレベル
**提案**: `tools/plantuml/` へ移動

#### 2. `node_modules/` + `package.json` + `package-lock.json`
**現状**: PlantUML図生成用Node.js環境
**意図**: PlantUML図の自動生成ツール
**問題**: 開発ツールがルートに散在
**提案**: `tools/` フォルダにまとめて移動

#### 3. `plantuml/`
**現状**: PlantUMLソースファイル（.puml）
**意図**: プロジェクト固有の図表ソース
**問題**: 成果物なのにルートレベル
**提案**: `docs/artifacts/design/diagrams/source/` へ移動

#### 4. `policies-template/`
**現状**: ポリシーテンプレート集
**意図**: 再利用可能なポリシーテンプレート
**問題**: docs体系と重複・混在
**提案**: `docs/templates/` へ移動または統合

#### 5. `temp_workspace/`
**現状**: 作業用一時フォルダ
**意図**: 実験・草稿・テスト用作業領域
**問題**: 一時ファイルがルートに存在
**提案**: `.gitignore`追加 or `tools/workspace/` へ移動

#### 6. `分析.xmind`
**現状**: XMindマインドマップファイル
**意図**: プロジェクト分析資料
**問題**: 日本語ファイル名、形式不統一
**提案**: `docs/artifacts/requirements/analysis.xmind` へ移動

## 処理方針の詳細

### 🛠️ ツール統合: `tools/` フォルダ作成

```
tools/
├── plantuml/                   # PlantUMLツール一式
│   ├── plantuml.jar           # (3rdpartyから移動)
│   ├── node_modules/          # (ルートから移動)
│   ├── package.json           # (ルートから移動)  
│   ├── package-lock.json      # (ルートから移動)
│   └── generate.sh            # 図生成スクリプト
├── workspace/                 # 作業領域
│   ├── drafts/                # (temp_workspaceから移動)
│   ├── experiments/
│   ├── notes/
│   └── references/
└── README.md                  # ツール使用方法
```

### 📊 成果物統合: `docs/artifacts/` 強化

```
docs/artifacts/design/diagrams/
├── generated/                 # 生成された画像
│   ├── phase1/
│   ├── phase2/
│   └── phase3/
└── source/                    # PlantUMLソース
    ├── 01_system_value_hierarchy.puml
    ├── 02_system_context.puml
    ├── 03_business_flow.puml
    ├── 04_business_usecase.puml
    ├── 05_requirements_specification.puml
    └── 06_system_architecture.puml
```

### 📋 テンプレート統合: `docs/templates/`

```
docs/templates/
├── policies/                  # ポリシーテンプレート
│   ├── universal/
│   │   ├── code-review-template.md
│   │   ├── git-workflow-template.md
│   │   └── testing-strategy-template.md
│   └── technology-specific/
│       └── rust/
│           └── rust-coding-standards-template.md
└── README.md                 # テンプレート使用方法
```

## 各フォルダの意図と判定

### ✅ 維持すべき理由があるもの

**なし** - 全て適切な場所への移動が推奨

### 🔄 移動すべきもの

| フォルダ | 現在の意図 | 移動先 | 理由 |
|----------|------------|--------|------|
| `3rdparty/` | 外部ツール格納 | `tools/plantuml/` | ツール統合 |
| `node_modules/` | Node依存関係 | `tools/plantuml/` | ツール統合 |
| `plantuml/` | 図表ソース | `docs/artifacts/design/diagrams/source/` | 成果物統合 |
| `policies-template/` | テンプレート | `docs/templates/` | 体系統合 |
| `temp_workspace/` | 作業領域 | `tools/workspace/` | ツール統合 |
| `分析.xmind` | 分析資料 | `docs/artifacts/requirements/` | 成果物統合 |

### ❌ 削除検討すべきもの

- 古い実験ファイル（temp_workspace/内）
- 空のフォルダ
- 重複するテンプレート

## 最終的なルート構造（理想形）

```
PROJECT_ROOT/
├── README.md                   # プロジェクト概要
├── CLAUDE.md                   # AI開発指示書  
├── PROJECT_FEATURES.md         # 機能概要
├── QUICKSTART.md               # クイックスタート
├── Cargo.toml                  # Rustプロジェクト設定
├── Cargo.lock                  # 依存関係ロック
├── build.rs                    # ビルドスクリプト
├── src/                        # ソースコード
├── tests/                      # テストコード
├── docs/                       # 体系化ドキュメント
└── tools/                      # 開発ツール一式
    ├── plantuml/              # 図生成ツール
    └── workspace/             # 作業領域
```

## 処理の優先順位

### Phase 1: ツール統合（高優先度）
- 開発効率に直結するツール環境の整備

### Phase 2: 成果物統合（中優先度）  
- ドキュメント体系の完全性確保

### Phase 3: テンプレート統合（低優先度）
- 将来の再利用性向上

### Phase 4: クリーンアップ（低優先度）
- 不要ファイルの除去

## 質問への回答

**Q: これらのフォルダの意図は？**

**A**: 
- `3rdparty/`, `node_modules/`: **ツール環境** - PlantUML図生成用
- `plantuml/`: **図表ソース** - プロジェクト分析図のソースファイル
- `policies-template/`: **再利用テンプレート** - 他プロジェクト用ポリシー雛形
- `temp_workspace/`: **作業領域** - 実験・草稿・テスト用
- `分析.xmind`: **分析資料** - プロジェクト初期分析

**結論**: 全て有用だが、ルートレベルには適さない。適切な階層への移動により、整理された構造と機能性を両立できます。

この整理を実行しますか？