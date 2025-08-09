---
reusability: tech-specific
version: 1.0.0
dependencies: []
customizable:
  - project_name
  - test_framework
---

# Rustコーディング標準 - {{PROJECT_NAME}}

## 基本原則

### Rustの哲学
- **所有権**: 明確な所有権と借用規則
- **型安全性**: コンパイル時の型チェック
- **並行性**: データ競合のない並行処理
- **ゼロコスト抽象化**: 実行時オーバーヘッドなし

## 命名規則

### 識別子の命名
| 要素 | 規則 | 例 |
|------|------|-----|
| モジュール | snake_case | `user_service` |
| 型・トレイト | PascalCase | `UserAccount` |
| 関数・メソッド | snake_case | `calculate_total` |
| 変数・引数 | snake_case | `user_name` |
| 定数 | SCREAMING_SNAKE_CASE | `MAX_CONNECTIONS` |
| ライフタイム | 短い小文字 | `'a`, `'ctx` |

### 頭字語の扱い
```rust
// Good
struct HttpRequest;
struct IoError;

// Bad
struct HTTPRequest;
struct IOError;
```

## エラーハンドリング

### Result型の使用
```rust
// 関数シグネチャに必ずResult型を使用
pub fn process_data(input: &str) -> Result<ProcessedData, Error> {
    // エラーを早期リターン
    let parsed = parse_input(input)?;
    
    // 処理
    Ok(ProcessedData::new(parsed))
}
```

### カスタムエラー型
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Parse error: {message}")]
    Parse { message: String },
    
    #[error("Invalid input: expected {expected}, got {actual}")]
    InvalidInput { expected: String, actual: String },
}
```

### パニックの使用
```rust
// パニックは以下の場合のみ使用
// 1. 明らかなプログラミングエラー
assert!(index < len, "Index out of bounds");

// 2. 初期化時の設定エラー
let config = Config::load().expect("Failed to load configuration");

// 通常のエラーではResult型を使用
```

## 所有権と借用

### 借用の優先順位
```rust
// 1. 不変借用を優先
fn process(data: &Data) -> Result<Output, Error>

// 2. 必要な場合のみ可変借用
fn update(data: &mut Data) -> Result<(), Error>

// 3. 所有権の移動は最後の手段
fn consume(data: Data) -> Result<NewData, Error>
```

### スマートポインタの使用
```rust
// Box: ヒープ割り当て
let large_data = Box::new(LargeStruct::new());

// Rc/Arc: 共有所有権（Arcは並行処理用）
let shared = Arc::new(SharedData::new());

// RefCell/Mutex: 内部可変性（Mutexは並行処理用）
let mutable = Arc::new(Mutex::new(MutableData::new()));
```

## 非同期処理

### async/awaitの使用
```rust
use tokio;

// 非同期関数の定義
pub async fn fetch_data(url: &str) -> Result<Data, Error> {
    let response = reqwest::get(url).await?;
    let data = response.json::<Data>().await?;
    Ok(data)
}

// エラーハンドリング
pub async fn process_async() -> Result<(), Error> {
    // ?演算子で早期リターン
    let data = fetch_data("https://api.example.com").await?;
    
    // タイムアウト処理
    tokio::time::timeout(
        Duration::from_secs(30),
        long_running_task()
    ).await??;
    
    Ok(())
}
```

## テスト

### 単体テスト
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_normal_case() {
        // Arrange
        let input = TestData::new();
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected_value);
    }
    
    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn test_panic_case() {
        trigger_panic();
    }
}
```

### Property-basedテスト
```rust
#[cfg(test)]
mod prop_tests {
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_invariant(input in any::<String>()) {
            let result = process(input);
            // 不変条件を検証
            prop_assert!(invariant_holds(&result));
        }
    }
}
```

## パフォーマンス

### 最適化の原則
```rust
// 1. プロファイリングしてから最適化
// 2. 読みやすさを犠牲にしない
// 3. ベンチマークで検証

#[bench]
fn bench_algorithm(b: &mut Bencher) {
    b.iter(|| {
        algorithm(black_box(&input))
    });
}
```

### メモリ効率
```rust
// Vecの事前割り当て
let mut vec = Vec::with_capacity(expected_size);

// 文字列の結合
let result = format!("{}{}{}", part1, part2, part3);

// イテレータの活用
let sum: i32 = numbers
    .iter()
    .filter(|&&x| x > 0)
    .map(|&x| x * 2)
    .sum();
```

## ドキュメント

### ドキュメントコメント
```rust
/// データを処理し、結果を返す
///
/// # Arguments
/// * `input` - 処理対象のデータ
/// * `options` - 処理オプション
///
/// # Returns
/// 処理結果、エラーの場合は`Error`
///
/// # Errors
/// - `ParseError`: 入力が不正な場合
/// - `ProcessError`: 処理に失敗した場合
///
/// # Examples
/// ```
/// let result = process_data("input", &Options::default())?;
/// assert_eq!(result.status(), Status::Success);
/// ```
///
/// # Panics
/// オプションが無効な場合パニックする
pub fn process_data(input: &str, options: &Options) -> Result<Output, Error> {
    // 実装
}
```

## Clippy設定

### 推奨設定
```toml
# Cargo.toml または .clippy.toml
[workspace.lints.clippy]
all = "warn"
pedantic = "warn"
nursery = "warn"
cargo = "warn"

# 特定の警告を無効化
too_many_arguments = "allow"
```

### CI/CDでの使用
```bash
# 警告をエラーとして扱う
cargo clippy -- -D warnings

# 自動修正
cargo clippy --fix
```

## 依存関係管理

### Cargo.toml
```toml
[dependencies]
# バージョン指定は保守的に
serde = "1.0"
tokio = { version = "1", features = ["full"] }

[dev-dependencies]
# テスト用依存関係
proptest = "1.0"
mockall = "0.11"

[profile.release]
# リリースビルド最適化
opt-level = 3
lto = true
codegen-units = 1
```

## セキュリティ

### 安全でないコードの制限
```rust
// unsafe は最小限に
// 必ず安全性の根拠をコメント

/// # Safety
/// - `ptr` は有効なメモリを指している必要がある
/// - `len` はバッファサイズを超えてはならない
unsafe fn process_raw(ptr: *const u8, len: usize) {
    // 実装
}
```

### 依存関係の監査
```bash
# セキュリティ監査
cargo audit

# ライセンスチェック
cargo license
```

---
**適用日**: {{POLICY_EFFECTIVE_DATE}}  
**レビュー周期**: 四半期ごと