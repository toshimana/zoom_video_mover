# CLAUDE.md - Zoom Video Mover プロジェクト

## プロジェクト概要
ZoomクラウドレコーディングをローカルにダウンロードするRust GUIアプリケーション

## プロジェクト構造
```
src/
├── main.rs          # GUIアプリケーションのエントリーポイント
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
# GUIアプリケーション
cargo run

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

### 二層トレーサビリティ管理体制

#### 1. 要件プロセス内トレーサビリティ (`requirements_traceability_matrix.md`)
要件定義Phase0-6内での成果物間関係性を追跡：

**対象範囲**:
- Phase間の成果物関連付け: 前フェーズ成果物→次フェーズ成果物  
- Phase内の要件間整合性: ステークホルダー要求→システム要件→非機能要件
- 要件変更の要件プロセス内影響範囲分析

#### 2. 全体プロセス間トレーサビリティ (`overall_traceability_matrix.md`)  
要件→設計→実装→テスト間の一貫性を追跡：

**対象範囲**:
- 要件プロセス→設計プロセス: システム要件→アーキテクチャ設計
- 設計プロセス→実装プロセス: 詳細設計→Rustコード実装  
- 実装プロセス→テストプロセス: コード→単体・統合・E2Eテスト
- プロセス間変更影響の全体分析
#### トレーサビリティ管理ルール
**要件プロセス内**:
- Phase間成果物関連付けの維持
- 要件変更時の要件プロセス内影響分析  
- Phase完了時の要件整合性監査

**プロセス間**:
- 要件→設計→実装→テストの完全トレース
- プロセス間変更影響の全体分析
- 品質ゲート通過時の整合性確認

#### 品質指標
- **要件プロセス内完全性**: Phase間関連付け率 = 100%
- **プロセス間完全性**: 要件→テスト追跡可能率 = 100%  
- **更新完全性**: 変更時のマトリックス更新率 = 100%

### トレーサビリティマトリックス参照

#### 詳細トレーサビリティ情報
詳細なトレーサビリティ情報は以下の専用ファイルを参照：

**要件プロセス内トレーサビリティ**:
- `docs/requirements/crosscutting/requirements_traceability_matrix.md`
- Phase0-6内の成果物間関係性
- 要件変更の要件プロセス内影響分析

**プロセス間トレーサビリティ**:
- `docs/requirements/crosscutting/overall_traceability_matrix.md`
- 要件→設計→実装→テストの完全トレース
- プロセス間変更影響の全体分析

#### 変更管理プロセス

1. **要件変更時**: 要件プロセス内影響分析 → プロセス間影響分析 → 実装・テスト更新
2. **設計変更時**: プロセス間影響分析 → 実装・テスト更新  
3. **実装変更時**: 事前条件・事後条件・不変条件の維持 → テスト更新

詳細な変更管理手順は以下を参照：
- `docs/requirements/crosscutting/change_management.md`

## トラブルシューティング参考
- README.mdの詳細なトラブルシューティングセクション参照
- 特にZoom OAuth設定とWindows環境の問題

## 🤖 人の判断が必要な事項ガイドライン

### 概要
Claude Code Assistantが効果的に動作するために、人からの明確な判断が必要となる事項の基準。
詳細は `human_judgment_guidelines.md` を参照。

### 主要カテゴリ

#### 1. アーキテクチャ・設計判断
- **新しい技術選択**: ライブラリ・フレームワークの導入判断
- **トレードオフ**: パフォーマンス vs 可読性の選択
- **API設計**: 破壊的変更の承認、後方互換性方針
- **セキュリティレベル**: 暗号化方式、認証強度の設定

#### 2. 機能仕様・要件判断
- **曖昧性解決**: 機能の詳細動作仕様、エラー処理方針
- **優先度設定**: ユーザー体験の優先順位
- **デフォルト値**: 設定値、閾値の具体化
- **非機能要件**: パフォーマンス目標値、可用性要件

#### 3. 実装戦略・優先度判断
- **開発優先度**: 機能実装 vs バグ修正のバランス
- **テスト戦略**: カバレッジ目標、テスト種別の優先度
- **リファクタリング**: タイミング、範囲の決定
- **技術負債**: 対応の優先度と方法

#### 4. ユーザー体験・インターフェース判断
- **UI/UX詳細**: インターフェース設計、操作フロー
- **エラー表現**: 技術的 vs ユーザーフレンドリー
- **進捗表示**: 詳細度、表示方法
- **アクセシビリティ**: 対応レベル、実装方法

#### 5. セキュリティ・プライバシー判断
- **データ保護**: 認証情報の保存方法、暗号化強度
- **ログ出力**: 含める情報の範囲、詳細度
- **外部通信**: 証明書検証、プロキシ対応
- **一時ファイル**: 扱い方、削除ポリシー

#### 6. 運用・保守判断
- **ログ・監視**: 出力レベル、保存期間
- **設定管理**: 可能項目の範囲、デフォルト値
- **エラー通知**: 方法、頻度
- **パフォーマンス監視**: 範囲、閾値

### 判断を求める際のフォーマット

```
## 判断が必要な事項：[事項名]

