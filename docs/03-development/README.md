# 💻 開発ドキュメント

## 目的
開発時に参照する全ポリシー・ガイドライン・標準を集約

## 📂 構成

### 開発ポリシー
- **[policies/](policies/)** - 開発ポリシー・規約
  - 必須🔴: git_workflow_policy.md, rust_coding_standards.md, testing_strategy_policy.md
  - 推奨🟡: requirements_policy.md, design_policy.md, component_management_policy.md
  - 参考🟢: project_quality_management_policy.md, human_judgment_guidelines.md
  - 技術固有: technology-specific/ (Rust, tokio, eGUI等)

### コーディング標準
- **[standards/](standards/)** - コーディング標準・品質基準
  - (将来拡張予定: 言語別標準、フレームワーク別標準)

### 開発ガイド
- **[guides/](guides/)** - 開発ガイドライン・手順書
  - PFD作成ガイドライン等

## 🎯 使い方

### 新規開発者
1. policies/README.md で優先度確認
2. 🔴必須ポリシーから習得
3. 担当技術のtechnology-specific/ポリシーを確認

### 既存開発者
1. 機能開発時は関連ポリシーを参照
2. コードレビュー時は品質基準を確認
3. 不明点はguides/の手順書を参照

## 🔗 関連リンク
- **[QUICKSTART.md](../../QUICKSTART.md)** - クイックスタートガイド
- **[DEVELOPMENT_CHECKLIST.md](../../DEVELOPMENT_CHECKLIST.md)** - 開発チェックリスト
- **[RUST_DEVELOPMENT.md](../../RUST_DEVELOPMENT.md)** - Rust開発環境

---
**総ファイル数**: 24文書  
**更新頻度**: ポリシー更新時