# トレーサビリティ関連性マトリックス

## 概要

このマトリックスは、プロジェクト内の要件、設計、実装、テスト、画面、操作、機能仕様間の関連性を○×表で表現します。

## トレーサビリティ要素一覧

### 要件（Requirements）
- FR001-1: OAuth 2.0認証フロー
- FR001-2: Client ID/Secret設定
- FR001-3: トークン取得・更新
- FR001-4: 認証URL生成
- FR002-1: Zoom API呼び出し
- FR002-2: 録画リスト表示
- FR002-3: 期間フィルタリング
- FR002-4: ページネーション
- FR003-1: 並列ダウンロード
- FR003-2: 進捗表示
- FR003-3: ファイル種別対応
- FR003-4: ファイル名生成・サニタイズ
- FR004-1: AI要約API呼び出し
- FR004-2: 要約データ構造
- FR005-1: egui/eframe UI
- FR005-2: 設定画面
- FR005-3: ファイル選択
- FR006-1: CLI実行
- NFR001-1: レート制限対応
- NFR001-2: 指数バックオフリトライ
- NFR001-3: 同時ダウンロード数制限
- NFR002-1: エラーハンドリング
- NFR002-2: 詳細エラー分類
- NFR002-3: ログ出力
- NFR003-1: OAuth情報保護
- NFR003-2: HTTPS通信強制
- NFR004-1: Windows日本語対応
- NFR004-2: 日本語ファイル名

### 画面仕様（Screen）
- SC001: メイン画面
- SC002: 設定画面
- SC003: 認証画面
- SC004: 録画リスト画面
- SC005: ダウンロード進捗画面
- SC006: エラー表示画面

### 操作仕様（Operation）
- OP001: アプリケーション起動
- OP002: 設定入力・保存
- OP003: OAuth認証実行
- OP004: 録画検索・一覧表示
- OP005: ファイル選択
- OP006: ダウンロード実行
- OP007: 進捗監視・制御
- OP008: エラー処理・回復
- OP009: CLI実行

### 機能仕様（Function）
- FN001: 設定管理機能
- FN002: OAuth認証機能
- FN003: 録画検索機能
- FN004: ファイルダウンロード機能
- FN005: AI要約取得機能
- FN006: 進捗管理機能
- FN007: エラー処理機能
- FN008: ファイル管理機能
- FN009: ログ出力機能
- FN010: Windows対応機能

### 実装（Implementation）
- IMPL001: Config::load_from_file()
- IMPL002: Config::save_to_file()
- IMPL003: AuthToken::is_valid()
- IMPL004: generate_auth_url()
- IMPL005: exchange_code()
- IMPL006: get_recordings()
- IMPL007: download_file()
- IMPL008: sanitize_filename()
- IMPL009: ZoomDownloaderApp
- IMPL010: render_config()
- IMPL011: render_recordings()
- IMPL012: render_progress()
- IMPL013: rate_limit_check()
- IMPL014: handle_rate_limit_response()
- IMPL015: retry_with_exponential_backoff()

### テスト（Test）
- TEST001: config_roundtrip_property
- TEST002: oauth_token_validation_property
- TEST003: filename_sanitization_property
- TEST004: date_range_validation_property
- TEST005: recording_response_structure_property

## 関連性の定義

- **○**: 直接的な関連性あり（一方が他方を実装・実現・検証する）
- **△**: 間接的な関連性あり（共通の目的や機能で関連）
- **×**: 関連性なし

---

**次のステップ**: この情報を基に○×マトリックステーブルを生成します。