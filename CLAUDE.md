# CLAUDE.md - Zoom Video Mover プロジェクト

## プロジェクト概要
ZoomクラウドレコーディングをローカルにダウンロードするGUIアプリケーション

## プロジェクト機能
主要機能: OAuth認証、録画ダウンロード（動画・音声・チャット・トランスクリプト）、AI要約、並列処理
詳細: [README.md](README.md)、[project_features_detailed.md](docs/requirements/project_features_detailed.md)

## Rust開発環境
**参照**: [docs/policies/rust_development.md](docs/policies/rust_development.md)
- プロジェクト構造・依存関係
- ビルド・テスト・品質チェックコマンド
- コーディング規約（関数コメント・アサーション）
- デバッグ・ログ設定

## 開発ポリシー・規約

| ポリシー | ファイル |
|---------|---------|
| Rust開発ガイド | [docs/policies/rust_development.md](docs/policies/rust_development.md) |
| テスト戦略 | [docs/policies/testing_strategy.md](docs/policies/testing_strategy.md) |
| Gitワークフロー | [docs/policies/git_workflow.md](docs/policies/git_workflow.md) |
| 人の判断ガイドライン | [docs/policies/human_judgment_guidelines.md](docs/policies/human_judgment_guidelines.md) |
| 開発チェックリスト | [docs/policies/development_checklist.md](docs/policies/development_checklist.md) |

## 要件定義（RDRA）

要件定義ドキュメント: `docs/requirements/` 配下
- Phase 0-6の要件定義（各Phaseが1ファイル）
- システム要件・機能仕様詳細・RDRAモデル分析
- トレーサビリティ: [docs/requirements/traceability_matrix.md](docs/requirements/traceability_matrix.md)
- 変更管理: [docs/requirements/change_management.md](docs/requirements/change_management.md)

## トラブルシューティング
- **詳細ガイド**: README.md のトラブルシューティングセクション
- **技術実装**: [docs/policies/rust_development.md](docs/policies/rust_development.md)

## ワークフロー設計

### 1. Planモードを基本とする
- 3ステップ以上 or アーキテクチャに関わるタスクは必ずPlanモードで開始する
- 途中でうまくいかなくなったら、無理に進めずすぐに立ち止まって再計画する
- 構築だけでなく、検証ステップにもPlanモードを使う
- 曖昧さを減らすため、実装前に詳細な仕様を書く

### 2. サブエージェント戦略
- メインのコンテキストウィンドウをクリーンに保つためにサブエージェントを積極的に活用する
- リサーチ・調査・並列分析はサブエージェントに任せる
- 複雑な問題には、サブエージェントを使ってより多くの計算リソースを投入する
- 集中して実行するために、サブエージェント1つにつき1タスクを割り当てる

### 3. 自己改善ループ
- ユーザーから修正を受けたら必ず `tasks/lessons.md` にそのパターンを記録する
- 同じミスを繰り返さないように、自分へのルールを書く
- ミス率が下がるまで、ルールを徹底的に改善し続ける
- セッション開始時に、そのプロジェクトに関連するlessonsをレビューする

### 4. 完了前に必ず検証する
- 動作を証明できるまで、タスクを完了とマークしない
- 必要に応じてmainブランチと自分の変更の差分を確認する
- 「スタッフエンジニアはこれを承認するか？」と自問する
- テストを実行し、ログを確認し、正しく動作することを示す

### 5. エレガントさを追求する（バランスよく）
- 重要な変更をする前に「もっとエレガントな方法はないか？」と一度立ち止まる
- ハック的な修正に感じたら「今知っていることをすべて踏まえて、エレガントな解決策を実装する」
- シンプルで明白な修正にはこのプロセスをスキップする（過剰設計しない）
- 提示する前に自分の作業に自問自答する

### 6. 自律的なバグ修正
- バグレポートを受けたら、手取り足取り教えてもらわずにそのまま修正する
- ログ・エラー・失敗しているテストを見て、自分で解決する
- ユーザーのコンテキスト切り替えをゼロにする
- 言われなくても、失敗しているCIテストを修正しに行く

### 7. タスク完了後に自動コミット
- 一連のタスクが完了したら、ユーザーの指示を待たずに自動でgitコミットする
- コミット前に `cargo check && cargo clippy && cargo fmt --check` で品質チェックを実行する
- 品質チェックが失敗した場合は修正してからコミットする
- 機密情報（パスワード、APIキー等）が含まれていないことを確認する
- コミットメッセージは [Gitワークフロー](docs/policies/git_workflow.md) のテンプレートに従う
- ユーザーが明示的に禁止しない限り、必ずコミットする

---

## タスク管理

1. **まず計画を立てる**：チェック可能な項目として `tasks/todo.md` に計画を書く
2. **計画を確認する**：実装を開始する前に確認する
3. **進捗を記録する**：完了した項目を随時マークしていく
4. **変更を説明する**：各ステップで高レベルのサマリーを提供する
5. **結果をドキュメント化する**：`tasks/todo.md` にレビューセクションを追加する
6. **学びを記録する**：修正を受けた後に `tasks/lessons.md` を更新する

---

## コア原則

- **シンプル第一**：すべての変更をできる限りシンプルにする。影響するコードを最小限にする。
- **手を抜かない**：根本原因を見つける。一時的な修正は避ける。シニアエンジニアの水準を保つ。
- **影響を最小化する**：変更は必要な箇所のみにとどめる。バグを新たに引き込まない。
