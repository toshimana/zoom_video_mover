# Zoom Cloud Recording Downloader

ZoomクラウドレコーディングをローカルにダウンロードするRustアプリケーション

## 📚 ドキュメント構成

プロジェクトドキュメントは以下のフォルダに整理されています：

### `/docs/requirements/` - 要求・要件仕様
- **requirements_policy.md** - 要件定義方針とプロセス
- **system_requirements_summary.md** - システム要求仕様書

### `/docs/design/` - 設計仕様
- **design_policy.md** - 設計方針とアーキテクチャガイドライン

### `/docs/implementation/` - 実装仕様
- **implementation_policy.md** - 実装方針とコーディング規約

### `/docs/testing/` - テスト仕様
- **testing_policy.md** - テスト戦略とProperty-based Testing方針

### `/docs/analysis/` - 分析・レポート
- **development_process_and_deliverables_report.md** - 開発プロセスと成果物の関係分析

### 🔗 トレーサビリティ
すべての文書は要件-設計-実装-テストの完全なトレーサビリティを提供し、`CLAUDE.md`の包括的な開発ガイドラインと連携しています。

## 必要なソフトウェア

### Windows 11
1. **Rust** - https://rustup.rs/ からインストール（Cargo 1.87.0 対応）
2. **Visual Studio Build Tools** - C++ build toolsが必要
   - Visual Studio Installer から "C++ によるデスクトップ開発" をインストール
   - または Visual Studio Community をインストール
3. **Windows SDK** (通常Visual Studio Build Toolsに含まれる)

## セットアップ

### 1. Zoom OAuth アプリを作成

**重要: 正しい手順で作成してください**

1. **Zoom Marketplace にアクセス**
   - https://marketplace.zoom.us/develop/create

2. **アプリタイプを選択**
   - "OAuth" を選択（Server-to-Server OAuthではない）

3. **基本情報を入力**
   - App Name: 任意の名前（例: "Recording Downloader"）
   - Choose App Type: **User-managed app** を選択
   - Would you like to publish this app?: **No** を選択

4. **OAuth情報を設定**
   - Redirect URL: `http://localhost:8080/callback`
   - Allow users to install: チェックを入れる

5. **Scopes (権限) を追加**
   - `recording:read` スコープを追加
   - `user:read` スコープを追加（必須）
   - `meeting:read` スコープを追加（AI要約機能用）

6. **認証情報を取得**
   - **App Credentials** セクションから:
     - Client ID をコピー
     - Client Secret をコピー

**注意事項:**
- Client IDは通常、英数字とハイフンの組み合わせです
- Server-to-Server OAuthアプリではなく、通常のOAuthアプリを作成してください
- アプリが "Development" ステータスであることを確認してください

### 2. 設定ファイルの作成
初回実行時に自動で `config.toml` が作成されます。以下の内容を編集してください:

```toml
client_id = "あなたのZoom Client ID"
client_secret = "あなたのZoom Client Secret"
redirect_uri = "http://localhost:8080/callback"
```

## 使用方法

### Windows PowerShellでの実行
```powershell
# プロジェクトディレクトリに移動
cd path\to\zoom_video_mover

# 初回ビルド・実行
cargo run

# 2回目以降
cargo run --release
```

### Windows コマンドプロンプトでの実行
```cmd
rem プロジェクトディレクトリに移動
cd path\to\zoom_video_mover

rem 初回ビルド・実行
cargo run

rem 2回目以降
cargo run --release
```

## 実行手順

1. プログラムを実行
2. 初回実行時はブラウザが自動で開きZoom認証画面が表示されます
3. Zoomアカウントでログインし、アプリを承認
4. 表示される認証コードをコピーしてターミナルに入力
5. ダウンロードしたい期間（開始日・終了日）を入力（YYYY-MM-DD形式）
6. 保存先ディレクトリを指定（デフォルト: ユーザーのダウンロードフォルダ\ZoomRecordings）
7. ダウンロード開始

## Windows固有の機能

- **日本語文字化け対策**: Windows環境での日本語表示問題を自動解決
  - コンソール出力をUTF-8エンコーディングに自動設定
  - 日本語メッセージが正しく表示されます
- **自動ブラウザ起動**: OAuth認証時にデフォルトブラウザが自動で開きます
- **Windowsパス対応**: ファイル名にWindows で使用できない文字が含まれる場合は自動で置換
- **ダウンロードフォルダ**: デフォルトでユーザーのダウンロードフォルダ内にZoomRecordingsフォルダを作成
- **Windows環境変数**: `set ZOOM_ACCESS_TOKEN=...` 形式で環境変数設定コマンドを表示

