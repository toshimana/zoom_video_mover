# システムアーキテクチャ設計 - Zoom Video Mover

**文書情報**:
- バージョン: v1.1
- 作成日: 2025-08-09  
- 最終更新: 2025-08-09
- 作成者: 開発チーム
- レビュー状況: ドラフト
- レビュー期限: 2025-08-16

**関連文書**:
- [設計ポリシー](../../policies/universal/design_policy.md)
- [セキュリティポリシー](../../policies/universal/security_policy.md)
- [API仕様書](./api_specifications.md)

## 1. アーキテクチャ決定記録 (ADR)

### ADR-001: GUI フレームワーク選択
**決定**: egui フレームワークの採用  
**理由**: 
- Rustネイティブ・高パフォーマンス
- クロスプラットフォーム対応
- 軽量・依存関係最小
- リアルタイム更新・プログレス表示に最適

**代替案検討**: 
- Tauri: Web技術だが重い
- gtk-rs: Linux依存が強い
- 検討結果: eguiが最適解

### ADR-002: 非同期ランタイム選択
**決定**: tokio ランタイムの採用  
**理由**:
- Rustエコシステムのデファクト標準
- HTTP通信・並行ダウンロードに最適
- 豊富なライブラリサポート
- パフォーマンス・安定性実績

## 2. システムアーキテクチャ設計

## 1. システムアーキテクチャ図

```mermaid
graph TB
    User[ユーザー] --> GUI[GUI Interface<br/>main.rs]
    
    GUI --> Core[Core Library<br/>lib.rs]
    
    Core --> Config[設定管理<br/>Config struct]
    Core --> Auth[OAuth認証<br/>ZoomRecordingDownloader]
    Core --> API[Zoom API通信<br/>reqwest]
    Core --> Download[ダウンロード管理]
    
    Config --> File1[config.toml]
    
    Auth --> Zoom[Zoom OAuth Server]
    API --> Zoom[Zoom Cloud API]
    
    Download --> Local[ローカルストレージ]
    
    GUI --> Windows[Windows Console<br/>windows_console.rs]
    Windows --> WinAPI[Windows API<br/>UTF-8 Support]
    
    subgraph "External Dependencies"
        eframe[eframe/egui]
        reqwest[reqwest]
        oauth2[oauth2]
        tokio[tokio]
        serde[serde]
    end
    
    GUI --> eframe
    Core --> reqwest
    Auth --> oauth2
    Core --> tokio
    Core --> serde
```

## 2. データフロー図

```mermaid
flowchart TD
    Start([アプリ開始]) --> LoadConfig{設定ファイル<br/>存在?}
    LoadConfig -->|Yes| ParseConfig[設定解析]
    LoadConfig -->|No| CreateConfig[サンプル設定作成]
    
    CreateConfig --> UserInput[ユーザー設定入力]
    ParseConfig --> AuthFlow[OAuth認証フロー]
    UserInput --> AuthFlow
    
    AuthFlow --> GenAuthURL[認証URL生成]
    GenAuthURL --> Browser[ブラウザ起動]
    Browser --> ZoomAuth[Zoom認証画面]
    ZoomAuth --> AuthCode[認証コード取得]
    AuthCode --> GetToken[アクセストークン取得]
    
    GetToken --> SetPeriod[ダウンロード期間設定]
    SetPeriod --> GetRecordings[録画リスト取得]
    
    GetRecordings --> ProcessMeeting{各ミーティング<br/>処理}
    ProcessMeeting --> DownloadFiles[ファイルダウンロード]
    ProcessMeeting --> GetAISummary[AI要約取得]
    
    DownloadFiles --> SaveFile[ローカル保存]
    GetAISummary --> SaveSummary[要約保存]
    
    SaveFile --> NextMeeting{次のミーティング?}
    SaveSummary --> NextMeeting
    NextMeeting -->|Yes| ProcessMeeting
    NextMeeting -->|No| Complete([完了])
```

## 2.1 AI要約取得の詳細フロー

