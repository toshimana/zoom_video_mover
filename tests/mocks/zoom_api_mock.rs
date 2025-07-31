// tests/mocks/zoom_api_mock.rs
// Zoom API用の統合Mockオブジェクト実装

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use mockall::*;
use serde_json::{json, Value};
use chrono::{DateTime, Utc};
use wiremock::{Mock, MockServer, ResponseTemplate, Request};
use wiremock::matchers::{method, path, query_param, header, body_string_contains};

/// Zoom OAuth API Mock
pub struct ZoomOAuthMock {
    mock_server: MockServer,
    auth_codes: Arc<Mutex<HashMap<String, String>>>,  // auth_code -> client_id
    access_tokens: Arc<Mutex<HashMap<String, TokenInfo>>>,  // access_token -> info
}

/// トークン情報
#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub client_id: String,
    pub client_secret: String,
    pub expires_at: DateTime<Utc>,
    pub refresh_token: Option<String>,
    pub scopes: Vec<String>,
}

impl ZoomOAuthMock {
    /// OAuth Mock サーバーを起動
    pub async fn start() -> Self {
        let mock_server = MockServer::start().await;
        let auth_codes = Arc::new(Mutex::new(HashMap::new()));
        let access_tokens = Arc::new(Mutex::new(HashMap::new()));
        
        Self {
            mock_server,
            auth_codes,
            access_tokens,
        }
    }
    
    /// Mock サーバーのベースURL取得
    pub fn base_url(&self) -> String {
        self.mock_server.uri()
    }
    
    /// 有効な認証コードを事前登録
    pub fn register_auth_code(&self, auth_code: &str, client_id: &str) {
        self.auth_codes.lock().unwrap().insert(auth_code.to_string(), client_id.to_string());
    }
    
    /// 成功するトークン交換エンドポイントのMock設定
    pub async fn mock_token_exchange_success(&self, client_id: &str, client_secret: &str) {
        let auth_codes = self.auth_codes.clone();
        let access_tokens = self.access_tokens.clone();
        let client_id_owned = client_id.to_string();
        let client_secret_owned = client_secret.to_string();
        
        Mock::given(method("POST"))
            .and(path("/oauth/token"))
            .and(header("content-type", "application/x-www-form-urlencoded"))
            .and(body_string_contains("grant_type=authorization_code"))
            .respond_with_fn(move |req: &Request| {
                // リクエストボディの解析
                let body = std::str::from_utf8(&req.body).unwrap_or("");
                let auth_code = extract_form_param(body, "code");
                
                // Authorization ヘッダーの検証
                let auth_header = req.headers.get("Authorization")
                    .and_then(|h| h.to_str().ok())
                    .unwrap_or("");
                
                let expected_auth = format!("Basic {}", 
                    base64::encode(format!("{}:{}", client_id_owned, client_secret_owned)));
                
                // 認証コードの検証
                let auth_codes_guard = auth_codes.lock().unwrap();
                let registered_client = auth_codes_guard.get(&auth_code);
                
                if auth_header == expected_auth && 
                   registered_client == Some(&client_id_owned) {
                    // 成功レスポンス
                    let access_token = format!("mock_access_token_{}_{}", client_id_owned, 
                        chrono::Utc::now().timestamp());
                    let refresh_token = format!("mock_refresh_token_{}_{}", client_id_owned,
                        chrono::Utc::now().timestamp());
                    
                    // トークン情報を保存
                    let token_info = TokenInfo {
                        client_id: client_id_owned.clone(),
                        client_secret: client_secret_owned.clone(),
                        expires_at: Utc::now() + chrono::Duration::hours(1),
                        refresh_token: Some(refresh_token.clone()),
                        scopes: vec!["recording:read".to_string(), "user:read".to_string(), "meeting:read".to_string()],
                    };
                    
                    access_tokens.lock().unwrap().insert(access_token.clone(), token_info);
                    
                    ResponseTemplate::new(200)
                        .set_body_json(&json!({
                            "access_token": access_token,
                            "token_type": "Bearer",
                            "expires_in": 3600,
                            "refresh_token": refresh_token,
                            "scope": "recording:read user:read meeting:read"
                        }))
                } else {
                    // 認証失敗レスポンス
                    ResponseTemplate::new(400)
                        .set_body_json(&json!({
                            "error": "invalid_grant",
                            "error_description": "Invalid authorization code or client credentials"
                        }))
                }
            })
            .mount(&self.mock_server)
            .await;
    }
    
