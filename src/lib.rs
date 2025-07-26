use std::fs;
use serde::{Deserialize, Serialize};
use reqwest::Client;

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
        // 事前条件のassertion
        assert!(!path.is_empty(), "path must not be empty");
        debug_assert!(path.len() > 0, "path should have reasonable length");
        
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        
        // 事後条件のassertion
        debug_assert!(!config.client_id.is_empty(), "loaded config must have valid client_id");
        debug_assert!(!config.client_secret.is_empty(), "loaded config must have valid client_secret");
        
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
        // 事前条件のassertion
        assert!(!path.is_empty(), "path must not be empty");
        debug_assert!(path.len() > 0, "path should have reasonable length");
        
        let sample_config = Config {
            client_id: "your_zoom_client_id".to_string(),
            client_secret: "your_zoom_client_secret".to_string(),
            redirect_uri: Some("http://localhost:8080/callback".to_string()),
        };
        
        let content = toml::to_string_pretty(&sample_config)?;
        
        // 事後条件のassertion（書き込み前にコンテンツの妥当性確認）
        debug_assert!(!content.is_empty(), "generated TOML content must not be empty");
        debug_assert!(content.contains("client_id"), "TOML must contain client_id field");
        
        fs::write(path, content)?;
        
        // 事後条件のassertion（ファイル作成確認）
        debug_assert!(std::path::Path::new(path).exists(), "file should be created successfully");
        
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
        // 事前条件のassertion
        assert!(!path.is_empty(), "path must not be empty");
        assert!(!self.client_id.is_empty(), "client_id must not be empty");
        assert!(!self.client_secret.is_empty(), "client_secret must not be empty");
        
        let content = toml::to_string_pretty(self)?;
        
        // 事後条件のassertion（書き込み前にコンテンツの妥当性確認）
        debug_assert!(!content.is_empty(), "generated TOML content must not be empty");
        debug_assert!(content.contains(&self.client_id), "TOML must contain the client_id");
        
        fs::write(path, content)?;
        
        // 事後条件のassertion（ファイル作成確認）
        debug_assert!(std::path::Path::new(path).exists(), "file should be created successfully");
        
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Recording {
    pub id: String,
    pub download_url: String,
    pub file_type: String,
    pub file_size: u64,
    pub recording_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MeetingRecording {
    pub uuid: String,
    pub id: u64,
    pub topic: String,
    pub start_time: String,
    pub recording_files: Vec<Recording>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordingResponse {
    pub meetings: Vec<MeetingRecording>,
    pub page_count: u32,
    pub page_size: u32,
    pub total_records: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicSummary {
    pub name: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedSection {
    pub section_name: String,
    pub section_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryDetail {
    pub detail_type: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AISummaryResponse {
    pub meeting_uuid: String,
    pub summary_start_time: String,
    pub summary_end_time: String,
    pub summary_created_time: String,
    pub summary_last_modified_time: String,
    pub meeting_topic: String,
    pub meeting_id: String,
    pub meeting_host: String,
    pub summary_created_by: String,
    pub summary_title: String,
    pub summary_overview: String,
    pub summary_keywords: Vec<String>,
    pub summary_details: Vec<SummaryDetail>,
    pub detailed_sections: Vec<DetailedSection>,
    pub topic_summaries: Vec<TopicSummary>,
    pub next_steps: Vec<String>,
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
        // 事前条件のassertion
        assert!(!access_token.is_empty(), "access_token must not be empty");
        debug_assert!(access_token.len() > 10, "access_token should be reasonable length");
        
        let instance = Self {
            client: Client::new(),
            access_token,
        };
        
        // 事後条件のassertion
        debug_assert!(!instance.access_token.is_empty(), "instance should have valid access_token");
        
        instance
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
        // 事前条件のassertion
        assert!(!self.access_token.is_empty(), "access_token must be valid");
        debug_assert!(self.access_token.len() > 10, "access_token should be reasonable length");
        
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
        
        // 事後条件のassertion
        debug_assert!(!messages.is_empty(), "messages should not be empty");
        debug_assert!(messages.len() >= 2, "messages should contain at least start and end");
        debug_assert!(messages[0].contains("Testing Zoom API Access"), "first message should be test start");
        
        Ok(messages)
    }

    /// API アクセスをテストし、結果を標準出力に表示する
    /// 
    /// # 副作用
    /// - HTTPリクエストの送信
    /// - 標準出力へのメッセージ表示
    pub async fn test_api_access(&self) -> Result<(), Box<dyn std::error::Error>> {
        let messages = self.test_api_access_pure().await?;
        for message in messages {
            crate::windows_console::println_japanese(&message);
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
        // 事前条件のassertion
        assert!(!user_id.is_empty(), "user_id must not be empty");
        assert!(!from.is_empty(), "from date must not be empty");
        assert!(!to.is_empty(), "to date must not be empty");
        debug_assert!(from.len() == 10, "from date should be YYYY-MM-DD format");
        debug_assert!(to.len() == 10, "to date should be YYYY-MM-DD format");
        debug_assert!(from <= to, "from date should be earlier than or equal to to date");
        
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
        
        // 事後条件のassertion
        // recordings.meetings.len() is always >= 0 for Vec, so no assertion needed
        // 各レコーディングの妥当性チェック
        for meeting in &recordings.meetings {
            debug_assert!(!meeting.uuid.is_empty(), "meeting UUID should not be empty");
            debug_assert!(meeting.id > 0, "meeting ID should be positive");
        }
        
        Ok(recordings)
    }

    // 他の関数は簡略化のため省略...
}

pub mod windows_console {
    pub fn println_japanese(text: &str) {
        println!("{}", text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}