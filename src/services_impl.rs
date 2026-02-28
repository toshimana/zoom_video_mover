//! 本番用サービス実装
//!
//! services.rsで定義されたtraitの本番実装。
//! 既存のgui.rsにハードコードされていた外部呼び出しをラップする。

use crate::components::api::{
    ApiComponent, ApiConfig, MeetingRecording, RecordingFile, RecordingFileType,
    RecordingSearchRequest, RecordingSearchResponse,
};
use crate::components::auth::AuthToken;
use crate::components::download::{DownloadComponent, DownloadConfig, DownloadEvent};
use crate::gui::AppMessage;
use crate::services::{
    AuthService, BrowserLauncher, ConfigService, DownloadService, RecordingService,
};
use crate::Config;
use chrono::{Datelike, NaiveDate};
use log;
use std::path::PathBuf;
use std::sync::{mpsc, Arc};
use tokio::sync::Semaphore;

/// 本番用設定サービス
pub struct RealConfigService;

impl ConfigService for RealConfigService {
    fn load_config(&self, path: &str) -> Result<Config, Box<dyn std::error::Error>> {
        Config::load_from_file(path)
    }

    fn save_config(&self, config: &Config, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        config.save_to_file(path)
    }

    fn create_sample_config(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        Config::create_sample_file(path)
    }
}

/// 本番用認証サービス
pub struct RealAuthService;

impl AuthService for RealAuthService {
    fn generate_auth_url(
        &self,
        client_id: &str,
        client_secret: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;
        let client_id = client_id.to_string();
        let client_secret = client_secret.to_string();
        rt.block_on(async { crate::gui::generate_auth_url_async(&client_id, &client_secret).await })
    }

    fn exchange_code_for_token(
        &self,
        client_id: &str,
        client_secret: &str,
        auth_code: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;
        let client_id = client_id.to_string();
        let client_secret = client_secret.to_string();
        let auth_code = auth_code.to_string();
        rt.block_on(async {
            crate::gui::exchange_code_for_token_async(&client_id, &client_secret, &auth_code).await
        })
    }
}

/// 本番用録画取得サービス
pub struct RealRecordingService;