### 背景・状況
[現在の状況、発生した問題、実装の文脈]

### 選択肢
A) [選択肢1] - [メリット/デメリット]
B) [選択肢2] - [メリット/デメリット] 
C) [選択肢3] - [メリット/デメリット]

### 影響範囲
- 実装への影響：[具体的な変更箇所]
- ユーザーへの影響：[体験の変化]
- 開発への影響：[工数、複雑度の変化]

### 推奨案
[Claudeの推奨とその理由]

### 判断をお願いします
[具体的に何を決めてほしいかを明記]
```

### 緊急度レベル
- **🔴 緊急**: 実装がブロックされる
- **🟡 重要**: 品質・体験に大きく影響  
- **🟢 通常**: 将来的な改善・最適化

### 効果的な指示のコツ

#### ✅ 良い指示例
```
OAuth認証失敗時に、ユーザーに再認証を促すダイアログを表示し、
3回失敗したらアプリケーションを終了する処理を実装して
```

#### ❌ 避けるべき指示例
```
エラー処理を改善して
```

### 自動判断可能な事項（参考）
- 一般的なコーディング規約の適用
- 既存パターンに沿った実装
- 明確な仕様に基づく実装
- バグ修正（明確な不具合）
- コードリファクタリング（可読性向上）

### 制約・前提の明示推奨
- 技術的制約（使用可能ライブラリ等）
- 時間的制約（リリース期限等）
- リソース制約（メモリ、ファイルサイズ等）
- 互換性要件（既存機能への影響）

**詳細ガイドライン**: `human_judgment_guidelines.md` 参照

## 📋 プロジェクト品質管理・報告体制

### 🚨 重要：矛盾・不整合の検出・報告プロセス

Claude Code Assistantは、指示されたタスク実施時にプロジェクト内ポリシー間で矛盾・不整合を検出した場合、以下の手順で対応します：

#### 1. 矛盾・不整合の検出
- **分析対象**: 全ポリシー文書間の整合性
- **分析観点**: 用語定義、プロセス手順、品質基準、技術選択、トレーサビリティ
- **検出基準**: 影響度（高/中/低）、緊急度（🔴緊急/🟡重要/🟢通常）

#### 2. 打ち上げ事項ファイルの出力
**出力先**: `docs/analysis/policy_consistency_issues.md`

**内容**:
- 🔴 重要な不整合（要対応）
- 🟡 中程度の不整合（改善推奨）  
- 🟢 軽微な不整合（将来対応）
- 整合性スコア・対応優先度
- 推奨アクションプラン

#### 3. CLAUDE.md への反映
**今回実施事項**:
- 要件定義プロセス・成果物・関係性の体系化
- RDRAベース7段階プロセスの実装
- 25種類成果物の依存関係図
- フェーズゲート基準18項目の確立

#### 4. 報告体制
**即座報告**: 重要な矛盾・不整合の発見
**定期報告**: プロジェクト品質向上提案
**フォローアップ**: 解決策実施後の効果確認

### 🎯 プロジェクト品質保証の基盤

#### Property-basedテスト戦略（基盤）
- **位置づけ**: 品質保証の基盤戦略
- **実行規模**: 1000ケース以上の自動検証
- **適用領域**: データ整合性・境界値・異常系の完全検証
- **効果**: 手動テストでは困難な網羅的品質保証を実現

#### 要件定義プロセス（RDRA基準）
- **7段階プロセス**: Phase 0（準備）→ Phase 6（承認）
- **25種類成果物**: 体系的な文書管理
- **18項目品質基準**: フェーズゲート管理
- **完全トレーサビリティ**: 要件→設計→実装→テスト

#### 継続的品質改善
- **矛盾検出の自動化**: タスク実施時の整合性チェック
- **段階的改善**: 優先度に基づく計画的解決
- **組織学習**: 改善内容の標準化・ナレッジ蓄積

### 📊 現在の整合性状況

**総合整合性スコア**: 87.5%  
**主要強み**: 要件管理・技術選択・トレーサビリティの高い統一性  
**改善領域**: Property-basedテスト位置づけの統一（最優先）  

**次回確認**: Phase 1改善完了後（24時間以内）