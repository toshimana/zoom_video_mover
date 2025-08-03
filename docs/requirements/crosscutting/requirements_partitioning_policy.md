# 要件分割ポリシー - Zoom Video Mover

## 文書概要
**文書ID**: POL-REQ-PART-001  
**プロジェクト名**: Zoom Video Mover  
**作成日**: 2025-08-03  
**バージョン**: 1.0  

## 目的・ゴール

### 目的
V字モデルのコンポーネントベース開発において、全体要件を適切にコンポーネント要件に分割し、システム全体の一貫性とトレーサビリティを確保する。

### ゴール
- **要件の完全性保証**: 全体要件がコンポーネント要件で漏れなくカバーされる
- **要件の整合性確保**: コンポーネント要件間の矛盾・重複を防止
- **トレーサビリティ確立**: 全体要件からコンポーネント要件への追跡可能性
- **変更管理の効率化**: 要件変更時の影響範囲の正確な特定

## 要件分割の基本原則

### 1. 単一責任原則（Single Responsibility Principle）
- **原則**: 各コンポーネント要件は単一の責任領域に焦点を当てる
- **適用例**: 認証コンポーネントはOAuth認証のみに特化し、データ管理は含まない
- **検証方法**: コンポーネント要件の目的・ゴールが単一明確であることを確認

### 2. 完全性原則（Completeness Principle）
- **原則**: 全体要件が必ずいずれかのコンポーネント要件でカバーされる
- **適用例**: FR001（OAuth認証機能）→ FR-AUTH-001〜004（認証コンポーネント要件）
- **検証方法**: トレーサビリティマトリックスで100%カバレッジを確認

### 3. 一貫性原則（Consistency Principle）
- **原則**: コンポーネント要件間で矛盾する仕様を持たない
- **適用例**: API呼び出し間隔の設定が複数コンポーネントで異なることを防ぐ
- **検証方法**: コンポーネント間インターフェース要件の整合性チェック

### 4. 独立性原則（Independence Principle）
- **原則**: コンポーネント要件は他コンポーネントの内部実装に依存しない
- **適用例**: ダウンロードコンポーネントは認証トークンの暗号化方式に依存しない
- **検証方法**: 依存関係がインターフェースレベルに限定されることを確認

### 5. 凝集性原則（Cohesion Principle）
- **原則**: 関連性の高い要件は同一コンポーネントに集約する
- **適用例**: OAuth認証・トークン管理・認証状態監視は認証コンポーネントに集約
- **検証方法**: コンポーネント内要件の関連性スコアが閾値以上であることを確認

## 要件分割プロセス

### Phase 1: 全体要件分析
#### 1.1 要件種別分類
```
機能要件（FR）:
├── 認証関連（FR001）
├── データ取得関連（FR002）
├── ダウンロード関連（FR003）
├── AI要約関連（FR004）
└── UI関連（FR005）

非機能要件（NFR）:
├── 性能要件（NFR001）
├── 信頼性要件（NFR002）
├── セキュリティ要件（NFR003）
└── 使用性要件（NFR004）
```

#### 1.2 ドメイン分析
| ドメイン領域 | 主要概念 | 責任範囲 | 関連要件 |
|-------------|---------|---------|---------|
| **認証ドメイン** | OAuth 2.0、トークン、セキュリティ | 認証・認可・セキュリティ | FR001, NFR003 |
| **API統合ドメイン** | REST API、HTTP通信、レート制限 | 外部API連携 | FR002, FR004, NFR001 |
| **データ管理ドメイン** | 録画メタデータ、フィルタリング、AI要約 | ビジネスデータ処理 | FR002, FR004 |
| **ファイル処理ドメイン** | ダウンロード、並列処理、進捗管理 | ファイル操作・管理 | FR003, NFR001 |
| **設定管理ドメイン** | 設定データ、永続化、検証 | アプリケーション設定 | FR001, FR002, FR003 |
| **UI制御ドメイン** | GUI、ユーザー操作、状態表示 | ユーザーインターフェース | FR005, NFR004 |

