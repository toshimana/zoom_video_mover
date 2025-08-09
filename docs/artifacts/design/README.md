# 🏗️ 設計ドキュメント

## 目的  
システム設計に関する全ドキュメントとUML図を集約

## 📂 構成

### アーキテクチャ設計
- **[system_architecture.md](system_architecture.md)** - システム全体構成
- **[data_model_design.md](data_model_design.md)** - データモデル
- **[interface_design.md](interface_design.md)** - API・UI設計
- **[security_design.md](security_design.md)** - セキュリティ設計
- **[performance_design.md](performance_design.md)** - 性能設計
- **[error_handling_design.md](error_handling_design.md)** - エラーハンドリング

### コンポーネント設計
- **[components/](components/)** - 各コンポーネントの詳細設計
  - 認証・ダウンロード・UI・API・設定・録画コンポーネント

### 図表・UML
- **[diagrams/](diagrams/)** - UML図・PlantUML図
  - phase1/: 概念設計図
  - phase2/: 詳細設計図  
  - phase3/: 実装設計図

### プロセス・手順
- **[design_process_flow_diagram.md](design_process_flow_diagram.md)** - 設計プロセス
- **[design_traceability_matrix.md](design_traceability_matrix.md)** - 設計トレーサビリティ
- **[change_impact_analysis_procedure.md](change_impact_analysis_procedure.md)** - 変更影響分析

## 🎯 使い方

### アーキテクト・設計者
1. system_architecture.md で全体像把握
2. components/ で担当領域の詳細確認
3. diagrams/ で視覚的理解

### 実装者
1. 該当コンポーネント設計文書を確認
2. diagrams/ で実装対象の詳細図を参照
3. interface_design.md でAPI仕様確認

---
**総ファイル数**: 18文書 + UML図  
**更新頻度**: 設計変更時