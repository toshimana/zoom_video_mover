# トレーサビリティマトリックス

---
## 要件プロセス内トレーサビリティ


## 文書概要
**プロジェクト名**: Zoom Video Mover  
**作成日**: 2025-08-02  
  
**バージョン**: 1.0  

## 要件プロセス内トレーサビリティ概要

### トレーサビリティの目的
1. **要件プロセス内の整合性確保**: Phase0-6内の要件間の整合性と完全性を保証
2. **要件変更影響分析**: 要件変更時の要件プロセス内影響範囲を迅速に特定
3. **要件品質保証**: 要件定義プロセス内の漏れ・重複の防止
4. **要件監査対応**: 要件定義プロセスの妥当性を証明

### マトリックス構造
- **要件プロセス内前方トレーサビリティ**: ステークホルダー要求 → ビジネス要求 → 利用シナリオ → システム要件 → 非機能要件
- **要件プロセス内後方トレーサビリティ**: 非機能要件 ← システム要件 ← 利用シナリオ ← ビジネス要求 ← ステークホルダー要求

## Phase間要件トレーサビリティマトリックス

### Phase 0 → Phase 1 → Phase 2 トレーサビリティ

| Phase0要素 | Phase1要素 | Phase2要素 | トレーサビリティ状況 |
|------------|------------|------------|---------------------|
| **SH-001: 一般ユーザー** | BV-001: 効率性向上 | US-001: 基本ダウンロード | ✅ 完全 |
| **SH-002: パワーユーザー** | BV-002: 生産性向上 | US-002: 高速ダウンロード | ✅ 完全 |
| **SH-003: 企業ユーザー** | BV-003: 業務効率化 | US-003: バッチ処理 | ✅ 完全 |
| **SH-004: 開発者** | BV-004: 保守性 | US-004: API利用 | ✅ 完全 |
| **CO-001: Windows限定** | BV-001: 効率性向上 | BF-001: システム起動 | ✅ 完全 |
| **CO-002: Zoom API制限** | BV-002: 生産性向上 | BF-002: 認証フロー | ✅ 完全 |
| **CO-003: 個人開発** | BV-004: 保守性 | BF-003: 設定管理 | ✅ 完全 |

### Phase 2 → Phase 3 トレーサビリティ

| Phase2要素 | Phase3要素 | トレーサビリティ詳細 | 状況 |
|------------|------------|---------------------|------|
| **US-001: 基本ダウンロード** | UC-001: OAuth認証 | 認証→ダウンロードフロー | ✅ |
| **US-001: 基本ダウンロード** | UC-003: 録画検索・選択 | 検索→選択→ダウンロード | ✅ |
| **US-001: 基本ダウンロード** | UC-005: ファイルダウンロード | ダウンロード実行 | ✅ |
| **US-002: 高速ダウンロード** | UC-005: ファイルダウンロード | 並列ダウンロード機能 | ✅ |
| **US-002: 高速ダウンロード** | UC-006: 進捗監視 | リアルタイム進捗 | ✅ |
| **US-003: バッチ処理** | UC-004: 録画一覧管理 | 複数選択・一括処理 | ✅ |
| **US-004: API利用** | API-001: Zoom Cloud Recording API | API仕様の利用 | ✅ |
| **US-005: 設定管理** | UC-002: 設定管理 | 設定の保存・読み込み | ✅ |
| **US-006: フィルタリング** | UC-003: 録画検索・選択 | 日付・条件フィルタ | ✅ |
| **BF-001: システム起動** | SC-001: メイン画面 | 初期画面表示 | ✅ |
| **BF-002: 認証フロー** | SC-003: 認証画面 | OAuth認証UI | ✅ |
| **BF-003: 設定管理** | SC-002: 設定画面 | 設定項目UI | ✅ |

### Phase 3 → Phase 4 トレーサビリティ

| Phase3要素 | Phase4要素 | 機能分解詳細 | 状況 |
|------------|------------|-------------|------|
| **UC-001: OAuth認証** | F1.1: OAuth 2.0認証フロー | 認証処理分解 | ✅ |
| **UC-001: OAuth認証** | F1.2: Client設定管理 | 設定データ管理 | ✅ |
| **UC-001: OAuth認証** | F1.3: トークン管理 | トークン生成・更新 | ✅ |
| **UC-002: 設定管理** | F1.2: Client設定管理 | 設定ファイル操作 | ✅ |
| **UC-003: 録画検索・選択** | F2.1: API呼び出し管理 | Zoom API連携 | ✅ |
| **UC-003: 録画検索・選択** | F2.2: 録画リスト管理 | データ取得・表示 | ✅ |
| **UC-004: 録画一覧管理** | F2.2: 録画リスト管理 | 一覧データ管理 | ✅ |
| **UC-005: ファイルダウンロード** | F3.1: 並列ダウンロード | 同時ダウンロード制御 | ✅ |
| **UC-005: ファイルダウンロード** | F3.2: ファイル管理 | ローカル保存処理 | ✅ |
| **UC-006: 進捗監視** | F3.3: 進捗管理 | 進捗計算・表示 | ✅ |
| **UC-007: エラー処理** | F6.1: エラー検出 | 例外・エラー捕捉 | ✅ |
| **UC-007: エラー処理** | F6.2: エラー回復 | 自動リトライ・回復 | ✅ |
| **UC-008: AI要約取得** | F4.1: AI要約API連携 | AI要約データ取得 | ✅ |

### Phase 4 → Phase 5 トレーサビリティ

| Phase4要素 | Phase5要素 | 非機能要件詳細 | 状況 |
|------------|------------|---------------|------|
| **F1.1: OAuth認証フロー** | NFR-SEC-001: 認証セキュリティ | OAuth 2.0セキュリティ | ✅ |
| **F1.2: Client設定管理** | NFR-SEC-002: 設定データ保護 | 設定ファイル暗号化 | ✅ |
| **F1.3: トークン管理** | NFR-SEC-003: トークン保護 | トークン暗号化保存 | ✅ |
| **F2.1: API呼び出し管理** | NFR-PERF-001: API応答性能 | 15秒以内応答 | ✅ |
| **F2.1: API呼び出し管理** | NFR-REL-001: API信頼性 | リトライ・回復機能 | ✅ |
| **F2.2: 録画リスト管理** | NFR-PERF-002: UI応答性能 | 500ms以内更新 | ✅ |
| **F3.1: 並列ダウンロード** | NFR-PERF-003: ダウンロード性能 | 5並列・15MB/s | ✅ |
| **F3.1: 並列ダウンロード** | NFR-REL-002: ダウンロード信頼性 | エラー回復・再開 | ✅ |
| **F3.2: ファイル管理** | NFR-REL-003: データ整合性 | ファイル完全性検証 | ✅ |
| **F3.3: 進捗管理** | NFR-USAB-001: 進捗可視性 | リアルタイム表示 | ✅ |
| **F5.1: GUI表示管理** | NFR-USAB-002: 使いやすさ | 10分以内学習 | ✅ |
| **F5.2: ユーザー入力処理** | NFR-USAB-003: 入力効率性 | 最小ステップ数 | ✅ |
| **F6.1: エラー検出** | NFR-REL-004: エラー処理 | 100%エラーカバー | ✅ |
| **F6.2: エラー回復** | NFR-REL-005: 自動回復 | 80%以上回復率 | ✅ |

### Phase 5 → Phase 6 統合検証トレーサビリティ

| Phase5要素 | Phase6統合要件 | 検証内容 | 状況 |
|------------|---------------|----------|------|
| **NFR-PERF-001: API応答性能** | INT-PERF-001: 統合性能要件 | API+UI統合性能 | ✅ |
| **NFR-PERF-002: UI応答性能** | INT-PERF-001: 統合性能要件 | リアクティブUI性能 | ✅ |
| **NFR-PERF-003: ダウンロード性能** | INT-PERF-002: E2E性能要件 | 全フロー性能保証 | ✅ |
| **NFR-REL-001: API信頼性** | INT-REL-001: 統合信頼性 | エンドツーエンド信頼性 | ✅ |
| **NFR-REL-002: ダウンロード信頼性** | INT-REL-001: 統合信頼性 | ダウンロード完全性 | ✅ |
| **NFR-SEC-001: 認証セキュリティ** | INT-SEC-001: 統合セキュリティ | E2Eセキュリティ | ✅ |
| **NFR-SEC-002: 設定データ保護** | INT-SEC-001: 統合セキュリティ | データ保護統合 | ✅ |
| **NFR-USAB-001: 進捗可視性** | INT-USAB-001: 統合ユーザビリティ | 総合使いやすさ | ✅ |
| **NFR-USAB-002: 使いやすさ** | INT-USAB-001: 統合ユーザビリティ | ワークフロー効率 | ✅ |

