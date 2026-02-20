# テスト戦略 - Zoom Video Mover

**適用範囲**: 全テストコード（tests/*, src/*/tests）

## テストピラミッド

```
       ┌─────────────────┐
       │  Manual Tests    │ ← 最小限
       └─────────────────┘
     ┌─────────────────────┐
     │    E2E / Integration │ ← 少数・重要シナリオ
     └─────────────────────┘
   ┌─────────────────────────┐
   │   Unit / Property-based  │ ← 大多数・高速実行
   └─────────────────────────┘
```

## Property-basedテスト（基盤品質保証）

1000ケース以上の自動検証による網羅的品質保証。

### 使用フレームワーク
- **`proptest`**: Property-basedテスト
- **`proptest-derive`**: 任意値生成の自動導出

### テスト対象プロパティ
1. **ラウンドトリップ**: シリアライゼーション→デシリアライゼーションの可逆性
2. **不変条件**: データ構造の整合性
3. **入力範囲**: 有効な入力範囲での動作保証
4. **境界条件**: エッジケースでの適切な動作
5. **冪等性**: 同じ操作を複数回実行しても結果が同じ

### 基本パターン

```rust
use proptest::prelude::*;

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
    #[test]
    fn config_toml_roundtrip(config in arb_config()) {
        let toml_str = toml::to_string(&config)?;
        let parsed: Config = toml::from_str(&toml_str)?;
        prop_assert_eq!(config, parsed);
    }
}
```

### 事前条件・事後条件・不変条件の検証

```rust
proptest! {
    #[test]
    fn function_postconditions(input in arb_input()) {
        let result = function_under_test(input).unwrap();
        prop_assert!(!result.is_empty());               // 事後条件
        let result2 = function_under_test(input.clone()).unwrap();
        prop_assert_eq!(result, result2);                // 冪等性
    }
}
```

### 実行コマンド

```bash
cargo test --test property_tests                           # 基本実行
PROPTEST_CASES=1000 cargo test --test property_tests       # 1000ケース
PROPTEST_VERBOSE=1 cargo test --test property_tests        # 詳細ログ
cargo test --test property_tests -- config_roundtrip       # 特定テスト
```

### 設定

```rust
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 1000,
        max_shrink_iters: 1000,
        timeout: 5000,
        ..ProptestConfig::default()
    })]
    // テスト関数
}
```

## 日時・日付の検証規約

### 必須要件
- 実際に存在する日付のみを生成・受け入れる
- `chrono::NaiveDate`を使用した日付検証
- `YYYY-MM-DD`形式（ISO 8601準拠）

### 日付生成パターン

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
        let date = NaiveDate::from_ymd_opt(year, month, day)
            .expect("Generated date should be valid");
        date.format("%Y-%m-%d").to_string()
    }
}
```

### 必須検証項目
- `generated_dates_are_actually_valid`: 生成日付の実在性
- `date_range_always_ordered`: 日付範囲の順序保持
- `month_day_limits_are_respected`: 月ごとの日数制限
- `leap_year_february_29_is_valid`: うるう年判定の正確性

## 品質保証指標

- **Property-basedテスト**: 重要関数100%カバレッジ
- **実行ケース数**: 1000+ケース/関数
- **テスト成功率**: 100%
- **境界値検証**: 全エッジケース網羅
