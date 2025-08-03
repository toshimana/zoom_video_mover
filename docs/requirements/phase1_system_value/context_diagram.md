# コンテキスト図・システム環境分析

## システムコンテキスト概要

**プロジェクト名**: Zoom Video Mover  
**作成日**: 2025-08-02  
  

## システムコンテキスト図

### 高レベルコンテキスト図

```plantuml
@startuml
!theme plain
title Zoom Video Mover - システムコンテキスト図

actor "企業ユーザー" as CompanyUser
actor "個人ユーザー" as IndividualUser  
actor "IT管理者" as ITAdmin

rectangle "Zoom Video Mover" as ZVM {
  component "GUI Application" as GUI
  component "Download Engine" as Engine
  component "OAuth Manager" as OAuth
}

cloud "Zoom Cloud Platform" as ZoomCloud {
  component "API Server" as ZoomAPI
  component "Recording Storage" as ZoomStorage
  component "AI Companion" as ZoomAI
}

node "Windows OS" as WindowsOS {
  component "File System" as FileSystem
  component "Network Stack" as Network
  component "Process Manager" as ProcessMgr
}

database "Local Storage" as LocalDB {
  folder "Downloaded Files" as Files
  folder "Configuration" as Config
  folder "Logs" as Logs
}

' User interactions
CompanyUser --> GUI : 設定・操作
IndividualUser --> GUI : ファイル選択・DL
ITAdmin --> GUI : 設定管理・監視

' System interactions  
GUI --> Engine : ダウンロード要求
Engine --> OAuth : 認証・トークン管理
OAuth --> ZoomAPI : OAuth認証
Engine --> ZoomAPI : 録画リスト取得
Engine --> ZoomStorage : ファイルダウンロード
Engine --> ZoomAI : AI要約取得

' OS interactions
Engine --> Network : HTTP/HTTPS通信
Engine --> FileSystem : ファイル保存
GUI --> ProcessMgr : ウィンドウ管理

' Storage interactions
Engine --> Files : 録画ファイル保存
OAuth --> Config : 設定保存・読込
Engine --> Logs : ログ出力

@enduml
```

### 詳細相互作用図

```plantuml
@startuml
!theme plain
title Zoom Video Mover - 詳細相互作用図

left to right direction

package "ユーザー" {
  actor "企業IT管理者" as Admin
  actor "プロジェクトマネージャー" as PM  
  actor "個人ユーザー" as Individual
}

package "Zoom Video Mover" {
  component "設定管理" as ConfigMgr
  component "認証システム" as AuthSys
  component "録画検索" as SearchEngine
  component "ダウンロード制御" as DownloadCtrl
  component "進捗管理" as ProgressMgr
  component "エラー処理" as ErrorHandler
}

package "外部システム" {
  cloud "Zoom OAuth Server" as ZoomOAuth
  cloud "Zoom API Gateway" as ZoomAPI
  cloud "Zoom Recording Storage" as ZoomFiles
  cloud "Zoom AI Services" as ZoomAI
}

package "Windowsプラットフォーム" {
  component "ファイルシステム" as WinFS
  component "ネットワーク" as WinNet
  component "レジストリ" as WinReg
  component "UI Framework" as WinUI
}

' 主要な相互作用の定義
Admin --> ConfigMgr : OAuth設定・ポリシー設定
PM --> SearchEngine : 期間指定・フィルタ設定
Individual --> DownloadCtrl : ファイル選択・実行

ConfigMgr --> WinReg : 設定永続化
AuthSys --> ZoomOAuth : 認証・トークン取得
SearchEngine --> ZoomAPI : 録画リスト取得
DownloadCtrl --> ZoomFiles : ファイルダウンロード
DownloadCtrl --> ZoomAI : AI要約取得

ProgressMgr --> WinUI : 進捗バー・通知表示
ErrorHandler --> WinUI : エラーメッセージ表示
DownloadCtrl --> WinFS : ローカルファイル保存
AuthSys --> WinNet : HTTPS通信

@enduml
```

## ステークホルダー分析

### プライマリアクター（直接利用者）

#### 1. 企業ユーザー
**特徴**:
- 組織: 中小企業～大企業
- 利用規模: 10-100人のチーム
- 利用頻度: 日常的（週3-5回）
- 技術レベル: 中級

**システムとの相互作用**:
- **入力**: OAuth設定、期間指定、ファイル選択
- **出力**: ダウンロード済みファイル、進捗情報、エラー通知
- **期待**: 大量ファイル一括処理、確実性、自動化