### GUIアプリケーション（実験的）

CLI版に加えて、使いやすいGUIアプリケーションも提供しています：

```powershell
# GUIアプリケーションを起動
cargo run --bin zoom_video_mover_gui --release
```

**GUI版の機能:**
- 直感的なユーザーインターフェース
- リアルタイムダウンロード進捗表示
- 設定ファイルの自動生成・編集
- 日付範囲の簡単選択
- **Windows環境での日本語フォント自動対応**
  - Yu Gothic UI, Meiryo UI, MS UI Gothic などの日本語フォント自動検出
  - フォント優先順位による最適な表示
  - 文字化け問題の自動解決

## トラブルシューティング

### 日本語の文字化け
**✅ 修正済み**: Windows環境での日本語文字化けは自動的に解決されるようになりました。

**修正内容:**
- Windows API (`SetConsoleOutputCP`, `SetConsoleCP`) を使用してUTF-8エンコーディングを自動設定
- 適切なエラーハンドリングを実装してすべてのWindows環境で動作
- 出力バッファリング問題も解決

それでも文字化けが発生する場合は以下を試してください：

**PowerShellの場合:**
```powershell
# UTF-8エンコーディングを設定
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
cargo run --release
```

**コマンドプロンプトの場合:**
```cmd
# UTF-8コードページを設定
chcp 65001
cargo run --release
```

### GUIアプリケーションでの日本語表示問題
**✅ 修正済み**: GUIアプリケーションでの日本語文字化けと「No font data found」エラーは解決されました。

**修正内容:**
- **フォントエラーの完全解決**: システムフォントに依存しない安全な設定
- **eGUIデフォルトフォント活用**: 組み込みフォントによる安定した日本語表示
- **エラーハンドリング強化**: フォント関連のパニックを防止
- **シンプルで堅牢な設計**: 複雑なフォント検出を排除し、確実な動作を保証

**技術的な変更:**
- カスタムフォント設定を削除し、eGUIの標準設定を使用
- システムフォント名の指定によるエラーを回避
- 「No font data found for "Yu Gothic UI"」エラーの根本解決

**現在の動作:**
```powershell
# 正常に起動するようになりました
cargo run --bin zoom_video_mover_gui --release
```

### エラー: "linker 'link.exe' not found"
Visual Studio Build Tools がインストールされていません。上記の必要なソフトウェアをインストールしてください。

### エラー: "failed to run custom build command for 'openssl-sys'"
このプロジェクトはrustls-tlsを使用しているため、OpenSSLは不要です。それでもエラーが発生する場合:
```powershell
# 依存関係をクリーンアップ
cargo clean
cargo build --release
```

### エラー: Cargo 1.87.0 での互換性問題
```powershell
# Rustツールチェーンを最新に更新
rustup update
cargo build --release
```

### エラー: "'client_id' は、内部コマンドまたは外部コマンド..."
このエラーはWindowsのコマンドライン解析の問題です：

**原因**: OAuth URLに含まれる `&` 文字がWindowsコマンドで別々のコマンドとして解釈される

**対処法**: 
1. ブラウザが自動で開かない場合は、表示されたURLを手動でコピー&ペーストしてください
2. 修正済みのコードでは `\"\"` を追加してURL全体を適切にエスケープしています

### AI要約のダウンロードが失敗する
AI要約のダウンロードが失敗する場合は、以下を確認してください：

#### 1. 必要な権限の確認
Zoom Marketplace でアプリの設定を確認し、以下のスコープが **すべて** 追加されていることを確認：
- `recording:read` ✓
- `user:read` ✓ 
- `meeting:read` ✓ **（AI要約用に必須）**

#### 2. Zoom AI Companionの有効化
- ミーティングでZoom AI Companionが有効になっている必要があります
- AI Companionが有効でないミーティングには要約は生成されません
- Zoom プロ/ビジネス/エンタープライズアカウントが必要です

#### 3. ミーティングの要件
- ミーティング時間が一定時間以上必要（通常10分以上）
- 参加者が複数人いる必要がある場合があります
- ミーティング終了から要約生成まで時間がかかる場合があります