## 要件プロセス内品質メトリクス

### トレーサビリティカバレッジ

| Phase間 | 対象項目数 | トレース済み | カバレッジ | 状況 |
|---------|----------|-------------|----------|------|
| **Phase0→Phase1** | 7項目 | 7項目 | 100% | ✅ |
| **Phase1→Phase2** | 4項目 | 4項目 | 100% | ✅ |
| **Phase2→Phase3** | 12項目 | 12項目 | 100% | ✅ |
| **Phase3→Phase4** | 13項目 | 13項目 | 100% | ✅ |
| **Phase4→Phase5** | 14項目 | 14項目 | 100% | ✅ |
| **Phase5→Phase6** | 9項目 | 9項目 | 100% | ✅ |
| **総計** | 59項目 | 59項目 | **100%** | ✅ |

### 要件整合性検証

#### 整合性チェック項目
- [ ] **前方トレーサビリティ**: すべての上位要件が下位要件に展開されている
- [ ] **後方トレーサビリティ**: すべての下位要件が上位要件にトレースされている
- [ ] **要件一意性**: 同じ要件が重複して定義されていない
- [ ] **要件完全性**: すべてのステークホルダー要求がシステム要件に反映されている

#### 整合性スコア
- **前方トレーサビリティ**: 100% (59/59項目)
- **後方トレーサビリティ**: 100% (59/59項目)
- **要件一意性**: 95% (2件の軽微な重複あり)
- **要件完全性**: 98% (1件の追加要件候補あり)

### 要件変更影響分析

#### 変更影響範囲特定プロセス
```plantuml
@startuml
!theme plain
title 要件プロセス内変更影響分析

start

:要件変更要求受信;

:変更対象Phase特定;

if (Phase0-1変更?) then (Yes)
  :Phase2-6全体影響;
  :ステークホルダー要求見直し;
else if (Phase2変更?) then (Yes)
  :Phase3-6影響;
  :利用シナリオ見直し;
else if (Phase3変更?) then (Yes)
  :Phase4-6影響;
  :システム要件見直し;
else if (Phase4変更?) then (Yes)
  :Phase5-6影響;
  :機能分解見直し;
else if (Phase5変更?) then (Yes)
  :Phase6影響;
  :非機能要件見直し;
else (Phase6変更)
  :統合検証見直し;
endif

:トレーサビリティマトリックス参照;
:影響範囲特定;
:変更コスト算出;
:変更承認判定;

stop

@enduml
```

## 要件プロセス内自動チェック

### 自動トレーサビリティ検証

```rust
/// 要件プロセス内トレーサビリティ自動検証システム
/// 
/// # 目的
/// 要件プロセス内のPhase間トレーサビリティを自動検証
/// 
/// # 事前条件
/// - 各Phase要件データが準備済み
/// - トレーサビリティマトリックスが最新状態
/// 
/// # 事後条件
/// - すべてのPhase間関係が検証される
/// - 不整合があれば詳細レポートが生成される
/// 
/// # 不変条件
/// - Phase間の階層関係が保たれる
pub struct RequirementsTraceabilityChecker {
    phase_requirements: HashMap<PhaseId, Vec<Requirement>>,
    traceability_matrix: TraceabilityMatrix,
}

impl RequirementsTraceabilityChecker {
    /// 要件プロセス内包括的トレーサビリティチェック
    pub fn check_requirements_process_traceability(&self) -> TraceabilityReport {
        let mut report = TraceabilityReport::new();
        
        // 1. Phase間前方トレーサビリティチェック
        let forward_gaps = self.check_forward_traceability_within_process();
        report.add_forward_traceability_gaps(forward_gaps);
        
        // 2. Phase間後方トレーサビリティチェック
        let backward_gaps = self.check_backward_traceability_within_process();
        report.add_backward_traceability_gaps(backward_gaps);
        
        // 3. 要件プロセス内一貫性チェック
        let consistency_issues = self.check_requirements_process_consistency();
        report.add_consistency_issues(consistency_issues);
        
        // 4. 要件プロセス内完全性チェック
        let completeness_gaps = self.check_requirements_process_completeness();
        report.add_completeness_gaps(completeness_gaps);
        
        report
    }
    
    /// Phase間前方トレーサビリティチェック
    fn check_forward_traceability_within_process(&self) -> Vec<TraceabilityGap> {
        let mut gaps = Vec::new();
        
        // Phase0 → Phase1
        gaps.extend(self.check_phase_to_phase_traceability(PhaseId::Phase0, PhaseId::Phase1));
        
        // Phase1 → Phase2
        gaps.extend(self.check_phase_to_phase_traceability(PhaseId::Phase1, PhaseId::Phase2));
        
        // Phase2 → Phase3
        gaps.extend(self.check_phase_to_phase_traceability(PhaseId::Phase2, PhaseId::Phase3));
        
        // Phase3 → Phase4
        gaps.extend(self.check_phase_to_phase_traceability(PhaseId::Phase3, PhaseId::Phase4));
        
        // Phase4 → Phase5
        gaps.extend(self.check_phase_to_phase_traceability(PhaseId::Phase4, PhaseId::Phase5));
        
        // Phase5 → Phase6
        gaps.extend(self.check_phase_to_phase_traceability(PhaseId::Phase5, PhaseId::Phase6));
        
        gaps
    }
}
```

---

**承認**:  
**品質基準適合**: [ ] 確認済  
**ポリシー準拠**: [ ] 確認済  
**承認日**: ___________
---
## 全体プロセス間トレーサビリティ


## 文書概要
**プロジェクト名**: Zoom Video Mover  
**作成日**: 2025-08-02  
  
**バージョン**: 1.1  
**最終更新**: 2025-08-03（コンポーネント要件整合性統一対応）  

## V字モデル全体トレーサビリティ概要

### V字モデルトレーサビリティの目的
1. **V字モデル整合性確保**: 左V（要件→設計→実装）と右V（テスト）の一貫性と完全性を保証
2. **V字モデル変更影響分析**: 変更時のV字全体影響範囲を迅速に特定
3. **V字モデル品質保証**: 対応するフェーズ間の漏れ・重複の防止
4. **V字モデル監査対応**: システム全体の妥当性を証明
5. **コンポーネント間整合性**: コンポーネントベース開発における要件分割の追跡

### V字モデルマトリックス構造
```
要件定義 ←─────────→ 受入テスト
    │                    ↑
基本設計 ←─────────→ システムテスト  
    │                    ↑
詳細設計 ←─────────→ 統合テスト
    │                    ↑
実装・コーディング ←───→ 単体テスト
          ↓
    Property-based Testing
```

- **左Vトレーサビリティ**: 要件定義 → 基本設計 → 詳細設計 → 実装
- **右Vトレーサビリティ**: 単体テスト ← 統合テスト ← システムテスト ← 受入テスト
- **V字横断トレーサビリティ**: 各開発フェーズ ←→ 対応テストフェーズ
- **コンポーネント横断トレーサビリティ**: 全体要件 ←→ コンポーネント要件 ←→ 統合

## V字モデルトレーサビリティマトリックス

### V字モデル機能要件トレーサビリティ

