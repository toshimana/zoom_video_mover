# Cargo・Clippyポリシー - Zoom Video Mover

**技術要素**: cargo, clippy, rustfmt  
**適用範囲**: 全Rustプロジェクト管理・品質チェック

## Cargoプロジェクト管理方針

### Cargo.toml設定標準

#### 基本プロジェクト情報
```toml
[package]
name = "zoom_video_mover"
version = "0.1.0"
edition = "2021"
rust-version = "1.70"
authors = ["Zoom Video Mover Team"]
license = "MIT OR Apache-2.0"
description = "Zoom cloud recording downloader with GUI"
homepage = "https://github.com/your-org/zoom_video_mover"
repository = "https://github.com/your-org/zoom_video_mover"
readme = "README.md"
keywords = ["zoom", "recording", "downloader", "gui"]
categories = ["multimedia::video", "gui"]
```

#### 依存関係管理原則
```toml
[dependencies]
# GUI フレームワーク
eframe = "0.22"
egui = "0.22"

# 非同期処理
tokio = { version = "1.0", features = ["full"] }

# HTTP クライアント
reqwest = { version = "0.11", features = ["json", "stream"] }

# OAuth 2.0
oauth2 = "4.4"

# シリアライゼーション
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"

# エラーハンドリング
thiserror = "1.0"
anyhow = "1.0"

# 日時処理
chrono = { version = "0.4", features = ["serde"] }

# ログ
log = "0.4"
env_logger = "0.10"

# Windows固有
[target.'cfg(windows)'.dependencies]
windows = { version = "0.51", features = ["Win32_System_Console"] }

# 開発依存関係
[dev-dependencies]
proptest = "1.0"
tokio-test = "0.4"
criterion = "0.5"
mockall = "0.11"
```

### 依存関係選定基準
1. **安定性**: メジャーバージョン1.0以上を優先
2. **保守性**: アクティブに保守されているクレート
3. **最小性**: 機能要件を満たす最小限の依存関係
4. **互換性**: 他の依存関係との競合なし
5. **セキュリティ**: 既知の脆弱性なし

## Clippyリント設定

### Cargo.toml設定
```toml
[lints.clippy]
# 全般的な警告
all = "warn"
pedantic = "warn"
nursery = "warn"

# 重要なエラー（コンパイル失敗）
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
todo = "deny"
unimplemented = "deny"

# パフォーマンス関連
inefficient_to_string = "deny"
clone_on_ref_ptr = "deny"
rc_buffer = "deny"

# 正確性関連
float_cmp = "deny"
indexing_slicing = "warn"
integer_arithmetic = "warn"

# セキュリティ関連
print_stdout = "warn"
print_stderr = "warn"
dbg_macro = "warn"

# コード品質関連
cognitive_complexity = "warn"
too_many_arguments = "warn"
type_complexity = "warn"
large_enum_variant = "warn"

# 許可する例外
unnecessary_wraps = "allow"  # Result型の一貫性のため
module_name_repetitions = "allow"  # モジュール名の明確性のため
```

### カスタムClippyルール
```rust
// 特定ファイルでのlint無効化（慎重に使用）
#![allow(clippy::too_many_arguments)]  // 初期化関数など

// 関数レベルでの無効化
#[allow(clippy::unwrap_used)]  // テストコードなど明確に安全な場合のみ
fn test_helper() {
    let value = some_operation().unwrap();  // テスト環境で安全
}
```

## Rustfmtフォーマット設定

### rustfmt.toml設定
```toml
# インデント設定
max_width = 100
hard_tabs = false
tab_spaces = 4

# インポート設定
reorder_imports = true
group_imports = "StdExternalCrate"
imports_granularity = "Crate"

# 関数・構造体設定
fn_single_line = false
where_single_line = false
force_multiline_blocks = false

# コメント設定
wrap_comments = true
format_code_in_doc_comments = true
normalize_comments = true

# 配列・構造体設定
trailing_comma = "Vertical"
match_block_trailing_comma = false

# 文字列設定
string_lit_normalization = true

# その他
use_small_heuristics = "Default"
unstable_features = false
```

## 品質チェック自動化

### GitHub Actions設定
```yaml
name: Quality Checks

on: [push, pull_request]

jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        components: rustfmt, clippy
        override: true
    
    - name: Check formatting
      run: cargo fmt -- --check
    
    - name: Clippy check
      run: cargo clippy --all-targets --all-features -- -D warnings
    
    - name: Build check
      run: cargo check --all-targets
    
    - name: Run tests
      run: cargo test
```