    /// トークンリフレッシュエンドポイントのMock設定
    pub async fn mock_token_refresh(&self) {
        let access_tokens = self.access_tokens.clone();
        
        Mock::given(method("POST"))
            .and(path("/oauth/token"))
            .and(body_string_contains("grant_type=refresh_token"))
            .respond_with_fn(move |req: &Request| {
                let body = std::str::from_utf8(&req.body).unwrap_or("");
                let refresh_token = extract_form_param(body, "refresh_token");
                
                // 既存のリフレッシュトークンを検索
                let tokens_guard = access_tokens.lock().unwrap();
                let existing_token = tokens_guard.iter()
                    .find(|(_, info)| info.refresh_token.as_ref() == Some(&refresh_token));
                
                if let Some((_, token_info)) = existing_token {
                    // 新しいアクセストークン生成
                    let new_access_token = format!("refreshed_access_token_{}_{}", 
                        token_info.client_id, chrono::Utc::now().timestamp());
                    let new_refresh_token = format!("refreshed_refresh_token_{}_{}", 
                        token_info.client_id, chrono::Utc::now().timestamp());
                    
                    ResponseTemplate::new(200)
                        .set_body_json(&json!({
                            "access_token": new_access_token,
                            "token_type": "Bearer",
                            "expires_in": 3600,
                            "refresh_token": new_refresh_token,
                            "scope": token_info.scopes.join(" ")
                        }))
                } else {
                    ResponseTemplate::new(400)
                        .set_body_json(&json!({
                            "error": "invalid_grant",
                            "error_description": "Invalid refresh token"
                        }))
                }
            })
            .mount(&self.mock_server)
            .await;
    }
    
    /// 認証エラーパターンのMock設定
    pub async fn mock_auth_error(&self, error_type: &str) {
        let (status_code, error_code, error_description) = match error_type {
            "invalid_client" => (401, "invalid_client", "Invalid client credentials"),
            "invalid_grant" => (400, "invalid_grant", "Invalid authorization code"),
            "expired_code" => (400, "invalid_grant", "Authorization code expired"),
            _ => (400, "invalid_request", "Bad request"),
        };
        
        Mock::given(method("POST"))
            .and(path("/oauth/token"))
            .respond_with(
                ResponseTemplate::new(status_code)
                    .set_body_json(&json!({
                        "error": error_code,
                        "error_description": error_description
                    }))
            )
            .mount(&self.mock_server)
            .await;
    }
}

/// Zoom Cloud Recording API Mock
pub struct ZoomRecordingApiMock {
    mock_server: MockServer,
    recordings: Arc<Mutex<Vec<MockRecording>>>,
    ai_summaries: Arc<Mutex<HashMap<String, Value>>>,  // meeting_id -> summary
}

/// Mock用録画データ
#[derive(Debug, Clone)]
pub struct MockRecording {
    pub meeting_id: String,
    pub uuid: String,
    pub topic: String,
    pub start_time: DateTime<Utc>,
    pub duration: u32,
    pub files: Vec<MockRecordingFile>,
}

#[derive(Debug, Clone)]
pub struct MockRecordingFile {
    pub id: String,
    pub file_type: String,
    pub file_size: u64,
    pub download_url: String,
    pub play_url: Option<String>,
}