#### 2. 個人ユーザー  
**特徴**:
- 職種: フリーランス、コンサルタント、教育者
- 利用規模: 個人単位
- 利用頻度: 不定期（週1-2回）
- 技術レベル: 初級～中級

**システムとの相互作用**:
- **入力**: 基本設定、選択的ダウンロード
- **出力**: 録画ファイル、簡潔な進捗表示
- **期待**: 簡単操作、直感的UI、エラー発生最小化

### セカンダリアクター（間接影響者）

#### 3. IT管理者
**特徴**:
- 役割: システム導入・運用・セキュリティ管理
- 責任範囲: 企業IT環境全体
- 技術レベル: 高級

**システムとの相互作用**:
- **入力**: セキュリティポリシー、運用設定、監視設定
- **出力**: 監査ログ、セキュリティレポート、運用状況
- **期待**: セキュリティ遵守、監査対応、運用負荷最小化

### 外部システム（システム境界外）

#### 4. Zoom Cloud Platform
**提供サービス**:
- **OAuth認証**: ユーザー認証・トークン発行
- **Recording API**: 録画リスト・メタデータ提供
- **File Storage**: 録画ファイル本体の配信
- **AI Services**: Zoom AI Companion要約データ

**制約・制限**:
- **レート制限**: 10 requests/second
- **認証要求**: OAuth 2.0必須
- **データアクセス**: 権限スコープ制限

#### 5. Windows Operating System
**提供機能**:
- **ファイルシステム**: NTFS・ReFS対応
- **ネットワーク**: TCP/IP・HTTPS通信
- **UI Framework**: Win32・WinUI対応
- **プロセス管理**: マルチタスク・メモリ管理

**制約・制限**:
- **ファイルパス**: 260文字制限（レガシー）
- **ファイル名**: 予約語・特殊文字制限
- **権限**: UAC・管理者権限

## システム環境・制約の整理

### 技術環境制約

#### プラットフォーム制約
| 項目 | 制約内容 | 影響度 | 対策 |
|------|----------|--------|------|
| **OS バージョン** | Windows 10/11 (x64) | 高 | サポート対象明確化 |
| **ファイルパス長** | 260文字制限 | 中 | パス短縮・長いパス対応 |
| **同時接続数** | TCP接続制限 | 中 | 接続プール管理 |
| **メモリ使用量** | プロセス上限 2GB | 中 | ストリーミング処理 |

#### ネットワーク制約
| 項目 | 制約内容 | 影響度 | 対策 |
|------|----------|--------|------|
| **プロキシ環境** | 企業プロキシ必須 | 高 | プロキシ設定対応 |
| **ファイアウォール** | HTTPS(443)のみ許可 | 中 | HTTPS通信強制 |
| **帯域制限** | 組織の帯域制限 | 中 | 帯域調整機能 |
| **接続タイムアウト** | 30秒制限 | 低 | タイムアウト調整 |

### 外部API制約

#### Zoom API制約
| API | 制限内容 | 対策 |
|-----|----------|------|
| **認証API** | OAuth 2.0必須 | 完全準拠実装 |
| **録画API** | 10 req/sec | レート制限監視 |
| **ダウンロード** | ファイルサイズ上限なし | ストリーミング対応 |
| **AI API** | 不安定・仕様変更有** | 複数エンドポイント試行 |

#### セキュリティ制約
| 項目 | 要件 | 実装方針 |
|------|------|----------|
| **通信暗号化** | TLS 1.2以上 | HTTPS強制・証明書検証 |
| **認証情報保護** | 暗号化保存 | Windows DPAPI利用 |
| **ログ出力** | 機密情報マスク | 構造化ログ・フィルタリング |
| **プロセス分離** | 最小権限実行 | 標準ユーザー権限 |

## 相互作用の可視化

### データフロー図

```plantuml
@startuml
!theme plain
title Zoom Video Mover - データフロー図

actor User as "ユーザー"
participant GUI as "GUI Interface"
participant Auth as "認証管理"
participant Search as "検索エンジン"
participant Download as "DL制御"
participant FileSystem as "ファイルシステム"

box "Zoom Cloud" #LightBlue
  participant OAuth as "OAuth Server"
  participant API as "Recording API"
  participant Storage as "File Storage"  
  participant AI as "AI Services"
end box

== 認証フェーズ ==
User -> GUI: 1. OAuth設定入力
GUI -> Auth: 2. 認証開始
Auth -> OAuth: 3. 認証要求
OAuth -> Auth: 4. 認証コード
Auth -> OAuth: 5. トークン要求  
OAuth -> Auth: 6. アクセストークン
Auth -> GUI: 7. 認証完了通知

== 検索フェーズ ==
User -> GUI: 8. 検索条件入力
GUI -> Search: 9. 検索実行
Search -> API: 10. 録画リスト要求
API -> Search: 11. 録画メタデータ
Search -> AI: 12. AI要約要求
AI -> Search: 13. 要約データ
Search -> GUI: 14. 検索結果表示

== ダウンロードフェーズ ==
User -> GUI: 15. ファイル選択
GUI -> Download: 16. DL要求
Download -> Storage: 17. ファイル要求
Storage -> Download: 18. ファイルストリーム
Download -> FileSystem: 19. ファイル保存
Download -> GUI: 20. 進捗通知
GUI -> User: 21. 完了通知

@enduml
```

