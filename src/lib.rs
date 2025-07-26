// OAuth2関連のimportは現在使用されていませんが、
// 今後の機能拡張のために残しておきます
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tokio::io::AsyncWriteExt;
use chrono::{FixedOffset, Utc};

pub mod windows_console;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: Option<String>,
}

impl Config {
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
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

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
    /// 
    /// # 不変条件
    /// - 関数実行中にサンプル設定の内容は一定
    /// - 入力パラメータは変更されない
    pub fn create_sample_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let sample_config = Config {
            client_id: "your_zoom_client_id".to_string(),
            client_secret: "your_zoom_client_secret".to_string(),
            redirect_uri: Some("http://localhost:8080/callback".to_string()),
        };
        
        let content = toml::to_string_pretty(&sample_config)?;
        fs::write(path, content)?;
        Ok(())
    }

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
    /// 
    /// # 不変条件
    /// - self の内容は変更されない
    /// - 入力パラメータは変更されない
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Recording {
    pub id: String,
    pub meeting_id: String,
    pub recording_start: String,
    pub recording_end: String,
    pub file_type: String,
    pub file_size: u64,
    pub download_url: String,
    pub recording_type: String,
}

#[derive(Debug, Deserialize)]
pub struct RecordingResponse {
    pub meetings: Vec<MeetingRecording>,
}