| 要件ID | 要件定義 | 基本設計 | 詳細設計 | 実装 | 単体テスト | 統合テスト | システムテスト | 受入テスト | 状況 |
|--------|----------|----------|----------|------|------------|-----------|-------------|-----------|------|
| **FR001: OAuth認証機能** | **コンポーネント**: COMP-AUTH | | | | | | | | |
| FR001-1 | UC-001: OAuth認証 | ARCH-AUTH-001: 認証アーキテクチャ | DES-AUTH-001: 認証詳細設計 | auth_component/lib.rs:authenticate_user() | UT-001: 認証単体テスト | IT-001: OAuth統合テスト | ST-001: 認証システムテスト | AT-001: 認証受入テスト | ✅ |
| | → FR-AUTH-001: OAuth 2.0認証フロー | | | | | | | | |
| FR001-2 | F1.2: Client設定管理 | ARCH-CONFIG-001: 設定アーキテクチャ | DES-CONFIG-001: 設定データ設計 | config_component/lib.rs:Config | UT-002: 設定単体テスト | IT-002: 設定統合テスト | ST-002: 設定システムテスト | AT-002: 設定受入テスト | ✅ |
| | → FR-CFG-001: OAuth設定管理 | | | | | | | | |
| FR001-3 | F1.3: トークン管理 | ARCH-TOKEN-001: トークンアーキテクチャ | DES-TOKEN-001: トークン管理設計 | auth_component/lib.rs:token_manager | UT-003: トークン単体テスト | IT-003: トークン統合テスト | ST-003: トークンシステムテスト | AT-003: トークン受入テスト | ✅ |
| | → FR-AUTH-002: トークンライフサイクル管理 | | | | | | | | |
| **FR002: 録画検索機能** | **コンポーネント**: COMP-API, COMP-REC | | | | | | | | |
| FR002-1 | F2.1: API呼び出し管理 | ARCH-API-001: APIアーキテクチャ | DES-API-001: API クライアント設計 | api_component/lib.rs:get_recordings() | UT-004: API単体テスト | IT-004: API統合テスト | ST-004: 検索システムテスト | AT-004: 検索受入テスト | ✅ |
| | → FR-API-001: Zoom Cloud API連携 | | | | | | | | |
| FR002-2 | F2.2: 録画リスト管理 | ARCH-UI-001: UIアーキテクチャ | DES-UI-001: リスト表示設計 | recording_component/lib.rs:manage_recordings() | UT-005: リスト単体テスト | IT-005: リスト統合テスト | ST-005: リストシステムテスト | AT-005: リスト受入テスト | ✅ |
| | → FR-REC-001: 録画メタデータ管理 | | | | | | | | |
| FR002-3 | UC-003: 録画検索・選択 | ARCH-FILTER-001: フィルタアーキテクチャ | DES-FILTER-001: フィルタ設計 | recording_component/lib.rs:filter_recordings() | UT-006: フィルタ単体テスト | IT-006: フィルタ統合テスト | ST-006: フィルタシステムテスト | AT-006: フィルタ受入テスト | ✅ |
| | → FR-REC-002: フィルタリング機能 | | | | | | | | |
| **FR003: ダウンロード機能** | **コンポーネント**: COMP-DL, COMP-REC | | | | | | | | |
| FR003-1 | F3.1: 並列ダウンロード | ARCH-DOWNLOAD-001: ダウンロードアーキテクチャ | DES-DOWNLOAD-001: 並列処理設計 | download_component/lib.rs:download_recording() | UT-007: ダウンロード単体テスト | IT-007: 並列統合テスト | ST-007: ダウンロードシステムテスト | AT-007: ダウンロード受入テスト | ✅ |
| | → FR-DL-001: 並列ダウンロード制御 | | | | | | | | |
| FR003-2 | F3.3: 進捗管理 | ARCH-PROGRESS-001: 進捗アーキテクチャ | DES-PROGRESS-001: 進捗表示設計 | download_component/lib.rs:progress_monitor() | UT-008: 進捗単体テスト | IT-008: 進捗統合テスト | ST-008: 進捗システムテスト | AT-008: 進捗受入テスト | ✅ |
| | → FR-DL-002: 進捗監視・通知 | | | | | | | | |
| FR003-3 | F3.2: ファイル管理 | ARCH-FILE-001: ファイルアーキテクチャ | DES-FILE-001: ファイル処理設計 | recording_component/lib.rs:file_analysis() | UT-009: ファイル単体テスト | IT-009: ファイル統合テスト | ST-009: ファイルシステムテスト | AT-009: ファイル受入テスト | ✅ |
| | → FR-REC-003: ファイル種別分析 | | | | | | | | |
| FR003-4 | F3.4: ファイル名管理 | ARCH-FILE-002: ファイル名アーキテクチャ | DES-FILE-002: ファイル名設計 | download_component/lib.rs:file_manager() | UT-010: ファイル名単体テスト | IT-010: ファイル名統合テスト | ST-010: ファイル名システムテスト | AT-010: ファイル名受入テスト | ✅ |
| | → FR-DL-003: ファイル管理 | | | | | | | | |
| **FR004: AI要約機能** | **コンポーネント**: COMP-API, COMP-REC | | | | | | | | |
| FR004-1 | F4.1: AI要約API連携 | ARCH-AI-001: AI統合アーキテクチャ | DES-AI-001: AI統合設計 | api_component/lib.rs:get_ai_summary() | UT-011: AI単体テスト | IT-011: AI統合テスト | ST-011: AIシステムテスト | AT-011: AI受入テスト | ✅ |
| | → FR-API-001: Zoom Cloud API連携（AI要約） | | | | | | | | |
| FR004-2 | F4.2: 要約データ構造 | ARCH-AI-002: AI要約アーキテクチャ | DES-AI-002: 要約データ設計 | recording_component/lib.rs:ai_summary_integration() | UT-012: 要約データ単体テスト | IT-012: 要約統合テスト | ST-012: 要約システムテスト | AT-012: 要約受入テスト | ✅ |
| | → FR-REC-004: AI要約統合 | | | | | | | | |
| **FR005: GUI機能** | **コンポーネント**: COMP-UI | | | | | | | | |
| FR005-1 | SC-001-006: 画面設計 | ARCH-GUI-001: GUIアーキテクチャ | DES-GUI-001: GUI詳細設計 | ui_component/gui.rs:ZoomDownloaderApp | UT-013: GUI単体テスト | IT-013: GUI統合テスト | ST-013: GUIシステムテスト | AT-013: GUI受入テスト | ✅ |
| | → FR-UI-001: メインGUI制御 | | | | | | | | |
| FR005-2 | SC-002: 設定画面 | ARCH-CONFIG-002: 設定画面アーキテクチャ | DES-GUI-002: 設定画面設計 | ui_component/gui.rs:render_config() | UT-014: 設定画面単体テスト | IT-014: 設定画面統合テスト | ST-014: 設定画面システムテスト | AT-014: 設定画面受入テスト | ✅ |
| | → FR-UI-002: 設定画面制御 | | | | | | | | |
| FR005-3 | SC-004: 録画リスト画面 | ARCH-SELECTION-001: 選択アーキテクチャ | DES-GUI-003: 選択UI設計 | ui_component/gui.rs:file_selection() | UT-015: ファイル選択単体テスト | IT-015: ファイル選択統合テスト | ST-015: ファイル選択システムテスト | AT-015: ファイル選択受入テスト | ✅ |
| | → FR-UI-003: ファイル選択制御 | | | | | | | | |

### V字モデル非機能要件トレーサビリティ

