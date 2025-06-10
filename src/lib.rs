// OAuth2関連のimportは現在使用されていませんが、
// 今後の機能拡張のために残しておきます
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tokio::io::AsyncWriteExt;

pub mod windows_console;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: Option<String>,
}

impl Config {
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

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

pub struct ZoomRecordingDownloader {
    client: Client,
    access_token: String,
}

impl ZoomRecordingDownloader {
    pub fn new(access_token: String) -> Self {
        Self {
            client: Client::new(),
            access_token,
        }
    }

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

    pub async fn download_recording(&self, recording: &Recording, output_dir: &str) -> Result<String, Box<dyn std::error::Error>> {
        let invalid_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
        let safe_filename = format!("{}_{}_{}.{}", 
            recording.meeting_id.chars().map(|c| if invalid_chars.contains(&c) { '_' } else { c }).collect::<String>(),
            recording.recording_type.chars().map(|c| if invalid_chars.contains(&c) { '_' } else { c }).collect::<String>(),
            recording.id.chars().map(|c| if invalid_chars.contains(&c) { '_' } else { c }).collect::<String>(),
            recording.file_type.to_lowercase()
        );
        
        let output_path = Path::new(output_dir).join(&safe_filename);
        
        if output_path.exists() {
            windows_console::println_japanese(&format!("ファイルは既に存在しています: {}", output_path.display()));
            return Ok(output_path.to_string_lossy().to_string());
        }

        windows_console::println_japanese(&format!("ダウンロード中: {} ({:.2} MB)", safe_filename, recording.file_size as f64 / 1024.0 / 1024.0));

        let response = self
            .client
            .get(&recording.download_url)
            .bearer_auth(&self.access_token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Download failed: {}", response.status()).into());
        }

        fs::create_dir_all(output_dir)?;
        
        let mut file = tokio::fs::File::create(&output_path).await?;
        let content = response.bytes().await?;
        file.write_all(&content).await?;

        windows_console::println_japanese(&format!("ダウンロード完了: {}", output_path.display()));
        Ok(output_path.to_string_lossy().to_string())
    }

    pub async fn download_all_recordings(&self, user_id: &str, from: &str, to: &str, output_dir: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let recordings = self.list_recordings(user_id, from, to).await?;
        let mut downloaded_files = Vec::new();

        windows_console::println_japanese(&format!("{}個の録画ミーティングが見つかりました", recordings.meetings.len()));

        for meeting in recordings.meetings {
            windows_console::println_japanese(&format!("ミーティングを処理中: {} ({})", meeting.topic, meeting.start_time));
            
            for recording in meeting.recording_files {
                if recording.file_type.to_lowercase() == "mp4" || recording.file_type.to_lowercase() == "m4a" {
                    match self.download_recording(&recording, output_dir).await {
                        Ok(path) => downloaded_files.push(path),
                        Err(e) => windows_console::println_japanese(&format!("ダウンロード失敗 {}: {}", recording.id, e)),
                    }
                }
            }
        }

        Ok(downloaded_files)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
