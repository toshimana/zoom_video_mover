# Rust コーディング規約 - Zoom Video Mover

**適用範囲**: 全Rustコード（src/*, tests/*, build.rs）  
**準拠レベル**: 必須（100%準拠）  
**更新日**: 2025-08-05

## 🎯 基本原則

### 1. 関数の副作用について
- **原則**: 関数は副作用がない（純粋関数）ように設計する
- **例外**: 要件達成のために副作用が必要な場合は、関数コメントに明記する
- **副作用の例**: ファイル操作、ネットワーク通信、グローバル状態の変更、標準出力への書き込みなど

## 📝 関数コメントの必須要素

すべての**public関数**には以下の要素を含むコメントを記載する：

### 1. 事前条件（Preconditions）
- 関数が正常に動作するために満たすべき条件
- 引数の有効性、システム状態、依存関係など

### 2. 事後条件（Postconditions）
- 関数実行後に保証される条件
- 戻り値の性質、システム状態の変化など

### 3. 不変条件（Invariants）
- 関数実行中に常に維持される条件
- データ構造の整合性、状態の一貫性など

### 4. 副作用（Side Effects）
- 関数が引き起こす外部への影響

## ⚡ アサーション（assertion）の追加

関数の事前条件・事後条件を実行時にチェックするため、適切なassertionを追加する：

### 1. 事前条件のassertion
- 関数の開始時に`assert!`、`debug_assert!`を使用
- 引数の有効性、前提条件をチェック

### 2. 事後条件のassertion  
- 関数の終了前に戻り値や結果の妥当性をチェック
- `debug_assert!`を使用（リリースビルドでは無効化）

### 3. assertionの使い分け
- **`assert!`**: 常にチェックが必要な重要な条件
- **`debug_assert!`**: デバッグ時のみチェック（パフォーマンス重視）
- **`unreachable!`**: 到達不可能なコードパス

## 📋 完全な関数コメントとassertion例

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

## 🔍 品質チェックコマンド

### 必須実行コマンド
```bash
# 型チェック（コンパイルせずに型エラーを確認）
cargo check

# より詳細な静的解析
cargo clippy

# すべてのターゲットとテストの型チェック
cargo check --all-targets

# コードフォーマット
cargo fmt
```

### 品質基準
- **clippy警告**: 0件（例外なし）
- **関数コメント**: public関数100%準拠
- **assertion**: 事前・事後条件の適切な配置

## 📊 準拠確認方法

### 自動チェック
```bash
# 全品質チェック実行
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo check --all-targets
```

### 手動確認項目
- [ ] すべてのpublic関数に完全コメント
- [ ] 副作用の明確な記載
- [ ] 適切なassertion配置
- [ ] clippy警告0件

## 🎯 継続的改善

### 週次確認
- 新規追加関数のコメント準拠確認
- clippy警告の解決
- assertion追加の適切性確認

### 月次レビュー
- コーディング規約の見直し
- ベストプラクティスの更新
- チーム内での規約共有

---

**準拠必須**: このコーディング規約は全Rustコードで100%準拠が必要です。