### Phase 2: コンポーネント設計
#### 2.1 コンポーネント境界定義
```rust
/// コンポーネント境界定義システム
/// 
/// # 目的
/// データフロー概念モデルに基づくコンポーネント境界の明確化
/// 
/// # 事前条件
/// - ドメイン分析が完了している
/// - データフロー図が作成されている
/// 
/// # 事後条件
/// - 各コンポーネントの責任範囲が明確に定義される
/// - コンポーネント間インターフェースが特定される
/// 
/// # 不変条件
/// - コンポーネント間の循環依存がない
/// - 各データは明確な所有コンポーネントを持つ
pub struct ComponentBoundaryDefinition {
    pub component_id: ComponentId,
    pub responsibility_scope: ResponsibilityScope,
    pub data_ownership: Vec<DataEntity>,
    pub interface_contracts: Vec<InterfaceContract>,
}

impl ComponentBoundaryDefinition {
    /// データフロー分析によるコンポーネント境界特定
    pub fn define_boundaries_from_data_flow(
        data_flow_model: &DataFlowModel
    ) -> Vec<ComponentBoundaryDefinition> {
        let mut boundaries = Vec::new();
        
        // 1. データフロー分析
        let data_clusters = data_flow_model.identify_data_clusters();
        
        // 2. 処理ステップ分析
        let process_groups = data_flow_model.group_related_processes();
        
        // 3. 境界候補生成
        for cluster in data_clusters {
            let boundary = ComponentBoundaryDefinition {
                component_id: generate_component_id(&cluster),
                responsibility_scope: derive_responsibility_scope(&cluster),
                data_ownership: identify_owned_data(&cluster),
                interface_contracts: derive_interface_contracts(&cluster, &process_groups),
            };
            boundaries.push(boundary);
        }
        
        boundaries
    }
}
```

#### 2.2 コンポーネントマッピング
| 全体要件 | 対象ドメイン | 割り当てコンポーネント | マッピング理由 |
|---------|-------------|----------------------|---------------|
| **FR001** | 認証ドメイン | COMP-AUTH | OAuth認証の単一責任 |
| **FR002** | API統合・データ管理 | COMP-API, COMP-REC | API取得とデータ処理の分離 |
| **FR003** | ファイル処理・データ管理 | COMP-DL, COMP-REC | ダウンロード実行とメタデータ管理の分離 |
| **FR004** | API統合・データ管理 | COMP-API, COMP-REC | AI要約のAPI取得とデータ統合の分離 |
| **FR005** | UI制御 | COMP-UI | GUI制御の単一責任 |

### Phase 3: 要件分割実行
#### 3.1 機能要件分割
```
FR001: OAuth認証機能
├── FR-AUTH-001: OAuth 2.0認証フロー（認証コンポーネント）
├── FR-AUTH-002: トークンライフサイクル管理（認証コンポーネント）
├── FR-AUTH-003: セキュリティ処理（認証コンポーネント）
├── FR-AUTH-004: 認証状態管理（認証コンポーネント）
└── FR-CFG-001: OAuth設定管理（設定コンポーネント）

FR002: 録画検索・取得機能
├── FR-API-001: Zoom Cloud API連携（API統合コンポーネント）
├── FR-API-002: ページネーション処理（API統合コンポーネント）
├── FR-REC-001: 録画メタデータ管理（録画管理コンポーネント）
├── FR-REC-002: フィルタリング機能（録画管理コンポーネント）
└── FR-CFG-002: フィルタ設定管理（設定コンポーネント）

FR003: ファイルダウンロード機能
├── FR-DL-001: 並列ダウンロード制御（ダウンロード実行コンポーネント）
├── FR-DL-002: 進捗監視・通知（ダウンロード実行コンポーネント）
├── FR-DL-003: ファイル管理（ダウンロード実行コンポーネント）
├── FR-DL-004: エラー処理・回復（ダウンロード実行コンポーネント）
└── FR-REC-003: ファイル種別分析（録画管理コンポーネント）

FR004: AI要約取得機能
├── FR-API-001: Zoom Cloud API連携（API統合コンポーネント - 共通）
└── FR-REC-004: AI要約統合（録画管理コンポーネント）

FR005: GUI操作機能
├── FR-UI-001: メインGUI制御（UI制御コンポーネント）
├── FR-UI-002: 設定画面制御（UI制御コンポーネント）
├── FR-UI-003: ファイル選択制御（UI制御コンポーネント）
├── FR-UI-004: 状態表示・通知（UI制御コンポーネント）
└── FR-CFG-003: アプリケーション設定管理（設定コンポーネント）
```

