# 📦 再利用可能なポリシーテンプレート

このディレクトリには、他プロジェクトでも再利用可能な汎用ポリシーテンプレートが含まれています。

## 📂 ディレクトリ構造

```
policies-template/
├── universal/           # 技術・業界非依存の汎用ポリシー
├── project-generic/     # プロジェクト汎用ポリシー
├── project-specific/    # プロジェクト固有ポリシー  
└── technology-specific/ # 技術スタック別ポリシー
    ├── rust/           # Rust言語固有
    ├── python/         # Python言語固有
    └── typescript/     # TypeScript言語固有
```

## 🎯 再利用レベル

### Universal（完全汎用）
- プロジェクト名以外の変更不要
- どんな技術スタックでも利用可能
- 例：Gitワークフロー、コードレビュー基準

### Project-Generic（プロジェクト汎用）
- 特定プロジェクト向けだが技術非依存
- 業務ルール・ドメイン知識を含む
- 例：ビジネスルール、用語集

### Project-Specific（プロジェクト固有）
- 特定プロジェクト・技術に完全依存
- 実装詳細を含む
- 例：API統合仕様、実装ガイド

### Technology-Specific（技術固有）
- 特定の言語・フレームワーク向け
- 技術的な詳細を含む
- 例：Rustコーディング規約、Pythonテスト戦略

## 🔧 使用方法

### 1. プロジェクト設定ファイルを作成
`.policy-config.yaml`をプロジェクトルートに配置

### 2. 必要なテンプレートを選択
universalから始めて、必要に応じてdomain-specific、tech-specificを追加

### 3. 変数を置換
`{{PROJECT_NAME}}`などのプレースホルダーを実際の値に置換

### 4. カスタマイズ
プロジェクト固有の要件に応じて調整

## 📝 テンプレート変数

| 変数名 | 説明 | 例 |
|--------|------|-----|
| `{{PROJECT_NAME}}` | プロジェクト名 | Zoom Video Mover |
| `{{LANGUAGE}}` | プログラミング言語 | Rust |
| `{{TEST_FRAMEWORK}}` | テストフレームワーク | proptest |
| `{{BUILD_TOOL}}` | ビルドツール | cargo |
| `{{TEST_CASES}}` | テストケース数 | 1000 |
| `{{COVERAGE_TARGET}}` | カバレッジ目標 | 80 |

## 🚀 クイックスタート

```bash
# 1. テンプレートをコピー
cp -r policies-template/universal/* docs/policies/

# 2. 設定ファイルを編集
vi .policy-config.yaml

# 3. 変数を置換（例：sedを使用）
find docs/policies -name "*.md" -exec sed -i 's/{{PROJECT_NAME}}/MyProject/g' {} \;
```

## 📊 メタデータ仕様

各ポリシーファイルの先頭に以下のメタデータを含めます：

```yaml
---
reusability: universal  # universal | domain-specific | tech-specific
version: 1.0.0
dependencies: []        # 依存する他のポリシー
customizable:          # カスタマイズ可能な項目
  - test_cases
  - coverage_target
---
```

## 🔄 更新とバージョン管理

- テンプレートの更新は後方互換性を保つ
- 破壊的変更はメジャーバージョンを上げる
- カスタマイズ部分は`custom/`ディレクトリで管理

## 📈 採用実績

このテンプレートを使用しているプロジェクト：
- Zoom Video Mover（オリジナル）
- （今後追加予定）

---
**バージョン**: 1.0.0  
**最終更新**: 2025-08-06