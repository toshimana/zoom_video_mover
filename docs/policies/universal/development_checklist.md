# ✅ 開発チェックリスト - Zoom Video Mover

このチェックリストを使用して、品質の高いコードを一貫して提供できます。

## 📝 タスク開始前

### 環境準備
- [ ] 最新のmainブランチを取得
  ```bash
  git checkout main
  git pull origin main
  ```
- [ ] 新規ブランチを作成
  ```bash
  git checkout -b feature/your-feature-name
  # または
  git checkout -b fix/your-bug-fix
  ```
- [ ] 依存関係が最新か確認
  ```bash
  cargo update --dry-run
  ```

### 要件確認
- [ ] **[PROJECT_FEATURES.md](PROJECT_FEATURES.md)** で機能仕様を確認
- [ ] 影響範囲を特定（どのモジュールに影響するか）
- [ ] 必要なAPIスコープを確認（Zoom OAuth）

## 💻 コーディング中

### コード記述
- [ ] **[rust_coding_standards.md](docs/policies/rust_coding_standards.md)** に従う
- [ ] 関数に事前条件・事後条件をコメントで記載
- [ ] エラーハンドリングを適切に実装
- [ ] 新しい依存関係を追加する場合は理由を明確に

### 定期的な確認（1時間ごと）
- [ ] コンパイルエラーがないか確認
  ```bash
  cargo check
  ```
- [ ] 未使用の変数・インポートがないか確認
  ```bash
  cargo clippy
  ```

## 🧪 テスト作成

### テスト実装
- [ ] 単体テストを作成
- [ ] 境界値テストを含める
- [ ] エラーケースをテスト
- [ ] Property-basedテストを検討（複雑なロジックの場合）

### テスト実行
- [ ] 単体テストを実行
  ```bash
  cargo test --lib
  ```
- [ ] 統合テストを実行
  ```bash
  cargo test --test '*'
  ```
- [ ] 全テストを実行
  ```bash
  cargo test
  ```

## 🔍 コミット前の品質チェック

### 必須チェック（自動化可能）
- [ ] コードフォーマット
  ```bash
  cargo fmt
  ```
- [ ] 静的解析（警告をエラーとして扱う）
  ```bash
  cargo clippy -- -D warnings
  ```
- [ ] 型チェック（全ターゲット）
  ```bash
  cargo check --all-targets
  ```
- [ ] テスト実行（簡易版）
  ```bash
  PROPTEST_CASES=10 cargo test
  ```

### 手動確認
- [ ] 不要なデバッグコード（println!等）を削除
- [ ] TODOコメントを適切に処理
- [ ] ドキュメントコメントを追加（公開API）

## 📤 コミット・プッシュ

### コミット作成
- [ ] 変更内容を確認
  ```bash
  git status
  git diff
  ```
- [ ] 段階的にコミット（機能単位で）
  ```bash
  git add -p  # 対話的に追加
  git commit -m "feat: 機能の簡潔な説明"
  ```
- [ ] **[git_workflow_policy.md](docs/policies/git_workflow_policy.md)** のコミット規則に従う

### プッシュ前の最終確認
- [ ] ローカルでビルドが成功することを確認
  ```bash
  cargo build --release
  ```
- [ ] 全テストが通ることを確認
  ```bash
  cargo test
  ```
- [ ] リモートにプッシュ
  ```bash
  git push origin feature/your-feature-name
  ```

## 🔄 プルリクエスト作成

### PR作成前
- [ ] ブランチが最新のmainと同期している
  ```bash
  git fetch origin
  git rebase origin/main
  ```
- [ ] コンフリクトを解決（必要な場合）
- [ ] 最終的な動作確認

### PR説明文
- [ ] 変更の概要を記載
- [ ] 関連するIssue番号をリンク
- [ ] テスト方法を記載
- [ ] スクリーンショット追加（UI変更の場合）

### レビュー対応
- [ ] レビューコメントに対応
- [ ] 必要に応じてコードを修正
- [ ] 修正後は再度品質チェックを実行

## 🚀 リリース準備

### 最終確認
- [ ] READMEの更新が必要か確認
- [ ] CHANGELOGの更新（大きな変更の場合）
- [ ] バージョン番号の更新（Cargo.toml）

### Windows固有の確認
- [ ] Windows環境でのビルド確認
- [ ] 日本語文字化けがないか確認
- [ ] パス区切り文字の処理確認

## 🔧 トラブルシューティング

### よくある問題と対処

**cargo testが遅い**
```bash
# テストケース数を減らす
PROPTEST_CASES=10 cargo test

# 特定のテストのみ実行
cargo test test_名前
```

**cargo clippyで大量の警告**
```bash
# 段階的に修正
cargo clippy --fix
```

**git rebaseでコンフリクト**
```bash
# 一旦中断して状態確認
git rebase --abort
# 個別にマージ
git merge origin/main
```

## 📊 品質メトリクス目標

- [ ] テストカバレッジ: 80%以上
- [ ] Clippy警告: 0
- [ ] コンパイル警告: 0
- [ ] ドキュメントカバレッジ: 公開APIの100%

## 🎯 クイックコマンド

一括実行用のコマンド集：

### 基本の品質チェック（3分）
```bash
cargo fmt && cargo clippy -- -D warnings && cargo test --lib
```

### 完全な品質チェック（10分）
```bash
cargo fmt && \
cargo clippy -- -D warnings && \
cargo check --all-targets && \
cargo test && \
cargo build --release
```

### コミット前の最速チェック（1分）
```bash
cargo fmt && cargo check && PROPTEST_CASES=5 cargo test --lib
```

---
**最終更新**: 2025-08-06  
**使用頻度**: 毎回の開発作業時  
**所要時間**: 基本チェック3-5分、完全チェック10-15分