#### 4. エラーメッセージの確認
アプリケーション実行時に表示されるエラーメッセージを確認：
- `401 (Unauthorized)` → アクセストークンの問題
- `403 (Forbidden)` → 権限不足（上記スコープを確認）
- `404 (Not found)` → 要約が存在しない（正常）
- `429 (Rate limit)` → API制限、時間をおいて再試行

### エラー: "申し訳ございません。リクエストを完了できませんでした。(4,700)"
このエラーはZoom OAuth設定の問題です。以下を順番に確認してください:

#### 1. Zoom Marketplace でのアプリ設定を確認

**App Type の確認:**
- アプリタイプが "OAuth" であることを確認
- "Server-to-Server OAuth" ではないことを確認

**App Information:**
- "Would you like to publish this app on Zoom Marketplace?" → **No** を選択
- "Choose your app type" → **User-managed app** を選択

**OAuth Information:**
- Redirect URL: `http://localhost:8080/callback` （完全一致必須）
- OAuth Allowlist: 空欄でOK

**Scopes:**
- `recording:read` を追加
- `user:read` を追加（**必須**）

#### 2. App Status を Activated にする方法

**重要**: アプリを使用するには必ずActivatedステータスにする必要があります。

**手順:**
1. **Zoom Marketplace** (https://marketplace.zoom.us/develop/create) にアクセス
2. 作成したOAuthアプリをクリック
3. **左サイドバー** の "App Credentials" をクリック
4. **"Activation" セクション** を確認:
   - Status が "Development" と表示されている場合
   - **"Activate your app"** ボタンまたは **"Activate"** ボタンをクリック
5. 確認ダイアログが表示されたら **"Activate"** をクリック
6. Status が **"Activated"** に変わることを確認

**注意事項:**
- アプリがActivatedステータスでないと OAuth認証が失敗します
- Development ステータスでも一部テストは可能ですが、本格的な使用にはActivation が必要
- 一度Activatedにすると、一部設定変更時に再Activationが必要な場合があります

**トラブルシューティング:**
- "Activate" ボタンが見つからない場合:
  1. App Information の設定が完了していることを確認
  2. 必要なScopes (`recording:read`, `user:read`) が追加されていることを確認
  3. Redirect URL が正しく設定されていることを確認

#### 3. Client ID/Secret の確認
- config.tomlのclient_idが正しいかチェック
- スペースや改行文字が含まれていないか確認
- Client Secret も正確にコピーされているか確認

#### 4. 一般的な解決方法
1. Zoom Marketplace でアプリを一度削除
2. 上記の設定で新しいOAuthアプリを作成
3. 新しいClient ID/Secretをconfig.tomlに設定
4. プログラムを再実行

### ダウンロードが失敗する
- インターネット接続を確認
- Zoom APIの制限（API rate limit）に達している可能性があります。しばらく待ってから再試行
- アクセストークンの有効期限が切れている場合は、再認証が必要

## ファイル形式

ダウンロードされるファイル:
- **MP4形式の動画ファイル** - メインの録画動画
- **M4A形式の音声ファイル** - 音声のみの録画
- **TXT形式のチャットファイル** - ミーティング中のチャット履歴
- **VTT形式のトランスクリプトファイル** - 自動生成された会話の文字起こし
- **CC.VTT形式のクローズドキャプション** - アクセシビリティ用字幕
- **CSV形式のデータファイル** - 投票結果、参加者リストなど
- **JSON形式のメタデータファイル** - ミーティングの詳細情報
- **🤖 AI Companion要約ファイル (JSON形式)** - Zoom AI Companionが生成した会議要約

**🆕 AI Companion機能**: 
- Zoom AI Companionが有効なミーティングでは、自動生成された要約をダウンロード
- 要約には以下が含まれます：
  - ミーティング概要
  - 詳細な内容要約
  - 次のアクションアイテム
  - キーワード抽出
  - 会議のメタデータ

ファイル名形式: 
- 録画ファイル: `{会議ID}_{録画タイプ}_{ファイル種別}_{録画ID}.{拡張子}`
- AI要約ファイル: `{会議ID}_ai_summary.json`

例:
- `1234567890_shared_screen_video_abcd1234.mp4` (画面共有の動画)
- `1234567890_audio_only_audio_efgh5678.m4a` (音声のみ)
- `1234567890_chat_file_chat_ijkl9012.txt` (チャット履歴)
- `1234567890_audio_transcript_transcript_mnop3456.vtt` (音声文字起こし)
- `1234567890_ai_summary.json` (🤖 AI Companion要約)