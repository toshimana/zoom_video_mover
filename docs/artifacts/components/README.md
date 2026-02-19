# コンポーネント設計成果物 (Components Artifacts)

本ディレクトリには、Zoom Video Moverプロジェクトの個別コンポーネント設計に関する成果物を格納しています。

## ディレクトリ構成

```
components/
├── README.md                     # 本ファイル
├── ui_specifications.md          # UI仕様書
├── error_handling_design.md      # エラーハンドリング設計
├── design/                       # 個別コンポーネント設計
│   ├── api_component_design.md           # APIコンポーネント設計
│   ├── auth_component_design.md          # 認証コンポーネント設計
│   ├── config_component_design.md        # 設定コンポーネント設計
│   ├── download_component_design.md      # ダウンロードコンポーネント設計
│   ├── recording_component_design.md     # 録画管理コンポーネント設計
│   └── ui_component_design.md            # UIコンポーネント設計
└── diagrams/                     # コンポーネント詳細図面
    ├── README.md                         # 図面説明
    ├── detailed_class_diagram.puml       # 詳細クラス図
    ├── detailed_component_diagram.puml   # 詳細コンポーネント図
    ├── download_execution_sequence.puml  # ダウンロード実行シーケンス図
    ├── download_task_state_diagram.puml  # ダウンロードタスク状態図
    ├── oauth_authentication_sequence.puml # OAuth認証シーケンス図
    └── oauth_token_state_diagram.puml    # OAuthトークン状態図
```

## コンポーネント設計成果物説明

### 1. 横断的設計
- **ui_specifications.md**: ユーザーインターフェース全体仕様
- **error_handling_design.md**: システム全体のエラーハンドリング設計

### 2. 個別コンポーネント設計 (design/)
各コンポーネントの詳細設計：

#### 認証・API関連
- **auth_component_design.md**: OAuth認証、トークン管理
- **api_component_design.md**: Zoom API連携、HTTP通信

#### データ管理関連  
- **recording_component_design.md**: 録画データ管理、検索・フィルタリング
- **config_component_design.md**: 設定管理、永続化

#### 機能実行関連
- **download_component_design.md**: ファイルダウンロード、進捗管理

#### ユーザーインターフェース
- **ui_component_design.md**: GUI実装、状態管理

### 3. 詳細設計図面 (diagrams/)
コンポーネント間の詳細な相互作用：

#### クラス・コンポーネント構造
- **detailed_class_diagram.puml**: 実装レベルのクラス設計
- **detailed_component_diagram.puml**: コンポーネント間関係

#### 動的振る舞い
- **download_execution_sequence.puml**: ダウンロード処理の詳細フロー
- **oauth_authentication_sequence.puml**: OAuth認証フローの詳細

#### 状態管理
- **download_task_state_diagram.puml**: ダウンロードタスクの状態遷移
- **oauth_token_state_diagram.puml**:認証トークンの状態管理

## 設計レベル分類

### Level 1: アーキテクチャ設計 (`../architecture/`)
- システム全体構造
- 非機能要件対応
- 技術基盤選択

### Level 2: コンポーネント設計 (本ディレクトリ)
- 個別コンポーネント詳細
- コンポーネント間インターフェース
- 実装レベル設計

### Level 3: 実装設計 (`../implementation/`)
- コード構造
- 実装進捗管理
- デプロイメント詳細

## コンポーネント間関係

```
┌─────────────────┐    ┌─────────────────┐
│  UI Component   │───▶│ Auth Component  │
└─────────────────┘    └─────────────────┘
         │                       │
         ▼                       ▼
┌─────────────────┐    ┌─────────────────┐
│Recording Mgmt   │───▶│  API Component  │
│   Component     │    │                 │
└─────────────────┘    └─────────────────┘
         │                       │
         ▼                       │
┌─────────────────┐              │
│Download Component│◀─────────────┘
└─────────────────┘
         │
         ▼
┌─────────────────┐
│Config Component │
└─────────────────┘
```

## 関連文書

- **アーキテクチャ**: `../architecture/` - システム全体設計
- **要件**: `../requirements/` - 機能・非機能要件
- **実装**: `../implementation/` - 実装詳細・進捗
- **テスト**: `../testing/` - コンポーネントテスト

## 更新履歴

- 2025-08-09: コンポーネント設計成果物の分離・整理
- 2025-08-09: 詳細設計図面の分類・配置