# API仕様書・インターフェース定義書 - Zoom Video Mover

## 文書概要
**プロジェクト名**: Zoom Video Mover  
**作成日**: 2025-08-02  
  
**バージョン**: 1.0  

## 外部API連携仕様

### Zoom Cloud Recording API連携

#### API概要
- **ベースURL**: `https://api.zoom.us/v2`
- **認証方式**: OAuth 2.0 (Authorization Code Flow)
- **レート制限**: 10 requests/second
- **データ形式**: JSON
- **文字エンコーディング**: UTF-8

#### 認証フロー仕様

##### OAuth 2.0 設定
```json
{
  "auth_endpoint": "https://zoom.us/oauth/authorize",
  "token_endpoint": "https://api.zoom.us/oauth/token",
  "client_id": "CLIENT_ID",
  "client_secret": "CLIENT_SECRET",
  "redirect_uri": "http://localhost:8080/oauth/callback",
  "scope": "recording:read user:read meeting:read",
  "response_type": "code",
  "state": "RANDOM_STATE_STRING"
}
```

##### 認証URL生成
```http
GET /oauth/authorize HTTP/1.1
Host: zoom.us

Parameters:
  response_type=code
  client_id={CLIENT_ID}
  redirect_uri={REDIRECT_URI}
  scope=recording:read user:read meeting:read
  state={STATE}
```

##### アクセストークン取得
```http
POST /oauth/token HTTP/1.1
Host: api.zoom.us
Content-Type: application/x-www-form-urlencoded

grant_type=authorization_code
&code={AUTHORIZATION_CODE}
&redirect_uri={REDIRECT_URI}
&client_id={CLIENT_ID}
&client_secret={CLIENT_SECRET}
```

**レスポンス**:
```json
{
  "access_token": "ACCESS_TOKEN",
  "token_type": "bearer",
  "refresh_token": "REFRESH_TOKEN",
  "expires_in": 3600,
  "scope": "recording:read user:read meeting:read"
}
```

#### 録画一覧取得API

##### エンドポイント: `/users/{userId}/recordings`

**リクエスト**:
```http
GET /v2/users/me/recordings HTTP/1.1
Host: api.zoom.us
Authorization: Bearer {ACCESS_TOKEN}

Parameters:
  from={YYYY-MM-DD}           # 必須: 検索開始日
  to={YYYY-MM-DD}             # 必須: 検索終了日
  page_size={NUMBER}          # オプション: ページサイズ(1-300, デフォルト30)
  next_page_token={TOKEN}     # オプション: 次ページトークン
  mc={BOOLEAN}                # オプション: 音声のみ会議を含む
  trash={BOOLEAN}             # オプション: ゴミ箱の録画を含む
```

**レスポンス**:
```json
{
  "from": "2025-07-01",
  "to": "2025-08-01",
  "page_count": 3,
  "page_size": 30,
  "total_records": 85,
  "next_page_token": "NEXT_PAGE_TOKEN",
  "meetings": [
    {
      "uuid": "MEETING_UUID",
      "id": 123456789,
      "account_id": "ACCOUNT_ID",
      "host_id": "HOST_ID",
      "topic": "週次進捗会議",
      "type": 2,
      "start_time": "2025-08-01T09:00:00Z",
      "timezone": "Asia/Tokyo",
      "duration": 60,
      "total_size": 1288490188,
      "recording_count": 3,
      "share_url": "https://zoom.us/rec/share/...",
      "recording_files": [
        {
          "id": "FILE_ID_1",
          "meeting_id": "123456789",
          "recording_start": "2025-08-01T09:00:05Z",
          "recording_end": "2025-08-01T10:00:02Z",
          "file_type": "MP4",
          "file_extension": "MP4",
          "file_size": 1258291027,
          "play_url": "https://zoom.us/rec/play/...",
          "download_url": "https://zoom.us/rec/download/...",
          "status": "completed",
          "deleted_time": null,
          "recording_type": "shared_screen_with_speaker_view"
        },
        {
          "id": "FILE_ID_2", 
          "meeting_id": "123456789",
          "recording_start": "2025-08-01T09:00:05Z",
          "recording_end": "2025-08-01T10:00:02Z",
          "file_type": "M4A",
          "file_extension": "M4A", 
          "file_size": 89088123,
          "play_url": "https://zoom.us/rec/play/...",
          "download_url": "https://zoom.us/rec/download/...",
          "status": "completed",
          "deleted_time": null,
          "recording_type": "audio_only"
        },
        {
          "id": "FILE_ID_3",
          "meeting_id": "123456789", 
          "recording_start": "2025-08-01T09:00:05Z",
          "recording_end": "2025-08-01T10:00:02Z",
          "file_type": "CHAT",
          "file_extension": "TXT",
          "file_size": 2048,
          "play_url": "",
          "download_url": "https://zoom.us/rec/download/...",
          "status": "completed",
          "deleted_time": null,
          "recording_type": "chat_file"
        }
      ]
    }
  ]
}
```

