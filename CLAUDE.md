# CLAUDE.md - Zoom Video Mover プロジェクト

## プロジェクト概要
ZoomクラウドレコーディングをローカルにダウンロードするRustアプリケーション（CLI・GUI両対応）

## プロジェクト構造
```
src/
├── main.rs          # CLIアプリケーションのエントリーポイント
├── main_gui.rs      # GUIアプリケーションのエントリーポイント
├── lib.rs           # コアライブラリ
├── gui.rs           # GUI実装
└── windows_console.rs # Windows固有のコンソール処理
```

## 主要依存関係
- **HTTP/OAuth**: `reqwest`, `oauth2`, `url`
- **GUI**: `eframe`, `egui`
- **非同期処理**: `tokio`
- **シリアライゼーション**: `serde`, `serde_json`
- **Windows固有**: `windows` crate

## ビルド・実行コマンド

### 開発・テスト
```bash
# CLIアプリケーション
cargo run

# GUIアプリケーション  
cargo run --bin zoom_video_mover_gui

# リリースビルド
cargo build --release
```

### テスト
```bash
# 全テスト実行
cargo test

# Property-basedテスト実行
cargo test property_tests

# 通常のテスト実行
cargo test tests
```

### リント・フォーマット・型チェック
```bash
cargo fmt
cargo clippy
cargo check  # 型チェック・コンパイルエラー確認
```

## コード変更時の品質チェック
ビルドに影響するファイル（`*.rs`, `Cargo.toml`, `build.rs`）を変更した際は、以下のコマンドを実行して型安全性を確認する：

```bash
# 型チェック（コンパイルせずに型エラーを確認）
cargo check

# より詳細な静的解析
cargo clippy

# すべてのターゲットとテストの型チェック
cargo check --all-targets
```

## 設定ファイル
- `config.toml`: Zoom OAuth設定（client_id, client_secret, redirect_uri）
- 初回実行時に自動生成される

## 主要機能
1. **OAuth認証**: Zoom APIアクセス用の認証処理
2. **録画ダウンロード**: 動画・音声・チャット・トランスクリプト
3. **AI要約**: Zoom AI Companion生成の会議要約
4. **Windows対応**: 日本語文字化け対策、パス処理
5. **GUI**: 直感的なユーザーインターface

## API権限（Scopes）
- `recording:read`: 録画ファイルアクセス
- `user:read`: ユーザー情報（必須）
- `meeting:read`: AI要約アクセス

## 開発時の注意点
- Windows環境での文字エンコーディング処理
- Zoom API rate limit対応
- OAuth flow の適切な実装
- エラーハンドリングの重要性

## コーディング規約

### 関数の副作用について
- **原則**: 関数は副作用がない（純粋関数）ように設計する
- **例外**: 要件達成のために副作用が必要な場合は、関数コメントに明記する
- **副作用の例**: ファイル操作、ネットワーク通信、グローバル状態の変更、標準出力への書き込みなど

### 関数コメントの必須要素
すべての関数には以下の要素を含むコメントを記載する：

#### 1. 事前条件（Preconditions）
- 関数が正常に動作するために満たすべき条件
- 引数の有効性、システム状態、依存関係など

#### 2. 事後条件（Postconditions）
- 関数実行後に保証される条件
- 戻り値の性質、システム状態の変化など

#### 3. 不変条件（Invariants）
- 関数実行中に常に維持される条件
- データ構造の整合性、状態の一貫性など

#### 4. 副作用（Side Effects）
- 関数が引き起こす外部への影響

### アサーション（assertion）の追加
関数の事前条件・事後条件を実行時にチェックするため、適切なassertionを追加する：

#### 1. 事前条件のassertion
- 関数の開始時に`assert!`、`debug_assert!`を使用
- 引数の有効性、前提条件をチェック

#### 2. 事後条件のassertion  
- 関数の終了前に戻り値や結果の妥当性をチェック
- `debug_assert!`を使用（リリースビルドでは無効化）

