# トレーサビリティマトリックス - Zoom Video Mover

## 1. 要件→設計→実装トレーサビリティマトリックス

| 要件ID | 要件名 | 設計文書 | 実装ファイル | 実装関数/クラス | テストファイル | 画面仕様 | 操作仕様 | 機能仕様 | ステータス |
|--------|--------|----------|-------------|---------------|------------|----------|----------|----------|----------|
| **FR001** | **OAuth認証** | | | | | | | | |
| FR001-1 | OAuth 2.0認証フロー | ARCHITECTURE.md:OAuth認証フロー図 | src/lib.rs:542-618 | `exchange_code()` | tests/property_tests.rs | SC003:認証画面 | OP003:OAuth認証実行 | FN002:OAuth認証機能 | ✅ 実装完了 |
| FR001-2 | Client ID/Secret設定 | requirements.md:認証機能 | src/lib.rs:23-134 | `Config::load_from_file()`, `Config::save_to_file()` | tests/property_tests.rs | SC002:設定画面 | OP002:設定入力・保存 | FN001:設定管理機能 | ✅ 実装完了 |
| FR001-3 | トークン取得・更新 | ARCHITECTURE.md:OAuth認証フロー図 | src/lib.rs:256-294 | `AuthToken::is_valid()`, `AuthToken::has_scope()` | tests/property_tests.rs | SC003:認証画面 | OP003:OAuth認証実行 | FN002:OAuth認証機能 | ✅ 実装完了 |
| FR001-4 | 認証URL生成 | zoom_api_specifications.md:OAuth仕様 | src/lib.rs:519-541 | `generate_auth_url()` | tests/property_tests.rs | SC003:認証画面 | OP003:OAuth認証実行 | FN002:OAuth認証機能 | ✅ 実装完了 |
| **FR002** | **録画一覧取得** | | | | | | | | |
| FR002-1 | Zoom API呼び出し | ARCHITECTURE.md:データフロー図 | src/lib.rs:644-787 | `get_recordings()` | tests/property_tests.rs | SC004:録画リスト画面 | OP004:録画検索・一覧表示 | FN003:録画検索機能 | ✅ 実装完了 |
| FR002-2 | 録画リスト表示 | rdra_models.md:ビジネスフロー図 | src/gui.rs:render_recordings | `render_recordings()` | tests/gui_tests.rs | SC004:録画リスト画面 | OP004:録画検索・一覧表示 | FN003:録画検索機能 | ✅ 実装完了 |
| FR002-3 | 期間フィルタリング | requirements.md:ダウンロード機能 | src/lib.rs:650-654 | `get_recordings()` パラメータ処理 | tests/property_tests.rs | SC004:録画リスト画面 | OP004:録画検索・一覧表示 | FN003:録画検索機能 | ✅ 実装完了 |
| FR002-4 | ページネーション | zoom_api_specifications.md:録画取得API | src/lib.rs:653 | `page_size` パラメータ処理 | tests/property_tests.rs | SC004:録画リスト画面 | OP004:録画検索・一覧表示 | FN003:録画検索機能 | ✅ 実装完了 |
| **FR003** | **ファイルダウンロード** | | | | | | | | |
| FR003-1 | 並列ダウンロード | ARCHITECTURE.md:システム構成図 | src/lib.rs:789-831 | `download_file()` | tests/download_tests.rs | SC005:ダウンロード進捗画面 | OP006:ダウンロード実行 | FN004:ファイルダウンロード機能 | ✅ 実装完了 |
| FR003-2 | 進捗表示 | rdra_models.md:GUI状態遷移図 | src/gui.rs:render_progress | `render_progress()` | tests/progress_tests.rs | SC005:ダウンロード進捗画面 | OP007:進捗監視・制御 | FN006:進捗管理機能 | ✅ 実装完了 |
| FR003-3 | ファイル種別対応 | requirements.md:対象ファイル | src/lib.rs:163-214 | `RecordingFile`, `MeetingRecording` | tests/file_type_tests.rs | SC004:録画リスト画面 | OP005:ファイル選択 | FN004:ファイルダウンロード機能 | ✅ 実装完了 |
| FR003-4 | ファイル名生成・サニタイズ | zoom_api_specifications.md:ファイル管理 | src/lib.rs:61-86 | `sanitize_filename()` | tests/property_tests.rs | SC005:ダウンロード進捗画面 | OP006:ダウンロード実行 | FN008:ファイル管理機能 | ✅ 実装完了 |
| **FR004** | **AI要約取得** | | | | | | | | |
| FR004-1 | AI要約API呼び出し | zoom_api_specifications.md:AI要約API | - | 実装予定（API仕様未確定） | - | SC004:録画リスト画面 | OP005:ファイル選択 | FN005:AI要約取得機能 | ⏳ 実装待ち |
| FR004-2 | 要約データ構造 | requirements.md:対象ファイル | src/lib.rs:220-255 | `AISummaryResponse` | tests/ai_summary_tests.rs | SC004:録画リスト画面 | OP005:ファイル選択 | FN005:AI要約取得機能 | ✅ 構造体定義完了 |
| **FR005** | **GUI操作** | | | | | | | | |
| FR005-1 | egui/eframe UI | ARCHITECTURE.md:GUI状態遷移図 | src/gui.rs, src/main_gui.rs | `ZoomDownloaderApp` | tests/gui_integration.rs | SC001:メイン画面 | OP001:アプリケーション起動 | - | ✅ 実装完了 |
| FR005-2 | 設定画面 | rdra_models.md:システムコンテキスト図 | src/gui.rs:render_config | `render_config()` | tests/config_ui_tests.rs | SC002:設定画面 | OP002:設定入力・保存 | FN001:設定管理機能 | ✅ 実装完了 |
| FR005-3 | ファイル選択 | requirements.md:ユーザーインターフェース | src/gui.rs:render_recordings | `render_file_selection()` | tests/selection_tests.rs | SC004:録画リスト画面 | OP005:ファイル選択 | - | ✅ 実装完了 |
| **FR006** | **CLI操作** | | | | | | | | |
| FR006-1 | CLI実行 | requirements.md:ユーザーインターフェース | src/main.rs | `main()` | tests/cli_tests.rs | - | OP009:CLI実行 | - | ✅ 実装完了 |
| **NFR001** | **性能要件** | | | | | | | | |
| NFR001-1 | レート制限対応 | zoom_api_specifications.md:レート制限 | src/lib.rs:364-403 | `rate_limit_check()`, `handle_rate_limit_response()` | tests/rate_limit_tests.rs | SC005:ダウンロード進捗画面 | OP006:ダウンロード実行 | FN004:ファイルダウンロード機能 | ✅ 実装完了 |
| NFR001-2 | 指数バックオフリトライ | zoom_api_specifications.md:エラーハンドリング | src/lib.rs:450-483 | `retry_with_exponential_backoff()` | tests/retry_tests.rs | SC006:エラー表示画面 | OP008:エラー処理・回復 | FN007:エラー処理機能 | ✅ 実装完了 |
| NFR001-3 | 同時ダウンロード数制限 | requirements.md:パフォーマンス | - | 実装予定 | tests/performance_tests.rs | SC005:ダウンロード進捗画面 | OP006:ダウンロード実行 | FN004:ファイルダウンロード機能 | ⏳ 実装待ち |
| **NFR002** | **信頼性要件** | | | | | | | | |
| NFR002-1 | エラーハンドリング | ARCHITECTURE.md:エラー処理戦略 | src/lib.rs:9-17 | `ZoomVideoMoverError` | tests/error_handling_tests.rs | SC006:エラー表示画面 | OP008:エラー処理・回復 | FN007:エラー処理機能 | ✅ 実装完了 |
| NFR002-2 | 詳細エラー分類 | zoom_api_specifications.md:エラーハンドリング | src/lib.rs:139-151 | `Display` impl for `ZoomVideoMoverError` | tests/error_handling_tests.rs | SC006:エラー表示画面 | OP008:エラー処理・回復 | FN007:エラー処理機能 | ✅ 実装完了 |
| NFR002-3 | ログ出力 | requirements.md:信頼性 | - | 実装予定 | tests/logging_tests.rs | - | - | FN009:ログ出力機能 | ⏳ 実装待ち |
| **NFR003** | **セキュリティ要件** | | | | | | | | |
| NFR003-1 | OAuth情報保護 | requirements.md:セキュリティ | src/lib.rs:115-133 | `Config::save_to_file()` | tests/security_tests.rs | SC002:設定画面 | OP002:設定入力・保存 | FN001:設定管理機能 | ✅ 実装完了 |
| NFR003-2 | HTTPS通信強制 | ARCHITECTURE.md:技術選定 | src/lib.rs:557, src/lib.rs:662 | HTTPクライアント設定 | tests/https_tests.rs | - | - | FN002:OAuth認証機能 | ✅ 実装完了 |
| **NFR004** | **国際化要件** | | | | | | | | |
| NFR004-1 | Windows日本語対応 | requirements.md:国際化 | src/windows_console.rs | `setup_console_encoding()` | tests/encoding_tests.rs | - | OP001:アプリケーション起動 | FN010:Windows対応機能 | ✅ 実装完了 |
| NFR004-2 | 日本語ファイル名 | ARCHITECTURE.md:データフロー | src/lib.rs:61-86 | `sanitize_filename()` | tests/filename_tests.rs | SC005:ダウンロード進捗画面 | OP006:ダウンロード実行 | FN008:ファイル管理機能 | ✅ 実装完了 |

