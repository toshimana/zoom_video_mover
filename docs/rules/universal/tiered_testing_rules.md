# 段階的テスト実行ルール（汎用）

**適用範囲**: すべてのソフトウェア開発プロジェクト  
**依存性レベル**: Level 1 (Universal) - プロジェクト・技術非依存  
**参照ポリシー**: [testing_strategy_policy.md](../../policies/universal/testing_strategy_policy.md), [performance_monitoring_policy.md](../../policies/universal/performance_monitoring_policy.md)

## 1. 段階的テスト戦略

### 1.1 テスト実行レベル定義
```markdown
Level 1 - CI/CD高速テスト:
- 目的: 継続的インテグレーション・高速フィードバック
- 実行時間制限: 5分以内
- Property-basedテスト: 100ケース
- カバレッジ目標: 80%以上
- 実行タイミング: プルリクエスト・マージ前

Level 2 - Pre-Deploy検証テスト:
- 目的: デプロイ前の包括的検証
- 実行時間制限: 15分以内  
- Property-basedテスト: 500ケース
- カバレッジ目標: 85%以上
- 実行タイミング: デプロイ承認前・ステージング環境

Level 3 - 包括的品質保証テスト:
- 目的: 最高品質保証・長期安定性確認
- 実行時間制限: 制限なし
- Property-basedテスト: 1000ケース以上
- カバレッジ目標: 90%以上
- 実行タイミング: 夜間バッチ・週次実行
```

### 1.2 パフォーマンス要件との整合
```markdown
応答時間要件との両立:
- Level 1: API応答時間 < 100ms 維持
- Level 2: API応答時間 < 500ms 許容
- Level 3: API応答時間制限なし

CI/CDパイプライン要件:
- 全体パイプライン時間: 30分以内
- Level 1テスト時間: 5分以内（16.7%）
- ビルド・デプロイ時間: 25分以内（83.3%）
```

## 2. テスト実行制御

### 2.1 環境変数による制御
```bash
# Level 1: CI/CD高速テスト
export TEST_LEVEL=1
export PROPTEST_CASES=100
export TEST_TIMEOUT=300  # 5分

# Level 2: Pre-Deploy検証テスト  
export TEST_LEVEL=2
export PROPTEST_CASES=500
export TEST_TIMEOUT=900  # 15分

# Level 3: 包括的品質保証テスト
export TEST_LEVEL=3
export PROPTEST_CASES=1000
export TEST_TIMEOUT=0    # 制限なし
```

### 2.2 テスト選択ルール
```rust
// Rustでの実装例
#[cfg(test)]
mod tests {
    use super::*;
    
    // Level 1: 基本機能テスト（常時実行）
    #[test]
    fn test_basic_functionality() {
        // 基本機能の高速テスト
    }
    
    // Level 2: 統合テスト（Pre-Deploy時実行）
    #[test]
    #[ignore = "level2"]
    fn test_integration_comprehensive() {
        // 統合テスト・中程度の実行時間
    }
    
    // Level 3: 包括テスト（夜間バッチ実行）
    #[test] 
    #[ignore = "level3"]
    fn test_exhaustive_scenarios() {
        // 包括的テスト・長時間実行
    }
    
    // Property-basedテストの段階的実行
    proptest! {
        #![proptest_config(ProptestConfig {
            cases: get_proptest_cases(),
            timeout: get_test_timeout(),
            ..ProptestConfig::default()
        })]
        
        #[test]
        fn test_property_based(input in strategy()) {
            // Property-basedテスト実装
        }
    }
}

fn get_proptest_cases() -> u32 {
    match std::env::var("TEST_LEVEL").unwrap_or("1".to_string()).as_str() {
        "1" => 100,   // CI/CD高速
        "2" => 500,   // Pre-Deploy
        "3" => 1000,  // 包括的
        _ => 100,     // デフォルト
    }
}
```

## 3. CI/CD統合

### 3.1 GitHub Actions設定例
```yaml
# .github/workflows/ci.yml
name: CI/CD Pipeline

on:
  pull_request:
    branches: [main]
  push:
    branches: [main]

jobs:
  level1_tests:
    name: "Level 1 - 高速CI テスト"
    runs-on: ubuntu-latest
    env:
      TEST_LEVEL: 1
      PROPTEST_CASES: 100
    steps:
      - uses: actions/checkout@v3
      - name: Run Level 1 Tests
        run: |
          cargo test --lib --bins
          timeout 5m cargo test --test property_tests

  level2_tests:
    name: "Level 2 - Pre-Deploy検証"
    runs-on: ubuntu-latest
    needs: level1_tests
    if: github.ref == 'refs/heads/main'
    env:
      TEST_LEVEL: 2
      PROPTEST_CASES: 500
    steps:
      - uses: actions/checkout@v3
      - name: Run Level 2 Tests
        run: |
          cargo test --ignored level2
          timeout 15m cargo test --test integration_tests

  level3_tests:
    name: "Level 3 - 包括的品質保証"
    runs-on: ubuntu-latest
    schedule:
      - cron: '0 2 * * *'  # 毎日午前2時実行
    env:
      TEST_LEVEL: 3
      PROPTEST_CASES: 1000
    steps:
      - uses: actions/checkout@v3
      - name: Run Level 3 Tests
        run: |
          cargo test --ignored level3
          PROPTEST_VERBOSE=1 cargo test --test property_tests_extended
```

## 4. 品質保証バランス

### 4.1 段階的品質保証
```markdown
Level 1品質基準:
- 基本機能動作: 100%確認
- 重要パス: 完全テスト
- 回帰テスト: 主要機能
- リスク評価: 高リスク機能重点

Level 2品質基準:
- 統合機能: 包括的テスト
- エラーハンドリング: 詳細確認
- パフォーマンス: 基準値確認
- セキュリティ: 基本検証

Level 3品質基準:
- エッジケース: 完全網羅
- 長期安定性: ストレステスト
- パフォーマンス限界: 詳細分析
- セキュリティ: 包括的監査
```

### 4.2 失敗時エスカレーション
```markdown
Level 1失敗時:
- 即座のプルリクエスト停止
- 開発者への即座通知
- 修正後の再実行

Level 2失敗時:
- デプロイ承認停止
- チームリーダー通知
- 根本原因分析・対策

Level 3失敗時:
- 品質課題の特定・記録
- 改善計画策定
- 長期品質向上施策
```

## 5. メトリクス・改善

### 5.1 測定指標
```markdown
効率性指標:
- Level 1実行時間: < 5分維持
- Level 2実行時間: < 15分維持
- 全体CI時間: < 30分維持

品質指標:
- Level 1バグ検出率: 基準値設定
- Level 2統合問題検出率: 目標値達成
- Level 3品質課題発見数: 継続改善

バランス指標:
- 開発速度 vs 品質バランス
- CI時間 vs テストカバレッジ
- 自動化率 vs 手動確認品質
```

### 5.2 継続改善
```markdown
改善サイクル:
- 週次: Level 1効率性評価
- 月次: Level 2品質評価
- 四半期: Level 3包括評価

最適化施策:
- テスト選択アルゴリズム改善
- 並列実行・リソース最適化
- 重要度ベーステスト優先順位
- 自動化拡大・手動作業削減
```

---

**策定日**: 2025-08-09  
**適用範囲**: 全プロジェクト・全テスト実行  
**見直し頻度**: 月次またはパフォーマンス課題発生時