impl ZoomRecordingApiMock {
    /// Recording API Mock サーバーを起動
    pub async fn start() -> Self {
        let mock_server = MockServer::start().await;
        let recordings = Arc::new(Mutex::new(Vec::new()));
        let ai_summaries = Arc::new(Mutex::new(HashMap::new()));
        
        Self {
            mock_server,
            recordings,
            ai_summaries,
        }
    }
    
    /// Mock サーバーのベースURL取得
    pub fn base_url(&self) -> String {
        self.mock_server.uri()
    }
    
    /// テスト用録画データを追加
    pub fn add_recording(&self, recording: MockRecording) {
        self.recordings.lock().unwrap().push(recording);
    }
    
    /// 複数の日本語録画データを一括追加
    pub fn add_sample_japanese_recordings(&self) {
        let recordings = vec![
            MockRecording {
                meeting_id: "123456789".to_string(),
                uuid: "meeting-uuid-123".to_string(),
                topic: "週次チーム会議".to_string(),
                start_time: Utc::now() - chrono::Duration::days(1),
                duration: 60,
                files: vec![
                    MockRecordingFile {
                        id: "video_123".to_string(),
                        file_type: "MP4".to_string(),
                        file_size: 1_073_741_824, // 1GB
                        download_url: format!("{}/rec/download/video_123", self.base_url()),
                        play_url: Some(format!("{}/rec/play/video_123", self.base_url())),
                    },
                    MockRecordingFile {
                        id: "audio_123".to_string(),
                        file_type: "MP3".to_string(),
                        file_size: 67_108_864, // 64MB
                        download_url: format!("{}/rec/download/audio_123", self.base_url()),
                        play_url: None,
                    },
                    MockRecordingFile {
                        id: "chat_123".to_string(),
                        file_type: "TXT".to_string(),
                        file_size: 2048, // 2KB
                        download_url: format!("{}/rec/download/chat_123", self.base_url()),
                        play_url: None,
                    },
                ],
            },
            MockRecording {
                meeting_id: "456789012".to_string(),
                uuid: "meeting-uuid-456".to_string(),
                topic: "プロジェクト進捗レビュー".to_string(),
                start_time: Utc::now() - chrono::Duration::days(3),
                duration: 90,
                files: vec![
                    MockRecordingFile {
                        id: "video_456".to_string(),
                        file_type: "MP4".to_string(),
                        file_size: 2_147_483_648, // 2GB
                        download_url: format!("{}/rec/download/video_456", self.base_url()),
                        play_url: Some(format!("{}/rec/play/video_456", self.base_url())),
                    },
                ],
            },
            MockRecording {
                meeting_id: "789012345".to_string(),
                uuid: "meeting-uuid-789".to_string(),
                topic: "クライアント要件確認ミーティング".to_string(),
                start_time: Utc::now() - chrono::Duration::days(7),
                duration: 120,
                files: vec![
                    MockRecordingFile {
                        id: "video_789".to_string(),
                        file_type: "MP4".to_string(),
                        file_size: 1_610_612_736, // 1.5GB
                        download_url: format!("{}/rec/download/video_789", self.base_url()),
                        play_url: Some(format!("{}/rec/play/video_789", self.base_url())),
                    },
                    MockRecordingFile {
                        id: "transcript_789".to_string(),
                        file_type: "VTT".to_string(),
                        file_size: 15360, // 15KB
                        download_url: format!("{}/rec/download/transcript_789", self.base_url()),
                        play_url: None,
                    },
                ],
            },
        ];
        
        let mut recordings_guard = self.recordings.lock().unwrap();
        recordings_guard.extend(recordings);
    }
    
    /// AI要約データを追加
    pub fn add_ai_summary(&self, meeting_id: &str, summary: Value) {
        self.ai_summaries.lock().unwrap().insert(meeting_id.to_string(), summary);
    }
    