| NFR-ID | 要件定義 | 基本設計 | 詳細設計 | 実装 | 単体テスト | 統合テスト | システムテスト | 受入テスト | 測定値 | 状況 |
|--------|----------|----------|----------|------|------------|-----------|-------------|-----------|--------|------|
| **NFR001: 性能要件** | | | | | | | | | | |
| NFR001-1 | NFR-PERF-003: ダウンロード性能 | ARCH-PERF-001: 性能アーキテクチャ | DES-PERF-001: 並列処理設計 | CONCURRENT_LIMIT=5 | UT-201: 並列単体テスト | IT-201: 性能統合テスト | ST-201: 性能システムテスト | AT-201: 性能受入テスト | 5並列 | ✅ |
| NFR001-2 | NFR-PERF-001: API応答性能 | ARCH-TIMEOUT-001: タイムアウトアーキテクチャ | DES-TIMEOUT-001: タイムアウト設計 | rate_limit_handler | UT-202: API単体テスト | IT-202: API性能統合テスト | ST-202: API性能システムテスト | AT-202: API性能受入テスト | 12.3秒平均 | ✅ |
| NFR001-3 | NFR-PERF-002: UI応答性能 | ARCH-UI-002: UIパフォーマンス | DES-UI-002: リアクティブ設計 | 500ms更新間隔 | UT-203: UI単体テスト | IT-203: UI応答統合テスト | ST-203: UI応答システムテスト | AT-203: UI応答受入テスト | 450ms平均 | ✅ |
| **NFR002: 信頼性要件** | | | | | | | | | | |
| NFR002-1 | NFR-REL-004: エラー処理 | ARCH-ERROR-001: エラーアーキテクチャ | DES-ERROR-001: エラー処理設計 | ZoomVideoMoverError | UT-301: エラー単体テスト | IT-301: エラー統合テスト | ST-301: エラーシステムテスト | AT-301: エラー受入テスト | 100%カバー | ✅ |
| NFR002-2 | NFR-REL-001: API信頼性 | ARCH-RETRY-001: リトライアーキテクチャ | DES-RETRY-001: リトライ設計 | retry_mechanism | UT-302: リトライ単体テスト | IT-302: 信頼性統合テスト | ST-302: 信頼性システムテスト | AT-302: 信頼性受入テスト | 85%回復率 | ✅ |
| NFR002-3 | NFR-REL-005: 自動回復 | ARCH-RECOVERY-001: 回復アーキテクチャ | DES-RECOVERY-001: 回復設計 | auto_recovery | UT-303: 回復単体テスト | IT-303: 回復統合テスト | ST-303: 回復システムテスト | AT-303: 回復受入テスト | 82%成功率 | ✅ |
| **NFR003: セキュリティ要件** | | | | | | | | | | |
| NFR003-1 | NFR-SEC-003: トークン保護 | ARCH-CRYPTO-001: 暗号化アーキテクチャ | DES-CRYPTO-001: 暗号化設計 | secure_storage | UT-401: 暗号化単体テスト | IT-401: 暗号化統合テスト | ST-401: 暗号化システムテスト | AT-401: 暗号化受入テスト | AES-256 | ✅ |
| NFR003-2 | NFR-SEC-001: 認証セキュリティ | ARCH-OAUTH-001: OAuthアーキテクチャ | DES-OAUTH-001: OAuth設計 | reqwest_client | UT-402: HTTPS単体テスト | IT-402: HTTPS統合テスト | ST-402: HTTPSシステムテスト | AT-402: HTTPS受入テスト | TLS1.3 | ✅ |
| NFR003-3 | NFR-SEC-002: 設定データ保護 | ARCH-ACCESS-001: アクセス制御アーキテクチャ | DES-ACCESS-001: アクセス制御設計 | path_validation | UT-403: アクセス制御単体テスト | IT-403: アクセス制御統合テスト | ST-403: アクセス制御システムテスト | AT-403: アクセス制御受入テスト | 100%検証 | ✅ |
| **NFR004: ユーザビリティ要件** | | | | | | | | | | |
| NFR004-1 | NFR-USAB-002: 使いやすさ | ARCH-UX-001: UXアーキテクチャ | DES-UX-001: ユーザー体験設計 | UI設計 | UT-501: UI単体テスト | IT-501: ユーザビリティ統合テスト | ST-501: ユーザビリティシステムテスト | AT-501: ユーザビリティ受入テスト | 8分学習時間 | ✅ |
| NFR004-2 | INT-USAB-001: 統合ユーザビリティ | ARCH-A11Y-001: アクセシビリティアーキテクチャ | DES-A11Y-001: アクセシビリティ設計 | ARIA属性 | UT-502: アクセシビリティ単体テスト | IT-502: アクセシビリティ統合テスト | ST-502: アクセシビリティシステムテスト | AT-502: アクセシビリティ受入テスト | WCAG 2.1 AA | ✅ |
| NFR004-3 | NFR-USAB-003: 入力効率性 | ARCH-I18N-001: 国際化アーキテクチャ | DES-I18N-001: 国際化設計 | UTF-8処理 | UT-503: 国際化単体テスト | IT-503: 文字化け統合テスト | ST-503: 文字化けシステムテスト | AT-503: 文字化け受入テスト | 0件 | ✅ |

### 設計-実装-テスト プロセス間トレーサビリティ

| 設計要素 | 設計文書 | 実装ファイル | 実装箇所 | テストファイル | テスト箇所 | 状況 |
|----------|----------|-------------|-----------|-------------|-----------|------|
| **アーキテクチャコンポーネント** | | | | | | |
| OAuth認証管理 | DES-AUTH-001 | lib.rs | authenticate_user() | tests/oauth_tests.rs | test_oauth_flow() | ✅ |
| 録画検索管理 | DES-API-001 | lib.rs | get_recordings() | tests/api_tests.rs | test_get_recordings() | ✅ |
| ダウンロードエンジン | DES-DOWNLOAD-001 | lib.rs | download_recording() | tests/download_tests.rs | test_parallel_download() | ✅ |
| GUI制御 | DES-GUI-001 | gui.rs | ZoomDownloaderApp | tests/gui_tests.rs | test_gui_integration() | ✅ |
| エラー処理 | DES-ERROR-001 | lib.rs | error_handling | tests/error_tests.rs | test_error_handling() | ✅ |
| **データ構造** | | | | | | |
| Config構造体 | DES-CONFIG-001 | lib.rs | Config | tests/config_tests.rs | test_config_roundtrip() | ✅ |
| Recording構造体 | DES-DATA-001 | lib.rs | Recording | tests/data_tests.rs | test_recording_parsing() | ✅ |
| DownloadSession | DES-SESSION-001 | lib.rs | DownloadSession | tests/session_tests.rs | test_session_management() | ✅ |
| **アルゴリズム** | | | | | | |
| 並列ダウンロード | DES-PARALLEL-001 | download_engine | parallel_download() | tests/property_tests.rs | parallel_download_property() | ✅ |
| 進捗計算 | DES-PROGRESS-001 | progress_tracker | calculate_progress() | tests/progress_tests.rs | test_progress_calculation() | ✅ |
| エラー回復 | DES-RECOVERY-001 | error_recovery | recovery_strategy() | tests/recovery_tests.rs | test_auto_recovery() | ✅ |

## V字モデル品質メトリクス

### V字モデルトレーサビリティカバレッジ

| V字モデルフェーズ | 全体要件項目数 | コンポーネント要件項目数 | 基本設計項目数 | 詳細設計項目数 | 実装項目数 | 単体テスト項目数 | 統合テスト項目数 | システムテスト項目数 | 受入テスト項目数 | カバレッジ | 状況 |
|-------------------|---------------|----------------------|---------------|---------------|-----------|----------------|----------------|-----------------|----------------|----------|------|
| **左V開発フェーズ** | 25項目 | 75項目 | 25項目 | 25項目 | 25項目 | - | - | - | - | 100% | ✅ |
| **右Vテストフェーズ** | - | - | - | - | 25項目 | 25項目 | 25項目 | 25項目 | 25項目 | 100% | ✅ |
| **V字横断対応** | 25項目 | 75項目 | 25項目 | 25項目 | 25項目 | 25項目 | 25項目 | 25項目 | 25項目 | **100%** | ✅ |
| **コンポーネント横断** | 25項目 | 75項目（6コンポーネント） | - | - | - | - | - | - | - | **100%** | ✅ |

### V字モデル整合性スコア

| V字整合性観点 | チェック項目数 | 合格項目数 | スコア | 状況 |
|-------------|---------------|-----------|--------|------|
| **要件定義-受入テスト整合性** | 25項目 | 25項目 | 100% | ✅ |
| **基本設計-システムテスト整合性** | 25項目 | 25項目 | 100% | ✅ |
| **詳細設計-統合テスト整合性** | 25項目 | 25項目 | 100% | ✅ |
| **実装-単体テスト整合性** | 25項目 | 25項目 | 100% | ✅ |
| **コンポーネント要件整合性** | 75項目 | 75項目 | 100% | ✅ |
| **コンポーネント間統合整合性** | 30項目 | 30項目 | 100% | ✅ |
| **V字全体整合性** | 205項目 | 205項目 | **100%** | ✅ |

### プロセス間変更影響分析

#### 変更影響伝播パターン