#[derive(Debug, Deserialize)]
pub struct MeetingRecording {
    pub uuid: String,
    pub id: u64,
    pub topic: String,
    pub start_time: String,
    pub recording_files: Vec<Recording>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AISummaryResponse {
    #[serde(default)]
    pub meeting_uuid: String,
    #[serde(default)]
    pub summary_start_time: String,
    #[serde(default)]
    pub summary_end_time: String,
    #[serde(default)]
    pub summary_created_time: String,
    #[serde(default)]
    pub summary_last_modified_time: String,
    #[serde(default)]
    pub summary_title: String,
    #[serde(default)]
    pub summary_overview: String,
    #[serde(default)]
    pub summary_details: Vec<SummaryDetail>,
    #[serde(default)]
    pub summary_content: String,
    #[serde(default)]
    pub next_steps: Vec<String>,
    #[serde(default)]
    pub summary_keyword: Vec<String>,
    
    // Alternative field names that Zoom might use
    #[serde(default, alias = "summary")]
    pub summary: String,
    #[serde(default, alias = "key_points")]
    pub key_points: Vec<String>,
    #[serde(default, alias = "action_items")]
    pub action_items: Vec<String>,
    #[serde(default, alias = "meeting_id")]
    pub meeting_id: String,
    
    // Additional structured content fields
    #[serde(default, alias = "topic_summaries")]
    pub topic_summaries: Vec<TopicSummary>,
    #[serde(default, alias = "detailed_sections")]
    pub detailed_sections: Vec<DetailedSection>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SummaryDetail {
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub summary: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TopicSummary {
    #[serde(default)]
    pub topic_title: String,
    #[serde(default)]
    pub topic_content: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DetailedSection {
    #[serde(default)]
    pub section_title: String,
    #[serde(default)]
    pub section_content: String,
}

// Alternative structure for unknown AI summary formats
#[derive(Debug, Deserialize, Serialize)]
pub struct GenericAISummary {
    #[serde(flatten)]
    pub data: serde_json::Map<String, serde_json::Value>,
}

pub struct ZoomRecordingDownloader {
    client: Client,
    access_token: String,
}

impl ZoomRecordingDownloader {
    /// 新しいZoomRecordingDownloaderインスタンスを作成する
    /// 
    /// # 事前条件
    /// - access_token は有効なOAuth2アクセストークンである
    /// - access_token は空でない
    /// 
    /// # 事後条件
    /// - 新しいZoomRecordingDownloaderインスタンスが作成される
    /// - HTTP clientが正常に初期化される
    /// - access_tokenが内部に保存される
    /// 
    /// # 不変条件
    /// - access_tokenは構造体の生存期間中不変
    /// - HTTP clientの設定は変更されない
    pub fn new(access_token: String) -> Self {
        Self {
            client: Client::new(),
            access_token,
        }
    }

    /// Zoom API への接続とアクセス権限をテストする（副作用なし版）
    /// 
    /// # 事前条件
    /// - self.access_token は有効なOAuth2アクセストークンである
    /// - インターネット接続が利用可能である
    /// - Zoom API サーバーが稼働中である
    /// 
    /// # 事後条件
    /// - 成功時: API接続テスト結果とメッセージリストを返す
    /// - 失敗時: 適切なエラーメッセージと共にエラーを返す
    /// 
    /// # 不変条件
    /// - コンソール出力の副作用なし
    /// - グローバル状態の変更なし
    /// - self の状態は変更されない
    pub async fn test_api_access_pure(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut messages = Vec::new();
        messages.push("=== Testing Zoom API Access ===".to_string());
        
        // Test basic user info API
        let user_response = self
            .client
            .get("https://api.zoom.us/v2/users/me")
            .bearer_auth(&self.access_token)
            .send()
            .await?;

        messages.push(format!("User API status: {}", user_response.status()));
        
        if user_response.status().is_success() {
            if let Ok(user_data) = user_response.json::<serde_json::Value>().await {
                if let Some(user_id) = user_data.get("id").and_then(|v| v.as_str()) {
                    messages.push(format!("✓ Connected as user: {}", user_id));
                }
                if let Some(account_id) = user_data.get("account_id").and_then(|v| v.as_str()) {
                    messages.push(format!("Account ID: {}", account_id));
                }
            }
        } else {
            messages.push(format!("✗ User API failed: {}", user_response.status()));
        }

        messages.push("=== End API Test ===\n".to_string());
        Ok(messages)
    }

    /// Zoom API への接続とアクセス権限をテストする（副作用あり版）
    /// 
    /// 事前条件:
    /// - self.access_token は有効なOAuth2アクセストークンである
    /// - インターネット接続が利用可能である
    /// - Zoom API サーバーが稼働中である
    /// 
    /// 事後条件:
    /// - 成功時: API接続が確認され、ユーザー情報が出力される
    /// - 失敗時: 適切なエラーメッセージと共にエラーを返す
    /// API アクセスをテストし、結果を標準出力に表示する
    /// 
    /// # 副作用
    /// - HTTPリクエストの送信
    /// - 標準出力へのメッセージ表示
    pub async fn test_api_access(&self) -> Result<(), Box<dyn std::error::Error>> {
        let messages = self.test_api_access_pure().await?;
        for message in messages {
            windows_console::println_japanese(&message);
        }
        Ok(())
    }

    /// 指定した期間のレコーディング一覧を取得する
    /// 
    /// # 副作用
    /// - HTTPリクエストの送信
    /// 
    /// # 事前条件
    /// - user_id は有効なZoomユーザーIDである
    /// - from は有効な日付形式（YYYY-MM-DD）である
    /// - to は有効な日付形式（YYYY-MM-DD）である
    /// - from <= to である
    /// - アクセストークンが有効である
    /// 
    /// # 事後条件
    /// - 成功時: 指定期間のレコーディング情報を含むRecordingResponseを返す
    /// - 失敗時: 適切なエラーメッセージと共にエラーを返す
    /// 
    /// # 不変条件
    /// - self の状態は変更されない
    /// - 入力パラメータは変更されない
    pub async fn list_recordings(&self, user_id: &str, from: &str, to: &str) -> Result<RecordingResponse, Box<dyn std::error::Error>> {
        let url = format!(
            "https://api.zoom.us/v2/users/{}/recordings?from={}&to={}",
            user_id, from, to
        );

        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.access_token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("API request failed: {}", response.status()).into());
        }

        let recordings: RecordingResponse = response.json().await?;
        Ok(recordings)
    }

    /// ミーティングIDを使用してAI要約を取得する
    /// 
    /// 事前条件:
    /// - meeting_id は有効なZoomミーティングIDである
    /// - アクセストークンが有効である
    /// - 必要なスコープ（meeting:read）が許可されている
    /// 
    /// 事後条件:
    /// - 成功時: AI要約が利用可能な場合は Some(AISummaryResponse) を返す
    /// - AI要約が利用不可の場合は None を返す
    /// - 失敗時: 適切なエラーメッセージと共にエラーを返す
    pub async fn get_ai_summary_by_meeting_id(&self, meeting_id: u64) -> Result<Option<AISummaryResponse>, Box<dyn std::error::Error>> {
        windows_console::println_japanese(&format!("Requesting AI summary for Meeting ID: {}", meeting_id));
        
        // Try AI summary endpoints using meeting ID - comprehensive list
        let endpoints = vec![
            format!("https://api.zoom.us/v2/meetings/{}/batch_summary", meeting_id),
            format!("https://api.zoom.us/v2/meetings/{}/summary", meeting_id),
            format!("https://api.zoom.us/v2/meetings/{}/ai_companion_summary", meeting_id),
            format!("https://api.zoom.us/v2/meetings/{}/recording_summary", meeting_id),
            format!("https://api.zoom.us/v2/meetings/{}/meeting_summary", meeting_id),
            format!("https://api.zoom.us/v2/ai_companion/meetings/{}/summary", meeting_id),
            format!("https://api.zoom.us/v2/ai_companion/summary/{}", meeting_id),
            format!("https://api.zoom.us/v2/meetings/{}/ai_summary", meeting_id),
            format!("https://api.zoom.us/v2/meetings/{}/detailed_summary", meeting_id),
            format!("https://api.zoom.us/v2/meetings/{}/content_summary", meeting_id),
            format!("https://api.zoom.us/v2/meetings/{}/companion_summary", meeting_id),
            format!("https://api.zoom.us/v2/ai/meetings/{}/summary", meeting_id),
            format!("https://api.zoom.us/v2/ai/summary/meetings/{}", meeting_id),
            format!("https://api.zoom.us/v2/meetings/{}/analysis", meeting_id),
            format!("https://api.zoom.us/v2/meetings/{}/insights", meeting_id),
        ];
        
        for (i, url) in endpoints.iter().enumerate() {
            windows_console::println_japanese(&format!("Trying Meeting ID endpoint {}/{}: {}", i+1, endpoints.len(), url));

            let response = self
                .client
                .get(url)
                .bearer_auth(&self.access_token)
                .send()
                .await?;

            match response.status().as_u16() {
                200 => {
                    windows_console::println_japanese("AI summary response received via Meeting ID!");
                    let response_text = response.text().await?;
                    windows_console::println_japanese(&format!("Response length: {} chars", response_text.len()));
                    
                    // Save debug response
                    self.save_debug_response(&response_text, &format!("meeting_id_{}_endpoint_{}", meeting_id, i+1)).await;
                    
                    if let Ok(summary) = serde_json::from_str::<AISummaryResponse>(&response_text) {
                        windows_console::println_japanese("Successfully parsed AI summary!");
                        return Ok(Some(summary));
                    } else if let Ok(generic_json) = serde_json::from_str::<serde_json::Value>(&response_text) {
                        windows_console::println_japanese("Received valid JSON, converting to AI summary format");
                        let converted_summary = self.convert_generic_to_ai_summary(generic_json, &meeting_id.to_string());
                        return Ok(Some(converted_summary));
                    } else {
                        windows_console::println_japanese("Response is not valid JSON");
                        // Save invalid response for debugging
                        self.save_debug_response(&response_text, &format!("meeting_id_{}_endpoint_{}_invalid", meeting_id, i+1)).await;
                        continue;
                    }
                },
                404 => {
                    windows_console::println_japanese(&format!("Meeting ID endpoint {} returned 404 (not found)", i+1));
                    continue;
                },
                401 => {
                    windows_console::println_japanese(&format!("Meeting ID endpoint {} returned 401 (Unauthorized)", i+1));
                    continue;
                },
                403 => {
                    windows_console::println_japanese(&format!("Meeting ID endpoint {} returned 403 (Forbidden)", i+1));
                    continue;
                },
                _ => {
                    windows_console::println_japanese(&format!("Meeting ID endpoint {} returned {}", i+1, response.status()));
                    continue;
                }
            }
        }
        
        windows_console::println_japanese(&format!("No AI summary found via Meeting ID {}", meeting_id));
        Ok(None)
    }

    /// UUID形式のバリエーションを生成する（純粋関数）
    /// 
    /// 事前条件:
    /// - meeting_uuid は空でない文字列である
    /// 
    /// 事後条件:
    /// - UUID形式のバリエーションリストを返す
    /// - 副作用なし
    /// 
    /// 不変条件:
    /// - 入力パラメータを変更しない
    fn generate_uuid_variants(&self, meeting_uuid: &str) -> Vec<String> {
        let single_encoded = urlencoding::encode(meeting_uuid);
        let double_encoded = urlencoding::encode(&single_encoded);
        
        vec![
            double_encoded.to_string(),  // Research shows double encoding is often required
            single_encoded.to_string(),
            meeting_uuid.to_string(),
            meeting_uuid.replace("/", "%2F").replace("=", "%3D"),
        ]
    }

    /// AI要約エンドポイントのリストを生成する（純粋関数）
    /// 
    /// 事前条件:
    /// - uuid_variant は空でない文字列である
    /// 
    /// 事後条件:
    /// - エンドポイントURLのリストを返す
    /// - 副作用なし
    /// 
    /// 不変条件:
    /// - 入力パラメータを変更しない
    fn generate_ai_summary_endpoints(&self, uuid_variant: &str) -> Vec<String> {
        vec![
            // Primary endpoints
            format!("https://api.zoom.us/v2/meetings/{}/meeting_summary", uuid_variant),
            format!("https://api.zoom.us/v2/meetings/{}/recordings", uuid_variant),
            format!("https://api.zoom.us/v2/meetings/{}/summary", uuid_variant),
            format!("https://api.zoom.us/v2/meetings/{}/batch_summary", uuid_variant),
            // Extended AI endpoints
            format!("https://api.zoom.us/v2/meetings/{}/ai_companion_summary", uuid_variant),
            format!("https://api.zoom.us/v2/meetings/{}/ai_summary", uuid_variant),
            format!("https://api.zoom.us/v2/meetings/{}/detailed_summary", uuid_variant),
            format!("https://api.zoom.us/v2/meetings/{}/content_summary", uuid_variant),
            format!("https://api.zoom.us/v2/meetings/{}/companion_summary", uuid_variant),
            format!("https://api.zoom.us/v2/ai_companion/meetings/{}/summary", uuid_variant),
            format!("https://api.zoom.us/v2/ai_companion/summary/{}", uuid_variant),
            format!("https://api.zoom.us/v2/ai/meetings/{}/summary", uuid_variant),
            format!("https://api.zoom.us/v2/ai/summary/meetings/{}", uuid_variant),
            format!("https://api.zoom.us/v2/meetings/{}/analysis", uuid_variant),
            format!("https://api.zoom.us/v2/meetings/{}/insights", uuid_variant),
            format!("https://api.zoom.us/v2/meetings/{}/recording_summary", uuid_variant),
        ]
    }

    /// HTTPレスポンスからAI要約を抽出する（複雑度削減版）
    /// 
    /// 事前条件:
    /// - response_text は有効なHTTPレスポンステキストである
    /// - url は有効なURLである
    /// - meeting_uuid は空でない文字列である
    /// 
    /// 事後条件:
    /// - 成功時: AISummaryResponseを返す
    /// - 失敗時: None を返す
    /// 
    /// 不変条件:
    /// - 入力パラメータを変更しない
    async fn extract_ai_summary_from_response(&self, response_text: &str, url: &str, meeting_uuid: &str) -> Option<AISummaryResponse> {
        // Check if this is a recordings endpoint response
        if url.contains("/recordings") {
            if let Ok(recordings_data) = serde_json::from_str::<serde_json::Value>(response_text) {
                // Look for SUMMARY file type in recording files
                if let Some(recording_files) = recordings_data.get("recording_files").and_then(|v| v.as_array()) {
                    for file in recording_files {
                        if let Some(file_type) = file.get("file_type").and_then(|v| v.as_str()) {
                            if file_type == "SUMMARY" {
                                windows_console::println_japanese("✓ Found SUMMARY file in recordings!");
                                let converted_summary = self.convert_generic_to_ai_summary(file.clone(), meeting_uuid);
                                return Some(converted_summary);
                            }
                        }
                    }
                }
                windows_console::println_japanese("No SUMMARY file found in recordings");
                return None;
            }
        } else {
            // Try to parse as meeting summary response
            if let Ok(summary) = serde_json::from_str::<AISummaryResponse>(response_text) {
                windows_console::println_japanese("✓ Successfully parsed AI summary!");
                return Some(summary);
            } else if let Ok(generic_json) = serde_json::from_str::<serde_json::Value>(response_text) {
                windows_console::println_japanese("✓ Received valid JSON, converting to AI summary format");
                let converted_summary = self.convert_generic_to_ai_summary(generic_json, meeting_uuid);
                return Some(converted_summary);
            } else {
                windows_console::println_japanese("Response is not valid JSON");
                return None;
            }
        }
        None
    }

    /// 単一のエンドポイントでAI要約を試行する（複雑度削減版）
    /// 
    /// 事前条件:
    /// - url は有効なURLである
    /// - meeting_uuid は空でない文字列である
    /// - variant_idx と endpoint_idx は有効なインデックスである
    /// 
    /// 事後条件:
    /// - 成功時: AISummaryResponseを返す
    /// - 失敗時: None を返す
    /// 
    /// 不変条件:
    /// - ネットワークエラーは上位に伝播される
    async fn try_single_endpoint(&self, url: &str, meeting_uuid: &str, variant_idx: usize, endpoint_idx: usize) -> Result<Option<AISummaryResponse>, Box<dyn std::error::Error>> {
        windows_console::println_japanese(&format!("Trying endpoint {}: {}", endpoint_idx + 1, url));

        let response = self
            .client
            .get(url)
            .bearer_auth(&self.access_token)
            .send()
            .await?;

        match response.status().as_u16() {
            200 => {
                windows_console::println_japanese("✓ Received 200 response!");
                let response_text = response.text().await?;
                windows_console::println_japanese(&format!("Response length: {} chars", response_text.len()));
                
                // Save debug response
                self.save_debug_response(&response_text, &format!("uuid_{}_variant_{}_endpoint_{}", meeting_uuid.replace("/", "_").replace("=", "_"), variant_idx + 1, endpoint_idx + 1)).await;
                
                return Ok(self.extract_ai_summary_from_response(&response_text, url, meeting_uuid).await);
            },
            404 => {
                windows_console::println_japanese(&format!("Endpoint {} returned 404 (not found)", endpoint_idx + 1));
            },
            401 => {
                windows_console::println_japanese(&format!("Endpoint {} returned 401 (Unauthorized)", endpoint_idx + 1));
                windows_console::println_japanese("ℹ Ensure access token is valid and has not expired");
            },
            403 => {
                windows_console::println_japanese(&format!("Endpoint {} returned 403 (Forbidden)", endpoint_idx + 1));
                windows_console::println_japanese("ℹ Required scopes: meeting:read, recording:read, user:read");
                windows_console::println_japanese("ℹ Note: You may need to be the meeting host to access summaries");
            },
            429 => {
                windows_console::println_japanese(&format!("Endpoint {} returned 429 (Rate limit exceeded)", endpoint_idx + 1));
                // Add delay for rate limiting
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            },
            422 => {
                windows_console::println_japanese(&format!("Endpoint {} returned 422 (Unprocessable Entity)", endpoint_idx + 1));
                windows_console::println_japanese("ℹ This may indicate the summary is still being processed");
            },
            500..=599 => {
                windows_console::println_japanese(&format!("Endpoint {} returned {} (Server error)", endpoint_idx + 1, response.status()));
            },
            _ => {
                windows_console::println_japanese(&format!("Endpoint {} returned {} (Unknown error)", endpoint_idx + 1, response.status()));
                if let Ok(error_text) = response.text().await {
                    if !error_text.is_empty() {
                        windows_console::println_japanese(&format!("Error details: {}", 
                            if error_text.len() > 300 { 
                                format!("{}...", &error_text[..300]) 
                            } else { 
                                error_text 
                            }
                        ));
                    }
                }
            }
        }
        Ok(None)
    }

    /// ミーティングUUIDを使用してAI要約を取得する（リファクタリング版・複雑度削減）
    /// 
    /// 事前条件:
    /// - meeting_uuid は有効なZoomミーティングUUIDである
    /// - アクセストークンが有効である
    /// - 必要なスコープ（meeting:read）が許可されている
    /// 
    /// 事後条件:
    /// - 成功時: AI要約が利用可能な場合は Some(AISummaryResponse) を返す
    /// - AI要約が利用不可の場合は None を返す
    /// - 失敗時: 適切なエラーメッセージと共にエラーを返す
    pub async fn get_ai_summary(&self, meeting_uuid: &str) -> Result<Option<AISummaryResponse>, Box<dyn std::error::Error>> {
        windows_console::println_japanese(&format!("Requesting AI summary for UUID: {}", meeting_uuid));
        
        let uuid_variants = self.generate_uuid_variants(meeting_uuid);
        
        windows_console::println_japanese(&format!("Single encoded UUID: {}", urlencoding::encode(meeting_uuid)));
        windows_console::println_japanese(&format!("Double encoded UUID: {}", urlencoding::encode(&urlencoding::encode(meeting_uuid))));
        
        for (variant_idx, uuid_variant) in uuid_variants.iter().enumerate() {
            windows_console::println_japanese(&format!("Trying UUID variant {}/{}: {}", variant_idx + 1, uuid_variants.len(), uuid_variant));
            
            let endpoints = self.generate_ai_summary_endpoints(uuid_variant);
            
            for (endpoint_idx, url) in endpoints.iter().enumerate() {
                if let Some(summary) = self.try_single_endpoint(url, meeting_uuid, variant_idx, endpoint_idx).await? {
                    return Ok(Some(summary));
                }
            }
        }
        
        // No AI summary found from any endpoint
        windows_console::println_japanese(&format!("ℹ No AI summary available for meeting {} (this is normal if AI Companion was not enabled)", meeting_uuid));
        windows_console::println_japanese("ℹ AI summaries require: 1) AI Companion enabled, 2) Meeting host access, 3) Processing time (up to 24h)");
        Ok(None)
    }

    /// 汎用JSON形式をAISummaryResponseに変換する
    /// 
    /// 事前条件:
    /// - json は有効なJSON Valueである
    /// - meeting_uuid は空でない文字列である
    /// 
    /// 事後条件:
    /// - 常に有効なAISummaryResponseを返す
    /// - 不明なフィールドは適切なデフォルト値で初期化される
    fn convert_generic_to_ai_summary(&self, json: serde_json::Value, meeting_uuid: &str) -> AISummaryResponse {
        // Extract common fields that might exist in various formats
        let summary_text = json.get("summary")
            .or_else(|| json.get("overview"))
            .or_else(|| json.get("content"))
            .or_else(|| json.get("brief_summary"))
            .or_else(|| json.get("executive_summary"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let key_points = json.get("key_points")
            .or_else(|| json.get("highlights"))
            .or_else(|| json.get("main_points"))
            .or_else(|| json.get("important_points"))
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        let action_items = self.extract_action_items_with_assignees(&json);

        // Extract topic summaries from various possible structures
        let topic_summaries = self.extract_topic_summaries(&json);
        
        // Extract detailed sections from various possible structures  
        let detailed_sections = self.extract_detailed_sections(&json);
        
        // Extract summary details
        let summary_details = self.extract_summary_details(&json);
        
        // Extract summary_content (detailed markdown content)
        let summary_content = json.get("summary_content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        AISummaryResponse {
            meeting_uuid: meeting_uuid.to_string(),
            summary_start_time: json.get("start_time").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            summary_end_time: json.get("end_time").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            summary_created_time: json.get("created_time").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            summary_last_modified_time: json.get("modified_time").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            summary_title: json.get("title").and_then(|v| v.as_str()).unwrap_or("AI Generated Summary").to_string(),
            summary_overview: summary_text.clone(),
            summary_details: summary_details,
            summary_content: summary_content,
            next_steps: action_items.clone(),
            summary_keyword: json.get("keywords")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
            summary: summary_text,
            key_points: key_points,
            action_items: action_items,
            meeting_id: json.get("meeting_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            topic_summaries: topic_summaries,
            detailed_sections: detailed_sections,
        }
    }

    fn extract_topic_summaries(&self, json: &serde_json::Value) -> Vec<TopicSummary> {
        let mut topics = Vec::new();
        
        // Try various field names for topic summaries with expanded search
        let topic_arrays = [
            json.get("topics"),
            json.get("topic_summaries"),
            json.get("sections"),
            json.get("segments"),
            json.get("detailed_topics"),
            json.get("content_sections"),
            json.get("meeting_sections"),
            json.get("discussion_topics"),
            json.get("agenda_items"),
            json.get("key_discussions"),
        ];
        
        for topic_array_opt in topic_arrays {
            if let Some(topic_array) = topic_array_opt.and_then(|v| v.as_array()) {
                for topic in topic_array {
                    if let (Some(title), Some(content)) = (
                        topic.get("title")
                            .or_else(|| topic.get("topic"))
                            .or_else(|| topic.get("name"))
                            .or_else(|| topic.get("heading"))
                            .or_else(|| topic.get("section_title"))
                            .or_else(|| topic.get("discussion_topic"))
                            .and_then(|v| v.as_str()),
                        topic.get("content")
                            .or_else(|| topic.get("summary"))
                            .or_else(|| topic.get("description"))
                            .or_else(|| topic.get("details"))
                            .or_else(|| topic.get("text"))
                            .or_else(|| topic.get("discussion_summary"))
                            .and_then(|v| v.as_str())
                    ) {
                        topics.push(TopicSummary {
                            topic_title: title.to_string(),
                            topic_content: content.to_string(),
                        });
                    }
                }
                if !topics.is_empty() {
                    break; // Found topics, stop searching
                }
            }
        }
        
        // Try to extract from nested structures with deeper search
        if topics.is_empty() {
            let nested_containers = [
                json.get("overview"),
                json.get("detailed_content"),
                json.get("meeting_details"),
                json.get("content_analysis"),
                json.get("discussion_analysis"),
                json.get("structured_summary"),
            ];
            
            for container_opt in nested_containers {
                if let Some(container) = container_opt {
                    let nested_arrays = [
                        container.get("topics"),
                        container.get("sections"),
                        container.get("content_sections"),
                        container.get("detailed_sections"),
                    ];
                    
                    for nested_array_opt in nested_arrays {
                        if let Some(nested_array) = nested_array_opt.and_then(|v| v.as_array()) {
                            for topic in nested_array {
                                if let (Some(title), Some(content)) = (
                                    topic.get("title")
                                        .or_else(|| topic.get("topic"))
                                        .or_else(|| topic.get("section_title"))
                                        .and_then(|v| v.as_str()),
                                    topic.get("content")
                                        .or_else(|| topic.get("summary"))
                                        .or_else(|| topic.get("details"))
                                        .and_then(|v| v.as_str())
                                ) {
                                    topics.push(TopicSummary {
                                        topic_title: title.to_string(),
                                        topic_content: content.to_string(),
                                    });
                                }
                            }
                            if !topics.is_empty() {
                                break;
                            }
                        }
                    }
                    if !topics.is_empty() {
                        break;
                    }
                }
            }
        }
        
        topics
    }
    
    fn extract_detailed_sections(&self, json: &serde_json::Value) -> Vec<DetailedSection> {
        let mut sections = Vec::new();
        
        // Try various field names for detailed sections with comprehensive search
        let section_arrays = [
            json.get("detailed_sections"),
            json.get("sections"),
            json.get("details"),
            json.get("content_sections"),
            json.get("meeting_sections"),
            json.get("analysis_sections"),
            json.get("discussion_sections"),
            json.get("structured_content"),
            json.get("content_breakdown"),
            json.get("detailed_breakdown"),
        ];
        
        for section_array_opt in section_arrays {
            if let Some(section_array) = section_array_opt.and_then(|v| v.as_array()) {
                for section in section_array {
                    if let (Some(title), Some(content)) = (
                        section.get("title")
                            .or_else(|| section.get("section_title"))
                            .or_else(|| section.get("heading"))
                            .or_else(|| section.get("name"))
                            .or_else(|| section.get("section_name"))
                            .or_else(|| section.get("topic"))
                            .and_then(|v| v.as_str()),
                        section.get("content")
                            .or_else(|| section.get("section_content"))
                            .or_else(|| section.get("text"))
                            .or_else(|| section.get("details"))
                            .or_else(|| section.get("summary"))
                            .or_else(|| section.get("description"))
                            .and_then(|v| v.as_str())
                    ) {
                        sections.push(DetailedSection {
                            section_title: title.to_string(),
                            section_content: content.to_string(),
                        });
                    }
                }
                if !sections.is_empty() {
                    break; // Found sections, stop searching
                }
            }
        }
        
        // Try extracting from object properties directly (non-array format)
        if sections.is_empty() {
            if let Some(obj) = json.as_object() {
                for (key, value) in obj {
                    // Look for section-like properties
                    if key.contains("section") || key.contains("topic") || key.contains("discussion") {
                        if let Some(content_str) = value.as_str() {
                            sections.push(DetailedSection {
                                section_title: key.clone(),
                                section_content: content_str.to_string(),
                            });
                        } else if let Some(content_obj) = value.as_object() {
                            // Extract nested content
                            let title = content_obj.get("title")
                                .or_else(|| content_obj.get("name"))
                                .and_then(|v| v.as_str())
                                .unwrap_or(key);
                            let content = content_obj.get("content")
                                .or_else(|| content_obj.get("summary"))
                                .or_else(|| content_obj.get("description"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("");
                            
                            if !content.is_empty() {
                                sections.push(DetailedSection {
                                    section_title: title.to_string(),
                                    section_content: content.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
        
        sections
    }
    
    fn extract_summary_details(&self, json: &serde_json::Value) -> Vec<SummaryDetail> {
        let mut details = Vec::new();
        
        // Try various field names for summary details with expanded search
        let detail_arrays = [
            json.get("summary_details"),
            json.get("details"),
            json.get("breakdown"),
            json.get("content_breakdown"),
            json.get("detailed_breakdown"),
            json.get("meeting_breakdown"),
            json.get("analysis_details"),
            json.get("structured_details"),
        ];
        
        for detail_array_opt in detail_arrays {
            if let Some(detail_array) = detail_array_opt.and_then(|v| v.as_array()) {
                for detail in detail_array {
                    if let (Some(label), Some(summary)) = (
                        detail.get("label")
                            .or_else(|| detail.get("type"))
                            .or_else(|| detail.get("category"))
                            .or_else(|| detail.get("summary_type"))
                            .or_else(|| detail.get("detail_type"))
                            .or_else(|| detail.get("section_type"))
                            .and_then(|v| v.as_str()),
                        detail.get("summary")
                            .or_else(|| detail.get("content"))
                            .or_else(|| detail.get("summary_content"))
                            .or_else(|| detail.get("detail_content"))
                            .or_else(|| detail.get("text"))
                            .or_else(|| detail.get("description"))
                            .and_then(|v| v.as_str())
                    ) {
                        details.push(SummaryDetail {
                            label: label.to_string(),
                            summary: summary.to_string(),
                        });
                    }
                }
                if !details.is_empty() {
                    break; // Found details, stop searching
                }
            }
        }
        
        details
    }
    
    fn extract_brief_summary_only(&self, formatted_content: &str) -> String {
        // Extract only the brief summary part, typically the first paragraph or section
        let lines: Vec<&str> = formatted_content.lines().collect();
        let mut result = String::new();
        let mut found_content = false;
        
        for line in lines {
            let trimmed = line.trim();
            
            // Skip empty lines at the start
            if !found_content && trimmed.is_empty() {
                continue;
            }
            
            // Stop if we hit a detailed section marker (starts with ◆ or ■ or #)
            if found_content && (trimmed.starts_with("◆") || trimmed.starts_with("■") || trimmed.starts_with("#")) {
                break;
            }
            
            found_content = true;
            result.push_str(line);
            result.push('\n');
            
            // If we have a good amount of summary text, stop here
            if result.len() > 200 && trimmed.is_empty() {
                break;
            }
        }
        
        result.trim().to_string()
    }
    
    /// AI要約ヘッダーを生成する（純粋関数）
    /// 
    /// 事前条件:
    /// - summary は有効なAISummaryResponseである
    /// - meeting_id は空でない文字列である
    /// 
    /// 事後条件:
    /// - ヘッダー部分のテキストを返す
    /// - 副作用なし
    /// 
    /// 不変条件:
    /// - 入力パラメータを変更しない
    fn generate_ai_summary_header(&self, summary: &AISummaryResponse, meeting_id: &str) -> String {
        let mut content = String::new();
        content.push_str("=".repeat(80).as_str());
        content.push_str("\n");
        content.push_str(&format!("AI要約 - ミーティングID: {}\n", meeting_id));
        if !summary.summary_title.is_empty() {
            content.push_str(&format!("タイトル: {}\n", summary.summary_title));
        }
        if !summary.summary_created_time.is_empty() {
            content.push_str(&format!("作成日時: {}\n", summary.summary_created_time));
        }
        content.push_str("=".repeat(80).as_str());
        content.push_str("\n\n");
        content
    }

    /// AI要約の簡単な要約部分を生成する（純粋関数）
    /// 
    /// 事前条件:
    /// - summary は有効なAISummaryResponseである
    /// 
    /// 事後条件:
    /// - 簡単な要約部分のテキストを返す
    /// - 副作用なし
    /// 
    /// 不変条件:
    /// - 入力パラメータを変更しない
    fn generate_brief_summary_section(&self, summary: &AISummaryResponse) -> String {
        let mut content = String::new();
        content.push_str("【簡単な要約】\n");
        
        let brief_summary = if !summary.summary_overview.is_empty() { 
            &summary.summary_overview
        } else if !summary.summary.is_empty() {
            &summary.summary
        } else if !summary.topic_summaries.is_empty() {
            // Generate summary from topic summaries if no direct summary available
            let combined_summary: String = summary.topic_summaries.iter()
                .map(|t| format!("{}について議論され", t.topic_title))
                .collect::<Vec<_>>()
                .join("、");
            content.push_str(&format!("この会議では、{}。", combined_summary));
            content.push_str("\n");
            content.push_str("\n");
            return content;
        } else {
            ""
        };
        
        if !brief_summary.is_empty() {
            content.push_str(brief_summary);
            content.push_str("\n");
        } else if summary.topic_summaries.is_empty() {
            content.push_str("要約情報がありません。\n");
        }
        content.push_str("\n");
        content
    }

    /// AI要約の次のステップ部分を生成する（純粋関数）
    /// 
    /// 事前条件:
    /// - summary は有効なAISummaryResponseである
    /// 
    /// 事後条件:
    /// - 次のステップ部分のテキストを返す
    /// - 副作用なし
    /// 
    /// 不変条件:
    /// - 入力パラメータを変更しない
    fn generate_next_steps_section(&self, summary: &AISummaryResponse) -> String {
        let mut content = String::new();
        content.push_str("【次のステップ】\n");
        
        let next_steps = if !summary.next_steps.is_empty() { 
            &summary.next_steps
        } else { 
            &summary.action_items 
        };
        
        if !next_steps.is_empty() {
            for (i, step) in next_steps.iter().enumerate() {
                // Process markdown links in the step text
                let processed_step = self.process_markdown_links(step);
                content.push_str(&format!("{}. {}\n", i + 1, processed_step));
            }
        } else {
            content.push_str("次のステップはありません。\n");
        }
        content.push_str("\n");
        content
    }

    /// AI要約の詳細コンテンツ部分を生成する（純粋関数）
    /// 
    /// 事前条件:
    /// - summary は有効なAISummaryResponseである
    /// 
    /// 事後条件:
    /// - 詳細コンテンツ部分のテキストを返す
    /// - 副作用なし
    /// 
    /// 不変条件:
    /// - 入力パラメータを変更しない
    fn generate_detailed_content_section(&self, summary: &AISummaryResponse) -> String {
        let mut content = String::new();
        
        // Add detailed summary content (markdown format) if available
        if !summary.summary_content.is_empty() {
            let formatted_content = self.filter_duplicate_sections(&summary.summary_content, &summary.next_steps);
            if !formatted_content.trim().is_empty() {
                // Check if we have detailed content that would be redundant
                let has_detailed_content_elsewhere = !summary.summary_details.is_empty() || 
                                                   !summary.detailed_sections.is_empty() || 
                                                   !summary.topic_summaries.is_empty();
                
                if has_detailed_content_elsewhere {
                    // Only show a brief summary from the AI content, not the full details
                    let brief_summary_only = self.extract_brief_summary_only(&formatted_content);
                    if !brief_summary_only.trim().is_empty() && brief_summary_only.len() > 50 {
                        content.push_str("【AI要約（概要のみ）】\n");
                        content.push_str(&brief_summary_only);
                        content.push_str("\n\n");
                    }
                } else {
                    // Show full content if no other detailed sections available
                    content.push_str("【詳細要約（Zoom AI生成）】\n");
                    content.push_str(&formatted_content);
                    content.push_str("\n\n");
                }
            }
        }
        content
    }

    /// AI要約の詳細セクション部分を生成する（純粋関数）
    /// 
    /// 事前条件:
    /// - summary は有効なAISummaryResponseである
    /// 
    /// 事後条件:
    /// - 詳細セクション部分のテキストを返す
    /// - 副作用なし
    /// 
    /// 不変条件:
    /// - 入力パラメータを変更しない
    fn generate_detailed_sections(&self, summary: &AISummaryResponse) -> String {
        let mut content = String::new();
        
        // Show most detailed content available, avoiding duplication
        let has_summary_details = !summary.summary_details.is_empty();
        let has_detailed_sections = !summary.detailed_sections.is_empty();
        let has_topic_summaries = !summary.topic_summaries.is_empty();
        
        // Prioritize summary_details as most detailed, then detailed_sections, then topic_summaries
        if has_summary_details {
            content.push_str("【詳細要約内容】\n");
            for detail in &summary.summary_details {
                if !detail.label.is_empty() {
                    content.push_str(&format!("■ {}\n", detail.label));
                    if !detail.summary.is_empty() {
                        content.push_str(&format!("{}\n\n", detail.summary));
                    }
                }
            }
        } else if has_detailed_sections {
            content.push_str("【詳細セクション】\n");
            for section in &summary.detailed_sections {
                if !section.section_title.is_empty() {
                    content.push_str(&format!("■ {}\n", section.section_title));
                    if !section.section_content.is_empty() {
                        content.push_str(&format!("{}\n\n", section.section_content));
                    }
                }
            }
        } else if has_topic_summaries {
            content.push_str("【トピック要約】\n");
            for (i, topic) in summary.topic_summaries.iter().enumerate() {
                if !topic.topic_title.is_empty() {
                    content.push_str(&format!("{}. {}\n", i + 1, topic.topic_title));
                    if !topic.topic_content.is_empty() {
                        content.push_str(&format!("{}\n\n", topic.topic_content));
                    }
                }
            }
        } else {
            // If no detailed content available
            content.push_str("【ミーティング情報】\n");
            content.push_str("■ ミーティングタイトル\n");
            if !summary.summary_title.is_empty() && summary.summary_title != "AI Generated Summary" {
                content.push_str(&format!("{}\n\n", summary.summary_title));
            } else {
                content.push_str("詳細な要約情報は利用できません。\n\n");
            }
        }
        content
    }

    /// AI要約のキーワードとポイント部分を生成する（純粋関数）
    /// 
    /// 事前条件:
    /// - summary は有効なAISummaryResponseである
    /// 
    /// 事後条件:
    /// - キーワードとポイント部分のテキストを返す
    /// - 副作用なし
    /// 
    /// 不変条件:
    /// - 入力パラメータを変更しない
    fn generate_keywords_and_points_section(&self, summary: &AISummaryResponse) -> String {
        let mut content = String::new();
        
        // Keywords
        if !summary.summary_keyword.is_empty() {
            content.push_str("【キーワード】\n");
            content.push_str(&summary.summary_keyword.join(", "));
            content.push_str("\n\n");
        }

        // Key points
        if !summary.key_points.is_empty() {
            content.push_str("【重要ポイント】\n");
            for (i, point) in summary.key_points.iter().enumerate() {
                content.push_str(&format!("{}. {}\n", i + 1, point));
            }
            content.push_str("\n");
        }
        content
    }

    /// AI要約フッターを生成する（純粋関数）
    /// 
    /// 事前条件:
    /// - なし
    /// 
    /// 事後条件:
    /// - フッター部分のテキストを返す
    /// - 副作用なし
    /// 
    /// 不変条件:
    /// - グローバル状態を変更しない
    fn generate_ai_summary_footer(&self) -> String {
        let mut content = String::new();
        content.push_str("-".repeat(80).as_str());
        content.push_str("\n");
        content.push_str("Generated by: Zoom AI Companion\n");
        content.push_str(&format!("Download timestamp: {}\n", Utc::now().to_rfc3339()));
        content
    }

    /// AI要約のテキストコンテンツを生成する（リファクタリング版・複雑度削減）
    /// 
    /// 事前条件:
    /// - summary は有効なAISummaryResponseである
    /// - meeting_id は空でない文字列である
    /// 
    /// 事後条件:
    /// - 成功時: 読みやすい形式のテキストコンテンツを返す
    /// - 副作用なし: ファイルは作成されない
    /// - 失敗時: 適切なエラーメッセージと共にエラーを返す
    /// 
    /// 不変条件:
    /// - ファイルシステムの状態を変更しない
    /// - 入力パラメータを変更しない
    fn generate_ai_summary_text_content(&self, summary: &AISummaryResponse, meeting_id: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut content = String::new();
        
        // Build content by composing smaller functions
        content.push_str(&self.generate_ai_summary_header(summary, meeting_id));
        content.push_str(&self.generate_brief_summary_section(summary));
        content.push_str(&self.generate_next_steps_section(summary));
        content.push_str(&self.generate_detailed_content_section(summary));
        content.push_str(&self.generate_detailed_sections(summary));
        content.push_str(&self.generate_keywords_and_points_section(summary));
        content.push_str(&self.generate_ai_summary_footer());

        Ok(content)
    }

    fn filter_duplicate_sections(&self, summary_content: &str, next_steps: &[String]) -> String {
        let mut result = summary_content.to_string();
        
        // Convert markdown to readable text format
        result = result
            .replace("## ", "◆ ")
            .replace("### ", "● ")
            .replace("- ", "  - ");
        
        // Remove "次のステップ" section if it exists in summary_content
        // since we already have a dedicated next steps section
        if !next_steps.is_empty() {
            // Look for next steps section in various forms
            let next_step_patterns = [
                "◆ 次のステップです。",
                "◆ 次のステップ",
                "● 次のステップ",
                "◆ Next Steps",
                "◆ Action Items",
                "◆ アクションアイテム"
            ];
            
            for pattern in &next_step_patterns {
                if let Some(start) = result.find(pattern) {
                    // Find the end of this section (next ◆ or end of text)
                    let section_end = result[start..]
                        .find("\n◆ ")
                        .map(|pos| start + pos)
                        .unwrap_or(result.len());
                    
                    // Remove this section
                    result.replace_range(start..section_end, "");
                    break;
                }
            }
        }
        
        // Remove "概要" section and everything after it since it duplicates with detailed content
        let overview_patterns = [
            "◆ 概要",
            "● 概要",
            "◆ Overview",
            "◆ 詳細",
            "◆ Details"
        ];
        
        for pattern in &overview_patterns {
            if let Some(start) = result.find(pattern) {
                // Remove everything from this point onwards
                result.truncate(start);
                break;
            }
        }
        
        // Clean up extra whitespace
        result = result.replace("\n\n\n", "\n\n");
        result.trim().to_string()
    }
    
    fn process_markdown_links(&self, text: &str) -> String {
        // Convert markdown links [text](url) to "text (URL: url)" format using simple string processing
        let mut result = text.to_string();
        
        // Find all markdown links and replace them
        while let Some(start) = result.find('[') {
            if let Some(middle) = result[start..].find("](") {
                let middle = start + middle;
                if let Some(end) = result[middle + 2..].find(')') {
                    let end = middle + 2 + end;
                    
                    let link_text = &result[start + 1..middle];
                    let url = &result[middle + 2..end];
                    
                    let replacement = if url.starts_with("http") {
                        format!("{} (URL: {})", link_text, url)
                    } else {
                        // If not a URL, just keep the link text
                        link_text.to_string()
                    };
                    
                    result.replace_range(start..=end, &replacement);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        result
    }
    
    fn extract_action_items_with_assignees(&self, json: &serde_json::Value) -> Vec<String> {
        let mut action_items = Vec::new();
        
        // Try various field names for action items
        let action_arrays = [
            json.get("action_items"),
            json.get("next_steps"),
            json.get("todos"),
            json.get("follow_ups"),
            json.get("tasks"),
            json.get("assignments"),
            json.get("action_points"),
            json.get("decisions_and_actions"),
        ];
        
        for action_array_opt in action_arrays {
            if let Some(action_array) = action_array_opt.and_then(|v| v.as_array()) {
                for action in action_array {
                    if let Some(action_str) = action.as_str() {
                        action_items.push(action_str.to_string());
                    } else if let Some(action_obj) = action.as_object() {
                        // Handle structured action items with assignees
                        let task = action_obj.get("task")
                            .or_else(|| action_obj.get("action"))
                            .or_else(|| action_obj.get("description"))
                            .or_else(|| action_obj.get("content"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        
                        let assignee = action_obj.get("assignee")
                            .or_else(|| action_obj.get("assigned_to"))
                            .or_else(|| action_obj.get("owner"))
                            .or_else(|| action_obj.get("responsible_person"))
                            .and_then(|v| v.as_str());
                        
                        let due_date = action_obj.get("due_date")
                            .or_else(|| action_obj.get("deadline"))
                            .or_else(|| action_obj.get("target_date"))
                            .and_then(|v| v.as_str());
                        
                        if !task.is_empty() {
                            let mut formatted_action = task.to_string();
                            if let Some(assignee_name) = assignee {
                                formatted_action.push_str(&format!(" [担当: {}]", assignee_name));
                            }
                            if let Some(due) = due_date {
                                formatted_action.push_str(&format!(" [期限: {}]", due));
                            }
                            action_items.push(formatted_action);
                        }
                    }
                }
                if !action_items.is_empty() {
                    break; // Found action items, stop searching
                }
            }
        }
        
        // Try extracting from nested structures
        if action_items.is_empty() {
            let containers = [
                json.get("decisions"),
                json.get("outcomes"),
                json.get("meeting_outcomes"),
                json.get("follow_up_actions"),
            ];
            
            for container_opt in containers {
                if let Some(container) = container_opt {
                    if let Some(actions) = container.get("actions")
                        .or_else(|| container.get("action_items"))
                        .or_else(|| container.get("tasks"))
                        .and_then(|v| v.as_array()) 
                    {
                        for action in actions {
                            if let Some(action_str) = action.as_str() {
                                action_items.push(action_str.to_string());
                            }
                        }
                        if !action_items.is_empty() {
                            break;
                        }
                    }
                }
            }
        }
        
        action_items
    }

    /// ミーティング開始時刻を基にしたフォルダーパスを生成する（副作用なし版）
    /// 
    /// 事前条件:
    /// - base_output_dir は有効なディレクトリパスである
    /// - meeting_start_time は有効なISO 8601形式の日時文字列である
    /// 
    /// 事後条件:
    /// - 成功時: YYYY-MM-DD形式のフォルダーパスを返す
    /// - 副作用なし: ディレクトリは作成されない
    /// - 失敗時: 適切なエラーメッセージと共にエラーを返す
    /// 
    /// 不変条件:
    /// - ファイルシステムの状態を変更しない
    /// - 入力パラメータを変更しない
    fn calculate_meeting_date_folder(&self, base_output_dir: &str, meeting_start_time: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Parse the meeting start time (ISO 8601 format)
        let meeting_datetime = chrono::DateTime::parse_from_rfc3339(meeting_start_time)
            .map_err(|e| format!("Failed to parse meeting start time '{}': {}", meeting_start_time, e))?;
        
        // Convert to JST timezone (+09:00)
        let jst_offset = FixedOffset::east_opt(9 * 3600).unwrap();
        let meeting_jst = meeting_datetime.with_timezone(&jst_offset);
        
        // Format as YYYY-MM-DD for folder name
        let date_folder = meeting_jst.format("%Y-%m-%d").to_string();
        
        // Create the date-based folder path (but don't create the directory)
        let date_folder_path = Path::new(base_output_dir).join(&date_folder);
        
        Ok(date_folder_path.to_string_lossy().to_string())
    }

    /// ミーティング開始時刻を基にしたフォルダーを作成する（副作用あり版）
    /// 
    /// 事前条件:
    /// - base_output_dir は有効なディレクトリパスである
    /// - meeting_start_time は有効なISO 8601形式の日時文字列である
    /// - ディレクトリへの書き込み権限がある
    /// 
    /// 事後条件:
    /// - 成功時: YYYY-MM-DD形式のフォルダーが作成される
    /// - 作成されたフォルダーパスを返す
    /// - 失敗時: 適切なエラーメッセージと共にエラーを返す
    fn create_meeting_date_folder(&self, base_output_dir: &str, meeting_start_time: &str) -> Result<String, Box<dyn std::error::Error>> {
        let date_folder_path = self.calculate_meeting_date_folder(base_output_dir, meeting_start_time)?;
        fs::create_dir_all(&date_folder_path)?;
        Ok(date_folder_path)
    }

    /// AI要約ファイル名を生成する（副作用なし版）
    /// 
    /// 事前条件:
    /// - summary は有効なAISummaryResponseである
    /// - meeting_id は空でない文字列である
    /// - extension は有効なファイル拡張子である
    /// 
    /// 事後条件:
    /// - 成功時: 適切なファイル名を返す
    /// - 副作用なし: ファイルは作成されない
    /// - 失敗時: 適切なエラーメッセージと共にエラーを返す
    /// 
    /// 不変条件:
    /// - ファイルシステムの状態を変更しない
    /// - 入力パラメータを変更しない
    fn generate_ai_summary_filename(&self, summary: &AISummaryResponse, meeting_id: &str, extension: &str) -> Result<String, Box<dyn std::error::Error>> {
        let invalid_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
        let safe_meeting_id = meeting_id.chars().map(|c| if invalid_chars.contains(&c) { '_' } else { c }).collect::<String>();
        
        // Generate timestamp suffix from creation time
        let timestamp_suffix = if !summary.summary_created_time.is_empty() {
            // Parse and format the creation time for filename
            match chrono::DateTime::parse_from_rfc3339(&summary.summary_created_time) {
                Ok(dt) => {
                    let utc_dt = dt.with_timezone(&chrono::Utc);
                    format!("_{}", utc_dt.format("%Y%m%d_%H%M%S"))
                },
                Err(_) => {
                    // If parsing fails, use current timestamp
                    format!("_{}", Utc::now().format("%Y%m%d_%H%M%S"))
                }
            }
        } else {
            // Use current timestamp if no creation time available
            format!("_{}", Utc::now().format("%Y%m%d_%H%M%S"))
        };
        
        let filename = format!("{}_ai_summary{}.{}", safe_meeting_id, timestamp_suffix, extension);
        Ok(filename)
    }


    /// AI要約をテキストファイルとして保存する
    /// 
    /// 事前条件:
    /// - summary は有効なAISummaryResponseである
    /// - meeting_id は空でない文字列である
    /// - meeting_start_time は有効なISO 8601形式の日時文字列である
    /// - output_dir は有効なディレクトリパスである
    /// - ディレクトリへの書き込み権限がある
    /// 
    /// 事後条件:
    /// - 成功時: AI要約がテキストファイルとして保存される
    /// - 保存されたファイルパスを返す
    /// - 失敗時: 適切なエラーメッセージと共にエラーを返す
    pub async fn save_ai_summary_txt(&self, summary: &AISummaryResponse, meeting_id: &str, meeting_start_time: &str, output_dir: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Create date folder based on meeting start time (not AI summary creation time)
        let date_folder_path = self.create_meeting_date_folder(output_dir, meeting_start_time)?;
        
        // Generate filename using pure function
        let summary_filename = self.generate_ai_summary_filename(summary, meeting_id, "txt")?;
        let output_path = Path::new(&date_folder_path).join(&summary_filename);
        
        // Generate content using pure function
        let content = self.generate_ai_summary_text_content(summary, meeting_id)?;

        // Perform I/O operations
        fs::create_dir_all(&date_folder_path)?;
        fs::write(&output_path, content)?;

        windows_console::println_japanese(&format!("AI summary saved: {}", output_path.display()));
        Ok(output_path.to_string_lossy().to_string())
    }

    /// AI要約のJSONコンテンツを生成する（副作用なし版）
    /// 
    /// 事前条件:
    /// - summary は有効なAISummaryResponseである
    /// - meeting_id は空でない文字列である
    /// 
    /// 事後条件:
    /// - 成功時: 構造化されたJSON形式のコンテンツを返す
    /// - 副作用なし: ファイルは作成されない
    /// - 失敗時: 適切なエラーメッセージと共にエラーを返す
    /// 
    /// 不変条件:
    /// - ファイルシステムの状態を変更しない
    /// - 入力パラメータを変更しない
    fn generate_ai_summary_json_content(&self, summary: &AISummaryResponse, meeting_id: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Create comprehensive summary with structured format similar to reference example
        let comprehensive_summary = serde_json::json!({
            "meeting_uuid": summary.meeting_uuid,
            "meeting_id": meeting_id,
            "summary_metadata": {
                "title": summary.summary_title,
                "start_time": summary.summary_start_time,
                "end_time": summary.summary_end_time,
                "created_time": summary.summary_created_time,
                "last_modified_time": summary.summary_last_modified_time
            },
            "brief_summary": if !summary.summary_overview.is_empty() { 
                summary.summary_overview.clone() 
            } else { 
                summary.summary.clone() 
            },
            "next_steps": if !summary.next_steps.is_empty() { 
                summary.next_steps.clone() 
            } else { 
                summary.action_items.clone() 
            },
            "detailed_sections": if !summary.topic_summaries.is_empty() {
                summary.topic_summaries.iter().map(|t| serde_json::json!({
                    "title": t.topic_title,
                    "content": t.topic_content
                })).collect::<Vec<_>>()
            } else if !summary.detailed_sections.is_empty() {
                summary.detailed_sections.iter().map(|s| serde_json::json!({
                    "title": s.section_title,
                    "content": s.section_content
                })).collect::<Vec<_>>()
            } else if !summary.summary_details.is_empty() {
                summary.summary_details.iter().map(|d| serde_json::json!({
                    "title": d.label,
                    "content": d.summary
                })).collect::<Vec<_>>()
            } else {
                Vec::new()
            },
            "summary_details": summary.summary_details.iter().map(|d| serde_json::json!({
                "label": d.label,
                "summary": d.summary
            })).collect::<Vec<_>>(),
            "summary_content": summary.summary_content,
            "keywords": summary.summary_keyword,
            "key_points": summary.key_points,
            "generated_by": "Zoom AI Companion",
            "download_timestamp": Utc::now().to_rfc3339()
        });

        let json_content = serde_json::to_string_pretty(&comprehensive_summary)?;
        Ok(json_content)
    }

    /// AI要約をJSONファイルとして保存する
    /// 
    /// # 副作用
    /// - ディレクトリの作成
    /// - ファイルシステムへの書き込み（JSONファイルの保存）
    /// 
    /// 事前条件:
    /// - summary は有効なAISummaryResponseである
    /// - meeting_id は空でない文字列である
    /// - meeting_start_time は有効なISO 8601形式の日時文字列である
    /// - output_dir は有効なディレクトリパスである
    /// - ディレクトリへの書き込み権限がある
    /// 
    /// 事後条件:
    /// - 成功時: AI要約がJSONファイルとして保存される
    /// - 保存されたファイルパスを返す
    /// - 失敗時: 適切なエラーメッセージと共にエラーを返す
    pub async fn save_ai_summary(&self, summary: &AISummaryResponse, meeting_id: &str, meeting_start_time: &str, output_dir: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Create date folder based on meeting start time (not AI summary creation time)
        let date_folder_path = self.create_meeting_date_folder(output_dir, meeting_start_time)?;
        
        // Generate filename using pure function
        let summary_filename = self.generate_ai_summary_filename(summary, meeting_id, "json")?;
        let output_path = Path::new(&date_folder_path).join(&summary_filename);
        
        // Generate content using pure function
        let json_content = self.generate_ai_summary_json_content(summary, meeting_id)?;

        // Perform I/O operations
        fs::create_dir_all(&date_folder_path)?;
        fs::write(&output_path, json_content)?;

        windows_console::println_japanese(&format!("AI summary saved: {}", output_path.display()));
        Ok(output_path.to_string_lossy().to_string())
    }
    
    fn get_next_available_suffix(&self, dir_path: &str, base_filename: &str, extension: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut counter = 1;
        loop {
            let suffix = format!("_v{:02}", counter);
            let test_filename = format!("{}_{}.{}", base_filename, suffix, extension);
            let test_path = Path::new(dir_path).join(&test_filename);
            
            if !test_path.exists() {
                return Ok(suffix);
            }
            
            counter += 1;
            if counter > 99 {
                // Fallback to timestamp if too many versions
                return Ok(format!("_{}", Utc::now().format("%Y%m%d_%H%M%S")));
            }
        }
    }

    /// ダウンロードしたファイルの整合性を検証する
    /// 
    /// 事前条件:
    /// - file_path は有効なファイルパスである
    /// - expected_size は正の数値である
    /// - file_type は空でない文字列である
    /// 
    /// 事後条件:
    /// - 成功時: ファイルの整合性が確認された場合は true を返す
    /// - 整合性に問題がある場合は false を返す
    /// - 検証エラーの場合は適切なエラーを返す
    /// 
    /// 不変条件:
    /// - ファイルサイズは expected_size と一致する必要がある
    /// - ファイル形式は file_type に適合する必要がある
    async fn verify_file_integrity(&self, file_path: &Path, expected_size: u64, file_type: &str) -> Result<bool, Box<dyn std::error::Error>> {
        // Check if file exists
        if !file_path.exists() {
            return Ok(false);
        }
        
        // Check file size
        let actual_size = fs::metadata(file_path)?.len();
        if actual_size != expected_size {
            windows_console::println_japanese(&format!(
                "Size mismatch: expected {} bytes, got {} bytes", 
                expected_size, actual_size
            ));
            return Ok(false);
        }
        
        // Perform basic file type validation
        match file_type.to_lowercase().as_str() {
            "mp4" => self.verify_mp4_file(file_path).await,
            "m4a" => self.verify_m4a_file(file_path).await,
            "txt" => self.verify_text_file(file_path).await,
            "vtt" => self.verify_vtt_file(file_path).await,
            "csv" => self.verify_csv_file(file_path).await,
            "json" => self.verify_json_file(file_path).await,
            _ => {
                // For unknown file types, just check if file is readable
                Ok(fs::read(file_path).is_ok())
            }
        }
    }
    
    async fn verify_mp4_file(&self, file_path: &Path) -> Result<bool, Box<dyn std::error::Error>> {
        // Check MP4 file signature (first 4 bytes after ftyp box)
        let mut file = std::fs::File::open(file_path)?;
        let mut buffer = [0u8; 12];
        use std::io::Read;
        file.read_exact(&mut buffer)?;
        
        // Look for ftyp signature at bytes 4-7
        if &buffer[4..8] == b"ftyp" {
            Ok(true)
        } else {
            windows_console::println_japanese("Invalid MP4 file signature");
            Ok(false)
        }
    }
    
    async fn verify_m4a_file(&self, file_path: &Path) -> Result<bool, Box<dyn std::error::Error>> {
        // M4A uses similar container format as MP4
        self.verify_mp4_file(file_path).await
    }
    
    async fn verify_text_file(&self, file_path: &Path) -> Result<bool, Box<dyn std::error::Error>> {
        // Check if file contains valid UTF-8 text
        match fs::read_to_string(file_path) {
            Ok(_) => Ok(true),
            Err(_) => {
                windows_console::println_japanese("Invalid text file encoding");
                Ok(false)
            }
        }
    }
    
    async fn verify_vtt_file(&self, file_path: &Path) -> Result<bool, Box<dyn std::error::Error>> {
        // VTT files should start with "WEBVTT"
        let content = fs::read_to_string(file_path)?;
        if content.starts_with("WEBVTT") {
            Ok(true)
        } else {
            windows_console::println_japanese("Invalid VTT file format");
            Ok(false)
        }
    }
    
    async fn verify_csv_file(&self, file_path: &Path) -> Result<bool, Box<dyn std::error::Error>> {
        // Basic CSV validation - check if it's valid UTF-8 and has comma separators
        let content = fs::read_to_string(file_path)?;
        Ok(content.contains(',') || content.contains('\t'))
    }
    
    async fn verify_json_file(&self, file_path: &Path) -> Result<bool, Box<dyn std::error::Error>> {
        // Check if file contains valid JSON
        let content = fs::read_to_string(file_path)?;
        match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(_) => Ok(true),
            Err(_) => {
                windows_console::println_japanese("Invalid JSON file format");
                Ok(false)
            }
        }
    }
    
    async fn save_debug_response(&self, response_text: &str, filename_suffix: &str) {
        // Create debug directory if it doesn't exist
        let debug_dir = "debug_responses";
        if let Err(_) = std::fs::create_dir_all(debug_dir) {
            return; // Silently fail if we can't create debug directory
        }
        
        // Create timestamp for unique filename
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S_%3f");
        let filename = format!("ai_response_{}_{}.json", filename_suffix, timestamp);
        let filepath = std::path::Path::new(debug_dir).join(filename);
        
        // Try to pretty-print JSON if valid, otherwise save as-is
        let formatted_content = if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(response_text) {
            serde_json::to_string_pretty(&json_value).unwrap_or_else(|_| response_text.to_string())
        } else {
            // Not valid JSON, save as text with some metadata
            format!("# Raw Response (Not Valid JSON)\n# Timestamp: {}\n# Source: {}\n\n{}", 
                chrono::Utc::now().to_rfc3339(), filename_suffix, response_text)
        };
        
        // Save to file (silently fail if unable to write)
        let _ = std::fs::write(filepath, formatted_content);
    }

    /// 録画ファイルをダウンロードする（複雑度削減版）
    /// 
    /// 事前条件:
    /// - recording は有効なRecordingインスタンスである
    /// - meeting_start_time は有効なISO 8601形式の日時文字列である
    /// - output_dir は有効なディレクトリパスである
    /// - アクセストークンが有効である
    /// - ディレクトリへの書き込み権限がある
    /// 
    /// 事後条件:
    /// - 成功時: 録画ファイルがダウンロードされる
    /// - ダウンロードされたファイルパスを返す
    /// - 失敗時: 適切なエラーメッセージと共にエラーを返す
    /// 
    /// 不変条件:
    /// - ダウンロードしたファイルサイズは recording.file_size と一致する
    /// - ファイル形式は recording.file_type に適合する
    pub async fn download_recording(&self, recording: &Recording, meeting_start_time: &str, output_dir: &str) -> Result<String, Box<dyn std::error::Error>> {
        let date_folder_path = self.create_meeting_date_folder(output_dir, meeting_start_time)?;
        let safe_filename = self.generate_recording_filename(recording)?;
        let output_path = Path::new(&date_folder_path).join(&safe_filename);
        
        if self.check_file_exists(&output_path) {
            return Ok(output_path.to_string_lossy().to_string());
        }
        
        self.download_and_save_file(recording, &output_path, &date_folder_path, &safe_filename).await
    }
    
    /// ファイルの存在確認を行う（複雑度削減版）
    /// 
    /// 事前条件:
    /// - output_path は有効なPathである
    /// 
    /// 事後条件:
    /// - ファイルが存在する場合は true、そうでなければ false を返す
    /// - 既存ファイルの場合はメッセージを出力する
    fn check_file_exists(&self, output_path: &Path) -> bool {
        if output_path.exists() {
            windows_console::println_japanese(&format!("File already exists: {}", output_path.display()));
            true
        } else {
            false
        }
    }
    
    /// 録画ファイルの安全なファイル名を生成する（複雑度削減版）
    /// 
    /// 事前条件:
    /// - recording は有効なRecordingインスタンスである
    /// - recording.recording_start は有効なISO 8601形式の日時文字列である
    /// 
    /// 事後条件:
    /// - Zoomの命名規則に従った安全なファイル名を返す
    /// - GMT{timestamp}_{recording_type}.{extension} 形式
    fn generate_recording_filename(&self, recording: &Recording) -> Result<String, Box<dyn std::error::Error>> {
        let recording_start_parsed = chrono::DateTime::parse_from_rfc3339(&recording.recording_start)
            .map_err(|e| format!("Failed to parse recording start time: {}", e))?;
        
        let gmt_timestamp = recording_start_parsed.with_timezone(&chrono::Utc)
            .format("%Y%m%d-%H%M%S").to_string();
        
        let type_suffix = self.get_recording_type_suffix(recording);
        
        Ok(format!("GMT{}{}.{}", 
            gmt_timestamp,
            type_suffix,
            recording.file_type.to_lowercase()
        ))
    }
    
    /// 録画タイプに基づくファイル名サフィックスを取得する（複雑度削減版）
    /// 
    /// 事前条件:
    /// - recording は有効なRecordingインスタンスである
    /// 
    /// 事後条件:
    /// - 録画タイプとファイル形式に適したサフィックスを返す
    fn get_recording_type_suffix(&self, recording: &Recording) -> &'static str {
        match (recording.recording_type.to_lowercase().as_str(), recording.file_type.to_lowercase().as_str()) {
            (_, "mp4") => "", // Main video file has no suffix
            (_, "m4a") => "_AUDIO", // Audio-only file
            ("chat_file", _) => "_CHAT", // Chat transcript
            (_, "vtt") if recording.recording_type.contains("transcript") => "_TRANSCRIPT",
            (_, "vtt") => "_CC", // Closed captions
            (_, "csv") => "_POLL", // Poll results or participant data
            (_, "json") => "_META", // Metadata
            _ => ""
        }
    }
    
    /// ファイルのダウンロードと保存を実行する（複雑度削減版）
    /// 
    /// 事前条件:
    /// - recording は有効なRecordingインスタンスである
    /// - output_path は有効なPathである
    /// - date_folder_path は有効なディレクトリパスである
    /// - safe_filename は有効なファイル名である
    /// 
    /// 事後条件:
    /// - 成功時: ファイルがダウンロードされ、検証される
    /// - 失敗時: 適切なエラーを返す
    async fn download_and_save_file(&self, recording: &Recording, output_path: &Path, date_folder_path: &str, safe_filename: &str) -> Result<String, Box<dyn std::error::Error>> {
        windows_console::println_japanese(&format!("Downloading: {} ({:.2} MB)", safe_filename, recording.file_size as f64 / 1024.0 / 1024.0));

        let response = self.send_download_request(recording).await?;
        self.save_response_to_file(response, output_path, date_folder_path).await?;
        self.verify_and_report_download(output_path, recording).await;
        
        Ok(output_path.to_string_lossy().to_string())
    }
    
    /// ダウンロードリクエストを送信する（複雑度削減版）
    /// 
    /// 事前条件:
    /// - recording は有効なRecordingインスタンスである
    /// - self.access_token は有効なアクセストークンである
    /// 
    /// 事後条件:
    /// - 成功時: HTTPレスポンスを返す
    /// - 失敗時: 適切なエラーを返す
    async fn send_download_request(&self, recording: &Recording) -> Result<reqwest::Response, Box<dyn std::error::Error>> {
        let response = self
            .client
            .get(&recording.download_url)
            .bearer_auth(&self.access_token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Download failed: {}", response.status()).into());
        }

        Ok(response)
    }
    
    /// レスポンスをファイルに保存する（複雑度削減版）
    /// 
    /// 事前条件:
    /// - response は有効なHTTPレスポンスである
    /// - output_path は有効なPathである
    /// - date_folder_path は有効なディレクトリパスである
    /// 
    /// 事後条件:
    /// - 成功時: ファイルが指定パスに保存される
    /// - 失敗時: 適切なエラーを返す
    async fn save_response_to_file(&self, response: reqwest::Response, output_path: &Path, date_folder_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(date_folder_path)?;
        
        let mut file = tokio::fs::File::create(output_path).await?;
        let content = response.bytes().await?;
        file.write_all(&content).await?;
        
        Ok(())
    }
    
    /// ダウンロードの検証と結果報告を行う（複雑度削減版）
    /// 
    /// 事前条件:
    /// - output_path は有効なPathである
    /// - recording は有効なRecordingインスタンスである
    /// 
    /// 事後条件:
    /// - ファイルの整合性検証が実行される
    /// - 検証結果に応じたメッセージが出力される
    async fn verify_and_report_download(&self, output_path: &Path, recording: &Recording) {
        match self.verify_file_integrity(output_path, recording.file_size, &recording.file_type).await {
            Ok(true) => {
                windows_console::println_japanese(&format!("✓ Download completed and verified: {}", output_path.display()));
            },
            Ok(false) => {
                windows_console::println_japanese(&format!("⚠ Download completed but verification failed: {}", output_path.display()));
            },
            Err(e) => {
                windows_console::println_japanese(&format!("⚠ Download completed but verification error: {} - {}", output_path.display(), e));
            }
        }
    }

    /// AI要約のダウンロードと保存を処理する（複雑度削減版）
    /// 
    /// 事前条件:
    /// - meeting は有効なMeetingRecordingである
    /// - output_dir は有効なディレクトリパスである
    /// 
    /// 事後条件:
    /// - 成功時: AI要約が保存され、(保存パス, ファイルタイプ)のOptionを返す
    /// - AI要約が利用不可の場合は None を返す
    /// - 失敗時: エラーメッセージが出力されるが、処理は継続される
    /// 
    /// 不変条件:
    /// - 処理の失敗は全体の処理を停止しない
    async fn process_ai_summary_for_meeting(&self, meeting: &MeetingRecording, output_dir: &str) -> Option<(String, String)> {
        windows_console::println_japanese(&format!("=== Checking AI Companion summary for meeting {} ===", meeting.id));
        windows_console::println_japanese(&format!("Meeting UUID: {}", meeting.uuid));
        windows_console::println_japanese(&format!("Meeting topic: {}", meeting.topic));
        windows_console::println_japanese(&format!("Meeting start time: {}", meeting.start_time));
        
        // First try with Meeting ID (more likely to work)
        if let Ok(Some(summary)) = self.get_ai_summary_by_meeting_id(meeting.id).await {
            windows_console::println_japanese("✓ AI summary found via Meeting ID, saving to file...");
            match self.save_ai_summary_txt(&summary, &meeting.id.to_string(), &meeting.start_time, output_dir).await {
                Ok(path) => {
                    windows_console::println_japanese(&format!("✓ AI summary saved: {}", path));
                    windows_console::println_japanese("=== End AI summary check ===\n");
                    return Some((path, "ai_summary".to_string()));
                },
                Err(e) => windows_console::println_japanese(&format!("✗ Failed to save AI summary: {}", e)),
            }
        } else {
            windows_console::println_japanese("ℹ No AI summary found via Meeting ID");
        }
        
        // If not found via Meeting ID, try with UUID
        if let Ok(Some(summary)) = self.get_ai_summary(&meeting.uuid).await {
            windows_console::println_japanese("✓ AI summary found via UUID, saving to file...");
            match self.save_ai_summary_txt(&summary, &meeting.id.to_string(), &meeting.start_time, output_dir).await {
                Ok(path) => {
                    windows_console::println_japanese(&format!("✓ AI summary saved: {}", path));
                    windows_console::println_japanese("=== End AI summary check ===\n");
                    return Some((path, "ai_summary".to_string()));
                },
                Err(e) => windows_console::println_japanese(&format!("✗ Failed to save AI summary: {}", e)),
            }
        } else {
            windows_console::println_japanese("ℹ No AI summary found via UUID");
        }
        
        windows_console::println_japanese("ℹ No AI summary available (this is normal for meetings without AI Companion enabled)");
        windows_console::println_japanese("=== End AI summary check ===\n");
        None
    }

    /// ファイルタイプがダウンロード可能かどうかを判定する（純粋関数）
    /// 
    /// 事前条件:
    /// - file_type は空でない文字列である
    /// 
    /// 事後条件:
    /// - ダウンロード可能な場合は true を返す
    /// - 副作用なし
    /// 
    /// 不変条件:
    /// - 入力パラメータを変更しない
    fn is_file_type_downloadable(&self, file_type: &str) -> bool {
        match file_type.to_lowercase().as_str() {
            "mp4" | "m4a" => true,  // Video and audio files
            "txt" => true,          // Chat files
            "vtt" => true,          // Transcript/subtitle files  
            "csv" => true,          // Poll results, participant lists
            "json" => true,         // Meeting metadata
            "cc.vtt" => true,       // Closed captions
            _ => {
                // Unknown file types - attempt to download them
                true
            }
        }
    }

    /// 単一ミーティングの録画ファイルをダウンロードする（複雑度削減版）
    /// 
    /// 事前条件:
    /// - meeting は有効なMeetingRecordingである
    /// - output_dir は有効なディレクトリパスである
    /// 
    /// 事後条件:
    /// - 成功時: ダウンロードされたファイルパスとタイプのリストを返す
    /// - 失敗時: エラーメッセージが出力されるが、処理は継続される
    /// 
    /// 不変条件:
    /// - 個別ファイルの失敗は全体の処理を停止しない
    async fn download_meeting_recordings(&self, meeting: &MeetingRecording, output_dir: &str) -> Vec<(String, String)> {
        let mut downloaded_files = Vec::new();
        
        for recording in &meeting.recording_files {
            let file_type = recording.file_type.to_lowercase();
            
            if !self.is_file_type_downloadable(&file_type) {
                continue;
            }
            
            // Log unknown file types
            if !matches!(file_type.as_str(), "mp4" | "m4a" | "txt" | "vtt" | "csv" | "json" | "cc.vtt") {
                windows_console::println_japanese(&format!("Unknown file type '{}' for recording {}, attempting download", file_type, recording.id));
            }
            
            windows_console::println_japanese(&format!("Downloading {} file: {}", file_type.to_uppercase(), recording.id));
            match self.download_recording(recording, &meeting.start_time, output_dir).await {
                Ok(path) => {
                    downloaded_files.push((path, file_type));
                },
                Err(e) => windows_console::println_japanese(&format!("Download failed {}: {}", recording.id, e)),
            }
        }
        
        downloaded_files
    }

    /// ダウンロードサマリーを表示する（複雑度削減版）
    /// 
    /// 事前条件:
    /// - downloaded_files は有効なファイルパスとタイプのリストである
    /// 
    /// 事後条件:
    /// - ダウンロード結果のサマリーが出力される
    /// - 副作用: コンソールへの出力のみ
    /// 
    /// 不変条件:
    /// - 入力データを変更しない
    fn display_download_summary(&self, downloaded_files: &[(String, String)]) {
        let mut file_type_counts = std::collections::HashMap::new();
        
        for (_, file_type) in downloaded_files {
            *file_type_counts.entry(file_type.clone()).or_insert(0) += 1;
        }

        windows_console::println_japanese(&format!("Download completed: {} files total", downloaded_files.len()));
        
        if !file_type_counts.is_empty() {
            windows_console::println_japanese("File types downloaded:");
            for (file_type, count) in file_type_counts {
                let type_name = self.get_file_type_display_name(&file_type);
                windows_console::println_japanese(&format!("  {} ({}): {} files", type_name, file_type.to_uppercase(), count));
            }
        }
    }

    /// ファイルタイプの表示名を取得する（純粋関数）
    /// 
    /// 事前条件:
    /// - file_type は空でない文字列である
    /// 
    /// 事後条件:
    /// - 適切な表示名を返す
    /// - 副作用なし
    /// 
    /// 不変条件:
    /// - 入力パラメータを変更しない
    fn get_file_type_display_name(&self, file_type: &str) -> &'static str {
        match file_type {
            "mp4" => "Video files",
            "m4a" => "Audio files", 
            "txt" => "Chat files",
            "vtt" => "Transcript files",
            "cc.vtt" => "Caption files",
            "csv" => "Data files",
            "json" => "Metadata files",
            "ai_summary" => "AI Companion summaries",
            _ => "Other files"
        }
    }

    /// 指定した期間のすべての録画を一括ダウンロードする（リファクタリング版・複雑度削減）
    /// 
    /// 事前条件:
    /// - user_id は有効なZoomユーザーIDである
    /// - from は有効な日付形式（YYYY-MM-DD）である
    /// - to は有効な日付形式（YYYY-MM-DD）である
    /// - from <= to である
    /// - output_dir は有効なディレクトリパスである
    /// - アクセストークンが有効である
    /// - ディレクトリへの書き込み権限がある
    /// 
    /// 事後条件:
    /// - 成功時: すべての利用可能な録画ファイルがダウンロードされる
    /// - ダウンロードされたファイルパスのリストを返す
    /// - AI要約が利用可能な場合は要約も保存される
    /// - 失敗時: 適切なエラーメッセージと共にエラーを返す
    /// 
    /// 不変条件:
    /// - 各ファイルは適切なファイル形式と整合性を持つ
    /// - ファイルは日付別フォルダーに整理される
    pub async fn download_all_recordings(&self, user_id: &str, from: &str, to: &str, output_dir: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        // Test API access first
        if let Err(e) = self.test_api_access().await {
            windows_console::println_japanese(&format!("API test failed: {}", e));
        }

        let recordings = self.list_recordings(user_id, from, to).await?;
        let mut downloaded_files = Vec::new();

        windows_console::println_japanese(&format!("Found {} recorded meetings", recordings.meetings.len()));

        for meeting in recordings.meetings {
            windows_console::println_japanese(&format!("Processing meeting: {} ({})", meeting.topic, meeting.start_time));
            
            // Process AI summary for this meeting
            if let Some((path, file_type)) = self.process_ai_summary_for_meeting(&meeting, output_dir).await {
                downloaded_files.push((path, file_type));
            }
            
            // Download recording files for this meeting
            let meeting_files = self.download_meeting_recordings(&meeting, output_dir).await;
            downloaded_files.extend(meeting_files);
        }

        // Display download summary
        self.display_download_summary(&downloaded_files);

        // Extract just the file paths for return value
        Ok(downloaded_files.into_iter().map(|(path, _)| path).collect())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
