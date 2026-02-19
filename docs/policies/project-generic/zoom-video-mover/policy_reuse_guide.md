# 📚 ポリシー再利用ガイド

本プロジェクトで作成したポリシーを他プロジェクトで再利用するためのガイドです。

## 🎯 概要

Zoom Video Moverプロジェクトで開発した高品質なポリシー群を、テンプレート化して他プロジェクトでも活用できるようにしました。

## 📂 ファイル構成

```
.policy-config.yaml         # プロジェクト固有の設定
policies-template/          # 再利用可能なテンプレート
├── README.md              # テンプレートの説明
├── universal/             # 完全汎用ポリシー
│   ├── git-workflow.md
│   ├── testing-strategy.md
│   └── code-review.md
├── domain-specific/       # 分野別ポリシー
│   ├── oauth-apps/
│   └── gui-apps/
└── tech-specific/         # 技術固有ポリシー
    └── rust/
        └── rust-coding-standards.md
```

## 🚀 クイックスタート（新規プロジェクト）

### Step 1: テンプレートをコピー
```bash
# リポジトリをクローン
git clone <this-repository> policy-templates
cd your-new-project

# テンプレートディレクトリをコピー
cp -r policy-templates/policies-template ./
cp policy-templates/.policy-config.yaml ./
```

### Step 2: 設定ファイルをカスタマイズ
`.policy-config.yaml`を編集してプロジェクト情報を設定：

```yaml
project:
  name: "Your Project Name"  # ← 変更
  type: "web-application"    # ← 変更

technology:
  language: "Python"          # ← 変更
  test_framework: "pytest"    # ← 変更
  build_tool: "pip"          # ← 変更
```

### Step 3: ポリシーを選択・適用
必要なポリシーを選んで、プロジェクトのdocsディレクトリにコピー：

```bash
# 汎用ポリシーを適用
mkdir -p docs/policies
cp policies-template/universal/*.md docs/policies/

# 技術固有ポリシーを適用（例：Python）
cp policies-template/tech-specific/python/*.md docs/policies/
```

### Step 4: 変数を置換
プレースホルダーを実際の値に置換：

```bash
# sedを使った一括置換（Linux/Mac）
find docs/policies -name "*.md" -exec sed -i \
  -e 's/{{PROJECT_NAME}}/Your Project Name/g' \
  -e 's/{{LANGUAGE}}/Python/g' \
  -e 's/{{TEST_FRAMEWORK}}/pytest/g' \
  -e 's/{{BUILD_TOOL}}/pip/g' \
  {} \;

# PowerShellを使った置換（Windows）
Get-ChildItem -Path "docs\policies" -Filter "*.md" -Recurse | ForEach-Object {
    (Get-Content $_.FullName) `
        -replace '{{PROJECT_NAME}}', 'Your Project Name' `
        -replace '{{LANGUAGE}}', 'Python' |
    Set-Content $_.FullName
}
```

## 📊 再利用レベル別ガイド

### 🟢 Universal（完全汎用）
**特徴**: プロジェクトタイプや技術に依存しない
**例**: 
- `git-workflow.md` - Gitワークフロー
- `code-review.md` - コードレビュー基準
- `testing-strategy.md` - テスト戦略

**使い方**:
1. そのままコピー
2. プロジェクト名だけ置換
3. 必要に応じて微調整

### 🟡 Domain-Specific（分野別）
**特徴**: 特定の分野・業界向け
**例**:
- `oauth-apps/` - OAuth認証アプリ向け
- `gui-apps/` - GUIアプリ向け

**使い方**:
1. 該当する分野のポリシーを選択
2. プロジェクト固有の部分を調整
3. 不要な部分を削除

### 🔴 Tech-Specific（技術固有）
**特徴**: 特定の言語・フレームワーク向け
**例**:
- `rust/rust-coding-standards.md`
- `python/python-coding-standards.md`

**使い方**:
1. 使用技術に対応するポリシーを選択
2. フレームワーク固有の部分を追加
3. バージョン情報を更新

## 🔄 既存プロジェクトへの適用

### 段階的導入アプローチ

#### Phase 1: 最小導入（1日）
```
1. git-workflow.md を導入
2. 既存のワークフローと調整
3. チームに周知
```

#### Phase 2: 基本導入（1週間）
```
1. code-review.md を追加
2. testing-strategy.md を追加
3. 既存ルールとマージ
```

#### Phase 3: 完全導入（1ヶ月）
```
1. 全ポリシーを評価
2. 必要なものを選択・導入
3. チーム教育実施
```

## 🛠️ カスタマイズ方法

### プロジェクト固有の拡張
```markdown
# docs/policies/custom/project-specific.md

---
extends: universal/git-workflow.md
---

## プロジェクト固有のルール

### デプロイブランチ
- `staging`: ステージング環境
- `production`: 本番環境

[プロジェクト固有の内容]
```

### オーバーライド設定
`.policy-config.yaml`で既存ポリシーを上書き：

```yaml
overrides:
  test_cases: 500  # デフォルト1000から変更
  coverage_target: 90  # デフォルト80から変更
```

## 📝 変数リファレンス

| 変数 | 説明 | デフォルト値 |
|------|------|------------|
| `{{PROJECT_NAME}}` | プロジェクト名 | - |
| `{{LANGUAGE}}` | プログラミング言語 | - |
| `{{TEST_FRAMEWORK}}` | テストフレームワーク | - |
| `{{BUILD_TOOL}}` | ビルドツール | - |
| `{{TEST_CASES}}` | Property-basedテストケース数 | 1000 |
| `{{COVERAGE_TARGET}}` | カバレッジ目標(%) | 80 |
| `{{LOAD_TEST_USERS}}` | 負荷テストユーザー数 | 100 |
| `{{REQUIRED_APPROVALS}}` | 必要な承認者数 | 1 |

## 🔍 トラブルシューティング

### 問題: 変数が置換されない
**解決策**: 
- 変数名が正確か確認（大文字小文字も含む）
- `{{`と`}}`の間にスペースがないか確認

### 問題: ポリシーが多すぎる
**解決策**:
- まずuniversalから3つだけ選んで開始
- 徐々に追加していく

### 問題: 既存ルールと矛盾
**解決策**:
- 既存ルールを優先
- 段階的に統合
- チームで議論して調整

## 📈 成功事例

### 事例1: 小規模Webアプリ
- **適用ポリシー**: universal/* のみ
- **所要時間**: 2時間
- **効果**: レビュー時間50%削減

### 事例2: 大規模エンタープライズ
- **適用ポリシー**: 全カテゴリから15個選択
- **所要時間**: 2週間（段階導入）
- **効果**: 品質指標30%改善

## 🤝 コントリビューション

### テンプレートの改善
1. 汎用性を高める変更を提案
2. 新しい分野のポリシーを追加
3. 変数化できる部分を特定

### フィードバック
- 使いにくい部分
- 不足している変数
- 新しいユースケース

## 📚 関連リソース

- [QUICKSTART.md](QUICKSTART.md) - プロジェクトのクイックスタート
- [DEVELOPMENT_CHECKLIST.md](DEVELOPMENT_CHECKLIST.md) - 開発チェックリスト
- [docs/policies/README.md](docs/policies/README.md) - 現在のポリシー一覧

---
**バージョン**: 1.0.0  
**最終更新**: 2025-08-06  
**ライセンス**: プロジェクトライセンスに準拠