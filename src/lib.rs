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
    pub summary_type: String,
    #[serde(default)]
    pub summary_content: String,
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
    pub fn new(access_token: String) -> Self {
        Self {
            client: Client::new(),
            access_token,
        }
    }

    // Diagnostic method to test API connectivity and permissions
    pub async fn test_api_access(&self) -> Result<(), Box<dyn std::error::Error>> {
        windows_console::println_japanese("=== Testing Zoom API Access ===");
        
        // Test basic user info API
        let user_response = self
            .client
            .get("https://api.zoom.us/v2/users/me")
            .bearer_auth(&self.access_token)
            .send()
            .await?;

        windows_console::println_japanese(&format!("User API status: {}", user_response.status()));
        
        if user_response.status().is_success() {
            if let Ok(user_data) = user_response.json::<serde_json::Value>().await {
                if let Some(user_id) = user_data.get("id").and_then(|v| v.as_str()) {
                    windows_console::println_japanese(&format!("✓ Connected as user: {}", user_id));
                }
                if let Some(account_id) = user_data.get("account_id").and_then(|v| v.as_str()) {
                    windows_console::println_japanese(&format!("Account ID: {}", account_id));
                }
            }
        } else {
            windows_console::println_japanese(&format!("✗ User API failed: {}", user_response.status()));
        }

        windows_console::println_japanese("=== End API Test ===\n");
        Ok(())
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

    pub async fn get_ai_summary_by_meeting_id(&self, meeting_id: u64) -> Result<Option<AISummaryResponse>, Box<dyn std::error::Error>> {
        windows_console::println_japanese(&format!("Requesting AI summary for Meeting ID: {}", meeting_id));
        
        // Try AI summary endpoints using meeting ID
        let endpoints = vec![
            format!("https://api.zoom.us/v2/meetings/{}/batch_summary", meeting_id),
            format!("https://api.zoom.us/v2/meetings/{}/summary", meeting_id),
            format!("https://api.zoom.us/v2/meetings/{}/ai_companion_summary", meeting_id),
            format!("https://api.zoom.us/v2/meetings/{}/recording_summary", meeting_id),
            format!("https://api.zoom.us/v2/meetings/{}/meeting_summary", meeting_id),
            format!("https://api.zoom.us/v2/ai_companion/meetings/{}/summary", meeting_id),
            format!("https://api.zoom.us/v2/ai_companion/summary/{}", meeting_id),
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
                    
                    if let Ok(summary) = serde_json::from_str::<AISummaryResponse>(&response_text) {
                        windows_console::println_japanese("Successfully parsed AI summary!");
                        return Ok(Some(summary));
                    } else if let Ok(generic_json) = serde_json::from_str::<serde_json::Value>(&response_text) {
                        windows_console::println_japanese("Received valid JSON, converting to AI summary format");
                        let converted_summary = self.convert_generic_to_ai_summary(generic_json, &meeting_id.to_string());
                        return Ok(Some(converted_summary));
                    } else {
                        windows_console::println_japanese("Response is not valid JSON");
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

    pub async fn get_ai_summary(&self, meeting_uuid: &str) -> Result<Option<AISummaryResponse>, Box<dyn std::error::Error>> {
        windows_console::println_japanese(&format!("Requesting AI summary for UUID: {}", meeting_uuid));
        
        // Implement proper double URL encoding as required by Zoom API research
        let single_encoded = urlencoding::encode(meeting_uuid);
        let double_encoded = urlencoding::encode(&single_encoded);
        windows_console::println_japanese(&format!("Single encoded UUID: {}", single_encoded));
        windows_console::println_japanese(&format!("Double encoded UUID: {}", double_encoded));
        
        // Try different UUID formats based on Zoom API research
        let uuid_variants = vec![
            double_encoded.to_string(),  // Research shows double encoding is often required
            single_encoded.to_string(),
            meeting_uuid.to_string(),
            meeting_uuid.replace("/", "%2F").replace("=", "%3D"),
        ];
        
        for (variant_idx, uuid_variant) in uuid_variants.iter().enumerate() {
            windows_console::println_japanese(&format!("Trying UUID variant {}/{}: {}", variant_idx+1, uuid_variants.len(), uuid_variant));
            
            // Focus on the documented working endpoints from research
            let endpoints = vec![
                // Primary endpoint - meeting_summary (documented as working)
                format!("https://api.zoom.us/v2/meetings/{}/meeting_summary", uuid_variant),
                // Check for summary files in recordings
                format!("https://api.zoom.us/v2/meetings/{}/recordings", uuid_variant),
                // Legacy endpoints for compatibility
                format!("https://api.zoom.us/v2/meetings/{}/summary", uuid_variant),
                format!("https://api.zoom.us/v2/meetings/{}/batch_summary", uuid_variant),
            ];
        
            for (i, url) in endpoints.iter().enumerate() {
                windows_console::println_japanese(&format!("Trying endpoint {}/{}: {}", i+1, endpoints.len(), url));

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
                        
                        // Check if this is a recordings endpoint response
                        if url.contains("/recordings") {
                            if let Ok(recordings_data) = serde_json::from_str::<serde_json::Value>(&response_text) {
                                // Look for SUMMARY file type in recording files
                                if let Some(recording_files) = recordings_data.get("recording_files").and_then(|v| v.as_array()) {
                                    for file in recording_files {
                                        if let Some(file_type) = file.get("file_type").and_then(|v| v.as_str()) {
                                            if file_type == "SUMMARY" {
                                                windows_console::println_japanese("✓ Found SUMMARY file in recordings!");
                                                // Extract summary content if available
                                                let converted_summary = self.convert_generic_to_ai_summary(file.clone(), meeting_uuid);
                                                return Ok(Some(converted_summary));
                                            }
                                        }
                                    }
                                }
                                windows_console::println_japanese("No SUMMARY file found in recordings");
                                continue;
                            }
                        } else {
                            // Try to parse as meeting summary response
                            if let Ok(summary) = serde_json::from_str::<AISummaryResponse>(&response_text) {
                                windows_console::println_japanese("✓ Successfully parsed AI summary!");
                                return Ok(Some(summary));
                            } else if let Ok(generic_json) = serde_json::from_str::<serde_json::Value>(&response_text) {
                                windows_console::println_japanese("✓ Received valid JSON, converting to AI summary format");
                                let converted_summary = self.convert_generic_to_ai_summary(generic_json, meeting_uuid);
                                return Ok(Some(converted_summary));
                            } else {
                                windows_console::println_japanese("Response is not valid JSON");
                                windows_console::println_japanese(&format!("Response preview: {}", 
                                    if response_text.len() > 200 { 
                                        format!("{}...", &response_text[..200]) 
                                    } else { 
                                        response_text.clone() 
                                    }
                                ));
                                continue;
                            }
                        }
                    },
                    404 => {
                        windows_console::println_japanese(&format!("Endpoint {} returned 404 (not found)", i+1));
                        continue;
                    },
                    401 => {
                        windows_console::println_japanese(&format!("Endpoint {} returned 401 (Unauthorized)", i+1));
                        windows_console::println_japanese("ℹ Ensure access token is valid and has not expired");
                        continue;
                    },
                    403 => {
                        windows_console::println_japanese(&format!("Endpoint {} returned 403 (Forbidden)", i+1));
                        windows_console::println_japanese("ℹ Required scopes: meeting:read, recording:read, user:read");
                        windows_console::println_japanese("ℹ Note: You may need to be the meeting host to access summaries");
                        continue;
                    },
                    429 => {
                        windows_console::println_japanese(&format!("Endpoint {} returned 429 (Rate limit exceeded)", i+1));
                        // Add delay for rate limiting
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                        continue;
                    },
                    422 => {
                        windows_console::println_japanese(&format!("Endpoint {} returned 422 (Unprocessable Entity)", i+1));
                        windows_console::println_japanese("ℹ This may indicate the summary is still being processed");
                        continue;
                    },
                    500..=599 => {
                        windows_console::println_japanese(&format!("Endpoint {} returned {} (Server error)", i+1, response.status()));
                        continue;
                    },
                    _ => {
                        windows_console::println_japanese(&format!("Endpoint {} returned {} (Unknown error)", i+1, response.status()));
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
                        continue;
                    }
                }
            }
        }
        
        // No AI summary found from any endpoint
        windows_console::println_japanese(&format!("ℹ No AI summary available for meeting {} (this is normal if AI Companion was not enabled)", meeting_uuid));
        windows_console::println_japanese("ℹ AI summaries require: 1) AI Companion enabled, 2) Meeting host access, 3) Processing time (up to 24h)");
        Ok(None)
    }

    fn convert_generic_to_ai_summary(&self, json: serde_json::Value, meeting_uuid: &str) -> AISummaryResponse {
        // Extract common fields that might exist in various formats
        let summary_text = json.get("summary")
            .or_else(|| json.get("overview"))
            .or_else(|| json.get("content"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let key_points = json.get("key_points")
            .or_else(|| json.get("highlights"))
            .or_else(|| json.get("main_points"))
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        let action_items = json.get("action_items")
            .or_else(|| json.get("next_steps"))
            .or_else(|| json.get("todos"))
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        AISummaryResponse {
            meeting_uuid: meeting_uuid.to_string(),
            summary_start_time: json.get("start_time").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            summary_end_time: json.get("end_time").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            summary_created_time: json.get("created_time").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            summary_last_modified_time: json.get("modified_time").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            summary_title: json.get("title").and_then(|v| v.as_str()).unwrap_or("AI Generated Summary").to_string(),
            summary_overview: summary_text.clone(),
            summary_details: vec![], // Will be populated if structured details exist
            next_steps: action_items,
            summary_keyword: json.get("keywords")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
            summary: summary_text,
            key_points: key_points,
            action_items: json.get("action_items")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default(),
            meeting_id: json.get("meeting_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            topic_summaries: vec![], // Will be populated if topic summaries exist
            detailed_sections: vec![], // Will be populated if detailed sections exist
        }
    }

    fn create_recording_date_folder(&self, base_output_dir: &str, recording_start: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Parse the recording start time (ISO 8601 format)
        let recording_datetime = chrono::DateTime::parse_from_rfc3339(recording_start)
            .map_err(|e| format!("Failed to parse recording start time '{}': {}", recording_start, e))?;
        
        // Convert to JST timezone (+09:00)
        let jst_offset = FixedOffset::east_opt(9 * 3600).unwrap();
        let recording_jst = recording_datetime.with_timezone(&jst_offset);
        
        // Format as YYYY-MM-DD for folder name
        let date_folder = recording_jst.format("%Y-%m-%d").to_string();
        
        // Create the date-based folder path
        let date_folder_path = Path::new(base_output_dir).join(&date_folder);
        fs::create_dir_all(&date_folder_path)?;
        
        Ok(date_folder_path.to_string_lossy().to_string())
    }

    pub async fn save_ai_summary_txt(&self, summary: &AISummaryResponse, meeting_id: &str, output_dir: &str) -> Result<String, Box<dyn std::error::Error>> {
        let invalid_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
        
        // Create date folder based on AI summary creation time
        // Priority: summary_created_time > summary_start_time > current time
        let creation_time = if !summary.summary_created_time.is_empty() {
            &summary.summary_created_time
        } else if !summary.summary_start_time.is_empty() {
            &summary.summary_start_time
        } else {
            // Fallback to current time if neither is available
            &Utc::now().to_rfc3339()
        };
        let date_folder_path = self.create_recording_date_folder(output_dir, creation_time)?;
        
        // Create AI summary filename (.txt)
        let safe_meeting_id = meeting_id.chars().map(|c| if invalid_chars.contains(&c) { '_' } else { c }).collect::<String>();
        let summary_filename = format!("{}_ai_summary.txt", safe_meeting_id);
        let output_path = Path::new(&date_folder_path).join(&summary_filename);
        
        if output_path.exists() {
            windows_console::println_japanese(&format!("AI summary file already exists: {}", output_path.display()));
            return Ok(output_path.to_string_lossy().to_string());
        }

        // Create readable text format
        let mut content = String::new();
        
        // Header
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

        // Brief summary
        content.push_str("【簡単な要約】\n");
        let brief_summary = if !summary.summary_overview.is_empty() { 
            &summary.summary_overview
        } else { 
            &summary.summary 
        };
        if !brief_summary.is_empty() {
            content.push_str(brief_summary);
            content.push_str("\n");
        } else {
            content.push_str("要約情報がありません。\n");
        }
        content.push_str("\n");

        // Next steps
        content.push_str("【次のステップ】\n");
        let next_steps = if !summary.next_steps.is_empty() { 
            &summary.next_steps
        } else { 
            &summary.action_items 
        };
        if !next_steps.is_empty() {
            for (i, step) in next_steps.iter().enumerate() {
                content.push_str(&format!("{}. {}\n", i + 1, step));
            }
        } else {
            content.push_str("次のステップはありません。\n");
        }
        content.push_str("\n");

        // Detailed sections
        if !summary.topic_summaries.is_empty() {
            content.push_str("【詳細セクション】\n");
            for topic in &summary.topic_summaries {
                if !topic.topic_title.is_empty() {
                    content.push_str(&format!("■ {}\n", topic.topic_title));
                }
                if !topic.topic_content.is_empty() {
                    content.push_str(&format!("{}\n\n", topic.topic_content));
                }
            }
        } else if !summary.detailed_sections.is_empty() {
            content.push_str("【詳細セクション】\n");
            for section in &summary.detailed_sections {
                if !section.section_title.is_empty() {
                    content.push_str(&format!("■ {}\n", section.section_title));
                }
                if !section.section_content.is_empty() {
                    content.push_str(&format!("{}\n\n", section.section_content));
                }
            }
        } else if !summary.summary_details.is_empty() {
            content.push_str("【詳細情報】\n");
            for detail in &summary.summary_details {
                if !detail.summary_type.is_empty() {
                    content.push_str(&format!("■ {}\n", detail.summary_type));
                }
                if !detail.summary_content.is_empty() {
                    content.push_str(&format!("{}\n\n", detail.summary_content));
                }
            }
        }

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

        // Footer
        content.push_str("-".repeat(80).as_str());
        content.push_str("\n");
        content.push_str("Generated by: Zoom AI Companion\n");
        content.push_str(&format!("Download timestamp: {}\n", Utc::now().to_rfc3339()));

        fs::create_dir_all(&date_folder_path)?;
        fs::write(&output_path, content)?;

        windows_console::println_japanese(&format!("AI summary saved: {}", output_path.display()));
        Ok(output_path.to_string_lossy().to_string())
    }

    pub async fn save_ai_summary(&self, summary: &AISummaryResponse, meeting_id: &str, output_dir: &str) -> Result<String, Box<dyn std::error::Error>> {
        let invalid_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
        
        // Create date folder based on AI summary creation time
        // Priority: summary_created_time > summary_start_time > current time
        let creation_time = if !summary.summary_created_time.is_empty() {
            &summary.summary_created_time
        } else if !summary.summary_start_time.is_empty() {
            &summary.summary_start_time
        } else {
            // Fallback to current time if neither is available
            &Utc::now().to_rfc3339()
        };
        let date_folder_path = self.create_recording_date_folder(output_dir, creation_time)?;
        
        // Create AI summary filename
        let safe_meeting_id = meeting_id.chars().map(|c| if invalid_chars.contains(&c) { '_' } else { c }).collect::<String>();
        let summary_filename = format!("{}_ai_summary.json", safe_meeting_id);
        let output_path = Path::new(&date_folder_path).join(&summary_filename);
        
        if output_path.exists() {
            windows_console::println_japanese(&format!("AI summary file already exists: {}", output_path.display()));
            return Ok(output_path.to_string_lossy().to_string());
        }

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
                    "title": d.summary_type,
                    "content": d.summary_content
                })).collect::<Vec<_>>()
            } else {
                Vec::new()
            },
            "keywords": summary.summary_keyword,
            "key_points": summary.key_points,
            "generated_by": "Zoom AI Companion",
            "download_timestamp": Utc::now().to_rfc3339()
        });

        fs::create_dir_all(&date_folder_path)?;
        
        let json_content = serde_json::to_string_pretty(&comprehensive_summary)?;
        fs::write(&output_path, json_content)?;

        windows_console::println_japanese(&format!("AI summary saved: {}", output_path.display()));
        Ok(output_path.to_string_lossy().to_string())
    }

    pub async fn download_recording(&self, recording: &Recording, output_dir: &str) -> Result<String, Box<dyn std::error::Error>> {
        let invalid_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
        
        // Create date folder based on recording start time
        let date_folder_path = self.create_recording_date_folder(output_dir, &recording.recording_start)?;
        
        // Create descriptive filename based on file type
        let file_type_desc = match recording.file_type.to_lowercase().as_str() {
            "mp4" => "video",
            "m4a" => "audio", 
            "txt" => "chat",
            "vtt" => "transcript",
            "cc.vtt" => "captions",
            "csv" => "data",
            "json" => "metadata",
            _ => &recording.file_type.to_lowercase()
        };
        
        let safe_filename = format!("{}_{}_{}_{}.{}", 
            recording.meeting_id.chars().map(|c| if invalid_chars.contains(&c) { '_' } else { c }).collect::<String>(),
            recording.recording_type.chars().map(|c| if invalid_chars.contains(&c) { '_' } else { c }).collect::<String>(),
            file_type_desc,
            recording.id.chars().map(|c| if invalid_chars.contains(&c) { '_' } else { c }).collect::<String>(),
            recording.file_type.to_lowercase()
        );
        
        let output_path = Path::new(&date_folder_path).join(&safe_filename);
        
        if output_path.exists() {
            windows_console::println_japanese(&format!("File already exists: {}", output_path.display()));
            return Ok(output_path.to_string_lossy().to_string());
        }

        windows_console::println_japanese(&format!("Downloading: {} ({:.2} MB)", safe_filename, recording.file_size as f64 / 1024.0 / 1024.0));

        let response = self
            .client
            .get(&recording.download_url)
            .bearer_auth(&self.access_token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Download failed: {}", response.status()).into());
        }

        fs::create_dir_all(&date_folder_path)?;
        
        let mut file = tokio::fs::File::create(&output_path).await?;
        let content = response.bytes().await?;
        file.write_all(&content).await?;

        windows_console::println_japanese(&format!("Download completed: {}", output_path.display()));
        Ok(output_path.to_string_lossy().to_string())
    }

    pub async fn download_all_recordings(&self, user_id: &str, from: &str, to: &str, output_dir: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        // Test API access first
        if let Err(e) = self.test_api_access().await {
            windows_console::println_japanese(&format!("API test failed: {}", e));
        }

        let recordings = self.list_recordings(user_id, from, to).await?;
        let mut downloaded_files = Vec::new();
        let mut file_type_counts = std::collections::HashMap::new();

        windows_console::println_japanese(&format!("Found {} recorded meetings", recordings.meetings.len()));

        for meeting in recordings.meetings {
            windows_console::println_japanese(&format!("Processing meeting: {} ({})", meeting.topic, meeting.start_time));
            
            // Try to download AI summary for this meeting
            windows_console::println_japanese(&format!("=== Checking AI Companion summary for meeting {} ===", meeting.id));
            windows_console::println_japanese(&format!("Meeting UUID: {}", meeting.uuid));
            windows_console::println_japanese(&format!("Meeting topic: {}", meeting.topic));
            windows_console::println_japanese(&format!("Meeting start time: {}", meeting.start_time));
            
            // Try both UUID and Meeting ID approaches
            let mut summary_found = false;
            
            // First try with Meeting ID (more likely to work)
            match self.get_ai_summary_by_meeting_id(meeting.id).await {
                Ok(Some(summary)) => {
                    windows_console::println_japanese("✓ AI summary found via Meeting ID, saving to file...");
                    match self.save_ai_summary_txt(&summary, &meeting.id.to_string(), output_dir).await {
                        Ok(path) => {
                            windows_console::println_japanese(&format!("✓ AI summary saved: {}", path));
                            downloaded_files.push(path);
                            *file_type_counts.entry("ai_summary".to_string()).or_insert(0) += 1;
                            summary_found = true;
                        },
                        Err(e) => windows_console::println_japanese(&format!("✗ Failed to save AI summary: {}", e)),
                    }
                },
                Ok(None) => {
                    windows_console::println_japanese("ℹ No AI summary found via Meeting ID");
                },
                Err(e) => {
                    windows_console::println_japanese(&format!("✗ Error checking AI summary via Meeting ID: {}", e));
                }
            }
            
            // If not found via Meeting ID, try with UUID
            if !summary_found {
                match self.get_ai_summary(&meeting.uuid).await {
                    Ok(Some(summary)) => {
                        windows_console::println_japanese("✓ AI summary found via UUID, saving to file...");
                        match self.save_ai_summary_txt(&summary, &meeting.id.to_string(), output_dir).await {
                            Ok(path) => {
                                windows_console::println_japanese(&format!("✓ AI summary saved: {}", path));
                                downloaded_files.push(path);
                                *file_type_counts.entry("ai_summary".to_string()).or_insert(0) += 1;
                                summary_found = true;
                            },
                            Err(e) => windows_console::println_japanese(&format!("✗ Failed to save AI summary: {}", e)),
                        }
                    },
                    Ok(None) => {
                        windows_console::println_japanese("ℹ No AI summary found via UUID");
                    },
                    Err(e) => {
                        windows_console::println_japanese(&format!("✗ Error checking AI summary via UUID: {}", e));
                    }
                }
            }
            
            if !summary_found {
                windows_console::println_japanese("ℹ No AI summary available (this is normal for meetings without AI Companion enabled)");
            }
            windows_console::println_japanese("=== End AI summary check ===\n");
            
            for recording in meeting.recording_files {
                // Download all file types: videos (MP4), audio (M4A), chat files (TXT), and other assets
                let file_type = recording.file_type.to_lowercase();
                let is_downloadable = match file_type.as_str() {
                    "mp4" | "m4a" => true,  // Video and audio files
                    "txt" => true,          // Chat files
                    "vtt" => true,          // Transcript/subtitle files  
                    "csv" => true,          // Poll results, participant lists
                    "json" => true,         // Meeting metadata
                    "cc.vtt" => true,       // Closed captions
                    _ => {
                        // Log unknown file types but attempt to download them
                        windows_console::println_japanese(&format!("Unknown file type '{}' for recording {}, attempting download", file_type, recording.id));
                        true
                    }
                };
                
                if is_downloadable {
                    windows_console::println_japanese(&format!("Downloading {} file: {}", file_type.to_uppercase(), recording.id));
                    match self.download_recording(&recording, output_dir).await {
                        Ok(path) => {
                            downloaded_files.push(path);
                            *file_type_counts.entry(file_type.clone()).or_insert(0) += 1;
                        },
                        Err(e) => windows_console::println_japanese(&format!("Download failed {}: {}", recording.id, e)),
                    }
                }
            }
        }

        // Display download summary with file type breakdown
        windows_console::println_japanese(&format!("Download completed: {} files total", downloaded_files.len()));
        if !file_type_counts.is_empty() {
            windows_console::println_japanese("File types downloaded:");
            for (file_type, count) in file_type_counts {
                let type_name = match file_type.as_str() {
                    "mp4" => "Video files",
                    "m4a" => "Audio files", 
                    "txt" => "Chat files",
                    "vtt" => "Transcript files",
                    "cc.vtt" => "Caption files",
                    "csv" => "Data files",
                    "json" => "Metadata files",
                    "ai_summary" => "AI Companion summaries",
                    _ => "Other files"
                };
                windows_console::println_japanese(&format!("  {} ({}): {} files", type_name, file_type.to_uppercase(), count));
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