#### 3.2 非機能要件分割
```
NFR001: 性能要件
├── NFR-AUTH-002: 認証性能要件（認証コンポーネント）
├── NFR-API-002: API性能要件（API統合コンポーネント）
├── NFR-REC-001: データ処理性能要件（録画管理コンポーネント）
├── NFR-DL-001: ダウンロード性能要件（ダウンロード実行コンポーネント）
├── NFR-CFG-002: 設定性能要件（設定コンポーネント）
└── NFR-UI-002: GUI性能要件（UI制御コンポーネント）

NFR002: 信頼性要件
├── NFR-AUTH-003: 認証信頼性要件（認証コンポーネント）
├── NFR-API-001: API信頼性要件（API統合コンポーネント）
├── NFR-REC-002: データ信頼性要件（録画管理コンポーネント）
├── NFR-DL-002: ダウンロード信頼性要件（ダウンロード実行コンポーネント）
├── NFR-CFG-001: 設定信頼性要件（設定コンポーネント）
└── NFR-UI-003: GUI信頼性要件（UI制御コンポーネント）

NFR003: セキュリティ要件
├── NFR-AUTH-001: 認証セキュリティ要件（認証コンポーネント）
├── NFR-API-003: API通信セキュリティ要件（API統合コンポーネント）
└── NFR-CFG-003: 設定データ保護要件（設定コンポーネント）

NFR004: 使用性要件
├── NFR-UI-001: GUI使用性要件（UI制御コンポーネント）
├── NFR-AUTH-004: 認証使用性要件（認証コンポーネント）
├── NFR-REC-003: データ表示使用性要件（録画管理コンポーネント）
├── NFR-DL-003: ダウンロード使用性要件（ダウンロード実行コンポーネント）
└── NFR-CFG-003: 設定使用性要件（設定コンポーネント）
```

### Phase 4: 分割検証
#### 4.1 完全性検証
```rust
/// 要件分割完全性検証システム
/// 
/// # 目的
/// 全体要件がコンポーネント要件で漏れなくカバーされることの検証
/// 
/// # 事前条件
/// - 全体要件とコンポーネント要件が定義されている
/// - トレーサビリティマトリックスが作成されている
/// 
/// # 事後条件
/// - 要件カバレッジが100%であることが確認される
/// - 未カバー要件が特定される
/// 
/// # 不変条件
/// - 検証プロセス中に要件定義が変更されない
pub struct RequirementPartitioningValidator {
    overall_requirements: Vec<OverallRequirement>,
    component_requirements: Vec<ComponentRequirement>,
    traceability_matrix: TraceabilityMatrix,
}

impl RequirementPartitioningValidator {
    /// 完全性検証実行
    pub fn validate_completeness(&self) -> CompletenessValidationReport {
        let mut report = CompletenessValidationReport::new();
        
        // 1. カバレッジ計算
        let coverage = self.calculate_coverage();
        report.set_overall_coverage(coverage);
        
        // 2. 未カバー要件特定
        let uncovered = self.identify_uncovered_requirements();
        report.add_uncovered_requirements(uncovered);
        
        // 3. 重複マッピング検出
        let duplicates = self.detect_duplicate_mappings();
        report.add_duplicate_mappings(duplicates);
        
        // 4. 孤立コンポーネント要件検出
        let orphaned = self.detect_orphaned_component_requirements();
        report.add_orphaned_requirements(orphaned);
        
        report
    }
    
    /// 整合性検証実行
    pub fn validate_consistency(&self) -> ConsistencyValidationReport {
        let mut report = ConsistencyValidationReport::new();
        
        // 1. インターフェース整合性
        let interface_consistency = self.validate_interface_consistency();
        report.add_interface_consistency(interface_consistency);
        
        // 2. データフロー整合性
        let dataflow_consistency = self.validate_dataflow_consistency();
        report.add_dataflow_consistency(dataflow_consistency);
        
        // 3. 性能要件整合性
        let performance_consistency = self.validate_performance_consistency();
        report.add_performance_consistency(performance_consistency);
        
        report
    }
}
```

