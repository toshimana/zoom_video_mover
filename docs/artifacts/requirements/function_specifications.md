# 機能仕様書 - Zoom Video Mover

## 機能仕様概要

本文書は、Zoom Video Moverの機能詳細仕様を定義します。各機能の入力・出力・処理ロジック・例外処理を詳細に記述し、実装の指針となる仕様を提供します。

## 機能一覧

| 機能ID | 機能名 | カテゴリ | 実装ファイル | 対応要件 | 優先度 |
|--------|--------|----------|-------------|----------|--------|
| **FN001** | 設定管理機能 | Core | lib.rs:Config | FR001-2 | 高 |
| **FN002** | OAuth認証機能 | Authentication | lib.rs:ZoomRecordingDownloader | FR001-1 | 高 |
| **FN003** | 録画検索機能 | API | lib.rs:get_recordings | FR002-1 | 高 |
| **FN004** | ファイルダウンロード機能 | Download | lib.rs:download_recording | FR003-1 | 高 |
| **FN005** | AI要約取得機能 | API | lib.rs:get_ai_summary | FR003-3 | 中 |
| **FN006** | 進捗管理機能 | UI | gui.rs:ProgressTracker | FR003-2 | 中 |
| **FN007** | エラー処理機能 | Core | lib.rs:Error handling | NFR002-1 | 高 |
| **FN008** | ファイル管理機能 | File System | lib.rs:FileManager | NFR004-2 | 中 |
| **FN009** | ログ出力機能 | Logging | lib.rs:Logger | NFR002-2 | 中 |
| **FN010** | Windows対応機能 | Platform | windows_console.rs | NFR004-1 | 中 |

---

## FN001: 設定管理機能

### 機能概要
- **目的**: OAuth認証情報とアプリケーション設定の永続化
- **実装**: `lib.rs:Config` struct
- **対応要件**: FR001-2（Client ID/Secret設定）

### 入力仕様

#### 設定項目
| 項目名 | 型 | 必須 | デフォルト値 | 検証ルール |
|--------|-----|------|-------------|------------|
| `client_id` | String | ○ | - | 空文字禁止、英数字・ハイフン・アンダースコア |
| `client_secret` | String | ○ | - | 空文字禁止、20文字以上 |
| `redirect_uri` | Option<String> | × | `http://localhost:8080/callback` | URL形式 |

#### 設定ファイル形式 (TOML)
```toml
client_id = "zoom_client_id_example"
client_secret = "zoom_client_secret_example_very_long"
redirect_uri = "http://localhost:8080/callback"
```

### 出力仕様