```mermaid
flowchart TD
    Start([AI要約取得開始]) --> CheckUUID{Meeting UUID<br/>有効?}
    CheckUUID -->|Yes| UUIDFlow[UUID使用フロー]
    CheckUUID -->|No| IDFlow[Meeting ID使用フロー]
    
    UUIDFlow --> GenVariants[UUID変形生成<br/>・原形<br/>・URLエンコード<br/>・ダブルエンコード]
    GenVariants --> UUIDEndpoints[UUID用エンドポイント生成<br/>16個のエンドポイント]
    
    IDFlow --> IDEndpoints[Meeting ID用エンドポイント生成<br/>15個のエンドポイント]
    
    UUIDEndpoints --> TryEndpoint{エンドポイント<br/>試行}
    IDEndpoints --> TryEndpoint
    
    TryEndpoint --> RateLimit[レート制限チェック]
    RateLimit --> SendRequest[HTTPリクエスト送信]
    
    SendRequest --> CheckStatus{レスポンス<br/>ステータス}
    CheckStatus -->|200| ProcessSuccess[成功レスポンス処理]
    CheckStatus -->|404| NextEndpoint[次のエンドポイント]
    CheckStatus -->|401/403| AuthError[認証エラー処理]
    CheckStatus -->|429| WaitRetry[レート制限待機]
    CheckStatus -->|422| ProcessingWait[要約処理中]
    CheckStatus -->|5xx| ServerError[サーバーエラー]
    
    ProcessSuccess --> SaveDebug[デバッグレスポンス保存]
    SaveDebug --> ParseJSON{JSONパース<br/>可能?}
    
    ParseJSON -->|AISummaryResponse| DirectParse[直接パース成功]
    ParseJSON -->|Generic JSON| ConvertJSON[汎用JSON変換]
    ParseJSON -->|Invalid| NextEndpoint
    
    DirectParse --> ReturnSummary[要約データ返却]
    ConvertJSON --> FieldMapping[フィールドマッピング<br/>・summary → summary_overview<br/>・key_points → key_points<br/>・action_items → next_steps]
    FieldMapping --> ReturnSummary
    
    AuthError --> NextEndpoint
    WaitRetry --> TryEndpoint
    ProcessingWait --> NextEndpoint
    ServerError --> NextEndpoint
    
    NextEndpoint --> HasMore{次のエンドポイント<br/>存在?}
    HasMore -->|Yes| TryEndpoint
    HasMore -->|No| CheckUUIDFallback{UUID→ID<br/>フォールバック?}
    
    CheckUUIDFallback -->|Yes| IDFlow
    CheckUUIDFallback -->|No| NoSummary[要約取得失敗<br/>None返却]
    
    ReturnSummary --> End([終了])
    NoSummary --> End
```

## 3. システム構成図

```mermaid
graph LR
    subgraph "Local Environment"
        subgraph "Rust Application"
            GUI[GUI Module<br/>eframe/egui]
            Core[Core Library<br/>Business Logic]
            WinConsole[Windows Console<br/>UTF-8 Support]
        end
        
        subgraph "Local Files"
            Config[config.toml]
            Downloads[Downloaded Files<br/>Videos/Audio/Chat]
            Debug[debug_responses/<br/>AI Response Logs]
        end
    end
    
    subgraph "External Services"
        ZoomOAuth[Zoom OAuth Server<br/>oauth.zoom.us]
        ZoomAPI[Zoom Cloud API<br/>api.zoom.us]
        Browser[Web Browser<br/>OAuth Flow]
    end
    
    GUI --> Core
    Core --> Config
    Core --> Downloads
    Core --> Debug
    WinConsole --> GUI
    
    Core -->|HTTPS/OAuth2| ZoomOAuth
    Core -->|HTTPS/REST API| ZoomAPI
    GUI -->|Launch| Browser
    Browser -->|Redirect| ZoomOAuth
```

## 4. OAuth認証フロー図

```mermaid
sequenceDiagram
    participant User as ユーザー
    participant App as Zoom Video Mover
    participant Browser as ブラウザ
    participant ZoomOAuth as Zoom OAuth Server
    participant ZoomAPI as Zoom API
    
    User->>App: アプリ起動
    App->>App: 設定ファイル読み込み
    
    User->>App: 認証開始
    App->>ZoomOAuth: 認証URL生成要求
    ZoomOAuth-->>App: 認証URL返却
    
    App->>Browser: 認証URLでブラウザ起動
    Browser->>ZoomOAuth: ユーザー認証画面表示
    
    User->>Browser: 認証情報入力
    Browser->>ZoomOAuth: 認証実行
    ZoomOAuth-->>Browser: 認証コード発行
    
    Browser->>App: 認証コード（リダイレクト/手動入力）
    App->>ZoomOAuth: アクセストークン要求<br/>(認証コード)
    ZoomOAuth-->>App: アクセストークン発行
    
    App->>ZoomAPI: API呼び出し<br/>(Bearer Token)
    ZoomAPI-->>App: 録画データ/AI要約
    
    App->>User: ダウンロード完了通知
```

## 5. GUI状態遷移図