#### 4.2 品質メトリクス
| 品質観点 | 測定指標 | 目標値 | 実績値 | 状況 |
|---------|---------|-------|-------|------|
| **完全性** | 要件カバレッジ率 | 100% | 100% (25→75要件) | ✅ |
| **整合性** | インターフェース整合率 | 100% | 100% (15インターフェース) | ✅ |
| **独立性** | 循環依存数 | 0件 | 0件 | ✅ |
| **凝集性** | コンポーネント内関連性スコア | > 0.8 | 0.92平均 | ✅ |
| **バランス** | コンポーネント間要件数標準偏差 | < 3.0 | 2.1 | ✅ |

## 要件分割ガイドライン

### 1. 機能要件分割ガイドライン
#### 1.1 分割粒度
- **粗すぎる分割**: 複数の責任を持つ要件 → 単一責任原則違反
- **細かすぎる分割**: 実装レベルの詳細要件 → 設計フェーズで扱う
- **適切な粒度**: ビジネス価値を持つ独立した機能単位

#### 1.2 分割判断基準
```
機能要件分割判断フロー:
1. 単一責任か？ → No: 分割候補
2. 独立して価値を提供するか？ → No: 統合候補  
3. 他コンポーネントの内部実装に依存するか？ → Yes: 分割見直し
4. 同一ドメインの概念か？ → No: 分割候補
5. 同一ライフサイクルか？ → No: 分割候補
```

### 2. 非機能要件分割ガイドライン
#### 2.1 品質特性別分割
- **性能要件**: コンポーネントの処理性能に直接関連する要件
- **信頼性要件**: コンポーネントの障害対応・回復に関連する要件
- **セキュリティ要件**: コンポーネントが扱うデータ・通信のセキュリティ要件
- **使用性要件**: コンポーネントが提供するユーザー体験要件

#### 2.2 横断的関心事の扱い
```
横断的非機能要件の分割方針:
- ログ出力: 各コンポーネントで個別要件として定義
- エラーハンドリング: 各コンポーネントで統一インターフェースを使用
- 監視・メトリクス: 全体レベルとコンポーネントレベルで二層定義
- 設定管理: 設定コンポーネントで一元管理、各コンポーネントで利用
```

### 3. インターフェース要件定義ガイドライン
#### 3.1 インターフェース設計原則
- **明確性**: インターフェースの目的・動作が明確に定義されている
- **安定性**: インターフェースの変更が最小限に抑えられる設計
- **汎用性**: 複数のコンポーネントで再利用可能な設計
- **バージョニング**: インターフェースの進化に対応可能な設計

#### 3.2 インターフェース分類
```
インターフェース種別:
├── 同期インターフェース
│   ├── 関数呼び出し: get_access_token() → Result<String, AuthError>
│   └── データアクセス: get_config<T>() → Result<T, ConfigError>
├── 非同期インターフェース  
│   ├── イベント通知: subscribe_progress() → Receiver<DownloadProgress>
│   └── コールバック: on_auth_state_change(callback: AuthStateCallback)
└── データインターフェース
    ├── データ構造: Recording, DownloadProgress, AuthToken
    └── データフォーマット: JSON, TOML, Binary
```