#### ファイル種別・録画タイプ定義

##### ファイル種別 (file_type)
| 種別 | 説明 | 拡張子 | 典型的サイズ |
|------|------|--------|-------------|
| **MP4** | 動画ファイル | .mp4 | 100MB-5GB |
| **M4A** | 音声ファイル | .m4a | 10MB-500MB |
| **CHAT** | チャット履歴 | .txt | 1KB-100KB |
| **CC** | クローズドキャプション | .vtt | 5KB-500KB |
| **CSV** | 投票・Q&A結果 | .csv | 1KB-50KB |

##### 録画タイプ (recording_type)
| タイプ | 説明 | 主な用途 |
|--------|------|----------|
| **shared_screen_with_speaker_view** | 画面共有+発話者表示 | プレゼンテーション |
| **shared_screen_with_gallery_view** | 画面共有+ギャラリー表示 | 大人数会議 |
| **speaker_view** | 発話者中心表示 | 講演・研修 |
| **gallery_view** | 全参加者表示 | ディスカッション |
| **audio_only** | 音声のみ | 音声会議 |
| **chat_file** | チャットファイル | テキスト記録 |

#### ファイルダウンロードAPI

##### ダウンロードURL取得
```http
GET {download_url} HTTP/1.1
Authorization: Bearer {ACCESS_TOKEN}
Range: bytes=0-1048575  # オプション: 範囲指定
```

**レスポンス**:
```http
HTTP/1.1 200 OK
Content-Type: video/mp4
Content-Length: 1258291027
Content-Range: bytes 0-1048575/1258291027
Accept-Ranges: bytes

[バイナリデータ]
```

#### AI要約取得API（実験的）

##### エンドポイント: `/meetings/{meetingId}/summary`

**リクエスト**:
```http
GET /v2/meetings/{meetingId}/summary HTTP/1.1
Host: api.zoom.us
Authorization: Bearer {ACCESS_TOKEN}
```

**レスポンス**:
```json
{
  "meeting_id": "123456789",
  "meeting_uuid": "MEETING_UUID",
  "summary": {
    "overview": "週次進捗会議の要約です。プロジェクトの進捗状況と課題について議論しました。",
    "key_points": [
      "新機能の開発が予定より2日遅れている",
      "バグ修正のため追加テストが必要",
      "次回リリース日程を1週間延期することで合意"
    ],
    "action_items": [
      {
        "description": "バグ修正とテスト完了",
        "assignee": "開発チーム",
        "due_date": "2025-08-05"
      },
      {
        "description": "クライアントへのスケジュール変更連絡",
        "assignee": "プロジェクトマネージャー",
        "due_date": "2025-08-02"
      }
    ],
    "participants_summary": [
      {
        "name": "田中太郎",
        "join_time": "2025-08-01T09:00:00Z",
        "leave_time": "2025-08-01T10:00:00Z",
        "speaking_time": "15分30秒"
      }
    ],
    "created_at": "2025-08-01T10:05:00Z"
  }
}
```

## エラーハンドリング仕様

### HTTPステータスコード対応

#### 2xx: 成功レスポンス
| コード | 説明 | 対応 |
|--------|------|------|
| **200 OK** | 正常処理完了 | データ取得・表示 |
| **206 Partial Content** | 部分コンテンツ取得 | 範囲指定ダウンロード継続 |

#### 4xx: クライアントエラー  
| コード | 説明 | 自動対応 | ユーザー対応 |
|--------|------|----------|-------------|
| **400 Bad Request** | リクエスト形式エラー | ログ記録 | エラー詳細表示 |
| **401 Unauthorized** | 認証エラー | トークン更新試行 | 再認証促進 |
| **403 Forbidden** | アクセス権限なし | - | 権限確認促進 |
| **404 Not Found** | リソース不存在 | - | ファイル削除済み通知 |
| **429 Too Many Requests** | レート制限超過 | 自動待機・リトライ | 待機状況表示 |