#### 3. assertionの使い分け
- **`assert!`**: 常にチェックが必要な重要な条件
- **`debug_assert!`**: デバッグ時のみチェック（パフォーマンス重視）
- **`unreachable!`**: 到達不可能なコードパス

#### 完全な関数コメントとassertion例
```rust
/// ユーザー認証を行い、トークンをファイルに保存する
/// 
/// # 副作用
/// - `config.toml`ファイルへの書き込み
/// - HTTPリクエストの送信
/// - 標準出力へのメッセージ表示
/// 
/// # 事前条件
/// - `client_id`と`client_secret`が空でない
/// - インターネット接続が利用可能
/// - `config.toml`への書き込み権限がある
/// 
/// # 事後条件
/// - 成功時：有効なAuthTokenが返される
/// - 失敗時：適切なエラーメッセージと共にErrorが返される
/// - 認証情報がファイルに保存される
/// 
/// # 不変条件
/// - 認証プロセス中にclient_idとclient_secretは変更されない
/// - ファイルシステムの整合性が保たれる
async fn authenticate_user(client_id: &str, client_secret: &str) -> Result<AuthToken, Error> {
    // 事前条件のassertion
    assert!(!client_id.is_empty(), "client_id must not be empty");
    assert!(!client_secret.is_empty(), "client_secret must not be empty");
    debug_assert!(client_id.len() > 5, "client_id should be reasonable length");
    
    // 実装
    let token = perform_oauth_flow(client_id, client_secret).await?;
    save_token_to_file(&token)?;
    
    // 事後条件のassertion
    debug_assert!(!token.access_token.is_empty(), "returned token must be valid");
    debug_assert!(token.expires_at > chrono::Utc::now(), "token must not be expired");
    
    Ok(token)
}
```

## テスト戦略

### Property-Based Testing（プロパティベーステスト）
関数の性質（プロパティ）を定義し、多数のランダム入力で検証する手法：

#### 使用フレームワーク
- **`proptest`**: Rust標準のproperty-basedテストフレームワーク
- **`quickcheck`**: QuickCheck-style testing
- **`proptest-derive`**: 任意値生成の自動導出

#### テスト対象プロパティ
1. **ラウンドトリップ性質**: シリアライゼーション→デシリアライゼーションの可逆性
2. **不変条件**: データ構造の整合性が常に保たれる
3. **入力範囲**: 有効な入力範囲での動作保証
4. **境界条件**: エッジケースでの適切な動作
5. **冪等性**: 同じ操作を複数回実行しても結果が変わらない

#### 関数単位でのProperty-basedテスト設計
各関数の**事前条件・事後条件・不変条件**を基にテストケースを作成：

##### 1. 事前条件のProperty検証
```rust
proptest! {
    #[test]
    fn function_preconditions(input in arb_valid_input()) {
        // 事前条件: 入力パラメータの妥当性
        prop_assert!(!input.is_empty());
        prop_assert!(input.len() >= MIN_LENGTH);
        
        // 関数実行は事前条件が満たされた場合のみ
        let result = function_under_test(input);
        // ...
    }
}
```

##### 2. 事後条件のProperty検証
```rust
proptest! {
    #[test]
    fn function_postconditions(input in arb_input()) {
        let result = function_under_test(input).unwrap();
        
        // 事後条件: 結果の妥当性
        prop_assert!(!result.is_empty());
        prop_assert!(result.contains_expected_format());
        
        // 結果の一貫性
        let result2 = function_under_test(input.clone()).unwrap();
        prop_assert_eq!(result, result2); // 冪等性
    }
}
```

##### 3. 不変条件のProperty検証
```rust
proptest! {
    #[test]
    fn function_invariants(input in arb_input()) {
        let original_input = input.clone();
        let system_state_before = capture_system_state();
        
        let result = function_under_test(input);
        
        // 不変条件: 入力が変更されない
        prop_assert_eq!(input, original_input);
        
        // 不変条件: システム状態の一貫性
        let system_state_after = capture_system_state();
        prop_assert_eq!(system_state_before, system_state_after);
    }
}
```