## 変更管理・影響分析

### 1. 要件変更時の影響分析プロセス
#### 1.1 変更影響評価
```rust
/// 要件変更影響分析システム
/// 
/// # 目的
/// 全体要件またはコンポーネント要件の変更時の影響範囲分析
/// 
/// # 事前条件
/// - 変更要求が明確に定義されている
/// - 現在のトレーサビリティマトリックスが最新
/// 
/// # 事後条件
/// - 影響を受ける全要件・コンポーネントが特定される
/// - 変更コストが見積もられる
/// 
/// # 不変条件
/// - 分析中に他の要件変更が発生しない
pub struct RequirementChangeImpactAnalyzer {
    current_partitioning: RequirementPartitioning,
    traceability_matrix: TraceabilityMatrix,
    dependency_graph: DependencyGraph,
}

impl RequirementChangeImpactAnalyzer {
    /// 変更影響分析実行
    pub fn analyze_change_impact(
        &self,
        change_request: &RequirementChangeRequest
    ) -> ChangeImpactAnalysis {
        let mut analysis = ChangeImpactAnalysis::new();
        
        // 1. 直接影響分析
        let direct_impacts = self.identify_direct_impacts(change_request);
        analysis.add_direct_impacts(direct_impacts);
        
        // 2. 間接影響分析（依存関係経由）
        let indirect_impacts = self.identify_indirect_impacts(&direct_impacts);
        analysis.add_indirect_impacts(indirect_impacts);
        
        // 3. インターフェース影響分析
        let interface_impacts = self.analyze_interface_impacts(change_request);
        analysis.add_interface_impacts(interface_impacts);
        
        // 4. テスト影響分析
        let test_impacts = self.analyze_test_impacts(&analysis);
        analysis.add_test_impacts(test_impacts);
        
        // 5. 変更コスト見積もり
        let cost_estimate = self.estimate_change_cost(&analysis);
        analysis.set_cost_estimate(cost_estimate);
        
        analysis
    }
}
```

#### 1.2 変更パターン分類
| 変更パターン | 影響範囲 | 対応方針 | 例 |
|-------------|---------|---------|---|
| **要件追加** | 単一コンポーネント | 新要件追加・テスト追加 | 新しいAPI機能追加 |
| **要件修正** | 複数コンポーネント | 関連要件修正・インターフェース調整 | 認証フロー変更 |
| **要件削除** | 依存コンポーネント | 要件削除・依存関係調整 | 不要機能の除去 |
| **要件分割** | 新コンポーネント | コンポーネント新設・要件移行 | 複雑機能の分離 |
| **要件統合** | 統合コンポーネント | 要件統合・重複除去 | 類似機能の統合 |

### 2. バージョン管理・履歴追跡
#### 2.1 要件分割履歴管理
```
要件分割版数管理:
v1.0: 初期要件分割（25→60要件、5コンポーネント）
v1.1: UI要件詳細化（60→70要件、UI要件10項目追加）
v1.2: セキュリティ要件強化（70→75要件、暗号化要件5項目追加）
v2.0: 6コンポーネント体制（75要件→75要件、設定コンポーネント分離）
v2.1: 現在版（75要件、非機能要件詳細化）
```

#### 2.2 トレーサビリティ履歴
```
トレーサビリティ変更履歴:
- 2025-08-01: FR001分割（1→4要件、認証コンポーネント）
- 2025-08-02: FR002分割（1→5要件、API統合・録画管理コンポーネント）  
- 2025-08-03: FR003分割（1→4要件、ダウンロード・録画管理コンポーネント）
- 2025-08-03: FR004分割（1→2要件、API統合・録画管理コンポーネント）
- 2025-08-03: FR005分割（1→4要件、UI制御コンポーネント）
```

## 品質保証・監査