### システム状態遷移図

```plantuml
@startuml
!theme plain
title Zoom Video Mover - システム状態遷移図

[*] --> 初期化
初期化 --> 設定確認 : アプリ起動

設定確認 --> 設定入力 : 設定不備
設定確認 --> 認証待機 : 設定完了

設定入力 --> 認証待機 : 設定保存

認証待機 --> 認証中 : 認証開始
認証中 --> 認証完了 : 認証成功
認証中 --> 認証エラー : 認証失敗
認証エラー --> 認証待機 : リトライ

認証完了 --> 検索待機 : 

検索待機 --> 検索実行中 : 検索開始
検索実行中 --> 検索完了 : 結果取得
検索実行中 --> 検索エラー : API エラー
検索エラー --> 検索待機 : リトライ

検索完了 --> ダウンロード準備 : ファイル選択

ダウンロード準備 --> ダウンロード実行中 : DL開始
ダウンロード実行中 --> ダウンロード完了 : 全ファイル完了
ダウンロード実行中 --> ダウンロード一時停止 : ユーザー操作
ダウンロード実行中 --> ダウンロードエラー : エラー発生

ダウンロード一時停止 --> ダウンロード実行中 : 再開
ダウンロードエラー --> ダウンロード実行中 : リトライ
ダウンロードエラー --> 検索待機 : エラー回復不能

ダウンロード完了 --> 検索待機 : 新規検索
ダウンロード完了 --> [*] : アプリ終了

@enduml
```

## システム品質属性

### 外部品質特性

#### 機能性（Functionality）
- **適合性**: Zoom API仕様完全準拠
- **正確性**: ファイル整合性100%保証
- **相互運用性**: Windows 10/11完全対応

#### 信頼性（Reliability）
- **成熟性**: 継続稼働時間24時間以上
- **障害許容性**: ネットワーク断・一時的API障害からの自動回復
- **回復性**: エラー発生後30秒以内の自動リトライ

#### 使用性（Usability）
- **理解性**: 初回利用時の設定完了率95%以上
- **習得性**: 基本操作習得時間30分以内
- **操作性**: 3クリック以内でのダウンロード開始

#### 効率性（Efficiency）
- **時間効率性**: 100ファイル処理10分以内
- **資源効率性**: メモリ使用量1GB以内
- **容量効率性**: ディスク使用量最小化

### 内部品質特性

#### 保守性（Maintainability）
- **解析性**: ログ・エラー情報による迅速な問題特定
- **変更性**: 新機能追加時の既存機能影響最小化
- **安定性**: マイナーバージョンアップでの後方互換性
- **試験性**: 自動テスト・Property-basedテストによる品質保証

#### 移植性（Portability）
- **適応性**: 異なるWindows環境での動作保証
- **設置性**: インストーラーレス・単一実行ファイル
- **共存性**: 他アプリケーションとの競合回避

## 制約事項・前提条件

### 運用制約

#### ユーザー制約
- **技術レベル**: 基本的なPC操作スキル必須
- **Zoomアカウント**: 有効なZoomアカウント・録画権限必須
- **ネットワーク**: 常時インターネット接続必須

#### システム制約
- **同時実行**: 1アカウントあたり1プロセスのみ
- **ファイルアクセス**: 排他制御によるファイル競合回避
- **リソース管理**: メモリ・CPU使用量の適切な制限

### 技術前提条件

#### 開発前提
- **言語**: Rust 1.70以上
- **フレームワーク**: egui/eframe最新安定版
- **ビルド環境**: Windows SDK・Visual Studio Build Tools

#### 実行前提
- **OS**: Windows 10 Version 1903以上
- **ハードウェア**: x64アーキテクチャ、RAM 4GB以上
- **ネットワーク**: HTTPS通信可能な環境

---

**承認**:  
**品質基準適合**: [ ] 確認済  
**ポリシー準拠**: [ ] 確認済  
**承認日**: ___________