#### Property-basedテストの書き方
```rust
use proptest::prelude::*;

// 任意値生成器の定義
prop_compose! {
    fn arb_config()
        (client_id in "[a-zA-Z0-9]{10,50}",
         client_secret in "[a-zA-Z0-9]{20,100}")
        -> Config
    {
        Config { client_id, client_secret, redirect_uri: None }
    }
}

proptest! {
    /// Property: ConfigのTOMLラウンドトリップ
    #[test]
    fn config_toml_roundtrip(config in arb_config()) {
        let toml_str = toml::to_string(&config)?;
        let parsed: Config = toml::from_str(&toml_str)?;
        prop_assert_eq!(config, parsed);
    }
}
```

#### テスト実行とデバッグ
```bash
# Property-basedテストのみ実行（統合テスト）
cargo test --test property_tests

# ライブラリのテストのみ実行
cargo test --lib

# 失敗ケースの最小化表示
PROPTEST_VERBOSE=1 cargo test --test property_tests

# 特定のテスト数で実行
PROPTEST_CASES=1000 cargo test --test property_tests
```

### 日時・日付の検証規約

#### 実際の日時であることの保証
すべての日時処理において、以下の規約を遵守する：

##### 1. 日付生成の要件
- **必須**: 実際に存在する日付のみを生成・受け入れる
- **検証**: `chrono::NaiveDate`を使用した実際の日付検証
- **形式**: `YYYY-MM-DD` 形式（ISO 8601準拠）

##### 2. 月別日数制限の遵守
```rust
// 月ごとの最大日数
match month {
    2 => {
        // うるう年判定: 4で割り切れ、かつ100で割り切れない、または400で割り切れる
        if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
            29 // うるう年の2月
        } else {
            28 // 平年の2月
        }
    },
    4 | 6 | 9 | 11 => 30, // 4月、6月、9月、11月
    _ => 31, // その他の月（1,3,5,7,8,10,12月）
}
```

##### 3. Property-basedテストでの日付検証
```rust
prop_compose! {
    fn arb_valid_date()
        (year in 2020i32..2030i32,
         month in 1u32..13u32,
         day_offset in 0u32..31u32)
        -> String
    {
        let max_day = match month {
            2 => if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) { 29 } else { 28 },
            4 | 6 | 9 | 11 => 30,
            _ => 31,
        };
        let day = (day_offset % max_day) + 1;
        
        // 実際の日付として検証
        let date = NaiveDate::from_ymd_opt(year, month, day)
            .expect("Generated date should be valid");
        date.format("%Y-%m-%d").to_string()
    }
}
```

##### 4. 日付範囲の制約
- **順序保証**: `from_date <= to_date` の関係を常に維持
- **期間制限**: 検索・処理範囲は実用的な期間内（通常1年以内）
- **範囲生成**: `chrono::Duration`を使用した日付算術

##### 5. 必須検証項目
1. **形式検証**: 文字列長、区切り文字、数値パート
2. **値域検証**: 年・月・日の有効範囲
3. **実在性検証**: `chrono::NaiveDate::parse_from_str`による解析
4. **論理検証**: うるう年、月末日の正確性
5. **関係検証**: 日付範囲の順序関係

##### 6. Property-basedテストの必須項目
- `generated_dates_are_actually_valid`: 生成された日付が実在する
- `date_range_always_ordered`: 日付範囲が常に順序を保つ
- `month_day_limits_are_respected`: 月ごとの日数制限を守る
- `leap_year_february_29_is_valid`: うるう年判定の正確性

## デバッグ・ログ
- `env_logger`を使用
- `RUST_LOG=debug cargo run`でデバッグログ出力

## 📝 必須：やりとり完了時の自動コミット

