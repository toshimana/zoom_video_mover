# データモデル設計書 - Zoom Video Mover

## 文書概要
**文書ID**: DES-DATA-001  
**作成日**: 2025-08-03  
**作成者**: データ設計者  
**レビューア**: ドメインエキスパート  
**バージョン**: 1.0  

## データモデル概要

### データモデル設計原則
1. **型安全性**: Rustの型システムを活用した安全なデータ構造
2. **不変性**: データの不変性を重視した設計
3. **シリアライゼーション**: serde対応による効率的なデータ変換
4. **バリデーション**: データ整合性を保証する検証機能
5. **拡張性**: 将来の機能追加に対応可能なスキーマ設計

### データレイヤー構成
```
┌─────────────────────────────────────────────────────────────────┐
│                    Domain Model Layer                           │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │   Core Domain   │  │   Value         │  │   Domain        │  │
│  │   Entities      │  │   Objects       │  │   Services      │  │
│  │                 │  │                 │  │                 │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘  │
├─────────────────────────────────────────────────────────────────┤
│                 Data Transfer Object Layer                      │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │   API DTOs      │  │   UI DTOs       │  │   Event DTOs    │  │
│  │  (External)     │  │ (Presentation)  │  │  (Messaging)    │  │
│  │                 │  │                 │  │                 │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘  │
├─────────────────────────────────────────────────────────────────┤
│                   Persistence Layer                             │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │   File System   │  │   Config        │  │   Cache         │  │
│  │   Models        │  │   Models        │  │   Models        │  │
│  │                 │  │                 │  │                 │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## ドメインモデル

### コアエンティティ

#### 1. 録画エンティティ (Recording Entity)
```rust
/// 録画エンティティ - ドメインの中核となるデータ構造
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recording {
    /// 録画ID（一意識別子）
    id: RecordingId,
    
    /// 会議情報
    meeting: MeetingInfo,
    
    /// 録画ファイル群
    files: Vec<RecordingFile>,
    
    /// ホスト情報
    host: HostInfo,
    
    /// 録画設定
    settings: RecordingSettings,
    
    /// メタデータ
    metadata: RecordingMetadata,
    
    /// ドメインイベント履歴
    #[serde(skip)]
    events: Vec<DomainEvent>,
}

impl Recording {
    /// 新しい録画インスタンス作成
    pub fn new(
        id: RecordingId,
        meeting: MeetingInfo,
        host: HostInfo,
        settings: RecordingSettings,
    ) -> Result<Self, DomainError> {
        // ビジネスルール検証
        Self::validate_recording_data(&meeting, &host, &settings)?;
        
        let recording = Self {
            id,
            meeting,
            files: Vec::new(),
            host,
            settings,
            metadata: RecordingMetadata::default(),
            events: vec![DomainEvent::RecordingCreated {
                recording_id: id.clone(),
                timestamp: chrono::Utc::now(),
            }],
        };
        
        Ok(recording)
    }
    
    /// 録画ファイル追加
    pub fn add_file(&mut self, file: RecordingFile) -> Result<(), DomainError> {
        // ビジネスルール: 同じファイルタイプの重複チェック
        if self.has_file_type(&file.file_type) && !file.file_type.allows_multiple() {
            return Err(DomainError::DuplicateFileType(file.file_type));
        }
        
        // ファイル追加
        self.files.push(file.clone());
        
        // ドメインイベント記録
        self.events.push(DomainEvent::FileAdded {
            recording_id: self.id.clone(),
            file_id: file.id,
            file_type: file.file_type,
            timestamp: chrono::Utc::now(),
        });
        
        // メタデータ更新
        self.update_aggregated_metadata();
        
        Ok(())
    }
    
    /// 総ファイルサイズ計算
    pub fn total_file_size(&self) -> FileSize {
        self.files.iter()
            .map(|f| f.size)
            .fold(FileSize::zero(), |acc, size| acc + size)
    }
    