/// 日付範囲を最大1ヶ月ごとのチャンクに分割する
///
/// Zoom APIの `from`/`to` パラメータは最大1ヶ月の範囲のみ許容するため、
/// 1ヶ月を超える期間を月単位に分割する。
/// 各チャンクの `to` は次チャンクの `from` と同じ値（API側でexclusive扱い対策として+1日）。
fn split_into_monthly_chunks(from: NaiveDate, to: NaiveDate) -> Vec<(NaiveDate, NaiveDate)> {
    let mut chunks = Vec::new();
    let mut chunk_start = from;

    while chunk_start <= to {
        // chunk_startの翌月1日を計算
        let next_month_first = if chunk_start.month() == 12 {
            NaiveDate::from_ymd_opt(chunk_start.year() + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(chunk_start.year(), chunk_start.month() + 1, 1)
        };

        let chunk_end = match next_month_first {
            Some(nm) if nm <= to => nm,
            // 最後のチャンク: toに+1日してexclusive対策
            _ => to + chrono::Duration::days(1),
        };

        chunks.push((chunk_start, chunk_end));
        chunk_start = chunk_end;
    }

    chunks
}

impl RecordingService for RealRecordingService {
    fn get_recordings(
        &self,
        access_token: &str,
        user_id: &str,
        from_date: &str,
        to_date: &str,
        progress_sender: mpsc::Sender<AppMessage>,
    ) -> Result<RecordingSearchResponse, Box<dyn std::error::Error + Send + Sync>> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;
        let access_token = access_token.to_string();
        let user_id = user_id.to_string();
        let from_date = from_date.to_string();
        let to_date = to_date.to_string();
        rt.block_on(async {
            // ApiComponent を生成し、認証トークンを設定
            let api = ApiComponent::new(ApiConfig::default());
            let token = AuthToken {
                access_token,
                token_type: "Bearer".to_string(),
                expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
                refresh_token: None,
                scopes: vec!["recording:read".to_string()],
            };
            api.set_auth_token(token).await;

            // 日付パース
            let from = NaiveDate::parse_from_str(&from_date, "%Y-%m-%d")
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                    Box::new(e)
                })?;
            let to = NaiveDate::parse_from_str(&to_date, "%Y-%m-%d")
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                    Box::new(e)
                })?;

            // 月単位チャンクに分割してページネーション対応で全件取得（並列）
            let chunks = split_into_monthly_chunks(from, to);
            let total_chunks = chunks.len();
            log::info!("Fetching recordings: from={} to={}, split into {} chunk(s)", from, to, total_chunks);

            let api = Arc::new(api);
            let mut all_meetings: Vec<MeetingRecording> = Vec::new();

            let chunk_semaphore = Arc::new(Semaphore::new(3));
            let mut handles = Vec::new();
            for (chunk_idx, (chunk_from, chunk_to)) in chunks.iter().enumerate() {
                let api = Arc::clone(&api);
                let sem = Arc::clone(&chunk_semaphore);
                let user_id = user_id.clone();
                let progress_sender = progress_sender.clone();
                let chunk_from = *chunk_from;
                let chunk_to = *chunk_to;
                handles.push(tokio::spawn(async move {
                    let _permit = sem.acquire().await.unwrap();
                    let _ = progress_sender.send(AppMessage::SearchProgress(
                        format!("録画データを取得中... ({}/{})", chunk_idx + 1, total_chunks),
                    ));
                    log::info!("Fetching chunk: from={} to={}", chunk_from, chunk_to);
                    let request = RecordingSearchRequest {
                        user_id: Some(user_id),
                        from: chunk_from,
                        to: chunk_to,
                        page_size: None,
                        next_page_token: None,
                    };
                    let result = api.get_all_recordings(request).await
                        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) });
                    log::info!("Chunk from={} to={}: completed", chunk_from, chunk_to);
                    result
                }));
            }
            for handle in handles {
                let chunk_meetings = handle.await
                    .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })??;
                all_meetings.extend(chunk_meetings);
            }

            log::info!("Total meetings fetched: {}", all_meetings.len());

            let mut recordings = RecordingSearchResponse {
                from: from_date.clone(),
                to: to_date.clone(),
                page_count: 1,
                page_size: all_meetings.len() as u32,
                total_records: all_meetings.len() as u32,
                next_page_token: None,
                meetings: all_meetings,
            };

            // 各ミーティングのrecording_filesにSUMMARYが含まれない場合、
            // Meeting Summary APIで自動チェックし、仮想エントリを追加（セマフォ付き並列化）
            let total_meetings = recordings.meetings.len();
            let semaphore = Arc::new(Semaphore::new(5));
            let mut summary_handles = Vec::new();

            for (meeting_idx, meeting) in recordings.meetings.iter().enumerate() {
                let has_summary = meeting.recording_files.iter()
                    .any(|f| f.file_type == RecordingFileType::Summary);
                if !has_summary {
                    let api = Arc::clone(&api);
                    let sem = Arc::clone(&semaphore);
                    let uuid = meeting.uuid.clone();
                    let topic = meeting.topic.clone();
                    let progress_sender = progress_sender.clone();
                    summary_handles.push((meeting_idx, tokio::spawn(async move {
                        let _permit = sem.acquire().await.unwrap();
                        let _ = progress_sender.send(AppMessage::SearchProgress(
                            format!("AI要約をチェック中... ({}/{})", meeting_idx + 1, total_meetings),
                        ));
                        log::info!("[DL-DIAG] No SUMMARY in recording_files, checking Meeting Summary API: meeting_uuid={}, topic='{}'",
                            uuid, topic);
                        api.get_meeting_summary(&uuid).await
                    })));
                }
            }

            for (meeting_idx, handle) in summary_handles {
                let meeting = &mut recordings.meetings[meeting_idx];
                match handle.await {
                    Ok(Ok(Some(_))) => {
                        log::info!("[DL-DIAG] Meeting summary available via API: meeting_uuid={}", meeting.uuid);
                        meeting.recording_files.push(RecordingFile {
                            id: String::new(),
                            meeting_id: meeting.id.to_string(),
                            recording_start: meeting.start_time.clone(),
                            recording_end: String::new(),
                            file_type: RecordingFileType::Summary,
                            file_extension: "json".to_string(),
                            file_size: 0,
                            play_url: None,
                            download_url: String::new(),
                            status: String::new(),
                            recording_type: String::new(),
                        });
                    }
                    Ok(Ok(None)) => {
                        log::info!("[DL-DIAG] No meeting summary available for meeting_uuid={}", meeting.uuid);
                    }
                    Ok(Err(e)) => {
                        log::warn!("[DL-DIAG] Failed to check meeting summary for meeting_uuid={}: {}", meeting.uuid, e);
                    }
                    Err(e) => {
                        log::warn!("[DL-DIAG] Summary check task panicked for meeting_uuid={}: {}", meeting.uuid, e);
                    }
                }
            }

            Ok(recordings)
        })
    }
}

