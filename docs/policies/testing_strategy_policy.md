# テスト戦略ポリシー - Zoom Video Mover

**適用範囲**: 全テストコード（tests/*, src/*/tests）  
**戦略位置づけ**: プロジェクト基盤品質保証戦略  
**更新日**: 2025-08-05

## 🎯 Property-basedテスト - 基盤品質保証戦略

**位置づけ**: プロジェクト基盤品質保証戦略として、1000ケース以上の自動検証による網羅的品質保証を実現

関数の性質（プロパティ）を定義し、多数のランダム入力で検証する手法：

### 🛠️ 使用フレームワーク
- **`proptest`**: Rust標準のproperty-basedテストフレームワーク
- **`quickcheck`**: QuickCheck-style testing
- **`proptest-derive`**: 任意値生成の自動導出

### 📋 テスト対象プロパティ
1. **ラウンドトリップ性質**: シリアライゼーション→デシリアライゼーションの可逆性
2. **不変条件**: データ構造の整合性が常に保たれる
3. **入力範囲**: 有効な入力範囲での動作保証
4. **境界条件**: エッジケースでの適切な動作
5. **冪等性**: 同じ操作を複数回実行しても結果が変わらない

## 🔬 関数単位でのProperty-basedテスト設計

各関数の**事前条件・事後条件・不変条件**を基にテストケースを作成：

### 1. 事前条件のProperty検証
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

### 2. 事後条件のProperty検証
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

### 3. 不変条件のProperty検証
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

## 📝 Property-basedテストの書き方

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

## 🚀 テスト実行規模・基盤戦略としての位置づけ

```bash
# Property-basedテストのみ実行（統合テスト）
cargo test --test property_tests

# ライブラリのテストのみ実行
cargo test --lib

# 基盤戦略として1000ケース以上の自動検証を実行
PROPTEST_CASES=1000 cargo test --test property_tests

# 失敗ケースの最小化表示
PROPTEST_VERBOSE=1 cargo test --test property_tests
```

### 基盤戦略としての特徴
- **実行規模**: 1000ケース以上の自動検証
- **適用領域**: データ整合性・境界値・異常系の完全検証
- **効果**: 手動テストでは困難な網羅的品質保証を実現
- **優先度**: 品質保証の最重要基盤として位置づけ

## 📅 日時・日付の検証規約

### 実際の日時であることの保証
すべての日時処理において、以下の規約を遵守する：

#### 1. 日付生成の要件
- **必須**: 実際に存在する日付のみを生成・受け入れる
- **検証**: `chrono::NaiveDate`を使用した実際の日付検証
- **形式**: `YYYY-MM-DD` 形式（ISO 8601準拠）

#### 2. 月別日数制限の遵守
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

#### 3. Property-basedテストでの日付検証
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

#### 4. 日付範囲の制約
- **順序保証**: `from_date <= to_date` の関係を常に維持
- **期間制限**: 検索・処理範囲は実用的な期間内（通常1年以内）
- **範囲生成**: `chrono::Duration`を使用した日付算術

#### 5. 必須検証項目
1. **形式検証**: 文字列長、区切り文字、数値パート
2. **値域検証**: 年・月・日の有効範囲
3. **実在性検証**: `chrono::NaiveDate::parse_from_str`による解析
4. **論理検証**: うるう年、月末日の正確性
5. **関係検証**: 日付範囲の順序関係

#### 6. Property-basedテストの必須項目
- `generated_dates_are_actually_valid`: 生成された日付が実在する
- `date_range_always_ordered`: 日付範囲が常に順序を保つ
- `month_day_limits_are_respected`: 月ごとの日数制限を守る
- `leap_year_february_29_is_valid`: うるう年判定の正確性

## 📊 品質保証指標

### 必須達成目標
- **Property-basedテスト**: 重要関数100%カバレッジ
- **実行ケース数**: 1000+ケース/関数
- **テスト成功率**: 100%
- **境界値検証**: 全エッジケース網羅

### 継続監視指標
- **日次**: 新規関数のProperty追加
- **週次**: テスト失敗の原因分析・修正
- **月次**: テスト戦略の見直し・改善

---

**戦略的位置づけ**: Property-basedテストはプロジェクトの基盤品質保証戦略として最優先で実施されます。