#### 5xx: サーバーエラー
| コード | 説明 | 自動対応 | ユーザー対応 |
|--------|------|----------|-------------|
| **500 Internal Server Error** | サーバー内部エラー | リトライ(最大3回) | エラー報告促進 |
| **502 Bad Gateway** | ゲートウェイエラー | リトライ(最大3回) | 一時的エラー通知 |
| **503 Service Unavailable** | サービス利用不可 | 指数バックオフリトライ | メンテナンス情報表示 |

### エラーレスポンス形式

#### 標準エラーレスポンス
```json
{
  "code": 124,
  "message": "Invalid access token."
}
```

#### 詳細エラーレスポンス（4xx系）
```json
{
  "code": 400,
  "message": "Bad Request",
  "errors": [
    {
      "field": "from",
      "message": "Invalid date format. Expected YYYY-MM-DD."
    },
    {
      "field": "to", 
      "message": "End date must be after start date."
    }
  ]
}
```

### リトライ戦略

#### 指数バックオフアルゴリズム
```rust
fn calculate_retry_delay(attempt: u32, base_delay_ms: u64) -> Duration {
    let delay_ms = base_delay_ms * 2_u64.pow(attempt);
    let jitter = rand::random::<f64>() * 0.1; // 10%のジッター
    Duration::from_millis((delay_ms as f64 * (1.0 + jitter)) as u64)
}

// 使用例
// 1回目: 1秒 + ジッター
// 2回目: 2秒 + ジッター  
// 3回目: 4秒 + ジッター
```

#### リトライ条件
| エラー種別 | リトライ | 最大回数 | 間隔 |
|------------|----------|----------|------|
| **ネットワークタイムアウト** | Yes | 3回 | 指数バックオフ |
| **レート制限 (429)** | Yes | 無制限 | Retry-After ヘッダー準拠 |
| **サーバーエラー (5xx)** | Yes | 3回 | 指数バックオフ |
| **認証エラー (401)** | No | - | トークン更新後再実行 |
| **権限エラー (403)** | No | - | ユーザー対応必要 |
| **クライアントエラー (400,404)** | No | - | ユーザー対応必要 |

## 内部API設計

### アプリケーション内部モジュール間API

#### 認証モジュールAPI

##### OAuthManager Interface
```rust
pub trait OAuthManager {
    async fn authenticate(&self, config: &OAuthConfig) -> Result<AccessToken, AuthError>;
    async fn refresh_token(&self, refresh_token: &str) -> Result<AccessToken, AuthError>;
    fn is_token_valid(&self, token: &AccessToken) -> bool;
    async fn revoke_token(&self, token: &AccessToken) -> Result<(), AuthError>;
}

pub struct ZoomOAuthManager {
    client: reqwest::Client,
    config: OAuthConfig,
}

impl OAuthManager for ZoomOAuthManager {
    async fn authenticate(&self, config: &OAuthConfig) -> Result<AccessToken, AuthError> {
        // OAuth認証フロー実装
        let auth_url = self.build_auth_url(config)?;
        let auth_code = self.start_oauth_flow(&auth_url).await?;
        let token = self.exchange_code_for_token(&auth_code).await?;
        Ok(token)
    }
    
    async fn refresh_token(&self, refresh_token: &str) -> Result<AccessToken, AuthError> {
        // リフレッシュトークンでアクセストークン更新
        let request = TokenRefreshRequest {
            grant_type: "refresh_token".to_string(),
            refresh_token: refresh_token.to_string(),
            client_id: self.config.client_id.clone(),
            client_secret: self.config.client_secret.clone(),
        };
        
        let response = self.client
            .post("https://api.zoom.us/oauth/token")
            .form(&request)
            .send()
            .await?;
            
        let token_response: TokenResponse = response.json().await?;
        Ok(AccessToken::from(token_response))
    }
}
```

#### 録画管理モジュールAPI