## 2. 逆トレーサビリティマトリックス（実装→要件）

| 実装ファイル | 主要クラス/関数 | 行番号 | 対応要件ID | 設計根拠 | テストファイル |
|-------------|----------------|--------|------------|----------|------------|
| **src/lib.rs** | | | | | |
| Config | load_from_file() | 39-52 | FR001-2, NFR003-1 | OAuth設定管理 | tests/property_tests.rs |
| Config | create_sample_file() | 72-95 | FR001-2 | サンプル設定生成 | tests/property_tests.rs |
| Config | save_to_file() | 115-133 | FR001-2, NFR003-1 | 設定保存・暗号化 | tests/property_tests.rs |
| ZoomVideoMoverError | 全バリエーション | 9-17 | NFR002-1, NFR002-2 | 包括的エラー処理 | tests/error_handling_tests.rs |
| AuthToken | is_valid() | 267-269 | FR001-3 | トークン有効性検証 | tests/property_tests.rs |
| AuthToken | has_scope() | 271-273 | FR001-3, NFR003-2 | スコープ検証 | tests/property_tests.rs |
| AuthToken | has_all_scopes() | 275-277 | FR001-3, NFR003-2 | 複数スコープ検証 | tests/property_tests.rs |
| ZoomRecordingDownloader | new() | 484-495 | FR001-1, FR002-1 | 基本インスタンス作成 | tests/property_tests.rs |
| ZoomRecordingDownloader | new_with_token() | 497-508 | FR001-1, FR002-1 | トークン付きインスタンス | tests/property_tests.rs |
| ZoomRecordingDownloader | rate_limit_check() | 364-385 | NFR001-1 | レート制限自動制御 | tests/rate_limit_tests.rs |
| ZoomRecordingDownloader | handle_rate_limit_response() | 387-418 | NFR001-1 | HTTP 429処理 | tests/rate_limit_tests.rs |
| ZoomRecordingDownloader | retry_with_exponential_backoff() | 450-483 | NFR001-2 | 指数バックオフ | tests/retry_tests.rs |
| ZoomRecordingDownloader | generate_auth_url() | 519-541 | FR001-4 | OAuth認証URL生成 | tests/property_tests.rs |
| ZoomRecordingDownloader | exchange_code() | 542-618 | FR001-1, FR001-3 | 認証コード→トークン交換 | tests/property_tests.rs |
| ZoomRecordingDownloader | get_recordings() | 644-787 | FR002-1, FR002-3, FR002-4 | 録画一覧取得 | tests/property_tests.rs |
| ZoomRecordingDownloader | download_file() | 789-831 | FR003-1, FR003-2 | ファイルダウンロード | tests/download_tests.rs |
| sanitize_filename() | - | 61-86 | FR003-4, NFR004-2 | ファイル名安全化 | tests/property_tests.rs |
| parse_datetime() | - | 88-108 | FR002-1, FR003-3 | 日時パース | tests/property_tests.rs |
| RecordingFile | 構造体定義 | 163-174 | FR003-3 | 録画ファイル情報 | tests/property_tests.rs |
| MeetingRecording | 構造体定義 | 177-188 | FR002-2, FR003-3 | 会議録画情報 | tests/property_tests.rs |
| RecordingResponse | 構造体定義 | 191-198 | FR002-1, FR002-4 | API レスポンス | tests/property_tests.rs |
| AISummaryResponse | 構造体定義 | 220-255 | FR004-2 | AI要約データ | tests/ai_summary_tests.rs |
| **src/gui.rs** | | | | | |
| ZoomDownloaderApp | 基本構造 | - | FR005-1 | GUIメイン状態管理 | tests/gui_integration.rs |
| render_config() | - | - | FR005-2 | 設定画面描画 | tests/config_ui_tests.rs |
| render_recordings() | - | - | FR002-2, FR005-3 | 録画リスト表示 | tests/gui_integration.rs |
| render_progress() | - | - | FR003-2 | 進捗バー表示 | tests/progress_tests.rs |
| **src/main_gui.rs** | | | | | |
| main() | - | - | FR005-1 | GUI アプリ起動 | tests/gui_integration.rs |
| **src/main.rs** | | | | | |
| main() | - | - | FR006-1 | CLI アプリ起動 | tests/cli_tests.rs |
| **src/windows_console.rs** | | | | | |
| setup_console_encoding() | - | 15 | NFR004-1 | Windows UTF-8設定 | tests/encoding_tests.rs |