#### 設定読み込み結果
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: Option<String>,
}
```

### 処理仕様

#### FN001-1: 設定ファイル読み込み
```rust
/// 設定ファイルから設定を読み込む
/// 
/// # 事前条件
/// - path は有効なファイルパスを指す
/// - ファイルが存在し、読み取り可能である
/// - ファイルの内容は有効な TOML 形式である
/// 
/// # 事後条件
/// - 成功時: 有効な Config インスタンスを返す
/// - client_id および client_secret は空でない
/// - 失敗時: 適切なエラーを返す
/// 
/// # 不変条件
/// - ファイルシステムの状態は変更されない
/// - 入力パラメータは変更されない
pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>>
```

**処理フロー:**
1. ファイル存在確認
2. ファイル内容読み込み (`fs::read_to_string`)
3. TOML解析 (`toml::from_str`)
4. 設定検証（必須項目チェック）
5. Config構造体返却

#### FN001-2: 設定ファイル保存
```rust
/// 設定をファイルに保存する
/// 
/// # 副作用
/// - ファイルシステムへの書き込み（指定されたパスにファイルを保存）
/// 
/// # 事前条件
/// - self は有効な Config インスタンスである
/// - path は有効なファイルパスを指す
/// - ファイルの親ディレクトリが存在するか作成可能である
/// - ファイルへの書き込み権限がある
/// 
/// # 事後条件
/// - 成功時: 設定が TOML 形式でファイルに保存される
/// - 失敗時: 適切なエラーを返す
pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>>
```

**処理フロー:**
1. 設定の事前検証
2. TOML形式シリアライズ (`toml::to_string_pretty`)
3. ファイル書き込み (`fs::write`)
4. 書き込み結果確認

#### FN001-3: サンプル設定ファイル作成
```rust
/// サンプル設定ファイルを作成する
/// 
/// # 副作用
/// - ファイルシステムへの書き込み（指定されたパスにファイルを作成）
/// 
/// # 事前条件
/// - path は有効なファイルパスを指す
/// - ファイルの親ディレクトリが存在するか作成可能である
/// - ファイルへの書き込み権限がある
/// 
/// # 事後条件
/// - 成功時: サンプル設定ファイルが作成される
/// - ファイルは有効な TOML 形式で保存される
/// - 失敗時: 適切なエラーを返す
pub fn create_sample_file(path: &str) -> Result<(), Box<dyn std::error::Error>>
```

### 例外処理

| 例外種別 | 発生条件 | エラーメッセージ | 処理方針 |
|----------|----------|-----------------|----------|
| **ファイル未存在** | 読み込み対象ファイルなし | "Configuration file not found" | サンプルファイル作成提案 |
| **TOML解析エラー** | 無効なTOML形式 | "Invalid TOML format" | 構文エラー詳細表示 |
| **必須項目不足** | client_id/secret空 | "Required fields missing" | 入力必須項目表示 |
| **権限エラー** | ファイル書き込み不可 | "Permission denied" | 管理者権限要求 |

---

## FN002: OAuth認証機能

### 機能概要
- **目的**: Zoom API アクセス用のOAuth 2.0認証
- **実装**: `lib.rs:ZoomRecordingDownloader`
- **対応要件**: FR001-1（OAuth 2.0認証フロー）

### 入力仕様

#### 認証パラメータ
| パラメータ | 型 | 必須 | 説明 |
|------------|-----|------|------|
| `client_id` | String | ○ | Zoom Developer App Client ID |
| `client_secret` | String | ○ | Zoom Developer App Client Secret |
| `redirect_uri` | String | ○ | OAuth リダイレクト URI |
| `scope` | Vec<String> | ○ | 要求権限: `["recording:read", "user:read", "meeting:read"]` |

### 出力仕様

#### 認証結果
```rust
#[derive(Debug, Clone)]
pub struct AuthToken {
    pub access_token: String,
    pub token_type: String,           // "Bearer"
    pub expires_in: u64,              // 有効期限（秒）
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub refresh_token: Option<String>,
    pub scope: String,
}
```

### 処理仕様

#### FN002-1: OAuth認証URL生成
```rust
/// OAuth認証URLを生成する
/// 
/// # 事前条件
/// - client_id が有効である
/// - redirect_uri が有効なURL形式である
/// - scope が空でない
/// 
/// # 事後条件
/// - 成功時: 有効な認証URLを返す
/// - URLにはstate パラメータが含まれる（CSRF対策）
/// - 失敗時: 適切なエラーを返す
/// 
/// # 不変条件
/// - client_id, redirect_uri は変更されない
pub fn generate_auth_url(&self) -> Result<String, Error>
```

**処理フロー:**
1. OAuth2クライアント初期化
2. CSRF対策用state生成
3. 認証URL組み立て
4. URL返却

**生成URL例:**
```
https://zoom.us/oauth/authorize?
response_type=code&
client_id=CLIENT_ID&
redirect_uri=http://localhost:8080/callback&
scope=recording:read user:read meeting:read&
state=RANDOM_STATE
```

#### FN002-2: 認証コード交換
```rust
/// 認証コードをアクセストークンに交換する
/// 
/// # 副作用
/// - HTTPリクエストの送信
/// 
/// # 事前条件
/// - auth_code が有効な認証コードである
/// - client_id, client_secret が有効である
/// 
/// # 事後条件
/// - 成功時: 有効なAuthTokenを返す
/// - アクセストークンの有効期限が設定される
/// - 失敗時: 適切なエラーを返す
pub async fn exchange_code(&self, auth_code: &str) -> Result<AuthToken, Error>
```

**処理フロー:**
1. トークン交換リクエスト作成
2. Zoom OAuth サーバーへPOST送信
3. レスポンス検証
4. AuthToken構造体作成
5. 有効期限計算・設定

#### FN002-3: トークンリフレッシュ
```rust
/// リフレッシュトークンを使用してアクセストークンを更新する
/// 
/// # 副作用
/// - HTTPリクエストの送信
/// 
/// # 事前条件
/// - refresh_token が有効である
/// - client_id, client_secret が有効である
/// 
/// # 事後条件
/// - 成功時: 新しいAuthTokenを返す
/// - 古いトークンは無効化される
/// - 失敗時: 再認証が必要
pub async fn refresh_token(&self, refresh_token: &str) -> Result<AuthToken, Error>
```

### Zoom OAuth API仕様

#### 認証URL
- **エンドポイント**: `https://zoom.us/oauth/authorize`
- **メソッド**: GET
- **パラメータ**:
  - `response_type=code`
  - `client_id`: アプリケーションID
  - `redirect_uri`: リダイレクト先
  - `scope`: 要求権限
  - `state`: CSRF対策用ランダム文字列

#### トークン交換API
- **エンドポイント**: `https://zoom.us/oauth/token`
- **メソッド**: POST
- **ヘッダー**: `Authorization: Basic base64(client_id:client_secret)`
- **ボディ**:
  ```json
  {
    "grant_type": "authorization_code",
    "code": "認証コード",
    "redirect_uri": "リダイレクトURI"
  }
  ```

### 例外処理

| 例外種別 | 発生条件 | HTTPステータス | 処理方針 |
|----------|----------|---------------|----------|
| **認証拒否** | ユーザーがキャンセル | - | 再認証要求 |
| **無効クライアント** | Client ID/Secret不正 | 401 | 設定確認要求 |
| **無効コード** | 認証コード期限切れ | 400 | 認証やり直し |
| **ネットワークエラー** | 接続失敗 | - | リトライ処理 |
| **レート制限** | API制限超過 | 429 | 待機後リトライ |

---

## FN003: 録画検索機能

### 機能概要
- **目的**: 指定期間のZoom録画リスト取得
- **実装**: `lib.rs:get_recordings`
- **対応要件**: FR002-1（Zoom API呼び出し）

### 入力仕様

#### 検索パラメータ
| パラメータ | 型 | 必須 | デフォルト | 制約 |
|------------|-----|------|-----------|------|
| `from_date` | String | ○ | 30日前 | YYYY-MM-DD形式 |
| `to_date` | String | ○ | 今日 | from_date <= to_date |
| `page_size` | u32 | × | 30 | 1-300の範囲 |
| `page_number` | u32 | × | 1 | 1以上 |

### 出力仕様