##### RecordingManager Interface
```rust
pub trait RecordingManager {
    async fn get_recordings(&self, filter: &RecordingFilter) -> Result<Vec<Meeting>, ApiError>;
    async fn get_recording_files(&self, meeting_id: &str) -> Result<Vec<RecordingFile>, ApiError>;
    async fn download_file(&self, file: &RecordingFile, output_path: &Path) -> Result<LocalFile, DownloadError>;
}

pub struct ZoomRecordingManager {
    client: reqwest::Client,
    auth_manager: Arc<dyn OAuthManager>,
}

#[derive(Debug, Clone)]
pub struct RecordingFilter {
    pub from_date: chrono::Date<chrono::Utc>,
    pub to_date: chrono::Date<chrono::Utc>,
    pub file_types: Vec<FileType>,
    pub meeting_name_pattern: Option<String>,
    pub page_size: Option<u32>,
}

impl RecordingManager for ZoomRecordingManager {
    async fn get_recordings(&self, filter: &RecordingFilter) -> Result<Vec<Meeting>, ApiError> {
        let token = self.auth_manager.get_valid_token().await?;
        
        let mut all_meetings = Vec::new();
        let mut next_page_token: Option<String> = None;
        
        loop {
            let url = format!("https://api.zoom.us/v2/users/me/recordings");
            let mut request = self.client
                .get(&url)
                .bearer_auth(&token.access_token)
                .query(&[
                    ("from", filter.from_date.format("%Y-%m-%d").to_string()),
                    ("to", filter.to_date.format("%Y-%m-%d").to_string()),
                ]);
                
            if let Some(page_token) = &next_page_token {
                request = request.query(&[("next_page_token", page_token)]);
            }
            
            let response = request.send().await?;
            let recordings_response: RecordingsResponse = response.json().await?;
            
            all_meetings.extend(recordings_response.meetings);
            
            if recordings_response.next_page_token.is_none() {
                break;
            }
            next_page_token = recordings_response.next_page_token;
        }
        
        Ok(all_meetings)
    }
}
```

#### ダウンロードエンジンAPI

##### DownloadEngine Interface
```rust
pub trait DownloadEngine {
    async fn start_download_session(&self, files: Vec<RecordingFile>) -> Result<DownloadSession, DownloadError>;
    async fn pause_session(&self, session_id: &str) -> Result<(), DownloadError>;
    async fn resume_session(&self, session_id: &str) -> Result<(), DownloadError>;
    async fn cancel_session(&self, session_id: &str) -> Result<(), DownloadError>;
    fn get_session_progress(&self, session_id: &str) -> Option<DownloadProgress>;
}

pub struct ParallelDownloadEngine {
    max_concurrent: usize,
    client: reqwest::Client,
    recording_manager: Arc<dyn RecordingManager>,
    progress_reporter: Arc<dyn ProgressReporter>,
}

#[derive(Debug, Clone)]
pub struct DownloadSession {
    pub id: String,
    pub files: Vec<RecordingFile>,
    pub status: SessionStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub output_directory: PathBuf,
}

#[derive(Debug, Clone)]
pub struct DownloadProgress {
    pub total_files: usize,
    pub completed_files: usize,
    pub failed_files: usize,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub transfer_rate: f64, // bytes/sec
    pub estimated_time_remaining: Option<Duration>,
}

impl DownloadEngine for ParallelDownloadEngine {
    async fn start_download_session(&self, files: Vec<RecordingFile>) -> Result<DownloadSession, DownloadError> {
        let session = DownloadSession {
            id: uuid::Uuid::new_v4().to_string(),
            files: files.clone(),
            status: SessionStatus::Running,
            created_at: chrono::Utc::now(),
            output_directory: self.get_output_directory(),
        };
        
        // 並列ダウンロードタスク開始
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent));
        let tasks: Vec<_> = files.into_iter().map(|file| {
            let semaphore = semaphore.clone();
            let engine = self.clone();
            let session_id = session.id.clone();
            
            tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                engine.download_single_file(&session_id, file).await
            })
        }).collect();
        
        // タスク完了を監視
        tokio::spawn(async move {
            for task in tasks {
                if let Err(e) = task.await {
                    error!("Download task failed: {}", e);
                }
            }
        });
        
        Ok(session)
    }
}
```

## 通信プロトコル・セキュリティ仕様

### HTTPS通信設定

#### TLS設定
```rust
use reqwest::ClientBuilder;
use reqwest::tls;

fn create_secure_client() -> Result<reqwest::Client, reqwest::Error> {
    ClientBuilder::new()
        .min_tls_version(tls::Version::TLS_1_2)  // TLS 1.2以上を強制
        .https_only(true)                        // HTTPS通信を強制
        .timeout(Duration::from_secs(30))        // タイムアウト設定
        .connect_timeout(Duration::from_secs(10)) // 接続タイムアウト
        .user_agent("ZoomVideoMover/1.0")        // User-Agent設定
        .build()
}
```

#### 証明書検証
- **標準CA証明書**: システム標準の証明書ストアを使用
- **証明書ピンニング**: Zoom.us 証明書の事前検証（オプション）
- **証明書エラー**: 検証失敗時の接続拒否

### 認証情報保護