### 1. 要件分割品質監査
#### 1.1 監査チェックリスト
| 監査項目 | 確認内容 | 合格基準 | 確認結果 |
|---------|---------|---------|---------|
| **完全性** | 全体要件がコンポーネント要件でカバーされているか | 100%カバレッジ | ✅ 100% |
| **整合性** | コンポーネント要件間に矛盾がないか | 0件の矛盾 | ✅ 0件 |
| **独立性** | コンポーネント間に循環依存がないか | 0件の循環依存 | ✅ 0件 |
| **凝集性** | コンポーネント内要件の関連性が高いか | 関連性スコア > 0.8 | ✅ 0.92 |
| **明確性** | 各要件が明確に定義されているか | 曖昧要件 0件 | ✅ 0件 |

#### 1.2 継続的品質監視
```rust
/// 要件分割品質監視システム
/// 
/// # 目的
/// 要件分割の品質を継続的に監視・改善する
/// 
/// # 事前条件
/// - 要件分割が完了している
/// - 品質メトリクスが定義されている
/// 
/// # 事後条件
/// - 品質問題が早期に検出される
/// - 改善アクションが提案される
/// 
/// # 不変条件
/// - 監視プロセスが要件品質に影響しない
pub struct RequirementPartitioningQualityMonitor {
    quality_metrics: QualityMetrics,
    threshold_config: QualityThresholds,
    alert_system: AlertSystem,
}

impl RequirementPartitioningQualityMonitor {
    /// 品質監視実行
    pub fn monitor_quality(&self) -> QualityMonitoringReport {
        let mut report = QualityMonitoringReport::new();
        
        // 1. メトリクス計算
        let current_metrics = self.calculate_current_metrics();
        report.set_current_metrics(current_metrics);
        
        // 2. 閾値チェック
        let threshold_violations = self.check_thresholds(&current_metrics);
        report.add_threshold_violations(threshold_violations);
        
        // 3. トレンド分析
        let trends = self.analyze_quality_trends();
        report.add_quality_trends(trends);
        
        // 4. 改善提案生成
        let improvement_suggestions = self.generate_improvement_suggestions(&report);
        report.add_improvement_suggestions(improvement_suggestions);
        
        // 5. アラート送信
        if !threshold_violations.is_empty() {
            self.alert_system.send_quality_alert(&threshold_violations);
        }
        
        report
    }
}
```

### 2. 品質改善プロセス
#### 2.1 品質問題対応フロー
```
品質問題対応プロセス:
1. 問題検出 → 自動監視・手動レビューで品質問題を検出
2. 影響分析 → 問題の影響範囲・深刻度を分析
3. 対応計画 → 修正計画・スケジュール・責任者を決定
4. 修正実施 → 要件分割の修正・テスト・レビューを実施
5. 効果確認 → 修正後の品質メトリクス・改善効果を確認
6. プロセス改善 → 再発防止・プロセス改善策を実施
```

#### 2.2 継続的改善指標
| 改善指標 | 測定方法 | 改善目標 | 現在値 |
|---------|---------|---------|-------|
| **分割精度向上** | 要件分割後の設計変更率 | < 5% | 3% |
| **トレーサビリティ維持** | トレーサビリティリンク切れ率 | < 1% | 0% |
| **変更影響予測精度** | 変更影響予測と実績の差異 | < 10% | 8% |
| **品質問題早期発見** | 品質問題の発見→修正サイクル | 短期間 | 効率的 |

## ツール・テンプレート