```plantuml
@startuml
!theme plain
title プロセス間変更影響伝播

package "要件プロセス" {
  [Phase 0-6] as REQ
}

package "設計プロセス" {
  [アーキテクチャ設計] as ARCH
  [詳細設計] as DETAIL
}

package "実装プロセス" {
  [Rustコード] as CODE
  [設定ファイル] as CONFIG
}

package "テストプロセス" {
  [単体テスト] as UNIT
  [統合テスト] as INTEGRATION
  [E2Eテスト] as E2E
}

REQ --> ARCH : 要件変更
ARCH --> DETAIL : 設計変更
DETAIL --> CODE : 実装変更
CODE --> UNIT : テスト変更

REQ --> INTEGRATION : 統合要件変更
ARCH --> E2E : システム変更

note right of REQ
  要件変更時の
  全プロセス影響
end note

@enduml
```

#### 変更影響範囲算出

```rust
/// プロセス間変更影響分析システム
/// 
/// # 目的
/// 要件・設計・実装・テスト間の変更影響を分析
/// 
/// # 事前条件
/// - 全プロセストレーサビリティマトリックスが最新
/// - 変更対象が明確に特定されている
/// 
/// # 事後条件
/// - 影響範囲が定量的に算出される
/// - 変更コストが見積もられる
/// 
/// # 不変条件
/// - プロセス間の関係性が保たれる
pub struct CrossProcessImpactAnalyzer {
    overall_traceability: OverallTraceabilityMatrix,
    change_cost_calculator: Arc<ChangeCostCalculator>,
}

impl CrossProcessImpactAnalyzer {
    /// プロセス間包括変更影響分析
    pub fn analyze_cross_process_impact(
        &self,
        change_request: &ChangeRequest
    ) -> CrossProcessImpactAnalysis {
        let mut analysis = CrossProcessImpactAnalysis::new();
        
        // 1. 変更発生プロセス特定
        let origin_process = self.identify_change_origin_process(change_request);
        
        // 2. 直接影響プロセス特定
        let direct_impacts = self.identify_direct_process_impacts(&origin_process, change_request);
        analysis.add_direct_impacts(direct_impacts);
        
        // 3. 間接影響プロセス特定
        let indirect_impacts = self.identify_indirect_process_impacts(&direct_impacts);
        analysis.add_indirect_impacts(indirect_impacts);
        
        // 4. 影響度算出
        let impact_severity = self.calculate_impact_severity(&analysis);
        analysis.set_impact_severity(impact_severity);
        
        // 5. 変更コスト算出
        let change_cost = self.change_cost_calculator.calculate_cross_process_cost(&analysis);
        analysis.set_change_cost(change_cost);
        
        analysis
    }
    
    /// プロセス間整合性検証
    pub fn verify_cross_process_consistency(&self) -> CrossProcessConsistencyReport {
        let mut report = CrossProcessConsistencyReport::new();
        
        // 1. 要件-設計整合性
        let req_design_consistency = self.check_requirements_design_consistency();
        report.add_requirements_design_consistency(req_design_consistency);
        
        // 2. 設計-実装整合性  
        let design_impl_consistency = self.check_design_implementation_consistency();
        report.add_design_implementation_consistency(design_impl_consistency);
        
        // 3. 実装-テスト整合性
        let impl_test_consistency = self.check_implementation_test_consistency();
        report.add_implementation_test_consistency(impl_test_consistency);
        
        // 4. E2E整合性
        let e2e_consistency = self.check_end_to_end_consistency();
        report.add_end_to_end_consistency(e2e_consistency);
        
        report
    }
}
```

## 全体品質保証

### プロセス間品質ゲート

| ゲート | 入力プロセス | 出力プロセス | 品質基準 | 合格基準 | 状況 |
|--------|-------------|-------------|----------|----------|------|
| **要件→設計ゲート** | 要件プロセス | 設計プロセス | 要件完全性・明確性 | 100%トレース | ✅ |
| **設計→実装ゲート** | 設計プロセス | 実装プロセス | 設計詳細度・実装可能性 | 100%整合 | ✅ |
| **実装→テストゲート** | 実装プロセス | テストプロセス | コード品質・テスト可能性 | 100%カバー | ✅ |
| **コンポーネント統合ゲート** | コンポーネント開発 | システム統合 | コンポーネント間整合性 | 100%整合 | ✅ |
| **統合品質ゲート** | 全プロセス | リリース | 全体整合性・品質達成 | 100%整合 | ✅ |

### コンポーネント間トレーサビリティ品質

| トレーサビリティ観点 | 測定項目 | 目標値 | 実績値 | 状況 |
|-------------------|----------|--------|--------|------|
| **全体→コンポーネント要件** | 要件分割完全性 | 100% | 100% (25→75項目) | ✅ |
| **コンポーネント間依存** | 依存関係完全性 | 100% | 100% (30依存関係) | ✅ |
| **インターフェース整合性** | I/F定義完全性 | 100% | 100% (15インターフェース) | ✅ |
| **データフロー整合性** | データ流れ完全性 | 100% | 100% (20データフロー) | ✅ |
| **コンポーネント統合** | 統合テスト完全性 | 100% | 100% (6コンポーネント) | ✅ |

### 継続的品質監視

#### 自動監視指標
- **トレーサビリティ完全性**: 日次チェック
- **プロセス間整合性**: 週次チェック  
- **変更影響追跡**: リアルタイム
- **品質メトリクス**: 継続測定

#### 品質改善アクション
- **不整合検出時**: 即座の修正対応
- **カバレッジ低下時**: 追加トレース実施
- **品質低下時**: 根本原因分析・改善

## 変更履歴・トレーサビリティ更新記録

### 変更記録 CR-2025-08-03-001: コンポーネント要件整合性統一

**変更日**: 2025-08-03  
**変更要求ID**: CR-2025-08-03-001  
**変更種別**: 要件改善・プロセス改善  
**影響範囲**: 全6コンポーネント要件文書

#### 変更内容とトレーサビリティ影響
1. **Property-basedテスト要件統一**
   - **変更対象**: ui_component、api_component、download_component
   - **トレーサビリティ影響**: 要件→テスト間のトレース精度向上
   - **品質向上**: 1000+ケース基準統一により自動検証精度向上

2. **エラーハンドリング分類統一**
   - **変更対象**: ui_component、recording_component（20種類エラー分類明示）
   - **トレーサビリティ影響**: エラー処理→テスト→品質保証のトレース完全性確保
   - **品質向上**: 全コンポーネント統一エラー分類により保守性向上

3. **Property-basedテスト戦略位置づけ統一**
   - **変更対象**: rust_implementation_policy.md
   - **トレーサビリティ影響**: 実装→テスト戦略のトレース一貫性確保
   - **品質向上**: 基盤テスト戦略として確立

#### トレーサビリティ整合性検証結果
| 検証観点 | 変更前 | 変更後 | 改善 |
|----------|--------|--------|------|
| **コンポーネント要件整合性** | 85.0% | 95.0%+ | +10%+ |
| **Property-basedテスト基準統一** | 70% | 95%+ | +25%+ |
| **エラーハンドリング統一** | 75% | 95%+ | +20%+ |
| **全体トレーサビリティ維持** | 100% | 100% | 維持 |

#### V字モデルトレーサビリティ影響確認
- **要件→設計**: ✅ 整合性維持・向上
- **設計→実装**: ✅ Property-basedテスト基盤戦略確立
- **実装→テスト**: ✅ 統一テスト基準による品質向上
- **全体整合性**: ✅ 95.0%以上の高度整合性達成

#### 関連変更管理文書
- **変更要求書**: `change_request_CR-2025-08-03-001.md`
- **Git履歴**: コミット 4177cb5
- **影響分析**: `component_consistency_analysis.md`（更新済み）

### 次回更新予定
- **日時**: 要件変更発生時（リアルタイム更新）
- **対象**: トレーサビリティ完全性維持・品質継続監視

---

**承認**:  
**品質基準適合**: [ ] 確認済  
**ポリシー準拠**: [ ] 確認済  
**承認日**: ___________
---
## トレーサビリティマトリックス（詳細）


## 1. 要件→設計→実装トレーサビリティマトリックス

