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

## トラブルシューティング参考
- README.mdの詳細なトラブルシューティングセクション参照
- 特にZoom OAuth設定とWindows環境の問題