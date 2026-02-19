---
reusability: universal
version: 1.0.0
dependencies: []
customizable:
  - branch_naming_convention
  - commit_message_format
---

# Git ワークフロー ポリシー - {{PROJECT_NAME}}

## 目的
{{PROJECT_NAME}}プロジェクトにおけるGit運用の標準化と品質維持

## ブランチ戦略

### メインブランチ
- `main` (または `master`): プロダクション環境
- `develop`: 開発統合ブランチ（オプション）

### フィーチャーブランチ
```
feature/機能名
fix/バグ修正内容
hotfix/緊急修正内容
refactor/リファクタリング内容
docs/ドキュメント更新
```

## コミット規則

### コミットメッセージフォーマット
```
<type>: <subject>

[optional body]

[optional footer]
```

### Type一覧
- `feat`: 新機能
- `fix`: バグ修正
- `docs`: ドキュメントのみの変更
- `style`: コードの意味に影響しない変更（空白、フォーマット等）
- `refactor`: バグ修正や機能追加を伴わないコード変更
- `perf`: パフォーマンス改善
- `test`: テストの追加・修正
- `chore`: ビルドプロセスやツールの変更

### コミット前チェックリスト
- [ ] コードがコンパイル/ビルドできる
- [ ] テストが全て通る
- [ ] リンター/フォーマッターを実行済み
- [ ] 不要なデバッグコードを削除済み

## プルリクエスト（PR）

### PR作成前
1. 最新のメインブランチをマージ/リベース
2. コンフリクトを解決
3. テストを実行

### PRテンプレート
```markdown
## 概要
変更内容の簡潔な説明

## 変更理由
なぜこの変更が必要か

## 変更内容
- [ ] 変更点1
- [ ] 変更点2

## テスト方法
テストの実行方法と確認項目

## 関連Issue
Closes #XXX
```

## コードレビュー

### レビュー観点
1. **機能性**: 要件を満たしているか
2. **可読性**: コードが理解しやすいか
3. **保守性**: 将来の変更が容易か
4. **性能**: パフォーマンスの問題はないか
5. **セキュリティ**: セキュリティリスクはないか

### レビューコメントの書き方
- 建設的で具体的な提案を行う
- 「なぜ」を説明する
- 良い点も積極的にコメントする

## マージ戦略

### マージ方法
- Squash and merge: フィーチャーブランチの場合（推奨）
- Create a merge commit: リリースブランチの場合
- Rebase and merge: 単一コミットの修正の場合

### マージ条件
- [ ] すべてのレビュー承認を取得
- [ ] CIが全て成功
- [ ] コンフリクトが解決済み

## 緊急時の対応

### Hotfixフロー
1. mainブランチから`hotfix/`ブランチを作成
2. 修正を実装
3. 最小限のテストを実行
4. 直接mainにマージ
5. developブランチにも反映

## 付録

### .gitignoreテンプレート
```gitignore
# ビルド成果物
{{BUILD_OUTPUT_DIR}}/
target/
dist/
*.exe
*.dll
*.so
*.dylib

# IDE設定
.idea/
.vscode/
*.swp
*.swo
*~

# OS固有
.DS_Store
Thumbs.db

# 環境固有
.env
.env.local
*.log
```

### 便利なGitエイリアス
```bash
git config --global alias.st status
git config --global alias.co checkout
git config --global alias.br branch
git config --global alias.cm commit
git config --global alias.lg "log --graph --pretty=format:'%Cred%h%Creset -%C(yellow)%d%Creset %s %Cgreen(%cr) %C(bold blue)<%an>%Creset' --abbrev-commit"
```

---
**適用日**: {{POLICY_EFFECTIVE_DATE}}  
**レビュー周期**: 四半期ごと