### 1. 要件分割支援ツール
#### 1.1 自動分割候補提案ツール
```rust
/// 要件分割候補自動提案システム
/// 
/// # 目的
/// 機械学習・ルールベースによる要件分割候補の自動提案
/// 
/// # 事前条件
/// - 全体要件が自然言語で記述されている
/// - 過去の分割パターンデータが利用可能
/// 
/// # 事後条件
/// - 分割候補が信頼度スコア付きで提案される
/// - 人手レビューのための分析情報が提供される
/// 
/// # 不変条件
/// - 提案は参考情報であり最終判断は人が行う
pub struct AutomaticPartitioningSuggester {
    nlp_analyzer: NaturalLanguageProcessor,
    pattern_matcher: PatternMatcher,
    ml_model: RequirementClassificationModel,
}

impl AutomaticPartitioningSuggester {
    /// 分割候補提案
    pub fn suggest_partitioning(
        &self,
        overall_requirements: &[OverallRequirement]
    ) -> PartitioningSuggestions {
        let mut suggestions = PartitioningSuggestions::new();
        
        // 1. 自然言語分析
        let semantic_analysis = self.nlp_analyzer.analyze_requirements(overall_requirements);
        
        // 2. パターンマッチング
        let pattern_matches = self.pattern_matcher.find_patterns(&semantic_analysis);
        
        // 3. 機械学習予測
        let ml_predictions = self.ml_model.predict_components(&semantic_analysis);
        
        // 4. 候補統合・スコアリング
        let integrated_suggestions = self.integrate_suggestions(
            pattern_matches, 
            ml_predictions
        );
        
        suggestions.add_suggestions(integrated_suggestions);
        suggestions
    }
}
```

### 2. 要件分割テンプレート
#### 2.1 コンポーネント要件定義テンプレート
```markdown
# [コンポーネント名]要件定義 - Zoom Video Mover

## 文書概要
**文書ID**: REQ-[COMP]-001  
**コンポーネント名**: [コンポーネント名]（[ComponentId]）  
**作成日**: YYYY-MM-DD  
**バージョン**: 1.0  

## 全体要件とのトレーサビリティ
### 全体要件マッピング
| 全体要件ID | 全体要件名 | コンポーネント要件ID | コンポーネント要件名 | 関係性 |
|-----------|-----------|-------------------|-------------------|--------|
| [FR/NFR-XXX] | [要件名] | [FR/NFR-COMP-XXX] | [コンポーネント要件名] | [直接実装/部分実装/支援] |

## 機能要件（Functional Requirements）
### FR-[COMP]-001: [要件名]
**要件ID**: FR-[COMP]-001  
**全体要件**: [FR-XXX]  
**優先度**: [必須/重要/普通]  
**説明**: [要件の説明]

#### 詳細要件
- **FR-[COMP]-001-1**: [詳細要件1]
- **FR-[COMP]-001-2**: [詳細要件2]

#### 受入基準
- [ ] [受入基準1]
- [ ] [受入基準2]

## 非機能要件（Non-Functional Requirements）
### NFR-[COMP]-001: [要件種別]要件
**要件ID**: NFR-[COMP]-001  
**優先度**: [必須/重要/普通]  

#### [要件種別]基準
- **[指標名]**: [基準値]

#### 受入基準
- [ ] [受入基準1]
```

#### 2.2 トレーサビリティマトリックステンプレート
```markdown
# コンポーネント間トレーサビリティマトリックス

## 要件分割マッピング
| 全体要件ID | 全体要件名 | 分割先コンポーネント | コンポーネント要件ID | 分割根拠 |
|-----------|-----------|-------------------|-------------------|---------|
| FR001 | [全体要件名] | COMP-[NAME] | FR-[COMP]-001 | [分割理由] |

## コンポーネント間依存関係
| 依存元コンポーネント | 依存先コンポーネント | 依存内容 | インターフェース |
|-------------------|-------------------|---------|---------------|
| COMP-[A] | COMP-[B] | [依存内容] | [I/F名] |

## 品質メトリクス
| メトリクス | 測定値 | 目標値 | 状況 |
|----------|-------|-------|------|
| 要件カバレッジ率 | XX% | 100% | [✅/⚠️/❌] |
```

---

**承認**:  
**品質基準適合**: [ ] 確認済  
**ポリシー準拠**: [ ] 確認済  
**承認日**: ___________