/// 本番用ブラウザ起動サービス
pub struct RealBrowserLauncher;

impl BrowserLauncher for RealBrowserLauncher {
    fn open_url(&self, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        open::that(url)?;
        Ok(())
    }
}

/// 本番用ダウンロードサービス
pub struct RealDownloadService;

impl RealDownloadService {
    /// 選択IDから対象ファイルを解決する
    ///
    /// - "uuid-fileid" 形式 → 特定ファイルを直接マッチ
    /// - "uuid" 形式 → そのミーティングの全ファイルをダウンロード
    fn resolve_selected_files<'a>(
        recordings: &'a RecordingSearchResponse,
        selected_recordings: &[String],
    ) -> Vec<(&'a MeetingRecording, &'a RecordingFile)> {
        let mut result = Vec::new();
        log::info!(
            "[DL-DIAG] resolve_selected_files: {} selections, {} meetings",
            selected_recordings.len(),
            recordings.meetings.len()
        );

        for selection in selected_recordings {
            for meeting in &recordings.meetings {
                if *selection == meeting.uuid {
                    // ミーティング全体が選択されている
                    log::info!(
                        "[DL-DIAG] Meeting-level selection: uuid={}, {} files",
                        meeting.uuid,
                        meeting.recording_files.len()
                    );
                    for file in &meeting.recording_files {
                        log::info!(
                            "[DL-DIAG]   file: type={}, stable_id={}, download_url_len={}",
                            file.file_type,
                            file.stable_id(),
                            file.download_url.len()
                        );
                        result.push((meeting, file));
                    }
                } else if let Some(file_id) = selection.strip_prefix(&format!("{}-", meeting.uuid))
                {
                    // 個別ファイルが選択されている
                    for file in &meeting.recording_files {
                        if file.stable_id() == file_id {
                            log::info!(
                                "[DL-DIAG] Individual file matched: selection={}, stable_id={}",
                                selection,
                                file.stable_id()
                            );
                            result.push((meeting, file));
                        }
                    }
                }
            }
        }

        log::info!(
            "[DL-DIAG] resolve result: {} files before dedup",
            result.len()
        );
        // 重複除去（同じ stable_id を持つものは1つだけ）
        result.dedup_by_key(|(_, f)| f.stable_id());
        log::info!(
            "[DL-DIAG] resolve result: {} files after dedup",
            result.len()
        );
        result
    }
}