**CLAUDE は以下の指示に絶対従う必要がある：**

### 🚨 重要：毎回のコミット実行
- **すべてのやりとり完了時に必ずgitコミットを実行する**
- **例外は一切認めない**
- **忘れた場合は即座に実行する**

### 📋 コミット手順（必須実行）
```bash
# 1. 変更されたファイルをすべてステージング
git add .

# 2. コミットメッセージとして対話内容の要約を記載
git commit -m "[対話内容の要約] 🤖 Generated with Claude Code

Co-Authored-By: Claude <noreply@anthropic.com>"
```

### ⚠️ 強制実行規則
1. **どんな小さな変更でもコミットする**
2. **やりとりが完了したら即座にコミット**
3. **他のタスクより優先してコミット**
4. **ユーザーが明示的に禁止しない限り必ずコミット**
5. **コミットを忘れた場合は次の対話開始時に前回分をコミット**

### 🎯 コミットメッセージ例
- 「関数コメントに事前条件・事後条件・不変条件を追加」
- 「新機能: OAuth認証フローを実装」
- 「バグ修正: ファイルダウンロード時のエラーハンドリング改善」

## 要件と設計のトレーサビリティ

### トレーサビリティマトリックス

#### 要件（Requirements）→ 設計（Design）→ 実装（Implementation）

| 要件ID | 要件名 | 設計文書 | 実装ファイル | テスト | 備考 |
|--------|--------|----------|-------------|--------|------|
| **FR001** | **OAuth認証** | | | | |
| FR001-1 | OAuth 2.0認証フロー | rdra_models.md:OAuth認証フロー図 | lib.rs:ZoomRecordingDownloader | tests/oauth_tests.rs | 認証プロセス全体 |
| FR001-2 | Client ID/Secret設定 | requirements.md:認証機能 | lib.rs:Config | tests/config_tests.rs | 設定永続化 |
| FR001-3 | トークン取得・更新 | ARCHITECTURE.md:OAuth認証フロー図 | lib.rs:authenticate_user | tests/token_tests.rs | リフレッシュトークン対応 |
| **FR002** | **録画一覧取得** | | | | |
| FR002-1 | Zoom API呼び出し | ARCHITECTURE.md:データフロー図 | lib.rs:get_recordings | tests/api_tests.rs | REST API通信 |
| FR002-2 | 録画リスト表示 | rdra_models.md:ビジネスフロー図 | gui.rs:render_recordings | tests/gui_tests.rs | UI表示処理 |
| FR002-3 | 期間フィルタリング | requirements.md:ダウンロード機能 | lib.rs:filter_by_date | tests/filter_tests.rs | 日付範囲検索 |
| **FR003** | **ファイルダウンロード** | | | | |
| FR003-1 | 並列ダウンロード | ARCHITECTURE.md:システム構成図 | lib.rs:download_recording | tests/download_tests.rs | 非同期処理 |
| FR003-2 | 進捗表示 | rdra_models.md:GUI状態遷移図 | gui.rs:render_progress | tests/progress_tests.rs | リアルタイム更新 |
| FR003-3 | ファイル種別対応 | requirements.md:対象ファイル | lib.rs:DownloadableFile | tests/file_type_tests.rs | MP4/MP3/TXT/JSON |
| **FR004** | **GUI操作** | | | | |
| FR004-1 | egui/eframe UI | ARCHITECTURE.md:GUI状態遷移図 | gui.rs:ZoomDownloaderApp | tests/gui_integration.rs | メインアプリ画面 |
| FR004-2 | 設定画面 | rdra_models.md:システムコンテキスト図 | gui.rs:render_config | tests/config_ui_tests.rs | 設定入力フォーム |
| FR004-3 | ファイル選択 | requirements.md:ユーザーインターフェース | gui.rs:render_file_selection | tests/selection_tests.rs | チェックボックス選択 |
| **NFR001** | **性能要件** | | | | |
| NFR001-1 | 同時ダウンロード数制限 | requirements.md:パフォーマンス | lib.rs:CONCURRENT_LIMIT | tests/performance_tests.rs | セマフォ制御 |
| NFR001-2 | API レート制限対応 | ARCHITECTURE.md:エラー処理戦略 | lib.rs:rate_limit_handler | tests/rate_limit_tests.rs | 指数バックオフ |
| **NFR002** | **信頼性要件** | | | | |
| NFR002-1 | エラーハンドリング | ARCHITECTURE.md:エラー処理戦略 | lib.rs:ZoomVideoMoverError | tests/error_handling_tests.rs | Result型エラー処理 |
| NFR002-2 | ログ出力 | requirements.md:信頼性 | lib.rs:logger_init | tests/logging_tests.rs | env_logger使用 |
| **NFR003** | **セキュリティ要件** | | | | |
| NFR003-1 | OAuth情報保護 | requirements.md:セキュリティ | lib.rs:secure_storage | tests/security_tests.rs | ローカル暗号化 |
| NFR003-2 | HTTPS通信強制 | ARCHITECTURE.md:技術選定 | lib.rs:reqwest_client | tests/https_tests.rs | TLS証明書検証 |
| **NFR004** | **国際化要件** | | | | |
| NFR004-1 | Windows日本語対応 | requirements.md:国際化 | windows_console.rs | tests/encoding_tests.rs | UTF-8エンコーディング |
| NFR004-2 | 日本語ファイル名 | ARCHITECTURE.md:データフロー | lib.rs:sanitize_filename | tests/filename_tests.rs | パス処理 |