    /// 録画リスト取得エンドポイントのMock設定
    pub async fn mock_recordings_list(&self, access_token: &str) {
        let recordings = self.recordings.clone();
        let expected_token = format!("Bearer {}", access_token);
        
        Mock::given(method("GET"))
            .and(path("/v2/users/me/recordings"))
            .and(header("Authorization", expected_token.as_str()))
            .respond_with_fn(move |req: &Request| {
                // クエリパラメータの取得
                let from_date = req.url.query_pairs()
                    .find(|(key, _)| key == "from")
                    .map(|(_, value)| value.to_string())
                    .unwrap_or_else(|| "2024-01-01".to_string());
                
                let to_date = req.url.query_pairs()
                    .find(|(key, _)| key == "to")
                    .map(|(_, value)| value.to_string())
                    .unwrap_or_else(|| "2024-12-31".to_string());
                
                // 日付範囲でフィルタリング
                let from_parsed = chrono::NaiveDate::parse_from_str(&from_date, "%Y-%m-%d")
                    .unwrap_or_else(|_| chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
                let to_parsed = chrono::NaiveDate::parse_from_str(&to_date, "%Y-%m-%d")
                    .unwrap_or_else(|_| chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap());
                
                let recordings_guard = recordings.lock().unwrap();
                let filtered_recordings: Vec<_> = recordings_guard.iter()
                    .filter(|r| {
                        let recording_date = r.start_time.date_naive();
                        recording_date >= from_parsed && recording_date <= to_parsed
                    })
                    .collect();
                
                // レスポンスJSON生成
                let meetings: Vec<Value> = filtered_recordings.iter()
                    .map(|recording| {
                        let recording_files: Vec<Value> = recording.files.iter()
                            .map(|file| json!({
                                "id": file.id,
                                "file_type": file.file_type,
                                "file_size": file.file_size,
                                "download_url": file.download_url,
                                "play_url": file.play_url,
                                "recording_start": recording.start_time.to_rfc3339(),
                                "recording_end": (recording.start_time + chrono::Duration::minutes(recording.duration as i64)).to_rfc3339()
                            }))
                            .collect();
                        
                        json!({
                            "uuid": recording.uuid,
                            "id": recording.meeting_id.parse::<u64>().unwrap_or(0),
                            "topic": recording.topic,
                            "start_time": recording.start_time.to_rfc3339(),
                            "duration": recording.duration,
                            "recording_files": recording_files
                        })
                    })
                    .collect();
                
                ResponseTemplate::new(200)
                    .set_body_json(&json!({
                        "from": from_date,
                        "to": to_date,
                        "page_count": 1,
                        "page_number": 1,
                        "page_size": meetings.len(),
                        "total_records": meetings.len(),
                        "meetings": meetings
                    }))
            })
            .mount(&self.mock_server)
            .await;
    }
    
    /// AI要約取得エンドポイントのMock設定
    pub async fn mock_ai_summary(&self, access_token: &str) {
        let ai_summaries = self.ai_summaries.clone();
        let expected_token = format!("Bearer {}", access_token);
        
        Mock::given(method("GET"))
            .and(path_regex(r"/v2/meetings/[0-9]+/ai_companion"))
            .and(header("Authorization", expected_token.as_str()))
            .respond_with_fn(move |req: &Request| {
                // URLから meeting_id を抽出
                let path_segments: Vec<&str> = req.url.path_segments()
                    .map(|c| c.collect())
                    .unwrap_or_default();
                
                let meeting_id = path_segments.get(2).unwrap_or(&"").to_string();
                
                let summaries_guard = ai_summaries.lock().unwrap();
                if let Some(summary) = summaries_guard.get(&meeting_id) {
                    ResponseTemplate::new(200)
                        .set_body_json(summary)
                } else {
                    ResponseTemplate::new(404)
                        .set_body_json(&json!({
                            "code": 3001,
                            "message": "Meeting does not exist or AI summary not available"
                        }))
                }
            })
            .mount(&self.mock_server)
            .await;
    }
    
    /// ファイルダウンロードエンドポイントのMock設定
    pub async fn mock_file_download(&self) {
        Mock::given(method("GET"))
            .and(path_regex(r"/rec/download/.*"))
            .respond_with_fn(|req: &Request| {
                let file_id = req.url.path_segments()
                    .and_then(|segments| segments.last())
                    .unwrap_or("unknown");
                
                // ファイルタイプによる異なるレスポンス
                let (content, content_type, size) = if file_id.contains("video") {
                    // 小さなビデオファイルのシミュレーション (10KB)
                    let content = "MOCK_VIDEO_DATA_".repeat(500); // 約10KB
                    (content, "video/mp4", 10240)
                } else if file_id.contains("audio") {
                    // 小さなオーディオファイルのシミュレーション (5KB)
                    let content = "MOCK_AUDIO_DATA_".repeat(250); // 約5KB
                    (content, "audio/mp3", 5120)
                } else if file_id.contains("chat") {
                    // チャットファイルのシミュレーション
                    let content = r#"
[10:00:15] Alice: おはようございます
[10:00:30] Bob: おはようございます。今日の議題を確認しましょう
[10:01:00] Charlie: 資料を共有画面で表示します
[10:15:30] Alice: 次回までのアクションアイテムを整理しました
"#.to_string();
                    (content, "text/plain", content.len())
                } else if file_id.contains("transcript") {
                    // トランスクリプトファイルのシミュレーション
                    let content = r#"WEBVTT

00:00:00.000 --> 00:00:05.000
皆さん、おはようございます。今日の会議を始めます。

00:00:05.000 --> 00:00:15.000
最初に前回のアクションアイテムの進捗を確認しましょう。

00:00:15.000 --> 00:00:25.000
プロジェクトの進捗状況について報告します。
"#.to_string();
                    (content, "text/vtt", content.len())
                } else {
                    // デフォルトファイル
                    let content = format!("Mock file content for {}", file_id);
                    (content, "application/octet-stream", content.len())
                };
                
                ResponseTemplate::new(200)
                    .set_body_string(content)
                    .insert_header("content-type", content_type)
                    .insert_header("content-length", size.to_string())
            })
            .mount(&self.mock_server)
            .await;
    }
    
    /// API エラーレスポンスのMock設定
    pub async fn mock_api_error(&self, status_code: u16, error_message: &str) {
        Mock::given(method("GET"))
            .and(path_regex(r"/v2/.*"))
            .respond_with(
                ResponseTemplate::new(status_code)
                    .set_body_json(&json!({
                        "code": status_code,
                        "message": error_message
                    }))
            )
            .mount(&self.mock_server)
            .await;
    }
}

/// 統合Mock管理 - OAuth + Recording API
pub struct ZoomApiMockSuite {
    pub oauth_mock: ZoomOAuthMock,
    pub recording_mock: ZoomRecordingApiMock,
}

impl ZoomApiMockSuite {
    /// 完全なMockスイートを起動
    pub async fn start() -> Self {
        let oauth_mock = ZoomOAuthMock::start().await;
        let recording_mock = ZoomRecordingApiMock::start().await;
        
        Self {
            oauth_mock,
            recording_mock,
        }
    }
    