    /// 録画時間計算
    pub fn duration(&self) -> Duration {
        self.meeting.end_time - self.meeting.start_time
    }
    
    /// ダウンロード可能性チェック
    pub fn is_downloadable(&self) -> bool {
        !self.files.is_empty() && 
        self.files.iter().any(|f| f.download_info.is_available()) &&
        !self.is_expired()
    }
    
    /// 期限切れチェック
    pub fn is_expired(&self) -> bool {
        if let Some(expiry) = self.settings.expiry_date {
            chrono::Utc::now() > expiry
        } else {
            false
        }
    }
    
    /// AI要約の有無
    pub fn has_ai_summary(&self) -> bool {
        self.metadata.ai_summary.is_some()
    }
    
    /// ビジネスルール検証
    fn validate_recording_data(
        meeting: &MeetingInfo,
        host: &HostInfo,
        settings: &RecordingSettings,
    ) -> Result<(), DomainError> {
        // 会議時間の妥当性
        if meeting.end_time <= meeting.start_time {
            return Err(DomainError::InvalidMeetingDuration);
        }
        
        // ホスト情報の妥当性
        if host.user_id.is_empty() || host.email.is_empty() {
            return Err(DomainError::InvalidHostInfo);
        }
        
        // 録画設定の妥当性
        if let Some(expiry) = settings.expiry_date {
            if expiry <= meeting.start_time {
                return Err(DomainError::InvalidExpiryDate);
            }
        }
        
        Ok(())
    }
}
```

#### 2. 録画ファイルエンティティ
```rust
/// 録画ファイル - 個別ファイルの詳細情報
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordingFile {
    /// ファイルID
    pub id: FileId,
    
    /// ファイル名
    pub name: FileName,
    
    /// ファイル種別
    pub file_type: RecordingFileType,
    
    /// ファイルサイズ
    pub size: FileSize,
    
    /// ファイル形式
    pub format: FileFormat,
    
    /// ダウンロード情報
    pub download_info: DownloadInfo,
    
    /// 録画期間
    pub recording_period: TimePeriod,
    
    /// ファイル品質情報
    pub quality_info: Option<QualityInfo>,
    
    /// チェックサム
    pub checksum: Option<Checksum>,
    
    /// 作成時刻
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl RecordingFile {
    /// 新しいファイルインスタンス作成
    pub fn new(
        id: FileId,
        name: FileName,
        file_type: RecordingFileType,
        size: FileSize,
        format: FileFormat,
    ) -> Self {
        Self {
            id,
            name,
            file_type,
            size,
            format,
            download_info: DownloadInfo::default(),
            recording_period: TimePeriod::default(),
            quality_info: None,
            checksum: None,
            created_at: chrono::Utc::now(),
        }
    }
    
    /// ダウンロード可能性
    pub fn is_downloadable(&self) -> bool {
        self.download_info.is_available() && 
        self.size > FileSize::zero()
    }
    
    /// ファイル整合性検証
    pub fn verify_integrity(&self, actual_checksum: &Checksum) -> bool {
        if let Some(expected_checksum) = &self.checksum {
            expected_checksum == actual_checksum
        } else {
            true  // チェックサムが設定されていない場合は検証スキップ
        }
    }
}
```

### 値オブジェクト (Value Objects)

#### 1. 識別子値オブジェクト
```rust
/// 録画ID値オブジェクト
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RecordingId(String);

impl RecordingId {
    pub fn new(id: String) -> Result<Self, ValidationError> {
        if id.is_empty() || id.len() > 100 {
            return Err(ValidationError::InvalidRecordingId(id));
        }
        
        // Zoom録画IDの形式検証 (英数字とハイフン)
        if !id.chars().all(|c| c.is_alphanumeric() || c == '-') {
            return Err(ValidationError::InvalidRecordingIdFormat(id));
        }
        
        Ok(Self(id))
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for RecordingId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// ファイルID値オブジェクト
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FileId(String);

impl FileId {
    pub fn new(id: String) -> Result<Self, ValidationError> {
        if id.is_empty() || id.len() > 100 {
            return Err(ValidationError::InvalidFileId(id));
        }
        Ok(Self(id))
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

#### 2. 時間関連値オブジェクト
```rust
/// 時間期間値オブジェクト
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimePeriod {
    pub start: chrono::DateTime<chrono::Utc>,
    pub end: chrono::DateTime<chrono::Utc>,
}

impl TimePeriod {
    pub fn new(
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Result<Self, ValidationError> {
        if end <= start {
            return Err(ValidationError::InvalidTimePeriod { start, end });
        }
        
        Ok(Self { start, end })
    }
    
    pub fn duration(&self) -> chrono::Duration {
        self.end - self.start
    }
    
    pub fn contains(&self, timestamp: chrono::DateTime<chrono::Utc>) -> bool {
        timestamp >= self.start && timestamp <= self.end
    }
    
    pub fn overlaps(&self, other: &TimePeriod) -> bool {
        self.start < other.end && other.start < self.end
    }
}

/// 日付範囲値オブジェクト
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DateRange {
    pub from: chrono::NaiveDate,
    pub to: chrono::NaiveDate,
}

impl DateRange {
    pub fn new(from: chrono::NaiveDate, to: chrono::NaiveDate) -> Result<Self, ValidationError> {
        if to < from {
            return Err(ValidationError::InvalidDateRange { from, to });
        }
        
        Ok(Self { from, to })
    }
    
    pub fn contains(&self, date: chrono::NaiveDate) -> bool {
        date >= self.from && date <= self.to
    }
    
    pub fn days_count(&self) -> i64 {
        (self.to - self.from).num_days() + 1
    }
}
```

#### 3. ファイル関連値オブジェクト
```rust
/// ファイルサイズ値オブジェクト
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FileSize(u64);

impl FileSize {
    pub fn new(bytes: u64) -> Self {
        Self(bytes)
    }
    
    pub fn zero() -> Self {
        Self(0)
    }
    
    pub fn bytes(&self) -> u64 {
        self.0
    }
    
    pub fn kb(&self) -> f64 {
        self.0 as f64 / 1024.0
    }
    
    pub fn mb(&self) -> f64 {
        self.0 as f64 / (1024.0 * 1024.0)
    }
    
    pub fn gb(&self) -> f64 {
        self.0 as f64 / (1024.0 * 1024.0 * 1024.0)
    }
    
    pub fn human_readable(&self) -> String {
        if self.0 < 1024 {
            format!("{} B", self.0)
        } else if self.0 < 1024 * 1024 {
            format!("{:.1} KB", self.kb())
        } else if self.0 < 1024 * 1024 * 1024 {
            format!("{:.1} MB", self.mb())
        } else {
            format!("{:.2} GB", self.gb())
        }
    }
}

impl std::ops::Add for FileSize {
    type Output = Self;
    
    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl std::fmt::Display for FileSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.human_readable())
    }
}

/// ファイル名値オブジェクト
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileName(String);

impl FileName {
    pub fn new(name: String) -> Result<Self, ValidationError> {
        if name.is_empty() {
            return Err(ValidationError::EmptyFileName);
        }
        
        if name.len() > 255 {
            return Err(ValidationError::FileNameTooLong(name.len()));
        }
        
        // Windows禁止文字チェック
        let forbidden_chars = ['<', '>', ':', '"', '|', '?', '*'];
        if name.chars().any(|c| forbidden_chars.contains(&c)) {
            return Err(ValidationError::InvalidFileNameCharacters(name));
        }
        
        Ok(Self(name))
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    pub fn extension(&self) -> Option<&str> {
        std::path::Path::new(&self.0)
            .extension()
            .and_then(|ext| ext.to_str())
    }
    
    pub fn stem(&self) -> Option<&str> {
        std::path::Path::new(&self.0)
            .file_stem()
            .and_then(|stem| stem.to_str())
    }
}

/// チェックサム値オブジェクト
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Checksum {
    pub algorithm: ChecksumAlgorithm,
    pub value: String,
}

impl Checksum {
    pub fn new(algorithm: ChecksumAlgorithm, value: String) -> Result<Self, ValidationError> {
        // アルゴリズムに応じた値の形式検証
        match algorithm {
            ChecksumAlgorithm::Sha256 => {
                if value.len() != 64 || !value.chars().all(|c| c.is_ascii_hexdigit()) {
                    return Err(ValidationError::InvalidChecksumFormat(algorithm, value));
                }
            },
            ChecksumAlgorithm::Md5 => {
                if value.len() != 32 || !value.chars().all(|c| c.is_ascii_hexdigit()) {
                    return Err(ValidationError::InvalidChecksumFormat(algorithm, value));
                }
            },
        }
        
        Ok(Self { algorithm, value })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChecksumAlgorithm {
    Sha256,
    Md5,
}
```

### 複合値オブジェクト

#### 1. 会議情報
```rust
/// 会議情報値オブジェクト
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeetingInfo {
    pub meeting_id: String,
    pub topic: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub participant_count: Option<u32>,
    pub meeting_type: MeetingType,
}

impl MeetingInfo {
    pub fn new(
        meeting_id: String,
        topic: String,
        start_time: chrono::DateTime<chrono::Utc>,
        end_time: chrono::DateTime<chrono::Utc>,
        meeting_type: MeetingType,
    ) -> Result<Self, ValidationError> {
        if meeting_id.is_empty() {
            return Err(ValidationError::EmptyMeetingId);
        }
        
        if topic.is_empty() {
            return Err(ValidationError::EmptyMeetingTopic);
        }
        
        if end_time <= start_time {
            return Err(ValidationError::InvalidMeetingDuration);
        }
        
        Ok(Self {
            meeting_id,
            topic,
            start_time,
            end_time,
            participant_count: None,
            meeting_type,
        })
    }
    
    pub fn duration(&self) -> chrono::Duration {
        self.end_time - self.start_time
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeetingType {
    ScheduledMeeting,
    InstantMeeting,
    RecurringMeeting,
    PersonalMeetingRoom,
}
```

#### 2. ホスト情報
```rust
/// ホスト情報値オブジェクト
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostInfo {
    pub user_id: String,
    pub email: String,
    pub display_name: String,
    pub department: Option<String>,
    pub role: UserRole,
}

impl HostInfo {
    pub fn new(
        user_id: String,
        email: String,
        display_name: String,
        role: UserRole,
    ) -> Result<Self, ValidationError> {
        if user_id.is_empty() {
            return Err(ValidationError::EmptyUserId);
        }
        
        if !Self::is_valid_email(&email) {
            return Err(ValidationError::InvalidEmail(email));
        }
        
        if display_name.is_empty() {
            return Err(ValidationError::EmptyDisplayName);
        }
        
        Ok(Self {
            user_id,
            email,
            display_name,
            department: None,
            role,
        })
    }
    
    fn is_valid_email(email: &str) -> bool {
        // 簡易メールアドレス検証
        email.contains('@') && email.contains('.') && email.len() > 5
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserRole {
    Admin,
    Licensed,
    Basic,
}
```

## データ転送オブジェクト (DTOs)

### API連携用DTO

#### 1. Zoom API レスポンスDTO
```rust
/// Zoom API録画レスポンスDTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoomRecordingResponseDto {
    pub meetings: Vec<ZoomMeetingDto>,
    pub next_page_token: Option<String>,
    pub page_size: u32,
    pub total_records: Option<u32>,
}

/// Zoom会議DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoomMeetingDto {
    pub uuid: String,
    pub id: i64,
    pub topic: String,
    pub host_id: String,
    pub host_email: String,
    pub start_time: String,  // ISO 8601 文字列
    pub duration: u32,
    pub total_size: u64,
    pub recording_count: u32,
    pub recording_files: Vec<ZoomRecordingFileDto>,
    pub participant_audio_files: Option<Vec<ZoomRecordingFileDto>>,
    pub summary_url: Option<String>,
}

/// Zoom録画ファイルDTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoomRecordingFileDto {
    pub id: String,
    pub meeting_id: String,
    pub recording_start: String,
    pub recording_end: String,
    pub file_type: String,
    pub file_size: u64,
    pub play_url: Option<String>,
    pub download_url: String,
    pub status: String,
    pub recording_type: String,
}

impl From<ZoomMeetingDto> for Recording {
    fn from(dto: ZoomMeetingDto) -> Self {
        // DTOからドメインエンティティへの変換
        let recording_id = RecordingId::new(dto.uuid)
            .expect("Invalid recording ID from Zoom API");
        
        let start_time = chrono::DateTime::parse_from_rfc3339(&dto.start_time)
            .expect("Invalid start time format")
            .with_timezone(&chrono::Utc);
        
        let end_time = start_time + chrono::Duration::seconds(dto.duration as i64);
        
        let meeting = MeetingInfo::new(
            dto.id.to_string(),
            dto.topic,
            start_time,
            end_time,
            MeetingType::ScheduledMeeting,
        ).expect("Invalid meeting info");
        
        let host = HostInfo::new(
            dto.host_id,
            dto.host_email,
            "".to_string(),  // 表示名はAPIレスポンスに含まれない
            UserRole::Licensed,
        ).expect("Invalid host info");
        
        let settings = RecordingSettings::default();
        
        let mut recording = Recording::new(recording_id, meeting, host, settings)
            .expect("Failed to create recording");
        
        // ファイル情報の追加
        for file_dto in dto.recording_files {
            if let Ok(file) = RecordingFile::try_from(file_dto) {
                let _ = recording.add_file(file);
            }
        }
        
        recording
    }
}
```

### UI表示用DTO

#### 1. 録画リスト表示DTO
```rust
/// 録画リスト表示用DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingListItemDto {
    pub id: String,
    pub topic: String,
    pub host_name: String,
    pub start_date: String,  // YYYY-MM-DD
    pub start_time: String,  // HH:MM
    pub duration: String,    // "1h 30m"
    pub file_count: usize,
    pub total_size: String,  // "123.5 MB"
    pub has_ai_summary: bool,
    pub download_status: DownloadStatusDto,
    pub thumbnail_url: Option<String>,
}

impl From<Recording> for RecordingListItemDto {
    fn from(recording: Recording) -> Self {
        let start_local = recording.meeting.start_time
            .with_timezone(&chrono_tz::Asia::Tokyo);  // 日本時間に変換
        
        Self {
            id: recording.id.to_string(),
            topic: recording.meeting.topic,
            host_name: recording.host.display_name,
            start_date: start_local.format("%Y-%m-%d").to_string(),
            start_time: start_local.format("%H:%M").to_string(),
            duration: format_duration(recording.duration()),
            file_count: recording.files.len(),
            total_size: recording.total_file_size().to_string(),
            has_ai_summary: recording.has_ai_summary(),
            download_status: DownloadStatusDto::from(recording.get_download_status()),
            thumbnail_url: recording.get_thumbnail_url(),
        }
    }
}

/// ダウンロード状況DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DownloadStatusDto {
    NotStarted,
    InProgress { progress: f32 },
    Completed,
    Failed { error: String },
    Paused,
}

/// 期間フォーマット関数
fn format_duration(duration: chrono::Duration) -> String {
    let total_seconds = duration.num_seconds();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    
    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}
```

#### 2. ファイル詳細表示DTO
```rust
/// ファイル詳細表示DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDetailDto {
    pub id: String,
    pub name: String,
    pub file_type: String,
    pub file_type_icon: String,
    pub size: String,
    pub format: String,
    pub duration: Option<String>,
    pub quality: Option<String>,
    pub is_downloadable: bool,
    pub download_url: Option<String>,
    pub local_path: Option<String>,
    pub checksum: Option<String>,
    pub created_at: String,
}

impl From<RecordingFile> for FileDetailDto {
    fn from(file: RecordingFile) -> Self {
        Self {
            id: file.id.to_string(),
            name: file.name.to_string(),
            file_type: format!("{:?}", file.file_type),
            file_type_icon: get_file_type_icon(&file.file_type),
            size: file.size.to_string(),
            format: file.format.to_string(),
            duration: file.quality_info.as_ref()
                .and_then(|q| q.duration)
                .map(|d| format_duration(d)),
            quality: file.quality_info.as_ref()
                .map(|q| format!("{}p @ {}fps", q.resolution.height, q.frame_rate)),
            is_downloadable: file.is_downloadable(),
            download_url: file.download_info.url.clone(),
            local_path: file.download_info.local_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string()),
            checksum: file.checksum.as_ref()
                .map(|c| format!("{}:{}", c.algorithm, c.value)),
            created_at: file.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

fn get_file_type_icon(file_type: &RecordingFileType) -> String {
    match file_type {
        RecordingFileType::Video => "🎥".to_string(),
        RecordingFileType::Audio => "🎵".to_string(),
        RecordingFileType::Chat => "💬".to_string(),
        RecordingFileType::Transcript => "📝".to_string(),
        RecordingFileType::SharedScreen => "🖥️".to_string(),
        RecordingFileType::Whiteboard => "📋".to_string(),
        RecordingFileType::Summary => "📄".to_string(),
        RecordingFileType::Other(_) => "📎".to_string(),
    }
}
```

## 永続化モデル

### ファイル形式設計

#### 1. 設定ファイル (TOML)
```rust
/// アプリケーション設定モデル
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub oauth: OAuthConfig,
    pub download: DownloadConfig,
    pub ui: UiConfig,
    pub advanced: AdvancedConfig,
}

/// OAuth設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    pub client_id: String,
    #[serde(skip_serializing)]  // セキュリティのため出力しない
    pub client_secret: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
}

/// ダウンロード設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadConfig {
    pub output_directory: PathBuf,
    pub concurrent_downloads: u32,
    pub chunk_size_mb: u32,
    pub auto_create_folders: bool,
    pub folder_structure: FolderStructure,
    pub file_naming: FileNamingPattern,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FolderStructure {
    Flat,
    ByDate,
    ByHost,
    ByDateAndHost,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileNamingPattern {
    Original,
    DateTopic,
    TopicDate,
    Custom(String),
}
```

#### 2. キャッシュモデル (JSON)
```rust
/// 録画キャッシュモデル
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingCache {
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub recordings: Vec<CachedRecording>,
    pub metadata: CacheMetadata,
}

/// キャッシュされた録画データ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedRecording {
    pub id: String,
    pub data: Recording,
    pub cached_at: chrono::DateTime<chrono::Utc>,
    pub ttl: Option<chrono::Duration>,
}

/// キャッシュメタデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    pub version: String,
    pub total_count: usize,
    pub size_bytes: u64,
    pub last_cleanup: chrono::DateTime<chrono::Utc>,
}

impl RecordingCache {
    /// キャッシュから録画取得
    pub fn get_recording(&self, id: &RecordingId) -> Option<&Recording> {
        self.recordings
            .iter()
            .find(|cached| cached.id == id.as_str())
            .filter(|cached| !cached.is_expired())
            .map(|cached| &cached.data)
    }
    
    /// キャッシュに録画追加
    pub fn add_recording(&mut self, recording: Recording, ttl: Option<chrono::Duration>) {
        let cached = CachedRecording {
            id: recording.id.to_string(),
            data: recording,
            cached_at: chrono::Utc::now(),
            ttl,
        };
        
        // 既存エントリの更新または新規追加
        if let Some(existing) = self.recordings
            .iter_mut()
            .find(|c| c.id == cached.id) {
            *existing = cached;
        } else {
            self.recordings.push(cached);
        }
        
        self.update_metadata();
    }
    
    /// 期限切れエントリの削除
    pub fn cleanup_expired(&mut self) {
        let initial_count = self.recordings.len();
        self.recordings.retain(|cached| !cached.is_expired());
        
        if self.recordings.len() != initial_count {
            self.update_metadata();
        }
    }
}

impl CachedRecording {
    fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            chrono::Utc::now() > self.cached_at + ttl
        } else {
            false
        }
    }
}
```

## バリデーション設計

### ドメインバリデーション
```rust
/// ドメインバリデーター
pub struct DomainValidator {
    rules: Vec<Box<dyn ValidationRule>>,
}

impl DomainValidator {
    pub fn new() -> Self {
        let mut rules: Vec<Box<dyn ValidationRule>> = vec![
            Box::new(RecordingIdValidationRule),
            Box::new(FileNameValidationRule),
            Box::new(TimePeriodValidationRule),
            Box::new(FileSizeValidationRule),
            Box::new(EmailValidationRule),
        ];
        
        Self { rules }
    }
    
    pub fn validate<T>(&self, data: &T) -> ValidationResult
    where
        T: Validate,
    {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        for rule in &self.rules {
            match rule.validate(data) {
                Ok(result) => {
                    warnings.extend(result.warnings);
                },
                Err(error) => {
                    errors.push(error);
                }
            }
        }
        
        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        }
    }
}

/// バリデーションルールトレイト
pub trait ValidationRule {
    fn validate<T>(&self, data: &T) -> Result<ValidationResult, ValidationError>
    where
        T: Validate;
}

/// 録画IDバリデーションルール
pub struct RecordingIdValidationRule;

impl ValidationRule for RecordingIdValidationRule {
    fn validate<T>(&self, data: &T) -> Result<ValidationResult, ValidationError>
    where
        T: Validate,
    {
        // 具体的なバリデーションロジック
        // ...
        Ok(ValidationResult::valid())
    }
}
```

### データ整合性チェック
```rust
/// データ整合性チェッカー
pub struct DataIntegrityChecker;

impl DataIntegrityChecker {
    /// 録画データの整合性チェック
    pub fn check_recording_integrity(recording: &Recording) -> IntegrityCheckResult {
        let mut issues = Vec::new();
        
        // 1. 時系列データの整合性
        if recording.meeting.end_time <= recording.meeting.start_time {
            issues.push(IntegrityIssue::InvalidTimeOrder {
                start: recording.meeting.start_time,
                end: recording.meeting.end_time,
            });
        }
        
        // 2. ファイルサイズの整合性
        let declared_total = recording.total_file_size();
        let actual_total = recording.files.iter()
            .map(|f| f.size)
            .fold(FileSize::zero(), |acc, size| acc + size);
        
        if declared_total != actual_total {
            issues.push(IntegrityIssue::FileSizeMismatch {
                declared: declared_total,
                actual: actual_total,
            });
        }
        
        // 3. 必須ファイルの存在チェック
        let has_video = recording.files.iter()
            .any(|f| f.file_type == RecordingFileType::Video);
        
        if !has_video {
            issues.push(IntegrityIssue::MissingRequiredFile {
                file_type: RecordingFileType::Video,
            });
        }
        
        IntegrityCheckResult {
            is_valid: issues.is_empty(),
            issues,
        }
    }
}

/// 整合性チェック結果
#[derive(Debug)]
pub struct IntegrityCheckResult {
    pub is_valid: bool,
    pub issues: Vec<IntegrityIssue>,
}

/// 整合性問題
#[derive(Debug)]
pub enum IntegrityIssue {
    InvalidTimeOrder {
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    },
    FileSizeMismatch {
        declared: FileSize,
        actual: FileSize,
    },
    MissingRequiredFile {
        file_type: RecordingFileType,
    },
    DuplicateFileType {
        file_type: RecordingFileType,
    },
}
```

## テスト支援

### テストデータビルダー
```rust
/// 録画テストデータビルダー
pub struct RecordingTestDataBuilder {
    id: Option<RecordingId>,
    meeting: Option<MeetingInfo>,
    host: Option<HostInfo>,
    settings: Option<RecordingSettings>,
    files: Vec<RecordingFile>,
}

impl RecordingTestDataBuilder {
    pub fn new() -> Self {
        Self {
            id: None,
            meeting: None,
            host: None,
            settings: None,
            files: Vec::new(),
        }
    }
    
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(RecordingId::new(id.into()).expect("Invalid test ID"));
        self
    }
    
    pub fn with_topic(mut self, topic: impl Into<String>) -> Self {
        let meeting = self.meeting.unwrap_or_else(|| default_meeting_info());
        self.meeting = Some(MeetingInfo {
            topic: topic.into(),
            ..meeting
        });
        self
    }
    
    pub fn with_file(mut self, file: RecordingFile) -> Self {
        self.files.push(file);
        self
    }
    
    pub fn with_video_file(self) -> Self {
        let file = RecordingFile::new(
            FileId::new("test_video".to_string()).unwrap(),
            FileName::new("video.mp4".to_string()).unwrap(),
            RecordingFileType::Video,
            FileSize::new(1024 * 1024 * 100), // 100MB
            FileFormat::new("mp4".to_string()).unwrap(),
        );
        self.with_file(file)
    }
    
    pub fn build(self) -> Recording {
        let id = self.id.unwrap_or_else(|| RecordingId::new("test_recording".to_string()).unwrap());
        let meeting = self.meeting.unwrap_or_else(|| default_meeting_info());
        let host = self.host.unwrap_or_else(|| default_host_info());
        let settings = self.settings.unwrap_or_else(|| RecordingSettings::default());
        
        let mut recording = Recording::new(id, meeting, host, settings)
            .expect("Failed to create test recording");
        
        for file in self.files {
            recording.add_file(file).expect("Failed to add test file");
        }
        
        recording
    }
}

fn default_meeting_info() -> MeetingInfo {
    MeetingInfo::new(
        "12345".to_string(),
        "Test Meeting".to_string(),
        chrono::Utc::now() - chrono::Duration::hours(1),
        chrono::Utc::now(),
        MeetingType::ScheduledMeeting,
    ).expect("Failed to create default meeting info")
}

fn default_host_info() -> HostInfo {
    HostInfo::new(
        "test_user".to_string(),
        "test@example.com".to_string(),
        "Test User".to_string(),
        UserRole::Licensed,
    ).expect("Failed to create default host info")
}
```

### Property-basedテスト生成器
```rust
/// 任意録画データ生成器
pub fn arb_recording() -> impl Strategy<Value = Recording> {
    (
        arb_recording_id(),
        arb_meeting_info(),
        arb_host_info(),
        arb_recording_settings(),
        prop::collection::vec(arb_recording_file(), 0..5),
    ).prop_map(|(id, meeting, host, settings, files)| {
        let mut recording = Recording::new(id, meeting, host, settings)
            .expect("Invalid generated recording data");
        
        for file in files {
            let _ = recording.add_file(file);
        }
        
        recording
    })
}

fn arb_recording_id() -> impl Strategy<Value = RecordingId> {
    "[a-zA-Z0-9\\-]{10,20}".prop_map(|s| {
        RecordingId::new(s).expect("Invalid generated recording ID")
    })
}

fn arb_file_size() -> impl Strategy<Value = FileSize> {
    (1u64..1_000_000_000u64).prop_map(FileSize::new)
}

fn arb_time_period() -> impl Strategy<Value = TimePeriod> {
    (
        0i64..86400i64,  // 開始時間のオフセット（秒）
        1i64..7200i64,   // 継続時間（秒）
    ).prop_map(|(start_offset, duration)| {
        let start = chrono::Utc::now() + chrono::Duration::seconds(start_offset);
        let end = start + chrono::Duration::seconds(duration);
        TimePeriod::new(start, end).expect("Invalid generated time period")
    })
}
```

---

**承認**:  
データ設計者: [ ] 承認  
ドメインエキスパート: [ ] 承認  
**承認日**: ___________