#### 逆トレーサビリティ（実装→要件）

| 実装ファイル | 主要クラス/関数 | 対応要件ID | 設計根拠 |
|-------------|----------------|------------|----------|
| **lib.rs** | | | |
| lib.rs:47 | `Config` struct | FR001-2, NFR003-1 | OAuth設定管理 |
| lib.rs:152 | `ZoomRecordingDownloader` | FR001, FR002, FR003 | コアビジネスロジック |
| lib.rs:298 | `authenticate_user()` | FR001-1, FR001-3 | OAuth認証実装 |
| lib.rs:445 | `get_recordings()` | FR002-1, FR002-3 | API呼び出し |
| lib.rs:521 | `download_recording()` | FR003-1, NFR001-1 | 並列ダウンロード |
| **gui.rs** | | | |
| gui.rs:58 | `ZoomDownloaderApp` | FR004-1 | GUIメイン状態管理 |
| gui.rs:195 | `render_config()` | FR004-2 | 設定画面描画 |
| gui.rs:267 | `render_recordings()` | FR002-2, FR004-3 | 録画リスト表示 |
| gui.rs:348 | `render_progress()` | FR003-2 | 進捗バー表示 |
| **windows_console.rs** | | | |
| windows_console.rs:15 | `setup_console_encoding()` | NFR004-1 | Windows UTF-8設定 |

#### テストトレーサビリティ

| テストファイル | テスト関数 | 検証要件 | テスト種別 | 合格基準 |
|---------------|------------|----------|------------|----------|
| **tests/oauth_tests.rs** | | | | |
| oauth_tests.rs | `test_oauth_flow()` | FR001-1 | 統合テスト | 認証完了まで正常 |
| oauth_tests.rs | `test_token_refresh()` | FR001-3 | 単体テスト | リフレッシュ成功 |
| **tests/config_tests.rs** | | | | |
| config_tests.rs | `test_config_roundtrip()` | FR001-2 | Property-based | TOML保存・読込一致 |
| config_tests.rs | `test_config_validation()` | NFR003-1 | 単体テスト | 無効設定をリジェクト |
| **tests/download_tests.rs** | | | | |
| download_tests.rs | `test_parallel_download()` | FR003-1, NFR001-1 | 統合テスト | 制限内同時実行 |
| download_tests.rs | `test_download_progress()` | FR003-2 | 単体テスト | 進捗イベント発火 |
| **tests/property_tests.rs** | | | | |
| property_tests.rs | `date_range_validation()` | FR002-3 | Property-based | 有効日付のみ生成 |
| property_tests.rs | `filename_sanitization()` | NFR004-2 | Property-based | 日本語文字正常処理 |