    /// 成功シナリオの一括Mock設定
    pub async fn setup_success_scenario(&self, client_id: &str, client_secret: &str) {
        let access_token = "test_access_token_success";
        
        // 1. OAuth認証成功の設定
        self.oauth_mock.register_auth_code("success_auth_code", client_id);
        self.oauth_mock.mock_token_exchange_success(client_id, client_secret).await;
        self.oauth_mock.mock_token_refresh().await;
        
        // 2. 録画データの追加
        self.recording_mock.add_sample_japanese_recordings();
        
        // 3. AI要約データの追加
        self.recording_mock.add_ai_summary("123456789", json!({
            "meeting_id": "123456789",
            "ai_summary": {
                "summary": "この会議では、週次の進捗報告と来週のアクションアイテムの確認を行いました。",
                "key_points": [
                    "プロジェクトA: スケジュール通り進行中",
                    "プロジェクトB: リソース追加が必要",
                    "来週のマイルストーン確認"
                ],
                "action_items": [
                    {
                        "description": "リソース計画の見直し",
                        "assignee": "project.manager@company.com",
                        "due_date": "2024-01-20"
                    }
                ],
                "generated_at": "2024-01-15T11:05:00Z"
            }
        }));
        
        // 4. API エンドポイントのMock設定
        self.recording_mock.mock_recordings_list(access_token).await;
        self.recording_mock.mock_ai_summary(access_token).await;
        self.recording_mock.mock_file_download().await;
    }
    