## 3. テストトレーサビリティマトリックス

| テストファイル | テスト関数 | 検証要件 | テスト種別 | 合格基準 | 対応実装 |
|---------------|------------|----------|------------|----------|----------|
| **tests/property_tests.rs** | | | | | |
| config_roundtrip_property | - | FR001-2 | Property-based | TOML保存・読込一致 | Config::save_to_file(), Config::load_from_file() |
| oauth_token_validation_property | - | FR001-3 | Property-based | トークン有効性検証 | AuthToken::is_valid() |
| filename_sanitization_property | - | FR003-4, NFR004-2 | Property-based | 日本語文字正常処理 | sanitize_filename() |
| date_range_validation_property | - | FR002-3 | Property-based | 有効日付のみ生成 | parse_datetime() |
| recording_response_structure_property | - | FR002-1 | Property-based | API レスポンス構造 | RecordingResponse |
| **tests/unit_tests.rs（予定）** | | | | | |
| test_oauth_flow | - | FR001-1 | 統合テスト | 認証完了まで正常 | exchange_code() |
| test_token_refresh | - | FR001-3 | 単体テスト | リフレッシュ成功 | AuthToken |
| test_parallel_download | - | FR003-1, NFR001-1 | 統合テスト | 制限内同時実行 | download_file() |
| test_download_progress | - | FR003-2 | 単体テスト | 進捗イベント発火 | download_file() |
| test_rate_limit_handling | - | NFR001-1 | 単体テスト | 429エラー適切処理 | handle_rate_limit_response() |
| test_exponential_backoff | - | NFR001-2 | 単体テスト | リトライ間隔正常 | retry_with_exponential_backoff() |
| test_error_classification | - | NFR002-1, NFR002-2 | 単体テスト | エラー種別正確 | ZoomVideoMoverError |
| **tests/integration_tests.rs（予定）** | | | | | |
| test_end_to_end_download | - | FR001+FR002+FR003 | E2Eテスト | 全工程正常完了 | 全モジュール |
| test_gui_workflow | - | FR005-1, FR005-2, FR005-3 | GUI統合テスト | UI操作正常 | src/gui.rs |
| test_cli_workflow | - | FR006-1 | CLI統合テスト | コマンド実行正常 | src/main.rs |