#### 品質保証トレーサビリティ

| 品質活動 | 対象 | 実行コマンド | 検証内容 | 成功基準 |
|----------|------|-------------|----------|----------|
| **型安全性チェック** | 全実装 | `cargo check` | コンパイルエラー | エラー0件 |
| **静的解析** | 全実装 | `cargo clippy` | コーディング規約 | 警告0件 |
| **フォーマット** | 全実装 | `cargo fmt` | コードスタイル | 差分なし |
| **単体テスト** | 個別関数 | `cargo test --lib` | 関数仕様 | 全テスト合格 |
| **統合テスト** | システム全体 | `cargo test --test integration` | 要件充足 | 全シナリオ合格 |
| **Property-based** | データ処理 | `cargo test --test property_tests` | データ整合性 | 1000ケース合格 |

#### 文書間相互参照

| 文書名 | セクション | 参照先文書 | 参照内容 |
|--------|------------|------------|----------|
| **requirements.md** | 機能要件 | ARCHITECTURE.md | システム構成との対応 |
| **requirements.md** | 非機能要件 | rdra_models.md | RDRA要求仕様書 |
| **ARCHITECTURE.md** | システム構成図 | lib.rs | 実装クラス構造 |
| **ARCHITECTURE.md** | OAuth認証フロー | gui.rs | GUI状態遷移 |
| **rdra_models.md** | ビジネスフロー | lib.rs | ビジネスロジック |
| **rdra_models.md** | 要求仕様書 | tests/ | テスト仕様 |
| **CLAUDE.md** | コーディング規約 | src/ | 実装ガイドライン |
| **CLAUDE.md** | テスト戦略 | tests/ | テスト実装方針 |

### トレーサビリティ管理プロセス

#### 1. 要件変更時の手順
1. **影響分析**: 変更要件に関連する設計・実装を特定
2. **設計更新**: アーキテクチャ文書の該当箇所を修正
3. **実装修正**: トレーサビリティマトリックスに基づいて対象ファイルを更新
4. **テスト更新**: 変更に対応するテスト仕様・実装を修正
5. **文書同期**: 関連文書間の整合性を確認・更新

#### 2. 設計変更時の手順
1. **要件確認**: 設計変更が要件逸脱でないことを確認
2. **実装影響**: 対象実装ファイル・関数を特定
3. **テスト影響**: 修正が必要なテストケースを特定
4. **品質保証**: 型チェック・静的解析で整合性確認

#### 3. 実装変更時の手順
1. **要件追跡**: 変更理由が要件・設計に由来することを確認
2. **テスト先行**: 変更前にテストケースを更新
3. **実装実行**: 事前条件・事後条件・不変条件を維持
4. **文書更新**: 必要に応じて設計文書を更新

### トレーサビリティ品質メトリクス

| メトリクス | 計算方法 | 目標値 | 現在値 |
|------------|----------|--------|--------|
| **要件カバレッジ** | 実装済み要件数 / 全要件数 | 100% | 98% |
| **設計カバレッジ** | 設計文書化要件数 / 全要件数 | 100% | 100% |
| **テストカバレッジ** | テスト済み要件数 / 全要件数 | 90% | 85% |
| **文書整合性** | 同期済み文書参照数 / 全文書参照数 | 100% | 95% |
| **変更追跡率** | 追跡可能変更数 / 全変更数 | 100% | 90% |

## トラブルシューティング参考
- README.mdの詳細なトラブルシューティングセクション参照
- 特にZoom OAuth設定とWindows環境の問題