    /// エラーシナリオの一括Mock設定
    pub async fn setup_error_scenarios(&self) {
        // 1. 認証エラー
        self.oauth_mock.mock_auth_error("invalid_client").await;
        
        // 2. API エラー
        self.recording_mock.mock_api_error(401, "Unauthorized - Invalid access token").await;
    }
    
    /// OAuth ベースURL取得
    pub fn oauth_base_url(&self) -> String {
        self.oauth_mock.base_url()
    }
    
    /// Recording API ベースURL取得
    pub fn api_base_url(&self) -> String {
        self.recording_mock.base_url()
    }
}

// ユーティリティ関数

/// URLエンコードされたフォームパラメータの抽出
fn extract_form_param(body: &str, param_name: &str) -> String {
    body.split('&')
        .find_map(|pair| {
            let mut parts = pair.split('=');
            if parts.next() == Some(param_name) {
                parts.next().map(|v| urlencoding::decode(v).unwrap_or_default().to_string())
            } else {
                None
            }
        })
        .unwrap_or_default()
}

/// Base64エンコード (簡易実装)
mod base64 {
    pub fn encode(input: String) -> String {
        // 実際の実装では base64 crateを使用
        format!("base64_encoded_{}", input.replace(":", "_"))
    }
}

// Mock トレーサビリティ
//
// 機能仕様 (function_specifications.md) との対応:
// ├─ FN002: OAuth認証機能         → ZoomOAuthMock
// ├─ FN003: 録画検索機能          → ZoomRecordingApiMock::mock_recordings_list
// ├─ FN004: ファイルダウンロード機能 → ZoomRecordingApiMock::mock_file_download
// ├─ FN005: AI要約取得機能        → ZoomRecordingApiMock::mock_ai_summary
// └─ FN007: エラー処理機能        → mock_auth_error, mock_api_error
//
// 操作仕様 (operation_specifications.md) との対応:
// ├─ OP003: OAuth認証実行         → ZoomOAuthMock (完全な認証フロー)
// ├─ OP004: 録画検索・一覧表示     → ZoomRecordingApiMock (日本語データ対応)
// ├─ OP006: ダウンロード実行       → mock_file_download (各種ファイル形式)
// └─ OP008: エラー処理・回復       → エラーレスポンスMock
//
// 画面仕様 (screen_specifications.md) との対応:
// ├─ SC003: 認証画面             → OAuth認証フローのMock
// ├─ SC004: 録画リスト画面        → 日本語録画データのMock
// ├─ SC005: ダウンロード進捗画面   → ファイルダウンロードのMock
// └─ SC006: エラー表示画面        → 各種エラーレスポンスのMock
//
// Mock の特徴:
// - 実際のZoom APIレスポンス形式に準拠
// - 日本語データの適切な処理
// - パラメータライズドテスト対応
// - エラーケースの網羅的カバー
// - 統合テスト・単体テスト両対応
// - リアルタイム応答のシミュレーション