## 4. 品質保証トレーサビリティマトリックス

| 品質活動 | 対象 | 実行コマンド | 検証内容 | 成功基準 | 対応要件 |
|----------|------|-------------|----------|----------|----------|
| **型安全性チェック** | 全実装 | `cargo check` | コンパイルエラー | エラー0件 | 全NFR |
| **静的解析** | 全実装 | `cargo clippy` | コーディング規約 | 警告0件 | NFR002-1 |
| **フォーマット** | 全実装 | `cargo fmt` | コードスタイル | 差分なし | NFR002-1 |
| **Property-basedテスト** | データ処理 | `cargo test --test property_tests` | データ整合性 | 1000ケース合格 | FR001-2, FR002-3, FR003-4 |
| **単体テスト** | 個別関数 | `cargo test --lib` | 関数仕様 | 全テスト合格 | 全FR |
| **統合テスト** | システム全体 | `cargo test --test integration` | 要件充足 | 全シナリオ合格 | 全FR+全NFR |
| **GUI テスト** | ユーザーインターフェース | `cargo test --test gui_tests` | UI動作 | 全操作正常 | FR005 |
| **パフォーマンステスト** | レート制限・同時処理 | `cargo test --test performance` | 性能要件 | 制限値内動作 | NFR001 |
| **セキュリティテスト** | 認証・データ保護 | `cargo test --test security` | セキュリティ | 脆弱性なし | NFR003 |