impl DownloadService for RealDownloadService {
    fn download_files(
        &self,
        access_token: &str,
        recordings: &RecordingSearchResponse,
        selected_recordings: &[String],
        output_dir: &str,
        sender: mpsc::Sender<AppMessage>,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let _ = sender.send(AppMessage::DownloadProgress(
            "Resolving selected files...".to_string(),
        ));

        // 選択されたファイルを解決
        let files_to_download = Self::resolve_selected_files(recordings, selected_recordings);

        if files_to_download.is_empty() {
            let _ = sender.send(AppMessage::DownloadComplete(vec![]));
            return Ok(vec![]);
        }

        let total_files = files_to_download.len();
        let _ = sender.send(AppMessage::DownloadProgress(format!(
            "Preparing to download {} files...",
            total_files
        )));

        // DownloadComponent用のタスク情報を事前に収集
        let access_token = access_token.to_string();
        let output_dir = output_dir.to_string();
        let mut tasks: Vec<(String, String, String, Option<u64>)> = Vec::new();
        let mut skipped_files: Vec<String> = Vec::new();
        let mut summary_fallback_targets: Vec<(MeetingRecording, RecordingFile)> = Vec::new();

        for (meeting, file) in &files_to_download {
            if file.download_url.is_empty() {
                if file.file_type == RecordingFileType::Summary {
                    // SUMMARYファイルはMeeting Summary APIでフォールバック取得
                    log::info!("[DL-DIAG] SUMMARY file has empty download_url, will use Meeting Summary API: meeting_id={}", meeting.id);
                    summary_fallback_targets.push(((*meeting).clone(), (*file).clone()));
                } else {
                    let msg = format!(
                        "{}: meeting='{}' ({})",
                        file.file_type, meeting.topic, meeting.start_time
                    );
                    log::warn!("[DL-DIAG] Skipping file with empty download_url: {}", msg);
                    skipped_files.push(msg);
                }
                continue;
            }
            let task_id = format!("{}-{}", meeting.uuid, file.stable_id());
            // Zoom APIではdownload_urlにaccess_tokenをクエリパラメータで付与する必要がある
            let download_url = if file.download_url.contains('?') {
                format!("{}&access_token={}", file.download_url, access_token)
            } else {
                format!("{}?access_token={}", file.download_url, access_token)
            };
            let file_name = crate::generate_file_path(meeting, file);
            let file_size = if file.file_size > 0 {
                Some(file.file_size)
            } else {
                None
            };
            log::info!(
                "[DL-DIAG] Task created: id={}, type={}, url_len={}",
                task_id,
                file.file_type,
                file.download_url.len()
            );
            tasks.push((task_id, download_url, file_name, file_size));
        }

        // GUIにスキップ通知
        if !skipped_files.is_empty() {
            let _ = sender.send(AppMessage::DownloadProgress(format!(
                "Warning: {} file(s) skipped (no download URL)",
                skipped_files.len()
            )));
            for msg in &skipped_files {
                let _ = sender.send(AppMessage::DownloadProgress(format!("  Skipped: {}", msg)));
            }
        }

        let sender_clone = sender.clone();

        // tokio runtimeで非同期ダウンロードを実行
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;

        let downloaded_files = rt.block_on(async move {
            // DownloadComponent を設定
            let config = DownloadConfig {
                output_directory: PathBuf::from(&output_dir),
                ..DownloadConfig::default()
            };
            let mut component = DownloadComponent::new(config);

            // イベントリスナーを設定
            let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel::<DownloadEvent>();
            component.set_event_listener(event_tx);

            // 出力ディレクトリを作成
            tokio::fs::create_dir_all(&output_dir).await.map_err(
                |e| -> Box<dyn std::error::Error + Send + Sync> {
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to create output directory: {}", e),
                    ))
                },
            )?;

            // タスクを追加
            for (task_id, download_url, file_name, file_size) in &tasks {
                component
                    .add_download_task(
                        task_id.clone(),
                        download_url.clone(),
                        file_name.clone(),
                        *file_size,
                    )
                    .await
                    .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Failed to add task: {}", e),
                        ))
                    })?;
            }

            // ダウンロード開始
            component.start_downloads().await.map_err(
                |e| -> Box<dyn std::error::Error + Send + Sync> {
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to start downloads: {}", e),
                    ))
                },
            )?;

            // イベントを受信して進捗を通知
            let mut completed_files: Vec<String> = Vec::new();
            let mut completed_count = 0u32;
            let mut failed_count = 0u32;
            let download_task_count = tasks.len() as u32;

            while download_task_count > 0 && completed_count + failed_count < download_task_count {
                match event_rx.recv().await {
                    Some(event) => match event {
                        DownloadEvent::TaskStarted { task_id } => {
                            // task_idからファイル名を取得
                            let file_name = tasks
                                .iter()
                                .find(|(id, _, _, _)| *id == task_id)
                                .map(|(_, _, name, _)| name.as_str())
                                .unwrap_or("unknown");
                            let _ = sender_clone.send(AppMessage::DownloadProgress(format!(
                                "Downloading: {}",
                                file_name
                            )));
                        }
                        DownloadEvent::ProgressUpdate { task_id, progress } => {
                            let file_name = tasks
                                .iter()
                                .find(|(id, _, _, _)| *id == task_id)
                                .map(|(_, _, name, _)| name.as_str())
                                .unwrap_or("unknown");
                            let speed_mbps = progress.current_speed / 1024.0 / 1024.0;
                            let _ = sender_clone.send(AppMessage::DownloadProgress(format!(
                                "{}: {:.1}% ({:.1} MB/s)",
                                file_name,
                                progress.percentage * 100.0,
                                speed_mbps
                            )));
                        }
                        DownloadEvent::TaskCompleted { output_path, .. } => {
                            completed_count += 1;
                            let path_str = output_path.to_string_lossy().to_string();
                            completed_files.push(path_str.clone());
                            let _ = sender_clone.send(AppMessage::DownloadProgress(format!(
                                "Completed ({}/{}): {}",
                                completed_count, download_task_count, path_str
                            )));
                        }
                        DownloadEvent::TaskFailed { task_id, error } => {
                            failed_count += 1;
                            let file_name = tasks
                                .iter()
                                .find(|(id, _, _, _)| *id == task_id)
                                .map(|(_, _, name, _)| name.as_str())
                                .unwrap_or("unknown");
                            let _ = sender_clone.send(AppMessage::DownloadProgress(format!(
                                "Failed: {} - {}",
                                file_name, error
                            )));
                        }
                        DownloadEvent::OverallProgressUpdate(_) => {
                            // 全体進捗更新（現時点では未実装部分のためスキップ）
                        }
                    },
                    None => {
                        // チャネルが閉じた
                        break;
                    }
                }
            }

            // シャットダウン
            let _ = component.stop_downloads().await;

            // Meeting Summary APIフォールバック処理
            if !summary_fallback_targets.is_empty() {
                let _ = sender_clone.send(AppMessage::DownloadProgress(format!(
                    "Fetching {} AI summary(ies) via Meeting Summary API...",
                    summary_fallback_targets.len()
                )));

                let api = ApiComponent::new(ApiConfig::default());
                let token = AuthToken {
                    access_token: access_token.clone(),
                    token_type: "Bearer".to_string(),
                    expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
                    refresh_token: None,
                    scopes: vec!["recording:read".to_string()],
                };
                api.set_auth_token(token).await;

                for (meeting, file) in &summary_fallback_targets {
                    let _ = sender_clone.send(AppMessage::DownloadProgress(format!(
                        "Fetching AI summary: {}",
                        meeting.topic
                    )));

                    match api.get_meeting_summary(&meeting.uuid).await {
                        Ok(Some(summary)) => {
                            let file_name = crate::generate_file_path(meeting, file);
                            let output_path = PathBuf::from(&output_dir).join(&file_name);

                            if let Some(parent) = output_path.parent() {
                                let _ = tokio::fs::create_dir_all(parent).await;
                            }

                            match serde_json::to_string_pretty(&summary) {
                                Ok(json_str) => {
                                    match tokio::fs::write(&output_path, json_str.as_bytes()).await
                                    {
                                        Ok(_) => {
                                            let path_str =
                                                output_path.to_string_lossy().to_string();
                                            completed_files.push(path_str.clone());
                                            let _ =
                                                sender_clone.send(AppMessage::DownloadProgress(
                                                    format!("AI summary saved: {}", path_str),
                                                ));
                                        }
                                        Err(e) => {
                                            log::error!("Failed to write summary file: {}", e);
                                            let _ =
                                                sender_clone.send(AppMessage::DownloadProgress(
                                                    format!("Failed to save AI summary: {}", e),
                                                ));
                                        }
                                    }
                                }
                                Err(e) => {
                                    log::error!("Failed to serialize summary: {}", e);
                                }
                            }
                        }
                        Ok(None) => {
                            log::info!("No AI summary available for meeting_id={}", meeting.id);
                            let _ = sender_clone.send(AppMessage::DownloadProgress(format!(
                                "AI summary not available: {}",
                                meeting.topic
                            )));
                        }
                        Err(e) => {
                            log::warn!(
                                "Failed to fetch AI summary for meeting_id={}: {}",
                                meeting.id,
                                e
                            );
                            let _ = sender_clone.send(AppMessage::DownloadProgress(format!(
                                "AI summary fetch failed: {} - {}",
                                meeting.topic, e
                            )));
                        }
                    }
                }
            }

            let _ = sender_clone.send(AppMessage::DownloadComplete(completed_files.clone()));

            Ok::<Vec<String>, Box<dyn std::error::Error + Send + Sync>>(completed_files)
        })?;

        Ok(downloaded_files)
    }
}