#### トークン暗号化保存
```rust
use ring::aead;
use ring::rand::{SecureRandom, SystemRandom};

struct SecureTokenStorage {
    key: aead::LessSafeKey,
    nonce_sequence: u64,
}

impl SecureTokenStorage {
    fn encrypt_token(&mut self, token: &AccessToken) -> Result<Vec<u8>, StorageError> {
        let nonce = self.generate_nonce()?;
        let plaintext = serde_json::to_vec(token)?;
        
        let mut ciphertext = Vec::new();
        ciphertext.extend_from_slice(&nonce.as_ref());
        ciphertext.extend_from_slice(&plaintext);
        
        self.key.seal_in_place_append_tag(nonce, aead::Aad::empty(), &mut ciphertext[12..])?;
        Ok(ciphertext)
    }
    
    fn decrypt_token(&self, encrypted_data: &[u8]) -> Result<AccessToken, StorageError> {
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = aead::Nonce::try_assume_unique_for_key(nonce_bytes)?;
        
        let mut decrypted = ciphertext.to_vec();
        let plaintext = self.key.open_in_place(nonce, aead::Aad::empty(), &mut decrypted)?;
        
        let token: AccessToken = serde_json::from_slice(plaintext)?;
        Ok(token)
    }
}
```

#### プロキシ対応
```rust
fn configure_proxy_client(proxy_url: Option<&str>) -> reqwest::Client {
    let mut builder = ClientBuilder::new();
    
    if let Some(proxy) = proxy_url {
        builder = builder.proxy(reqwest::Proxy::all(proxy).unwrap());
    }
    
    // システムプロキシ設定の自動検出
    builder = builder.proxy(reqwest::Proxy::system());
    
    builder.build().unwrap()
}
```

## API使用量制限・監視

### レート制限対応

#### レート制限監視
```rust
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::time::{Duration, Instant};

pub struct RateLimiter {
    requests_per_second: u32,
    last_request_time: AtomicU64,
    request_count: AtomicU64,
}

impl RateLimiter {
    pub async fn acquire_permit(&self) -> Result<(), RateLimitError> {
        let now = Instant::now().elapsed().as_millis() as u64;
        let last_time = self.last_request_time.load(Ordering::Relaxed);
        
        // 1秒経過したらカウンタリセット
        if now - last_time >= 1000 {
            self.request_count.store(0, Ordering::Relaxed);
            self.last_request_time.store(now, Ordering::Relaxed);
        }
        
        let current_count = self.request_count.fetch_add(1, Ordering::Relaxed);
        
        if current_count >= self.requests_per_second as u64 {
            let wait_time = 1000 - (now - last_time);
            tokio::time::sleep(Duration::from_millis(wait_time)).await;
        }
        
        Ok(())
    }
}
```

#### Retry-After ヘッダー対応
```rust
async fn handle_rate_limit_response(response: &reqwest::Response) -> Result<Duration, ApiError> {
    if response.status() == 429 {
        if let Some(retry_after) = response.headers().get("Retry-After") {
            let retry_seconds: u64 = retry_after.to_str()?.parse()?;
            return Ok(Duration::from_secs(retry_seconds));
        }
    }
    Ok(Duration::from_secs(1)) // デフォルト待機時間
}
```

### API使用量監視

#### 使用量追跡
```rust
#[derive(Debug, Clone)]
pub struct ApiUsageMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub rate_limited_requests: u64,
    pub total_bytes_downloaded: u64,
    pub session_start_time: chrono::DateTime<chrono::Utc>,
}

impl ApiUsageMetrics {
    pub fn record_request(&mut self, success: bool, rate_limited: bool, bytes: u64) {
        self.total_requests += 1;
        
        if success {
            self.successful_requests += 1;
            self.total_bytes_downloaded += bytes;
        } else {
            self.failed_requests += 1;
        }
        
        if rate_limited {
            self.rate_limited_requests += 1;
        }
    }
    
    pub fn get_success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        self.successful_requests as f64 / self.total_requests as f64
    }
    
    pub fn get_average_transfer_rate(&self) -> f64 {
        let elapsed = chrono::Utc::now().signed_duration_since(self.session_start_time);
        let elapsed_seconds = elapsed.num_seconds() as f64;
        
        if elapsed_seconds > 0.0 {
            self.total_bytes_downloaded as f64 / elapsed_seconds
        } else {
            0.0
        }
    }
}
```

---

**承認**:  
**品質基準適合**: [ ] 確認済  
**ポリシー準拠**: [ ] 確認済  
**承認日**: ___________