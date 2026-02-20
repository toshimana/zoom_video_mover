# 開発チェックリスト - Zoom Video Mover

## タスク開始前

### 環境準備
- [ ] 最新のmainブランチを取得
- [ ] 新規ブランチを作成（`feature/` or `fix/`）
- [ ] 依存関係の確認（`cargo update --dry-run`）

### 要件確認
- [ ] README.md で機能仕様を確認
- [ ] 影響範囲を特定
- [ ] 必要なAPIスコープを確認（Zoom OAuth）

## コーディング中

### コード記述
- [ ] `docs/policies/rust_development.md` に従う
- [ ] 関数に事前条件・事後条件をコメントで記載
- [ ] エラーハンドリングを適切に実装

### 定期確認（1時間ごと）
```bash
cargo check && cargo clippy
```

## テスト作成

- [ ] 単体テストを作成
- [ ] 境界値テスト・エラーケースを含める
- [ ] Property-basedテストを検討（複雑なロジック）

```bash
cargo test --lib          # 単体テスト
cargo test --test '*'     # 統合テスト
cargo test                # 全テスト
```

## コミット前の品質チェック

### 必須チェック
```bash
cargo fmt && cargo clippy -- -D warnings && cargo check --all-targets && PROPTEST_CASES=10 cargo test
```

### 手動確認
- [ ] 不要なデバッグコード（println!等）を削除
- [ ] TODOコメントを適切に処理
- [ ] ドキュメントコメントを追加（公開API）

## コミット・プッシュ

- [ ] `docs/policies/git_workflow.md` のコミット規則に従う
- [ ] 段階的にコミット（機能単位で）
- [ ] ローカルでビルド成功を確認（`cargo build --release`）

## クイックコマンド

### 基本チェック（3分）
```bash
cargo fmt && cargo clippy -- -D warnings && cargo test --lib
```

### 完全チェック（10分）
```bash
cargo fmt && cargo clippy -- -D warnings && cargo check --all-targets && cargo test && cargo build --release
```

### コミット前の最速チェック（1分）
```bash
cargo fmt && cargo check && PROPTEST_CASES=5 cargo test --lib
```

## 品質メトリクス目標

- テストカバレッジ: 80%以上
- Clippy警告: 0
- コンパイル警告: 0
- ドキュメントカバレッジ: 公開APIの100%