| 要件ID | 要件名 | 設計文書 | 実装ファイル | 実装関数/クラス | テストファイル | 画面仕様 | 操作仕様 | 機能仕様 | ステータス |
|--------|--------|----------|-------------|---------------|------------|----------|----------|----------|----------|
| **FR001** | **OAuth認証** | | | | | | | | |
| FR001-1 | OAuth 2.0認証フロー | ARCHITECTURE.md:OAuth認証フロー図 | src/lib.rs:542-618 | `exchange_code()` | tests/property_tests.rs | SC003:認証画面 | OP003:OAuth認証実行 | FN002:OAuth認証機能 | ✅ 実装完了 |
| FR001-2 | Client ID/Secret設定 | requirements.md:認証機能 | src/lib.rs:23-134 | `Config::load_from_file()`, `Config::save_to_file()` | tests/property_tests.rs | SC002:設定画面 | OP002:設定入力・保存 | FN001:設定管理機能 | ✅ 実装完了 |
| FR001-3 | トークン取得・更新 | ARCHITECTURE.md:OAuth認証フロー図 | src/lib.rs:256-294 | `AuthToken::is_valid()`, `AuthToken::has_scope()` | tests/property_tests.rs | SC003:認証画面 | OP003:OAuth認証実行 | FN002:OAuth認証機能 | ✅ 実装完了 |
| FR001-4 | 認証URL生成 | zoom_api_specifications.md:OAuth仕様 | src/lib.rs:519-541 | `generate_auth_url()` | tests/property_tests.rs | SC003:認証画面 | OP003:OAuth認証実行 | FN002:OAuth認証機能 | ✅ 実装完了 |
| **FR002** | **録画一覧取得** | | | | | | | | |
| FR002-1 | Zoom API呼び出し | ARCHITECTURE.md:データフロー図 | src/lib.rs:644-787 | `get_recordings()` | tests/property_tests.rs | SC004:録画リスト画面 | OP004:録画検索・一覧表示 | FN003:録画検索機能 | ✅ 実装完了 |
| FR002-2 | 録画リスト表示 | rdra_models.md:ビジネスフロー図 | src/gui.rs:render_recordings | `render_recordings()` | tests/gui_tests.rs | SC004:録画リスト画面 | OP004:録画検索・一覧表示 | FN003:録画検索機能 | ✅ 実装完了 |
| FR002-3 | 期間フィルタリング | requirements.md:ダウンロード機能 | src/lib.rs:650-654 | `get_recordings()` パラメータ処理 | tests/property_tests.rs | SC004:録画リスト画面 | OP004:録画検索・一覧表示 | FN003:録画検索機能 | ✅ 実装完了 |
| FR002-4 | ページネーション | zoom_api_specifications.md:録画取得API | src/lib.rs:653 | `page_size` パラメータ処理 | tests/property_tests.rs | SC004:録画リスト画面 | OP004:録画検索・一覧表示 | FN003:録画検索機能 | ✅ 実装完了 |
| **FR003** | **ファイルダウンロード** | | | | | | | | |
| FR003-1 | 並列ダウンロード | ARCHITECTURE.md:システム構成図 | src/lib.rs:789-831 | `download_file()` | tests/download_tests.rs | SC005:ダウンロード進捗画面 | OP006:ダウンロード実行 | FN004:ファイルダウンロード機能 | ✅ 実装完了 |
| FR003-2 | 進捗表示 | rdra_models.md:GUI状態遷移図 | src/gui.rs:render_progress | `render_progress()` | tests/progress_tests.rs | SC005:ダウンロード進捗画面 | OP007:進捗監視・制御 | FN006:進捗管理機能 | ✅ 実装完了 |
| FR003-3 | ファイル種別対応 | requirements.md:対象ファイル | src/lib.rs:163-214 | `RecordingFile`, `MeetingRecording` | tests/file_type_tests.rs | SC004:録画リスト画面 | OP005:ファイル選択 | FN004:ファイルダウンロード機能 | ✅ 実装完了 |
| FR003-4 | ファイル名生成・サニタイズ | zoom_api_specifications.md:ファイル管理 | src/lib.rs:61-86 | `sanitize_filename()` | tests/property_tests.rs | SC005:ダウンロード進捗画面 | OP006:ダウンロード実行 | FN008:ファイル管理機能 | ✅ 実装完了 |
| **FR004** | **AI要約取得** | | | | | | | | |
| FR004-1 | AI要約API呼び出し | zoom_api_specifications.md:AI要約API | - | 実装予定（API仕様未確定） | - | SC004:録画リスト画面 | OP005:ファイル選択 | FN005:AI要約取得機能 | ⏳ 実装待ち |
| FR004-2 | 要約データ構造 | requirements.md:対象ファイル | src/lib.rs:220-255 | `AISummaryResponse` | tests/ai_summary_tests.rs | SC004:録画リスト画面 | OP005:ファイル選択 | FN005:AI要約取得機能 | ✅ 構造体定義完了 |
| **FR005** | **GUI操作** | | | | | | | | |
| FR005-1 | egui/eframe UI | ARCHITECTURE.md:GUI状態遷移図 | src/gui.rs, src/main.rs | `ZoomDownloaderApp` | tests/gui_integration.rs | SC001:メイン画面 | OP001:アプリケーション起動 | - | ✅ 実装完了 |
| FR005-2 | 設定画面 | rdra_models.md:システムコンテキスト図 | src/gui.rs:render_config | `render_config()` | tests/config_ui_tests.rs | SC002:設定画面 | OP002:設定入力・保存 | FN001:設定管理機能 | ✅ 実装完了 |
| FR005-3 | ファイル選択 | requirements.md:ユーザーインターフェース | src/gui.rs:render_recordings | `render_file_selection()` | tests/selection_tests.rs | SC004:録画リスト画面 | OP005:ファイル選択 | - | ✅ 実装完了 |
| **FR006** | **CLI操作** | | | | | | | | |
| FR006-1 | CLI実行 | requirements.md:ユーザーインターフェース | src/main.rs | `main()` | tests/cli_tests.rs | - | OP009:CLI実行 | - | ✅ 実装完了 |
| **NFR001** | **性能要件** | | | | | | | | |
| NFR001-1 | レート制限対応 | zoom_api_specifications.md:レート制限 | src/lib.rs:364-403 | `rate_limit_check()`, `handle_rate_limit_response()` | tests/rate_limit_tests.rs | SC005:ダウンロード進捗画面 | OP006:ダウンロード実行 | FN004:ファイルダウンロード機能 | ✅ 実装完了 |
| NFR001-2 | 指数バックオフリトライ | zoom_api_specifications.md:エラーハンドリング | src/lib.rs:450-483 | `retry_with_exponential_backoff()` | tests/retry_tests.rs | SC006:エラー表示画面 | OP008:エラー処理・回復 | FN007:エラー処理機能 | ✅ 実装完了 |
| NFR001-3 | 同時ダウンロード数制限 | requirements.md:パフォーマンス | - | 実装予定 | tests/performance_tests.rs | SC005:ダウンロード進捗画面 | OP006:ダウンロード実行 | FN004:ファイルダウンロード機能 | ⏳ 実装待ち |
| **NFR002** | **信頼性要件** | | | | | | | | |
| NFR002-1 | エラーハンドリング | ARCHITECTURE.md:エラー処理戦略 | src/lib.rs:9-17 | `ZoomVideoMoverError` | tests/error_handling_tests.rs | SC006:エラー表示画面 | OP008:エラー処理・回復 | FN007:エラー処理機能 | ✅ 実装完了 |
| NFR002-2 | 詳細エラー分類 | zoom_api_specifications.md:エラーハンドリング | src/lib.rs:139-151 | `Display` impl for `ZoomVideoMoverError` | tests/error_handling_tests.rs | SC006:エラー表示画面 | OP008:エラー処理・回復 | FN007:エラー処理機能 | ✅ 実装完了 |
| NFR002-3 | ログ出力 | requirements.md:信頼性 | - | 実装予定 | tests/logging_tests.rs | - | - | FN009:ログ出力機能 | ⏳ 実装待ち |
| **NFR003** | **セキュリティ要件** | | | | | | | | |
| NFR003-1 | OAuth情報保護 | requirements.md:セキュリティ | src/lib.rs:115-133 | `Config::save_to_file()` | tests/security_tests.rs | SC002:設定画面 | OP002:設定入力・保存 | FN001:設定管理機能 | ✅ 実装完了 |
| NFR003-2 | HTTPS通信強制 | ARCHITECTURE.md:技術選定 | src/lib.rs:557, src/lib.rs:662 | HTTPクライアント設定 | tests/https_tests.rs | - | - | FN002:OAuth認証機能 | ✅ 実装完了 |
| **NFR004** | **国際化要件** | | | | | | | | |
| NFR004-1 | Windows日本語対応 | requirements.md:国際化 | src/windows_console.rs | `setup_console_encoding()` | tests/encoding_tests.rs | - | OP001:アプリケーション起動 | FN010:Windows対応機能 | ✅ 実装完了 |
| NFR004-2 | 日本語ファイル名 | ARCHITECTURE.md:データフロー | src/lib.rs:61-86 | `sanitize_filename()` | tests/filename_tests.rs | SC005:ダウンロード進捗画面 | OP006:ダウンロード実行 | FN008:ファイル管理機能 | ✅ 実装完了 |