/// サービスコンテナ - 全サービスをまとめて保持
pub struct AppServices {
    pub config_service: Box<dyn ConfigService>,
    pub auth_service: Arc<dyn AuthService>,
    pub recording_service: Arc<dyn RecordingService>,
    pub browser_launcher: Box<dyn BrowserLauncher>,
    pub download_service: Arc<dyn DownloadService>,
}

impl Default for AppServices {
    fn default() -> Self {
        Self {
            config_service: Box::new(RealConfigService),
            auth_service: Arc::new(RealAuthService),
            recording_service: Arc::new(RealRecordingService),
            browser_launcher: Box::new(RealBrowserLauncher),
            download_service: Arc::new(RealDownloadService),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::api::{RecordingFile, RecordingFileType};

    fn make_meeting(uuid: &str, files: Vec<RecordingFile>) -> MeetingRecording {
        MeetingRecording {
            uuid: uuid.to_string(),
            id: 123,
            account_id: String::new(),
            host_id: "host1".to_string(),
            topic: "Test Meeting".to_string(),
            meeting_type: 2,
            start_time: "2025-01-01T00:00:00Z".to_string(),
            timezone: String::new(),
            duration: 60,
            total_size: 0,
            recording_count: 0,
            recording_files: files,
        }
    }

    fn make_file(id: &str, file_type: RecordingFileType, download_url: &str) -> RecordingFile {
        RecordingFile {
            id: id.to_string(),
            meeting_id: String::new(),
            recording_start: String::new(),
            recording_end: String::new(),
            file_type,
            file_extension: String::new(),
            file_size: 1000,
            play_url: None,
            download_url: download_url.to_string(),
            status: String::new(),
            recording_type: String::new(),
        }
    }

    #[test]
    fn test_split_into_monthly_chunks_within_one_month() {
        let from = NaiveDate::from_ymd_opt(2025, 1, 5).unwrap();
        let to = NaiveDate::from_ymd_opt(2025, 1, 20).unwrap();
        let chunks = split_into_monthly_chunks(from, to);
        // 1ヶ月以内なら1チャンク、toに+1日
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].0, from);
        assert_eq!(chunks[0].1, NaiveDate::from_ymd_opt(2025, 1, 21).unwrap());
    }