```mermaid
stateDiagram-v2
    [*] --> Initial : アプリ起動
    
    Initial --> ConfigLoading : 設定読み込み
    ConfigLoading --> ConfigFound : 設定あり
    ConfigLoading --> ConfigNotFound : 設定なし
    
    ConfigNotFound --> ConfigInput : 設定入力画面
    ConfigInput --> ConfigFound : 設定保存
    
    ConfigFound --> Ready : 認証準備完了
    Ready --> Authenticating : 認証開始
    
    Authenticating --> AuthURL : 認証URL生成
    AuthURL --> WaitingAuth : 認証待機
    WaitingAuth --> AuthComplete : 認証完了
    WaitingAuth --> AuthError : 認証エラー
    
    AuthError --> Ready : リトライ
    AuthComplete --> Authenticated : 認証済み
    
    Authenticated --> DateInput : 期間設定
    DateInput --> Downloading : ダウンロード開始
    
    Downloading --> Progress : 進行中
    Progress --> Complete : 完了
    Progress --> DownloadError : エラー
    
    DownloadError --> Authenticated : リトライ
    Complete --> Authenticated : 新規ダウンロード
    
    Complete --> [*] : アプリ終了
```

## 6. コンポーネント依存関係図

```mermaid
graph TD
    subgraph "Binary Target"
        MainGUI[main.rs<br/>GUI Entry Point]
    end
    
    subgraph "Core Modules"
        Lib[lib.rs<br/>Core Library]
        GUI[gui.rs<br/>GUI Implementation]
        WinConsole[windows_console.rs<br/>Windows Support]
    end
    
    subgraph "Data Structures"
        Config[Config]
        Recording[Recording]
        AISummary[AISummaryResponse]
        Downloader[ZoomRecordingDownloader]
    end
    
    subgraph "External Crates"
        EFrame[eframe]
        Reqwest[reqwest]
        OAuth2[oauth2]
        Tokio[tokio]
        Serde[serde]
        Windows[windows]
    end
    
    MainGUI --> GUI
    MainGUI --> WinConsole
    
    GUI --> Lib
    GUI --> EFrame
    WinConsole --> Windows
    
    Lib --> Config
    Lib --> Recording
    Lib --> AISummary
    Lib --> Downloader
    
    Lib --> Reqwest
    Lib --> OAuth2
    Lib --> Tokio
    Lib --> Serde
    
    Downloader --> Reqwest
    Downloader --> OAuth2
```

## 7. ファイル構造とモジュール関係

```
zoom_video_mover/
├── src/
│   ├── main.rs          # GUI アプリケーションエントリーポイント
│   ├── lib.rs           # コアライブラリ
│   │   ├── Config       # 設定管理
│   │   ├── Recording    # 録画データ構造
│   │   ├── AISummary    # AI要約データ構造
│   │   └── ZoomDownloader # API通信・ダウンロード
│   ├── gui.rs           # GUI実装
│   │   ├── ZoomDownloaderApp # メインアプリ状態
│   │   ├── AppMessage   # 非同期メッセージング
│   │   └── UI Rendering # 各セクション描画
│   └── windows_console.rs # Windows固有処理
│       ├── Console Encoding # UTF-8設定
│       └── Japanese Output  # 日本語出力サポート
├── config.toml          # OAuth設定
├── debug_responses/     # AI要約レスポンスログ
└── Cargo.toml          # 依存関係・ビルド設定
```

## 7.1 AI要約データ構造の詳細

```rust
// 主要なAI要約レスポンス構造
pub struct AISummaryResponse {
    // 基本情報
    pub meeting_uuid: String,
    pub meeting_id: String,
    pub summary_title: String,
    
    // タイムスタンプ
    pub summary_start_time: String,
    pub summary_end_time: String,
    pub summary_created_time: String,
    pub summary_last_modified_time: String,
    
    // 要約コンテンツ
    pub summary_overview: String,      // 概要
    pub summary_content: String,       // 詳細内容（Markdown）
    pub summary_keyword: Vec<String>,  // キーワード
    
    // 構造化データ
    pub summary_details: Vec<SummaryDetail>,
    pub topic_summaries: Vec<TopicSummary>,
    pub detailed_sections: Vec<DetailedSection>,
    pub next_steps: Vec<String>,
    
    // 代替フィールド（Zoom APIの変動対応）
    pub summary: String,               // alias for summary_overview
    pub key_points: Vec<String>,       // alias for important points
    pub action_items: Vec<String>,     // alias for next_steps
}

// 支援構造体
pub struct SummaryDetail {
    pub label: String,
    pub summary: String,
}

pub struct TopicSummary {
    pub topic_title: String,
    pub topic_content: String,
}

pub struct DetailedSection {
    pub section_title: String,
    pub section_content: String,
}

// 汎用フォーマット対応
pub struct GenericAISummary {
    #[serde(flatten)]
    pub data: serde_json::Map<String, serde_json::Value>,
}
```