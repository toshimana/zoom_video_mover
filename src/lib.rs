use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    RedirectUrl, Scope, TokenUrl, AccessToken, AuthorizationCode, PkceCodeVerifier,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tokio::io::AsyncWriteExt;

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
        let filename = format!("{}_{}_{}.{}", 
            recording.meeting_id, 
            recording.recording_type,
            recording.id,
            recording.file_type.to_lowercase()
        );
        
        let output_path = Path::new(output_dir).join(&filename);
        
        if output_path.exists() {
            println!("File already exists: {}", output_path.display());
            return Ok(output_path.to_string_lossy().to_string());
        }

        println!("Downloading: {} ({:.2} MB)", filename, recording.file_size as f64 / 1024.0 / 1024.0);

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

        println!("Downloaded: {}", output_path.display());
        Ok(output_path.to_string_lossy().to_string())
    }

    pub async fn download_all_recordings(&self, user_id: &str, from: &str, to: &str, output_dir: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let recordings = self.list_recordings(user_id, from, to).await?;
        let mut downloaded_files = Vec::new();

        println!("Found {} meetings with recordings", recordings.meetings.len());

        for meeting in recordings.meetings {
            println!("Processing meeting: {} ({})", meeting.topic, meeting.start_time);
            
            for recording in meeting.recording_files {
                if recording.file_type.to_lowercase() == "mp4" || recording.file_type.to_lowercase() == "m4a" {
                    match self.download_recording(&recording, output_dir).await {
                        Ok(path) => downloaded_files.push(path),
                        Err(e) => eprintln!("Failed to download {}: {}", recording.id, e),
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