    #[test]
    fn test_split_into_monthly_chunks_exact_one_month() {
        let from = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(2025, 1, 31).unwrap();
        let chunks = split_into_monthly_chunks(from, to);
        // 1月1日〜1月31日 → 1チャンク（翌月1日 = 2/1 > to なので最後のチャンク）
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].0, from);
        assert_eq!(chunks[0].1, NaiveDate::from_ymd_opt(2025, 2, 1).unwrap());
    }

    #[test]
    fn test_split_into_monthly_chunks_multi_month() {
        let from = NaiveDate::from_ymd_opt(2025, 12, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(2026, 2, 27).unwrap();
        let chunks = split_into_monthly_chunks(from, to);
        // 3チャンク: 12/1-1/1, 1/1-2/1, 2/1-2/28
        assert_eq!(chunks.len(), 3);
        assert_eq!(
            chunks[0],
            (
                NaiveDate::from_ymd_opt(2025, 12, 1).unwrap(),
                NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            )
        );
        assert_eq!(
            chunks[1],
            (
                NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
            )
        );
        assert_eq!(
            chunks[2],
            (
                NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
                NaiveDate::from_ymd_opt(2026, 2, 28).unwrap(),
            )
        );
    }

    #[test]
    fn test_split_into_monthly_chunks_year_boundary() {
        let from = NaiveDate::from_ymd_opt(2025, 11, 15).unwrap();
        let to = NaiveDate::from_ymd_opt(2026, 1, 10).unwrap();
        let chunks = split_into_monthly_chunks(from, to);
        // 3チャンク: 11/15-12/1, 12/1-1/1, 1/1-1/11
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].0, NaiveDate::from_ymd_opt(2025, 11, 15).unwrap());
        assert_eq!(chunks[0].1, NaiveDate::from_ymd_opt(2025, 12, 1).unwrap());
        assert_eq!(chunks[1].0, NaiveDate::from_ymd_opt(2025, 12, 1).unwrap());
        assert_eq!(chunks[1].1, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        assert_eq!(chunks[2].0, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        assert_eq!(chunks[2].1, NaiveDate::from_ymd_opt(2026, 1, 11).unwrap());
    }

    #[test]
    fn test_split_into_monthly_chunks_same_day() {
        let date = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        let chunks = split_into_monthly_chunks(date, date);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].0, date);
        assert_eq!(chunks[0].1, NaiveDate::from_ymd_opt(2025, 6, 16).unwrap());
    }

    #[test]
    fn test_resolve_selected_files_meeting_level_includes_summary() {
        let summary_file = make_file("", RecordingFileType::Summary, "");
        let mp4_file = make_file(
            "file1",
            RecordingFileType::MP4,
            "https://example.com/video.mp4",
        );

        let meeting = make_meeting("uuid-1", vec![mp4_file, summary_file]);
        let recordings = RecordingSearchResponse {
            from: "2025-01-01".to_string(),
            to: "2025-01-31".to_string(),
            page_count: 1,
            page_size: 30,
            total_records: 1,
            next_page_token: None,
            meetings: vec![meeting],
        };

        let selected = vec!["uuid-1".to_string()];
        let result = RealDownloadService::resolve_selected_files(&recordings, &selected);

        // ミーティング全体選択ではSUMMARYファイルも含まれる
        assert_eq!(result.len(), 2);
        let types: Vec<_> = result.iter().map(|(_, f)| &f.file_type).collect();
        assert!(types.contains(&&RecordingFileType::Summary));
        assert!(types.contains(&&RecordingFileType::MP4));
    }

    #[test]
    fn test_resolve_selected_files_individual_summary() {
        let summary_file = make_file("", RecordingFileType::Summary, "");
        let mp4_file = make_file(
            "file1",
            RecordingFileType::MP4,
            "https://example.com/video.mp4",
        );

        let meeting = make_meeting("uuid-1", vec![mp4_file, summary_file]);
        let recordings = RecordingSearchResponse {
            from: "2025-01-01".to_string(),
            to: "2025-01-31".to_string(),
            page_count: 1,
            page_size: 30,
            total_records: 1,
            next_page_token: None,
            meetings: vec![meeting],
        };

        // "uuid-auto_summary" でSUMMARYファイルを個別選択
        let selected = vec!["uuid-1-auto_summary".to_string()];
        let result = RealDownloadService::resolve_selected_files(&recordings, &selected);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].1.file_type, RecordingFileType::Summary);
        assert_eq!(result[0].1.stable_id(), "auto_summary");
    }
}
