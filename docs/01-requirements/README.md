# 📋 要件定義ドキュメント

## 目的
Zoom Video Moverプロジェクトの要件定義に関する全ドキュメントを集約

## 📂 構成

### フェーズ別要件定義（RDRA手法）
- **[phase0_project_preparation/](phase0_project_preparation/)** - プロジェクト準備
- **[phase1_system_value/](phase1_system_value/)** - システム価値定義
- **[phase2_external_environment/](phase2_external_environment/)** - 外部環境分析
- **[phase3_system_boundary/](phase3_system_boundary/)** - システム境界定義
- **[phase4_system_internal/](phase4_system_internal/)** - システム内部構造
- **[phase5_non_functional/](phase5_non_functional/)** - 非機能要件
- **[phase6_integration/](phase6_integration/)** - 統合・検証

### 横断的要件
- **[crosscutting/](crosscutting/)** - 変更管理・トレーサビリティ・リスク管理

### 要件サマリ
- **[system_requirements_summary.md](system_requirements_summary.md)** - 要件全体概要

## 🎯 使い方

### 新規開発者
1. system_requirements_summary.md で全体把握
2. phase1-3 で機能要件理解
3. phase5 で非機能要件確認

### 機能変更時
1. crosscutting/change_management.md で変更プロセス確認
2. 該当フェーズの要件を更新
3. crosscutting/overall_traceability_matrix.md で影響分析

---
**総ファイル数**: 27文書  
**更新頻度**: 要件変更時