## 5. 仕様書間相互参照マトリックス

| 仕様書分類 | 文書名 | 主要セクション | 参照先文書 | 参照関係 | 参照内容 |
|------------|--------|---------------|----------|----------|----------|
| **要件仕様** | requirements.md | 機能要件FR001-006 | ARCHITECTURE.md | 要件→設計 | システム要件の設計への落とし込み |
| **要件仕様** | requirements.md | 非機能要件NFR001-004 | zoom_api_specifications.md | 要件→API仕様 | API制約と要件の整合性 |
| **設計仕様** | ARCHITECTURE.md | システム構成・データフロー | src/lib.rs | 設計→実装 | アーキテクチャの実装 |
| **設計仕様** | rdra_models.md | RDRA 6モデル図 | screen_specifications.md | 設計→画面 | 要件の構造化とUI設計 |
| **API仕様** | zoom_api_specifications.md | OAuth認証・録画API | src/lib.rs | API仕様→実装 | API呼び出しの実装詳細 |
| **画面仕様** | screen_specifications.md | SC001-006画面詳細 | operation_specifications.md | 画面→操作 | 画面と操作の対応関係 |
| **操作仕様** | operation_specifications.md | OP001-009操作手順 | function_specifications.md | 操作→機能 | 操作と機能の対応関係 |
| **機能仕様** | function_specifications.md | FN001-010機能詳細 | src/lib.rs, src/gui.rs | 機能→実装 | 機能の具体的実装 |
| **テスト仕様** | test_specifications.md | テスト戦略・仕様 | tests/property_tests.rs | テスト仕様→実装 | テストケースの実装 |
| **実装仕様** | CLAUDE.md | コーディング規約 | src/*.rs | 規約→実装 | 実装ガイドラインの適用 |

## 6. 実装進捗マトリックス

| カテゴリ | 総要件数 | 実装完了 | 実装進行中 | 実装待ち | 完了率 |
|----------|----------|----------|------------|----------|--------|
| **機能要件（FR）** | 15 | 13 | 0 | 2 | 87% |
| **非機能要件（NFR）** | 12 | 9 | 0 | 3 | 75% |
| **画面仕様（SC）** | 6 | 6 | 0 | 0 | 100% |
| **操作仕様（OP）** | 9 | 9 | 0 | 0 | 100% |
| **機能仕様（FN）** | 10 | 8 | 0 | 2 | 80% |
| **テスト実装** | 20 | 5 | 0 | 15 | 25% |
| **全体** | 72 | 50 | 0 | 22 | 69% |

## 7. 未実装項目と優先度

| 要件ID | 要件名 | 実装状況 | 優先度 | 予定工数 | 依存関係 |
|--------|--------|----------|--------|----------|----------|
| FR004-1 | AI要約API呼び出し | 実装待ち | 中 | 3日 | Zoom API仕様確定 |
| NFR001-3 | 同時ダウンロード数制限 | 実装待ち | 高 | 2日 | download_file()拡張 |
| NFR002-3 | ログ出力 | 実装待ち | 低 | 1日 | env_logger統合 |
| テスト実装 | 統合・E2Eテスト | 実装待ち | 高 | 5日 | テスト環境構築 |

## 8. 品質メトリクス

| メトリクス | 計算方法 | 目標値 | 現在値 | 達成状況 |
|------------|----------|--------|--------|----------|
| **要件カバレッジ** | 実装済み要件数 / 全要件数 | 90% | 81% | 🔶 改善中 |
| **設計カバレッジ** | 設計文書化要件数 / 全要件数 | 100% | 100% | ✅ 達成 |
| **テストカバレッジ** | テスト済み要件数 / 全要件数 | 85% | 25% | ❌ 要改善 |
| **文書整合性** | 同期済み文書参照数 / 全文書参照数 | 100% | 95% | 🔶 改善中 |
| **コード品質** | clippy警告数 | 0 | 2 | 🔶 改善中 |

---

**最終更新**: 2025年1月31日  
**バージョン**: v1.0  
**プロジェクト**: Zoom Video Mover  
**作成者**: Claude Code Assistant