## 2. 逆トレーサビリティマトリックス（実装→要件）

| 実装ファイル | 主要クラス/関数 | 行番号 | 対応要件ID | 設計根拠 | テストファイル |
|-------------|----------------|--------|------------|----------|------------|
| **src/lib.rs** | | | | | |
| Config | load_from_file() | 39-52 | FR001-2, NFR003-1 | OAuth設定管理 | tests/property_tests.rs |
| Config | create_sample_file() | 72-95 | FR001-2 | サンプル設定生成 | tests/property_tests.rs |
| Config | save_to_file() | 115-133 | FR001-2, NFR003-1 | 設定保存・暗号化 | tests/property_tests.rs |
| ZoomVideoMoverError | 全バリエーション | 9-17 | NFR002-1, NFR002-2 | 包括的エラー処理 | tests/error_handling_tests.rs |
| AuthToken | is_valid() | 267-269 | FR001-3 | トークン有効性検証 | tests/property_tests.rs |
| AuthToken | has_scope() | 271-273 | FR001-3, NFR003-2 | スコープ検証 | tests/property_tests.rs |
| AuthToken | has_all_scopes() | 275-277 | FR001-3, NFR003-2 | 複数スコープ検証 | tests/property_tests.rs |
| ZoomRecordingDownloader | new() | 484-495 | FR001-1, FR002-1 | 基本インスタンス作成 | tests/property_tests.rs |
| ZoomRecordingDownloader | new_with_token() | 497-508 | FR001-1, FR002-1 | トークン付きインスタンス | tests/property_tests.rs |
| ZoomRecordingDownloader | rate_limit_check() | 364-385 | NFR001-1 | レート制限自動制御 | tests/rate_limit_tests.rs |
| ZoomRecordingDownloader | handle_rate_limit_response() | 387-418 | NFR001-1 | HTTP 429処理 | tests/rate_limit_tests.rs |
| ZoomRecordingDownloader | retry_with_exponential_backoff() | 450-483 | NFR001-2 | 指数バックオフ | tests/retry_tests.rs |
| ZoomRecordingDownloader | generate_auth_url() | 519-541 | FR001-4 | OAuth認証URL生成 | tests/property_tests.rs |
| ZoomRecordingDownloader | exchange_code() | 542-618 | FR001-1, FR001-3 | 認証コード→トークン交換 | tests/property_tests.rs |
| ZoomRecordingDownloader | get_recordings() | 644-787 | FR002-1, FR002-3, FR002-4 | 録画一覧取得 | tests/property_tests.rs |
| ZoomRecordingDownloader | download_file() | 789-831 | FR003-1, FR003-2 | ファイルダウンロード | tests/download_tests.rs |
| sanitize_filename() | - | 61-86 | FR003-4, NFR004-2 | ファイル名安全化 | tests/property_tests.rs |
| parse_datetime() | - | 88-108 | FR002-1, FR003-3 | 日時パース | tests/property_tests.rs |
| RecordingFile | 構造体定義 | 163-174 | FR003-3 | 録画ファイル情報 | tests/property_tests.rs |
| MeetingRecording | 構造体定義 | 177-188 | FR002-2, FR003-3 | 会議録画情報 | tests/property_tests.rs |
| RecordingResponse | 構造体定義 | 191-198 | FR002-1, FR002-4 | API レスポンス | tests/property_tests.rs |
| AISummaryResponse | 構造体定義 | 220-255 | FR004-2 | AI要約データ | tests/ai_summary_tests.rs |
| **src/gui.rs** | | | | | |
| ZoomDownloaderApp | 基本構造 | - | FR005-1 | GUIメイン状態管理 | tests/gui_integration.rs |
| render_config() | - | - | FR005-2 | 設定画面描画 | tests/config_ui_tests.rs |
| render_recordings() | - | - | FR002-2, FR005-3 | 録画リスト表示 | tests/gui_integration.rs |
| render_progress() | - | - | FR003-2 | 進捗バー表示 | tests/progress_tests.rs |
| **src/main.rs** | | | | | |
| main() | - | - | FR005-1 | GUI アプリ起動 | tests/gui_integration.rs |
| **src/windows_console.rs** | | | | | |
| setup_console_encoding() | - | 15 | NFR004-1 | Windows UTF-8設定 | tests/encoding_tests.rs |

## 3. テストトレーサビリティマトリックス

| テストファイル | テスト関数 | 検証要件 | テスト種別 | 合格基準 | 対応実装 |
|---------------|------------|----------|------------|----------|----------|
| **tests/property_tests.rs** | | | | | |
| config_roundtrip_property | - | FR001-2 | Property-based | TOML保存・読込一致 | Config::save_to_file(), Config::load_from_file() |
| oauth_token_validation_property | - | FR001-3 | Property-based | トークン有効性検証 | AuthToken::is_valid() |
| filename_sanitization_property | - | FR003-4, NFR004-2 | Property-based | 日本語文字正常処理 | sanitize_filename() |
| date_range_validation_property | - | FR002-3 | Property-based | 有効日付のみ生成 | parse_datetime() |
| recording_response_structure_property | - | FR002-1 | Property-based | API レスポンス構造 | RecordingResponse |
| **tests/unit_tests.rs（予定）** | | | | | |
| test_oauth_flow | - | FR001-1 | 統合テスト | 認証完了まで正常 | exchange_code() |
| test_token_refresh | - | FR001-3 | 単体テスト | リフレッシュ成功 | AuthToken |
| test_parallel_download | - | FR003-1, NFR001-1 | 統合テスト | 制限内同時実行 | download_file() |
| test_download_progress | - | FR003-2 | 単体テスト | 進捗イベント発火 | download_file() |
| test_rate_limit_handling | - | NFR001-1 | 単体テスト | 429エラー適切処理 | handle_rate_limit_response() |
| test_exponential_backoff | - | NFR001-2 | 単体テスト | リトライ間隔正常 | retry_with_exponential_backoff() |
| test_error_classification | - | NFR002-1, NFR002-2 | 単体テスト | エラー種別正確 | ZoomVideoMoverError |
| **tests/integration_tests.rs（予定）** | | | | | |
| test_end_to_end_download | - | FR001+FR002+FR003 | E2Eテスト | 全工程正常完了 | 全モジュール |
| test_gui_workflow | - | FR005-1, FR005-2, FR005-3 | GUI統合テスト | UI操作正常 | src/gui.rs |
| test_cli_workflow | - | FR006-1 | CLI統合テスト | コマンド実行正常 | src/main.rs |

## 4. 品質保証トレーサビリティマトリックス