#### 録画情報構造
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recording {
    pub meeting_id: String,
    pub topic: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub duration: u32,                    // 分
    pub recording_files: Vec<RecordingFile>,
    pub ai_summary_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingFile {
    pub id: String,
    pub file_type: String,               // MP4, MP3, TXT, JSON, VTT
    pub file_size: u64,                  // バイト
    pub download_url: String,
    pub play_url: Option<String>,
    pub recording_start: chrono::DateTime<chrono::Utc>,
    pub recording_end: chrono::DateTime<chrono::Utc>,
}
```

### 処理仕様

#### FN003-1: 録画リスト取得
```rust
/// 指定期間の録画リストを取得する
/// 
/// # 副作用
/// - HTTPリクエストの送信
/// 
/// # 事前条件
/// - access_token が有効である
/// - from_date <= to_date である
/// - 日付形式が YYYY-MM-DD である
/// 
/// # 事後条件
/// - 成功時: Recording のベクターを返す
/// - 録画が存在しない場合は空のベクターを返す
/// - 失敗時: 適切なエラーを返す
/// 
/// # 不変条件
/// - 入力パラメータは変更されない
pub async fn get_recordings(
    &self,
    from_date: &str,
    to_date: &str,
) -> Result<Vec<Recording>, Error>
```

**処理フロー:**
1. 日付形式検証
2. API リクエスト作成
3. ページネーション処理（必要に応じて複数回API呼び出し）
4. 録画詳細情報取得
5. Recording構造体変換
6. 結果返却

#### FN003-2: 録画詳細取得
```rust
/// 特定の録画の詳細情報を取得する
/// 
/// # 副作用
/// - HTTPリクエストの送信
/// 
/// # 事前条件
/// - meeting_id が有効である
/// - access_token が有効である
/// 
/// # 事後条件
/// - 成功時: 詳細なRecording情報を返す
/// - 録画が削除されている場合はエラーを返す
pub async fn get_recording_detail(&self, meeting_id: &str) -> Result<Recording, Error>
```

### Zoom Cloud Recording API仕様

#### 録画リスト取得API
- **エンドポイント**: `GET https://api.zoom.us/v2/users/me/recordings`
- **ヘッダー**: `Authorization: Bearer {access_token}`
- **クエリパラメータ**:
  - `from`: 開始日（YYYY-MM-DD）
  - `to`: 終了日（YYYY-MM-DD）
  - `page_size`: ページサイズ（デフォルト: 30、最大: 300）
  - `page_number`: ページ番号

#### API応答例
```json
{
  "from": "2024-01-01",
  "to": "2024-01-31",
  "page_count": 1,
  "page_number": 1,
  "page_size": 30,
  "total_records": 5,
  "meetings": [
    {
      "uuid": "meeting-uuid",
      "id": 123456789,
      "topic": "Weekly Team Meeting",
      "start_time": "2024-01-15T10:00:00Z",
      "duration": 90,
      "recording_files": [
        {
          "id": "file-id",
          "file_type": "MP4",
          "file_size": 1073741824,
          "download_url": "https://zoom.us/rec/download/...",
          "play_url": "https://zoom.us/rec/play/...",
          "recording_start": "2024-01-15T10:00:00Z",
          "recording_end": "2024-01-15T11:30:00Z"
        }
      ]
    }
  ]
}
```

### データ変換・検証

#### 日付形式変換
```rust
fn parse_date(date_str: &str) -> Result<chrono::NaiveDate, Error> {
    chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|_| Error::InvalidDateFormat(date_str.to_string()))
}

fn validate_date_range(from: &str, to: &str) -> Result<(), Error> {
    let from_date = parse_date(from)?;
    let to_date = parse_date(to)?;
    
    if from_date > to_date {
        return Err(Error::InvalidDateRange { from, to });
    }
    
    let duration = to_date.signed_duration_since(from_date);
    if duration.num_days() > 365 {
        return Err(Error::DateRangeTooLarge(duration.num_days()));
    }
    
    Ok(())
}
```

### 例外処理

| 例外種別 | 発生条件 | 処理方針 | リトライ |
|----------|----------|----------|----------|
| **日付形式エラー** | 無効な日付形式 | エラーメッセージ表示 | なし |
| **日付範囲エラー** | from > to | 日付修正要求 | なし |
| **認証エラー** | トークン無効 | 再認証要求 | なし |
| **ネットワークエラー** | 接続失敗 | 3回自動リトライ | あり |
| **API制限エラー** | レート制限 | 待機後リトライ | あり |
| **データなし** | 録画未存在 | 空リスト返却 | なし |

---

## FN004: ファイルダウンロード機能

### 機能概要
- **目的**: 選択された録画ファイルのローカルダウンロード
- **実装**: `lib.rs:download_recording`
- **対応要件**: FR003-1（並列ダウンロード）

### 入力仕様

#### ダウンロードパラメータ
```rust
#[derive(Debug, Clone)]
pub struct DownloadRequest {
    pub file_id: String,
    pub file_name: String,
    pub download_url: String,
    pub file_size: u64,
    pub output_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct DownloadConfig {
    pub max_concurrent: usize,           // 最大並列数（デフォルト: 5）
    pub chunk_size: usize,               // チャンクサイズ（デフォルト: 8192）
    pub timeout_seconds: u64,            // タイムアウト（デフォルト: 300）
    pub retry_count: u32,                // リトライ回数（デフォルト: 3）
}
```

### 出力仕様

#### ダウンロード進捗情報
```rust
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    pub file_id: String,
    pub file_name: String,
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
    pub transfer_rate: u64,              // bytes per second
    pub eta_seconds: Option<u64>,        // estimated time remaining
    pub status: DownloadStatus,
}

#[derive(Debug, Clone)]
pub enum DownloadStatus {
    Pending,
    InProgress,
    Paused,
    Completed,
    Failed(String),
    Cancelled,
}
```

### 処理仕様

#### FN004-1: 単一ファイルダウンロード
```rust
/// 単一ファイルをダウンロードする
/// 
/// # 副作用
/// - HTTPリクエストの送信
/// - ファイルシステムへの書き込み
/// - 進捗通知の送信
/// 
/// # 事前条件
/// - download_url が有効である
/// - output_path の親ディレクトリが存在する
/// - ファイル書き込み権限がある
/// 
/// # 事後条件
/// - 成功時: ファイルが指定パスに保存される
/// - ファイルサイズが期待値と一致する
/// - 失敗時: 一時ファイルが削除される
/// 
/// # 不変条件
/// - ダウンロード中にリクエスト情報は変更されない
pub async fn download_file(
    &self,
    request: DownloadRequest,
    progress_sender: Option<mpsc::Sender<DownloadProgress>>,
) -> Result<PathBuf, Error>
```

**処理フロー:**
1. 出力ディレクトリ作成
2. HTTPリクエスト開始
3. レスポンスヘッダー検証（Content-Length等）
4. 一時ファイル作成（`.tmp`拡張子）
5. チャンク単位ダウンロード・書き込み
6. 進捗通知送信（1秒間隔）
7. ダウンロード完了確認
8. 一時ファイルを最終ファイル名にリネーム

#### FN004-2: 並列ダウンロード制御
```rust
/// 複数ファイルを並列ダウンロードする
/// 
/// # 副作用
/// - 複数のHTTPリクエスト送信
/// - 複数ファイルの同時書き込み
/// - 進捗通知の送信
/// 
/// # 事前条件
/// - requests が空でない
/// - すべてのリクエストが有効である
/// - 並列数制限が適切に設定されている
/// 
/// # 事後条件
/// - 成功時: すべてのファイルがダウンロード完了
/// - 一部失敗時: 成功分は保持、失敗分は報告
/// - 全失敗時: エラー返却
pub async fn download_files_parallel(
    &self,
    requests: Vec<DownloadRequest>,
    config: DownloadConfig,
    progress_sender: mpsc::Sender<DownloadProgress>,
) -> Result<Vec<PathBuf>, Vec<Error>>
```

**並列制御実装:**
```rust
use tokio::sync::Semaphore;

async fn download_files_parallel(&self, requests: Vec<DownloadRequest>) -> Result<Vec<PathBuf>, Vec<Error>> {
    let semaphore = Arc::new(Semaphore::new(self.config.max_concurrent));
    let mut tasks = Vec::new();
    
    for request in requests {
        let permit = semaphore.clone().acquire_owned().await?;
        let downloader = self.clone();
        let progress_sender = self.progress_sender.clone();
        
        let task = tokio::spawn(async move {
            let _permit = permit; // Keep permit until task completes
            downloader.download_file(request, Some(progress_sender)).await
        });
        
        tasks.push(task);
    }
    
    // Collect results
    let mut results = Vec::new();
    let mut errors = Vec::new();
    
    for task in tasks {
        match task.await? {
            Ok(path) => results.push(path),
            Err(error) => errors.push(error),
        }
    }
    
    if errors.is_empty() {
        Ok(results)
    } else {
        Err(errors)
    }
}
```

#### FN004-3: レジュームダウンロード
```rust
/// 中断したダウンロードを再開する
/// 
/// # 副作用
/// - HTTPリクエストの送信（Range指定）
/// - ファイルの追記書き込み
/// 
/// # 事前条件
/// - 一時ファイルが存在する
/// - サーバーがRange リクエストをサポートする
/// 
/// # 事後条件
/// - 成功時: 中断位置からダウンロード再開
/// - 失敗時: フルダウンロードにフォールバック
pub async fn resume_download(
    &self,
    request: DownloadRequest,
    temp_file_path: &Path,
) -> Result<PathBuf, Error>
```

### ファイル管理

#### ファイル名サニタイズ
```rust
fn sanitize_filename(filename: &str) -> String {
    let invalid_chars = ['<', '>', ':', '"', '|', '?', '*', '/', '\\'];
    let mut sanitized = filename.to_string();
    
    for ch in invalid_chars {
        sanitized = sanitized.replace(ch, "_");
    }
    
    // Windowsの予約語チェック
    let reserved_names = ["CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9"];
    
    if reserved_names.contains(&sanitized.to_uppercase().as_str()) {
        sanitized = format!("_{}", sanitized);
    }
    
    // 長さ制限（Windows: 255文字）
    if sanitized.len() > 255 {
        sanitized.truncate(252);
        sanitized.push_str("...");
    }
    
    sanitized
}
```

#### ディスク容量チェック
```rust
fn check_disk_space(path: &Path, required_bytes: u64) -> Result<(), Error> {
    let available = fs::available_space(path)?;
    if available < required_bytes {
        return Err(Error::InsufficientDiskSpace {
            required: required_bytes,
            available,
        });
    }
    Ok(())
}
```

### 例外処理・リトライ戦略

| 例外種別 | 検出タイミング | リトライ戦略 | 最大試行回数 |
|----------|---------------|-------------|-------------|
| **ネットワークタイムアウト** | HTTP接続時 | 指数バックオフ | 3回 |
| **HTTP 4xx エラー** | ダウンロード開始時 | リトライしない | 1回 |
| **HTTP 5xx エラー** | ダウンロード中 | 線形バックオフ | 3回 |
| **ディスク容量不足** | 書き込み時 | ユーザー通知 | 手動対応 |
| **権限エラー** | ファイル作成時 | ユーザー通知 | 手動対応 |
| **接続切断** | ダウンロード中 | レジューム試行 | 3回 |

---

## FN005: AI要約取得機能

### 機能概要
- **目的**: Zoom AI Companion生成の会議要約取得
- **実装**: `lib.rs:get_ai_summary`
- **対応要件**: FR003-3（AI要約対応）

### 入力仕様

#### AI要約リクエスト
```rust
#[derive(Debug, Clone)]
pub struct AiSummaryRequest {
    pub meeting_id: String,
    pub meeting_uuid: String,
    pub include_details: bool,           // 詳細情報を含むか
}
```

### 出力仕様

#### AI要約レスポンス
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSummaryResponse {
    pub meeting_id: String,
    pub summary: String,
    pub key_points: Vec<String>,
    pub action_items: Vec<ActionItem>,
    pub participants_summary: Vec<ParticipantSummary>,
    pub meeting_duration_minutes: u32,
    pub summary_generated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    pub description: String,
    pub assignee: Option<String>,
    pub due_date: Option<chrono::NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantSummary {
    pub name: String,
    pub email: String,
    pub join_time: chrono::DateTime<chrono::Utc>,
    pub leave_time: chrono::DateTime<chrono::Utc>,
    pub duration_minutes: u32,
}
```

### 処理仕様

#### FN005-1: AI要約取得
```rust
/// 指定されたミーティングのAI要約を取得する
/// 
/// # 副作用
/// - HTTPリクエストの送信
/// - デバッグログファイルの出力（オプション）
/// 
/// # 事前条件
/// - meeting_id が有効である
/// - access_token が有効である
/// - ミーティングでAI要約が生成されている
/// 
/// # 事後条件
/// - 成功時: AiSummaryResponseを返す
/// - AI要約が生成されていない場合はNoneを返す
/// - 失敗時: 適切なエラーを返す
pub async fn get_ai_summary(
    &self,
    meeting_id: &str,
) -> Result<Option<AiSummaryResponse>, Error>
```

**処理フロー:**
1. AI要約可用性確認
2. AI要約API呼び出し
3. JSON応答解析
4. 構造体変換
5. デバッグファイル出力（開発時）
6. 結果返却

### Zoom AI Companion API仕様

#### AI要約取得API
- **エンドポイント**: `GET https://api.zoom.us/v2/meetings/{meetingId}/ai_companion`
- **ヘッダー**: `Authorization: Bearer {access_token}`
- **クエリパラメータ**: なし

#### API応答例
```json
{
  "meeting_id": "123456789",
  "ai_summary": {
    "summary": "This meeting focused on project planning and resource allocation...",
    "key_points": [
      "Budget approval required by end of month",
      "New team member starting next week",
      "Client presentation scheduled for Friday"
    ],
    "action_items": [
      {
        "description": "Prepare budget proposal",
        "assignee": "john.doe@company.com",
        "due_date": "2024-01-31"
      }
    ],
    "participants": [
      {
        "name": "John Doe",
        "email": "john.doe@company.com",
        "join_time": "2024-01-15T10:00:00Z",
        "leave_time": "2024-01-15T11:00:00Z",
        "duration": 60
      }
    ],
    "generated_at": "2024-01-15T11:05:00Z"
  }
}
```

### 例外処理

| 例外種別 | 発生条件 | 処理方針 |
|----------|----------|----------|
| **AI要約未生成** | API が404を返す | None返却 |
| **権限不足** | meeting:read権限なし | 権限エラー表示 |
| **AI機能無効** | ZoomプランでAI機能利用不可 | 機能無効通知 |
| **データ形式エラー** | JSON解析失敗 | エラーログ出力 |

---

## FN006: 進捗管理機能

### 機能概要
- **目的**: ダウンロード進捗のリアルタイム追跡・表示
- **実装**: `gui.rs:ProgressTracker`
- **対応要件**: FR003-2（進捗表示）

### 入力仕様

#### 進捗イベント
```rust
#[derive(Debug, Clone)]
pub enum ProgressEvent {
    DownloadStarted {
        file_id: String,
        file_name: String,
        total_bytes: u64,
    },
    ProgressUpdate {
        file_id: String,
        bytes_downloaded: u64,
        transfer_rate: u64,
    },
    DownloadCompleted {
        file_id: String,
        final_path: PathBuf,
    },
    DownloadFailed {
        file_id: String,
        error: String,
    },
    AllCompleted {
        total_files: usize,
        total_bytes: u64,
        duration: Duration,
    },
}
```

### 出力仕様

#### 進捗状態
```rust
#[derive(Debug, Clone)]
pub struct ProgressState {
    pub total_files: usize,
    pub completed_files: usize,
    pub failed_files: usize,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub current_file: Option<String>,
    pub overall_progress: f32,           // 0.0 - 1.0
    pub transfer_rate: u64,              // bytes per second
    pub eta_seconds: Option<u64>,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub active_downloads: HashMap<String, FileProgress>,
}

#[derive(Debug, Clone)]
pub struct FileProgress {
    pub file_id: String,
    pub file_name: String,
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
    pub progress: f32,                   // 0.0 - 1.0
    pub transfer_rate: u64,
    pub status: DownloadStatus,
}
```

### 処理仕様

#### FN006-1: 進捗状態更新
```rust
/// 進捗イベントを受信して状態を更新する
/// 
/// # 事前条件
/// - self が初期化されている
/// - event が有効なProgressEventである
/// 
/// # 事後条件
/// - 進捗状態が最新情報で更新される
/// - 統計情報が再計算される
/// - UI更新通知が送信される
/// 
/// # 不変条件
/// - 完了ファイル数 + 失敗ファイル数 <= 総ファイル数
/// - ダウンロード済みバイト数 <= 総バイト数
pub fn update_progress(&mut self, event: ProgressEvent)
```

#### FN006-2: 転送速度計算
```rust
/// 移動平均による転送速度を計算する
/// 
/// # 事前条件
/// - 少なくとも2つの測定点が存在する
/// 
/// # 事後条件
/// - 過去N秒間の平均転送速度を返す
/// - 異常値は除外される
fn calculate_transfer_rate(&self, time_window_seconds: u64) -> u64
```

**転送速度計算ロジック:**
```rust
struct SpeedSample {
    timestamp: Instant,
    bytes_downloaded: u64,
}

impl ProgressTracker {
    fn calculate_transfer_rate(&self, window_seconds: u64) -> u64 {
        let now = Instant::now();
        let cutoff = now - Duration::from_secs(window_seconds);
        
        let recent_samples: Vec<_> = self.speed_samples
            .iter()
            .filter(|sample| sample.timestamp >= cutoff)
            .collect();
        
        if recent_samples.len() < 2 {
            return 0;
        }
        
        let first = recent_samples.first().unwrap();
        let last = recent_samples.last().unwrap();
        
        let bytes_diff = last.bytes_downloaded - first.bytes_downloaded;
        let time_diff = last.timestamp.duration_since(first.timestamp).as_secs_f64();
        
        if time_diff > 0.0 {
            (bytes_diff as f64 / time_diff) as u64
        } else {
            0
        }
    }
}
```

#### FN006-3: ETA（残り時間）計算
```rust
/// 現在の転送速度から残り時間を推定する
/// 
/// # 事前条件
/// - transfer_rate > 0
/// - remaining_bytes が正確である
/// 
/// # 事後条件
/// - 残り時間の推定値を秒単位で返す
/// - 転送速度が0の場合はNoneを返す
fn calculate_eta(&self, remaining_bytes: u64, transfer_rate: u64) -> Option<u64>
```

### UI更新制御

#### 更新頻度制限
```rust
pub struct ProgressTracker {
    last_ui_update: Instant,
    ui_update_interval: Duration,       // デフォルト: 100ms
}

impl ProgressTracker {
    pub fn should_update_ui(&self) -> bool {
        self.last_ui_update.elapsed() >= self.ui_update_interval
    }
    
    pub fn mark_ui_updated(&mut self) {
        self.last_ui_update = Instant::now();
    }
}
```

---

## FN007: エラー処理機能

### 機能概要
- **目的**: 統一されたエラー処理・報告・回復メカニズム
- **実装**: `lib.rs:Error handling`
- **対応要件**: NFR002-1（エラーハンドリング）

### エラー型定義

```rust
#[derive(Debug, thiserror::Error)]
pub enum ZoomVideoMoverError {
    #[error("Authentication failed: {message}")]
    AuthenticationError { message: String },
    
    #[error("Network error: {source}")]
    NetworkError { 
        #[from]
        source: reqwest::Error 
    },
    
    #[error("File system error: {message}")]
    FileSystemError { message: String },
    
    #[error("Configuration error: {field} is invalid")]
    ConfigurationError { field: String },
    
    #[error("Zoom API error: HTTP {status} - {message}")]
    ZoomApiError { 
        status: u16, 
        message: String 
    },
    
    #[error("Invalid date format: {date}")]
    InvalidDateFormat { date: String },
    
    #[error("Date range invalid: from {from} to {to}")]
    InvalidDateRange { from: String, to: String },
    
    #[error("Insufficient disk space: need {required} bytes, have {available}")]
    InsufficientDiskSpace { required: u64, available: u64 },
    
    #[error("Download cancelled by user")]
    DownloadCancelled,
    
    #[error("Timeout after {seconds} seconds")]
    TimeoutError { seconds: u64 },
}
```

### 処理仕様

#### FN007-1: エラー分類・変換
```rust
/// HTTPエラーレスポンスをアプリケーションエラーに変換する
/// 
/// # 事前条件
/// - response が有効なHTTPレスポンスである
/// 
/// # 事後条件
/// - 適切なZoomVideoMoverErrorが生成される
/// - エラーコード・メッセージが保持される
pub fn categorize_http_error(response: &reqwest::Response) -> ZoomVideoMoverError {
    match response.status().as_u16() {
        401 => ZoomVideoMoverError::AuthenticationError {
            message: "Invalid or expired access token".to_string(),
        },
        403 => ZoomVideoMoverError::AuthenticationError {
            message: "Insufficient permissions".to_string(),
        },
        404 => ZoomVideoMoverError::ZoomApiError {
            status: 404,
            message: "Resource not found".to_string(),
        },
        429 => ZoomVideoMoverError::ZoomApiError {
            status: 429,
            message: "Rate limit exceeded".to_string(),
        },
        500..=599 => ZoomVideoMoverError::ZoomApiError {
            status: response.status().as_u16(),
            message: "Server error".to_string(),
        },
        _ => ZoomVideoMoverError::NetworkError {
            source: reqwest::Error::from(response.error_for_status_ref().unwrap_err()),
        },
    }
}
```

#### FN007-2: リトライ戦略
```rust
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub backoff_factor: f64,
    pub retryable_errors: Vec<ErrorCategory>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorCategory {
    Network,
    ServerError,
    RateLimit,
    Timeout,
}

/// 指数バックオフでリトライを実行する
/// 
/// # 事前条件
/// - operation が有効なクロージャである
/// - retry_config が適切に設定されている
/// 
/// # 事後条件
/// - 成功時: 操作結果を返す
/// - 全試行失敗時: 最後のエラーを返す
pub async fn retry_with_backoff<F, T, E>(
    operation: F,
    retry_config: RetryConfig,
) -> Result<T, E>
where
    F: Fn() -> BoxFuture<'static, Result<T, E>>,
    E: std::error::Error + Send + Sync + 'static,
{
    let mut attempt = 0;
    let mut delay = retry_config.base_delay;
    
    loop {
        attempt += 1;
        
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                if attempt >= retry_config.max_attempts 
                    || !should_retry(&error, &retry_config) {
                    return Err(error);
                }
                
                tokio::time::sleep(delay).await;
                delay = std::cmp::min(
                    Duration::from_millis(
                        (delay.as_millis() as f64 * retry_config.backoff_factor) as u64
                    ),
                    retry_config.max_delay,
                );
            }
        }
    }
}
```

#### FN007-3: エラー報告・ログ出力
```rust
/// エラー情報を構造化ログとして出力する
/// 
/// # 副作用
/// - ログファイルへの書き込み
/// - コンソール出力（デバッグモード）
/// 
/// # 事前条件
/// - error が有効なエラーインスタンスである
/// - ログシステムが初期化されている
pub fn log_error(error: &ZoomVideoMoverError, context: Option<&str>) {
    let context_str = context.unwrap_or("unknown");
    
    match error {
        ZoomVideoMoverError::AuthenticationError { message } => {
            log::error!(
                target: "auth",
                "Authentication failed in {}: {}",
                context_str,
                message
            );
        }
        ZoomVideoMoverError::NetworkError { source } => {
            log::error!(
                target: "network",
                "Network error in {}: {}",
                context_str,
                source
            );
        }
        ZoomVideoMoverError::ZoomApiError { status, message } => {
            log::error!(
                target: "api",
                "Zoom API error in {} (HTTP {}): {}",
                context_str,
                status,
                message
            );
        }
        _ => {
            log::error!(
                target: "general",
                "Error in {}: {}",
                context_str,
                error
            );
        }
    }
}
```

### 回復戦略

| エラー分類 | 自動回復 | 回復戦略 | ユーザー操作 |
|------------|----------|----------|--------------|
| **認証エラー** | トークンリフレッシュ試行 | 再認証フロー | 手動認証 |
| **ネットワークエラー** | 3回リトライ | 指数バックオフ | 接続確認 |
| **レート制限** | 待機後リトライ | API制限遵守 | 自動待機 |
| **ファイルエラー** | 権限確認 | 代替パス提案 | 手動修正 |
| **設定エラー** | デフォルト値適用 | 設定検証 | 設定修正 |

---

## FN008: ファイル管理機能

### 機能概要
- **目的**: ファイル操作・パス処理・日本語対応
- **実装**: `lib.rs:FileManager`
- **対応要件**: NFR004-2（日本語ファイル名対応）

### 処理仕様

#### FN008-1: ファイル名サニタイズ
```rust
/// Windows対応のファイル名サニタイズ
/// 
/// # 事前条件
/// - filename が空でない
/// 
/// # 事後条件
/// - Windows で有効なファイル名を返す
/// - 日本語文字が保持される
/// - 長さ制限内に収まる
pub fn sanitize_filename_windows(filename: &str) -> String {
    // 無効文字の置換
    let invalid_chars = ['<', '>', ':', '"', '|', '?', '*', '/', '\\'];
    let mut sanitized = filename.to_string();
    
    for ch in invalid_chars {
        sanitized = sanitized.replace(ch, "_");
    }
    
    // 制御文字の除去
    sanitized = sanitized.chars()
        .filter(|c| !c.is_control())
        .collect();
    
    // 予約語の回避
    let reserved_names = [
        "CON", "PRN", "AUX", "NUL", 
        "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
        "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9"
    ];
    
    let name_upper = sanitized.to_uppercase();
    if reserved_names.contains(&name_upper.as_str()) {
        sanitized = format!("_{}", sanitized);
    }
    
    // 長さ制限（Windowsファイルシステム制限: 255文字）
    if sanitized.len() > 255 {
        let extension_start = sanitized.rfind('.').unwrap_or(sanitized.len());
        let extension = if extension_start < sanitized.len() {
            &sanitized[extension_start..]
        } else {
            ""
        };
        
        let max_base_len = 255 - extension.len();
        sanitized = format!("{}{}", &sanitized[..max_base_len], extension);
    }
    
    // 末尾の空白・ピリオド除去
    sanitized = sanitized.trim_end_matches([' ', '.']).to_string();
    
    // 空文字の場合のフォールバック
    if sanitized.is_empty() {
        sanitized = "unnamed_file".to_string();
    }
    
    sanitized
}
```

#### FN008-2: ディレクトリ作成
```rust
/// 必要に応じてディレクトリを再帰的に作成する
/// 
/// # 副作用
/// - ファイルシステムへのディレクトリ作成
/// 
/// # 事前条件
/// - path が有効なパスである
/// - 親ディレクトリへの書き込み権限がある
/// 
/// # 事後条件
/// - 成功時: 指定パスにディレクトリが存在する
/// - 失敗時: 適切なエラーを返す
pub fn ensure_directory_exists(path: &Path) -> Result<(), ZoomVideoMoverError> {
    if !path.exists() {
        std::fs::create_dir_all(path)
            .map_err(|e| ZoomVideoMoverError::FileSystemError {
                message: format!("Failed to create directory {}: {}", path.display(), e)
            })?;
    } else if !path.is_dir() {
        return Err(ZoomVideoMoverError::FileSystemError {
            message: format!("Path {} exists but is not a directory", path.display())
        });
    }
    
    Ok(())
}
```

#### FN008-3: 一意ファイル名生成
```rust
/// 既存ファイルと重複しない一意なファイル名を生成する
/// 
/// # 事前条件
/// - base_path が有効なパスである
/// - filename が有効なファイル名である
/// 
/// # 事後条件
/// - 既存ファイルと重複しないパスを返す
/// - ファイル名に連番が付加される場合がある
pub fn generate_unique_filename(base_path: &Path, filename: &str) -> PathBuf {
    let sanitized = sanitize_filename_windows(filename);
    let mut candidate = base_path.join(&sanitized);
    
    if !candidate.exists() {
        return candidate;
    }
    
    // 拡張子の分離
    let stem = Path::new(&sanitized).file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file");
    let extension = Path::new(&sanitized).extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    
    let extension_with_dot = if extension.is_empty() {
        String::new()
    } else {
        format!(".{}", extension)
    };
    
    // 連番付きファイル名の生成
    for i in 1..=9999 {
        let numbered_filename = format!("{}_{:04}{}", stem, i, extension_with_dot);
        candidate = base_path.join(numbered_filename);
        
        if !candidate.exists() {
            return candidate;
        }
    }
    
    // フォールバック: タイムスタンプ付き
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let timestamped_filename = format!("{}_{}{}", stem, timestamp, extension_with_dot);
    base_path.join(timestamped_filename)
}
```

---

## FN009: ログ出力機能

### 機能概要
- **目的**: 構造化ログ出力・デバッグ情報管理
- **実装**: `lib.rs:Logger`
- **対応要件**: NFR002-2（ログ出力）

### ログレベル・分類

```rust
use log::{debug, info, warn, error};

// ログターゲット分類
const LOG_TARGET_AUTH: &str = "zoom_video_mover::auth";
const LOG_TARGET_API: &str = "zoom_video_mover::api";
const LOG_TARGET_DOWNLOAD: &str = "zoom_video_mover::download";
const LOG_TARGET_CONFIG: &str = "zoom_video_mover::config";
const LOG_TARGET_GUI: &str = "zoom_video_mover::gui";
```

### 処理仕様

#### FN009-1: ログシステム初期化
```rust
/// ログシステムを初期化する
/// 
/// # 副作用
/// - ログファイルの作成
/// - コンソール出力の設定
/// 
/// # 事前条件
/// - ログディレクトリへの書き込み権限がある
/// 
/// # 事後条件
/// - ログシステムが利用可能になる
/// - 適切なフィルタレベルが設定される
pub fn init_logger() -> Result<(), ZoomVideoMoverError> {
    let log_level = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "info".to_string());
    
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or(log_level)
    )
    .format_timestamp_secs()
    .format_module_path(false)
    .format_target(true)
    .init();
    
    info!(target: LOG_TARGET_CONFIG, "Logger initialized");
    Ok(())
}
```

#### FN009-2: 構造化ログ出力
```rust
/// API呼び出しログを出力する
pub fn log_api_call(method: &str, url: &str, status: Option<u16>, duration: Duration) {
    match status {
        Some(code) if code >= 200 && code < 300 => {
            info!(
                target: LOG_TARGET_API,
                "{} {} -> {} ({:.2}s)",
                method, url, code, duration.as_secs_f64()
            );
        }
        Some(code) => {
            warn!(
                target: LOG_TARGET_API,
                "{} {} -> {} ({:.2}s)",
                method, url, code, duration.as_secs_f64()
            );
        }
        None => {
            error!(
                target: LOG_TARGET_API,
                "{} {} -> ERROR ({:.2}s)",
                method, url, duration.as_secs_f64()
            );
        }
    }
}

/// ダウンロード進捗ログを出力する
pub fn log_download_progress(file_name: &str, progress: f32, rate: u64) {
    debug!(
        target: LOG_TARGET_DOWNLOAD,
        "Downloading {} - {:.1}% ({} MB/s)",
        file_name,
        progress * 100.0,
        rate / 1_000_000
    );
}
```

---

## FN010: Windows対応機能

### 機能概要
- **目的**: Windows環境での文字エンコーディング・コンソール処理
- **実装**: `windows_console.rs`
- **対応要件**: NFR004-1（Windows日本語対応）

### 処理仕様

#### FN010-1: コンソールエンコーディング設定
```rust
/// Windows コンソールのUTF-8エンコーディングを設定する
/// 
/// # 副作用
/// - Windows APIの呼び出し
/// - コンソール設定の変更
/// 
/// # 事前条件
/// - Windows環境で実行されている
/// 
/// # 事後条件
/// - コンソール出力がUTF-8で処理される
/// - 日本語文字が正常に表示される
#[cfg(target_os = "windows")]
pub fn setup_console_encoding() -> Result<(), Box<dyn std::error::Error>> {
    use windows::Win32::System::Console::SetConsoleOutputCP;
    
    // UTF-8 コードページ (65001) を設定
    unsafe {
        SetConsoleOutputCP(65001)?;
    }
    
    println!("Console encoding set to UTF-8");
    Ok(())
}
```

#### FN010-2: 日本語パス処理
```rust
/// 日本語を含むWindowsパスの正規化
/// 
/// # 事前条件
/// - path が有効なパス文字列である
/// 
/// # 事後条件
/// - Windows APIで処理可能なパス形式を返す
/// - 日本語文字が保持される
#[cfg(target_os = "windows")]
pub fn normalize_japanese_path(path: &str) -> Result<PathBuf, ZoomVideoMoverError> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    
    // UTF-16変換による正規化
    let wide: Vec<u16> = path.encode_utf16().collect();
    let os_string = OsString::from_wide(&wide);
    let path_buf = PathBuf::from(os_string);
    
    // パス存在確認
    if let Some(parent) = path_buf.parent() {
        if !parent.exists() {
            return Err(ZoomVideoMoverError::FileSystemError {
                message: format!("Parent directory does not exist: {}", parent.display())
            });
        }
    }
    
    Ok(path_buf)
}
```

## 機能品質要件

### パフォーマンス要件

| 機能 | 性能指標 | 目標値 | 測定方法 |
|------|----------|--------|----------|
| **設定ファイル読み込み** | 応答時間 | 100ms以内 | 単体テスト |
| **OAuth認証** | 応答時間 | 5秒以内 | 統合テスト |
| **録画リスト取得** | 応答時間 | 3秒以内（30件） | APIテスト |
| **ダウンロード速度** | 転送速度 | 回線速度の80%以上 | 実測テスト |
| **進捗更新** | UI応答性 | 100ms以内 | UIテスト |

### 信頼性要件

| 機能 | 信頼性指標 | 目標値 | 検証方法 |
|------|------------|--------|----------|
| **エラー回復** | 自動回復率 | 80%以上 | 障害注入テスト |
| **データ整合性** | 破損率 | 0.01%以下 | ダウンロード検証 |
| **メモリリーク** | メモリ使用量 | 安定状態維持 | 長時間実行テスト |
| **並列処理** | デッドロック | 0件 | 並行テスト |

### セキュリティ要件

| 機能 | セキュリティ指標 | 実装方針 |
|------|-----------------|----------|
| **認証情報保護** | 暗号化保存 | OS標準機能使用 |
| **通信暗号化** | HTTPS強制 | TLS 1.2以上 |
| **ログ出力** | 機密情報除外 | トークン・パスワードマスク |
| **ファイル権限** | 適切な権限設定 | 最小権限原則 |