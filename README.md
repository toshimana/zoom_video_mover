# Zoom Cloud Recording Downloader

ZoomクラウドレコーディングをローカルにダウンロードするRustアプリケーション

## 必要なソフトウェア

### Windows 11
1. **Rust** - https://rustup.rs/ からインストール
2. **Visual Studio Build Tools** - C++ build toolsが必要
   - Visual Studio Installer から "C++ によるデスクトップ開発" をインストール
   - または Visual Studio Community をインストール

## セットアップ

### 1. Zoom OAuth アプリを作成
1. https://marketplace.zoom.us/develop/create にアクセス
2. "OAuth" アプリを選択
3. アプリ情報を入力:
   - App Name: 任意の名前
   - Choose App Type: User-managed app
   - Redirect URL: `http://localhost:8080/callback`
4. Scopes で `recording:read` を追加
5. Client ID と Client Secret を控える

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

- **自動ブラウザ起動**: OAuth認証時にデフォルトブラウザが自動で開きます
- **Windowsパス対応**: ファイル名にWindows で使用できない文字が含まれる場合は自動で置換
- **ダウンロードフォルダ**: デフォルトでユーザーのダウンロードフォルダ内にZoomRecordingsフォルダを作成
- **Windows環境変数**: `set ZOOM_ACCESS_TOKEN=...` 形式で環境変数設定コマンドを表示

## トラブルシューティング

### エラー: "linker 'link.exe' not found"
Visual Studio Build Tools がインストールされていません。上記の必要なソフトウェアをインストールしてください。

### エラー: "failed to run custom build command for 'openssl-sys'"
以下のコマンドでrust-opensslの代わりにnative-tlsを使用します（既に設定済み）:
```powershell
$env:OPENSSL_NO_VENDOR="1"
cargo build
```

### ダウンロードが失敗する
- インターネット接続を確認
- Zoom APIの制限（API rate limit）に達している可能性があります。しばらく待ってから再試行
- アクセストークンの有効期限が切れている場合は、再認証が必要

## ファイル形式

ダウンロードされるファイル:
- MP4形式の動画ファイル
- M4A形式の音声ファイル

ファイル名形式: `{会議ID}_{録画タイプ}_{録画ID}.{拡張子}`