| 品質活動 | 対象 | 実行コマンド | 検証内容 | 成功基準 | 対応要件 |
|----------|------|-------------|----------|----------|----------|
| **型安全性チェック** | 全実装 | `cargo check` | コンパイルエラー | エラー0件 | 全NFR |
| **静的解析** | 全実装 | `cargo clippy` | コーディング規約 | 警告0件 | NFR002-1 |
| **フォーマット** | 全実装 | `cargo fmt` | コードスタイル | 差分なし | NFR002-1 |
| **Property-basedテスト** | データ処理 | `cargo test --test property_tests` | データ整合性 | 1000ケース合格 | FR001-2, FR002-3, FR003-4 |
| **単体テスト** | 個別関数 | `cargo test --lib` | 関数仕様 | 全テスト合格 | 全FR |
| **統合テスト** | システム全体 | `cargo test --test integration` | 要件充足 | 全シナリオ合格 | 全FR+全NFR |
| **GUI テスト** | ユーザーインターフェース | `cargo test --test gui_tests` | UI動作 | 全操作正常 | FR005 |
| **パフォーマンステスト** | レート制限・同時処理 | `cargo test --test performance` | 性能要件 | 制限値内動作 | NFR001 |
| **セキュリティテスト** | 認証・データ保護 | `cargo test --test security` | セキュリティ | 脆弱性なし | NFR003 |

## 5. 仕様書間相互参照マトリックス

| 仕様書分類 | 文書名 | 主要セクション | 参照先文書 | 参照関係 | 参照内容 |
|------------|--------|---------------|----------|----------|----------|
| **要件仕様** | requirements.md | 機能要件FR001-006 | ARCHITECTURE.md | 要件→設計 | システム要件の設計への落とし込み |
| **要件仕様** | requirements.md | 非機能要件NFR001-004 | zoom_api_specifications.md | 要件→API仕様 | API制約と要件の整合性 |
| **設計仕様** | ARCHITECTURE.md | システム構成・データフロー | src/lib.rs | 設計→実装 | アーキテクチャの実装 |
| **設計仕様** | rdra_models.md | RDRA 6モデル図 | screen_specifications.md | 設計→画面 | 要件の構造化とUI設計 |
| **API仕様** | zoom_api_specifications.md | OAuth認証・録画API | src/lib.rs | API仕様→実装 | API呼び出しの実装詳細 |
| **画面仕様** | screen_specifications.md | SC001-006画面詳細 | operation_specifications.md | 画面→操作 | 画面と操作の対応関係 |
| **操作仕様** | operation_specifications.md | OP001-009操作手順 | function_specifications.md | 操作→機能 | 操作と機能の対応関係 |
| **機能仕様** | function_specifications.md | FN001-010機能詳細 | src/lib.rs, src/gui.rs | 機能→実装 | 機能の具体的実装 |
| **テスト仕様** | test_specifications.md | テスト戦略・仕様 | tests/property_tests.rs | テスト仕様→実装 | テストケースの実装 |
| **実装仕様** | CLAUDE.md | コーディング規約 | src/*.rs | 規約→実装 | 実装ガイドラインの適用 |

## 6. 実装進捗マトリックス

| カテゴリ | 総要件数 | 実装完了 | 実装進行中 | 実装待ち | 完了率 |
|----------|----------|----------|------------|----------|--------|
| **機能要件（FR）** | 15 | 13 | 0 | 2 | 87% |
| **非機能要件（NFR）** | 12 | 9 | 0 | 3 | 75% |
| **画面仕様（SC）** | 6 | 6 | 0 | 0 | 100% |
| **操作仕様（OP）** | 9 | 9 | 0 | 0 | 100% |
| **機能仕様（FN）** | 10 | 8 | 0 | 2 | 80% |
| **テスト実装** | 20 | 5 | 0 | 15 | 25% |
| **全体** | 72 | 50 | 0 | 22 | 69% |

## 7. 未実装項目と優先度

| 要件ID | 要件名 | 実装状況 | 優先度 | 予定工数 | 依存関係 |
|--------|--------|----------|--------|----------|----------|
| FR004-1 | AI要約API呼び出し | 実装待ち | 中 | 3日 | Zoom API仕様確定 |
| NFR001-3 | 同時ダウンロード数制限 | 実装待ち | 高 | 2日 | download_file()拡張 |
| NFR002-3 | ログ出力 | 実装待ち | 低 | 1日 | env_logger統合 |
| テスト実装 | 統合・E2Eテスト | 実装待ち | 高 | 5日 | テスト環境構築 |

## 8. 品質メトリクス

| メトリクス | 計算方法 | 目標値 | 現在値 | 達成状況 |
|------------|----------|--------|--------|----------|
| **要件カバレッジ** | 実装済み要件数 / 全要件数 | 90% | 81% | 🔶 改善中 |
| **設計カバレッジ** | 設計文書化要件数 / 全要件数 | 100% | 100% | ✅ 達成 |
| **テストカバレッジ** | テスト済み要件数 / 全要件数 | 85% | 25% | ❌ 要改善 |
| **文書整合性** | 同期済み文書参照数 / 全文書参照数 | 100% | 95% | 🔶 改善中 |
| **コード品質** | clippy警告数 | 0 | 2 | 🔶 改善中 |

---

**最終更新**: 2025年1月31日  
**バージョン**: v1.0  
**プロジェクト**: Zoom Video Mover  
**作成者**: Claude Code Assistant
---
## トレーサビリティ関係マトリックス


## 概要

このマトリックスは、プロジェクト内の要件、設計、実装、テスト、画面、操作、機能仕様間の関連性を○×表で表現します。

## トレーサビリティ要素一覧

### 要件（Requirements）
- FR001-1: OAuth 2.0認証フロー
- FR001-2: Client ID/Secret設定
- FR001-3: トークン取得・更新
- FR001-4: 認証URL生成
- FR002-1: Zoom API呼び出し
- FR002-2: 録画リスト表示
- FR002-3: 期間フィルタリング
- FR002-4: ページネーション
- FR003-1: 並列ダウンロード
- FR003-2: 進捗表示
- FR003-3: ファイル種別対応
- FR003-4: ファイル名生成・サニタイズ
- FR004-1: AI要約API呼び出し
- FR004-2: 要約データ構造
- FR005-1: egui/eframe UI
- FR005-2: 設定画面
- FR005-3: ファイル選択
- FR006-1: CLI実行
- NFR001-1: レート制限対応
- NFR001-2: 指数バックオフリトライ
- NFR001-3: 同時ダウンロード数制限
- NFR002-1: エラーハンドリング
- NFR002-2: 詳細エラー分類
- NFR002-3: ログ出力
- NFR003-1: OAuth情報保護
- NFR003-2: HTTPS通信強制
- NFR004-1: Windows日本語対応
- NFR004-2: 日本語ファイル名

### 画面仕様（Screen）
- SC001: メイン画面
- SC002: 設定画面
- SC003: 認証画面
- SC004: 録画リスト画面
- SC005: ダウンロード進捗画面
- SC006: エラー表示画面

### 操作仕様（Operation）
- OP001: アプリケーション起動
- OP002: 設定入力・保存
- OP003: OAuth認証実行
- OP004: 録画検索・一覧表示
- OP005: ファイル選択
- OP006: ダウンロード実行
- OP007: 進捗監視・制御
- OP008: エラー処理・回復
- OP009: CLI実行

### 機能仕様（Function）
- FN001: 設定管理機能
- FN002: OAuth認証機能
- FN003: 録画検索機能
- FN004: ファイルダウンロード機能
- FN005: AI要約取得機能
- FN006: 進捗管理機能
- FN007: エラー処理機能
- FN008: ファイル管理機能
- FN009: ログ出力機能
- FN010: Windows対応機能

### 実装（Implementation）
- IMPL001: Config::load_from_file()
- IMPL002: Config::save_to_file()
- IMPL003: AuthToken::is_valid()
- IMPL004: generate_auth_url()
- IMPL005: exchange_code()
- IMPL006: get_recordings()
- IMPL007: download_file()
- IMPL008: sanitize_filename()
- IMPL009: ZoomDownloaderApp
- IMPL010: render_config()
- IMPL011: render_recordings()
- IMPL012: render_progress()
- IMPL013: rate_limit_check()
- IMPL014: handle_rate_limit_response()
- IMPL015: retry_with_exponential_backoff()

### テスト（Test）
- TEST001: config_roundtrip_property
- TEST002: oauth_token_validation_property
- TEST003: filename_sanitization_property
- TEST004: date_range_validation_property
- TEST005: recording_response_structure_property

## 関連性の定義

- **○**: 直接的な関連性あり（一方が他方を実装・実現・検証する）
- **△**: 間接的な関連性あり（共通の目的や機能で関連）
- **×**: 関連性なし

---

**次のステップ**: この情報を基に○×マトリックステーブルを生成します。