### pre-commitフック設定
```bash
#!/bin/sh
# .git/hooks/pre-commit

set -e

echo "Running pre-commit quality checks..."

# フォーマットチェック
echo "Checking code formatting..."
cargo fmt -- --check
if [ $? -ne 0 ]; then
    echo "❌ Code formatting check failed. Run 'cargo fmt' to fix."
    exit 1
fi

# Clippyチェック
echo "Running Clippy lints..."
cargo clippy --all-targets --all-features -- -D warnings
if [ $? -ne 0 ]; then
    echo "❌ Clippy check failed. Fix the warnings above."
    exit 1
fi

# ビルドチェック
echo "Checking build..."
cargo check --all-targets
if [ $? -ne 0 ]; then
    echo "❌ Build check failed."
    exit 1
fi

# 高速テスト実行
echo "Running quick tests..."
cargo test --lib
if [ $? -ne 0 ]; then
    echo "❌ Quick tests failed."
    exit 1
fi

echo "✅ All pre-commit checks passed!"
```

## 品質メトリクス収集

### カバレッジ測定
```bash
# テストカバレッジ生成
cargo install cargo-tarpaulin
cargo tarpaulin --out Html --output-dir target/coverage

# カバレッジ目標: 90%以上
```

### 複雑度測定
```bash
# 循環的複雑度測定
cargo install cargo-cyclocomp
cargo cyclocomp --threshold 10
```

### 依存関係監査
```bash
# セキュリティ監査
cargo install cargo-audit
cargo audit

# 古い依存関係チェック
cargo install cargo-outdated
cargo outdated

# 未使用依存関係チェック
cargo install cargo-udeps
cargo +nightly udeps
```

## ビルド最適化

### リリースビルド設定
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true

# デバッグビルドの高速化
[profile.dev]
opt-level = 0
debug = true
incremental = true

# テスト用最適化
[profile.test]
opt-level = 1
debug = true
```

### ビルドスクリプト（build.rs）
```rust
// build.rs
use std::env;

fn main() {
    // Windows固有の設定
    if env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS");
        println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_OS");
    }
    
    // バージョン情報埋め込み
    println!("cargo:rustc-env=BUILD_TIME={}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));
    println!("cargo:rerun-if-changed=build.rs");
}
```

## ドキュメント生成

### Rustdoc設定
```bash
# ドキュメント生成
cargo doc --no-deps --document-private-items

# ドキュメントサーバー起動
cargo doc --open
```

### doc.rs対応
```rust
//! クレート全体のドキュメント
//!
//! # Examples
//! ```
//! use zoom_video_mover::ZoomDownloader;
//! 
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let downloader = ZoomDownloader::new("client_id", "client_secret")?;
//! let recordings = downloader.get_recordings("2024-01-01", "2024-01-31").await?;
//! # Ok(())
//! # }
//! ```

#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
```

## エラー対応ガイド

### よくあるClippyエラーとその対処

#### 1. unwrap_used
```rust
// ❌ 悪い例
let value = some_operation().unwrap();

// ✅ 良い例
let value = some_operation()
    .map_err(|e| MyError::OperationFailed(e))?;
```

#### 2. too_many_arguments
```rust
// ❌ 悪い例
fn create_user(name: String, email: String, age: u32, city: String, country: String, phone: String) {}

// ✅ 良い例
struct UserInfo {
    name: String,
    email: String,
    age: u32,
    contact: ContactInfo,
}

fn create_user(user_info: UserInfo) {}
```

#### 3. cognitive_complexity
```rust
// ❌ 複雑すぎる関数
fn complex_function(input: &str) -> Result<String, Error> {
    if condition1 {
        if condition2 {
            for item in items {
                if item.is_valid() {
                    // ネストが深すぎる
                }
            }
        }
    }
    // ...
}

// ✅ 関数分割
fn complex_function(input: &str) -> Result<String, Error> {
    if !initial_validation(input) {
        return Err(Error::Invalid);
    }
    
    process_items(input)
}

fn process_items(input: &str) -> Result<String, Error> {
    // 処理を分割
}
```

## 品質目標

- **Clippyワーニング**: 0件
- **フォーマット準拠**: 100%
- **ビルド成功率**: 100%
- **テストカバレッジ**: 90%以上
- **セキュリティ監査**: パス
- **依存関係最新性**: 月次更新

継続的な品質改善により、保守性の高いRustコードベースを維持します。

---

**承認**:  
**ポリシー版本**: 1.0  
**最終更新**: 2025-08-04  
**適用範囲**: 全Rustプロジェクト管理・品質チェック  
**承認日**: ___________