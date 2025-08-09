# 録画管理コンポーネント詳細設計書 - Zoom Video Mover

## 文書概要
**文書ID**: DES-RECORDING-001  
**コンポーネント名**: 録画管理コンポーネント（Recording Management Component）  
**作成日**: 2025-08-03  
  
**バージョン**: 1.0  

## コンポーネント概要

### 責任・役割
- **録画メタデータ管理**: API取得データの解析・構造化・品質保証
- **フィルタリング機能**: 多様な条件による録画データの検索・絞り込み
- **ファイル種別分析**: 録画ファイルの分類・命名・重複処理
- **AI要約統合**: AI Companion要約データの取得・構造化・管理

### アーキテクチャ位置
```
┌─────────────────────────────────────────────────────────────────┐
│                   Application Layer                             │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │              Recording Management Component                  │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │ │
│  │  │   Metadata  │  │   Filter    │  │     File Type       │ │ │
│  │  │   Manager   │  │   Engine    │  │     Analyzer        │ │ │
│  │  │             │  │             │  │                     │ │ │
│  │  └─────────────┘  └─────────────┘  └─────────────────────┘ │ │
│  │  ┌─────────────────────────────────────────────────────────┐ │ │
│  │  │           AI Summary Integrator                         │ │ │
│  │  └─────────────────────────────────────────────────────────┘ │ │
│  └─────────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                 Infrastructure Layer                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │   Data      │  │   Search    │  │    Validation           │  │
│  │ Normalizer  │  │   Index     │  │    Engine               │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## モジュール構造設計

### 内部モジュール構成
```rust
pub mod recording {
    /// メタデータ管理
    pub mod metadata_manager;
    
    /// フィルタリングエンジン
    pub mod filter_engine;
    
    /// ファイル種別分析
    pub mod file_analyzer;
    
    /// AI要約統合
    pub mod ai_summary_integrator;
    
    /// データ正規化
    pub mod data_normalizer;
    
    /// 検索インデックス
    pub mod search_index;
    
    /// 検証エンジン
    pub mod validation_engine;
    
    /// エラー定義
    pub mod error;
    
    /// 設定・定数
    pub mod config;
}
```

### モジュール依存関係
```
metadata_manager
    ├── → data_normalizer
    ├── → validation_engine
    ├── → search_index
    └── → error

filter_engine
    ├── → search_index
    ├── → data_normalizer
    └── → error

file_analyzer
    ├── → validation_engine
    └── → error

ai_summary_integrator
    ├── → metadata_manager
    ├── → validation_engine
    └── → error

data_normalizer
    └── → error

search_index
    └── → error

validation_engine
    └── → error
```

## データ構造設計

### コアデータ構造

#### 1. 録画メタデータ
```rust
/// 録画メタデータ（内部管理用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingMetadata {
    /// 録画ID（Zoom固有）
    pub id: String,
    
    /// 会議ID
    pub meeting_id: String,
    
    /// 会議トピック（正規化済み）
    pub topic: String,
    
    /// 録画開始時刻
    pub start_time: chrono::DateTime<chrono::Utc>,
    
    /// 録画終了時刻
    pub end_time: chrono::DateTime<chrono::Utc>,
    
    /// 録画時間（秒）
    pub duration_seconds: u32,
    
    /// 録画ファイル一覧
    pub files: Vec<RecordingFileInfo>,
    
    /// ホスト情報
    pub host_info: HostInfo,
    
    /// 参加者情報
    pub participants: Vec<ParticipantInfo>,
    
    /// 録画設定
    pub settings: RecordingSettings,
    
    /// AI要約情報
    pub ai_summary: Option<AISummaryData>,
    
    /// メタデータ品質情報
    pub quality_info: MetadataQualityInfo,
    
    /// 処理状態
    pub processing_state: ProcessingState,
    
    /// 最終更新時刻
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// 録画ファイル情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingFileInfo {
    /// ファイルID
    pub id: String,
    
    /// ファイル名（正規化済み）
    pub file_name: String,
    
    /// 元のファイル名
    pub original_name: String,
    
    /// ファイル種別
    pub file_type: RecordingFileType,
    
    /// ファイルサイズ（バイト）
    pub file_size: u64,
    
    /// ダウンロードURL
    pub download_url: String,
    
    /// ファイル作成時刻
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// ファイル形式
    pub format: FileFormat,
    
    /// 品質情報
    pub quality: Option<FileQualityInfo>,
    
    /// ハッシュ値（検証用）
    pub hash: Option<FileHash>,
    
    /// 処理状態
    pub state: FileProcessingState,
}

/// ファイル種別（詳細分類）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecordingFileType {
    /// 動画ファイル
    Video {
        resolution: Option<VideoResolution>,
        codec: Option<String>,
    },
    
    /// 音声ファイル
    Audio {
        quality: Option<AudioQuality>,
        codec: Option<String>,
    },
    
    /// チャットファイル
    Chat {
        format: ChatFormat,
        encoding: Option<String>,
    },
    
    /// トランスクリプト
    Transcript {
        format: TranscriptFormat,
        language: Option<String>,
    },
    
    /// 共有画面録画
    SharedScreen {
        resolution: Option<VideoResolution>,
    },
    
    /// ホワイトボード
    Whiteboard {
        format: WhiteboardFormat,
    },
    
    /// AI生成要約
    AISummary {
        format: SummaryFormat,
        confidence: Option<f64>,
    },
    
    /// その他
    Other {
        type_name: String,
        mime_type: Option<String>,
    },
}
```

#### 2. フィルタリング条件
```rust
/// フィルタリング条件（複合条件対応）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterCriteria {
    /// 日付範囲フィルタ
    pub date_range: Option<DateRangeFilter>,
    
    /// ファイル種別フィルタ
    pub file_types: Option<Vec<RecordingFileType>>,
    
    /// ファイルサイズフィルタ
    pub file_size: Option<FileSizeFilter>,
    
    /// テキスト検索条件
    pub text_search: Option<TextSearchFilter>,
    
    /// 会議時間フィルタ
    pub duration: Option<DurationFilter>,
    
    /// ホストフィルタ
    pub hosts: Option<Vec<String>>,
    
    /// AI要約フィルタ
    pub ai_summary: Option<AISummaryFilter>,
    
    /// カスタムフィルタ
    pub custom_filters: HashMap<String, FilterValue>,
    
    /// 論理演算子
    pub logic_operator: LogicOperator,
}

/// 日付範囲フィルタ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRangeFilter {
    /// 開始日（YYYY-MM-DD）
    pub from_date: chrono::NaiveDate,
    
    /// 終了日（YYYY-MM-DD）
    pub to_date: chrono::NaiveDate,
    
    /// 時刻範囲（オプション）
    pub time_range: Option<TimeRangeFilter>,
    
    /// タイムゾーン
    pub timezone: Option<chrono_tz::Tz>,
}

/// テキスト検索フィルタ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSearchFilter {
    /// 検索クエリ
    pub query: String,
    
    /// 検索対象フィールド
    pub fields: Vec<SearchField>,
    
    /// 検索モード
    pub mode: SearchMode,
    
    /// 大文字小文字区別
    pub case_sensitive: bool,
    
    /// 正規表現使用
    pub regex: bool,
}

/// 検索モード
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchMode {
    /// 部分一致
    Partial,
    
    /// 完全一致
    Exact,
    
    /// 前方一致
    StartsWith,
    
    /// 後方一致
    EndsWith,
    
    /// 全文検索
    FullText,
}
```

#### 3. AI要約データ
```rust
/// AI要約統合データ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AISummaryData {
    /// 要約ID
    pub summary_id: String,
    
    /// 会議要約
    pub meeting_summary: String,
    
    /// キーポイント
    pub key_points: Vec<KeyPoint>,
    
    /// アクションアイテム
    pub action_items: Vec<ActionItem>,
    
    /// 参加者インサイト
    pub participant_insights: Vec<ParticipantInsight>,
    
    /// 会議メトリクス
    pub meeting_metrics: MeetingMetrics,
    
    /// 要約品質情報
    pub quality_score: f64,
    
    /// 生成時刻
    pub generated_at: chrono::DateTime<chrono::Utc>,
    
    /// AI モデル情報
    pub model_info: AIModelInfo,
}

/// キーポイント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPoint {
    /// ポイントテキスト
    pub text: String,
    
    /// 重要度スコア（0.0-1.0）
    pub importance_score: f64,
    
    /// タイムスタンプ（会議内での位置）
    pub timestamp: Option<Duration>,
    
    /// カテゴリ
    pub category: KeyPointCategory,
    
    /// 関連する発言者
    pub speakers: Vec<String>,
}

/// アクションアイテム
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    /// アクション内容
    pub description: String,
    
    /// 担当者
    pub assignee: Option<String>,
    
    /// 期限
    pub due_date: Option<chrono::NaiveDate>,
    
    /// 優先度
    pub priority: ActionPriority,
    
    /// ステータス
    pub status: ActionStatus,
    
    /// 関連する会議内容
    pub related_content: Option<String>,
}
```

## インターフェース設計

### 公開API

#### 1. 録画管理マネージャー
```rust
/// 録画管理マネージャー - コンポーネントのメインインターフェース
#[async_trait]
pub trait RecordingManager: Send + Sync {
    /// 録画メタデータの解析・管理
    async fn process_recording_metadata(&self, raw_data: Vec<serde_json::Value>) -> Result<Vec<RecordingMetadata>, RecordingError>;
    
    /// フィルタリング条件による録画検索
    async fn search_recordings(&self, criteria: FilterCriteria) -> Result<Vec<RecordingMetadata>, RecordingError>;
    
    /// 録画データの詳細取得
    async fn get_recording_details(&self, recording_id: &str) -> Result<RecordingMetadata, RecordingError>;
    
    /// AI要約データの統合
    async fn integrate_ai_summary(&self, recording_id: &str, summary_data: AISummaryData) -> Result<(), RecordingError>;
    
    /// ファイル種別分析・分類
    async fn analyze_file_types(&self, recording: &mut RecordingMetadata) -> Result<(), RecordingError>;
    
    /// データ品質検証
    async fn validate_data_quality(&self, recording: &RecordingMetadata) -> Result<DataQualityReport, RecordingError>;
    
    /// 録画統計情報取得
    fn get_recording_statistics(&self) -> RecordingStatistics;
    
    /// フィルタ履歴の管理
    async fn save_filter_preset(&self, name: String, criteria: FilterCriteria) -> Result<(), RecordingError>;
    async fn load_filter_preset(&self, name: &str) -> Result<FilterCriteria, RecordingError>;
}
```

#### 2. 実装クラス
```rust
/// 録画管理マネージャー実装
pub struct ZoomRecordingManager {
    /// メタデータマネージャー
    metadata_manager: Arc<MetadataManager>,
    
    /// フィルタリングエンジン
    filter_engine: Arc<FilterEngine>,
    
    /// ファイル分析器
    file_analyzer: Arc<FileAnalyzer>,
    
    /// AI要約統合器
    ai_integrator: Arc<AISummaryIntegrator>,
    
    /// データ正規化器
    data_normalizer: Arc<DataNormalizer>,
    
    /// 検証エンジン
    validation_engine: Arc<ValidationEngine>,
    
    /// 設定情報
    config: RecordingConfig,
    
    /// 統計収集器
    statistics_collector: Arc<StatisticsCollector>,
}

impl ZoomRecordingManager {
    /// 新しい録画管理マネージャーを作成
    pub fn new(config: RecordingConfig) -> Result<Self, RecordingError> {
        let metadata_manager = Arc::new(MetadataManager::new(&config)?);
        let filter_engine = Arc::new(FilterEngine::new(&config)?);
        let file_analyzer = Arc::new(FileAnalyzer::new(&config)?);
        let ai_integrator = Arc::new(AISummaryIntegrator::new(&config)?);
        let data_normalizer = Arc::new(DataNormalizer::new());
        let validation_engine = Arc::new(ValidationEngine::new(&config)?);
        let statistics_collector = Arc::new(StatisticsCollector::new());
        
        Ok(Self {
            metadata_manager,
            filter_engine,
            file_analyzer,
            ai_integrator,
            data_normalizer,
            validation_engine,
            config,
            statistics_collector,
        })
    }
}

#[async_trait]
impl RecordingManager for ZoomRecordingManager {
    async fn process_recording_metadata(&self, raw_data: Vec<serde_json::Value>) -> Result<Vec<RecordingMetadata>, RecordingError> {
        let start_time = Instant::now();
        let mut processed_recordings = Vec::new();
        
        for raw_recording in raw_data {
            // 1. 基本メタデータ解析
            let mut recording = self.metadata_manager.parse_raw_metadata(raw_recording).await?;
            
            // 2. データ正規化
            self.data_normalizer.normalize_recording(&mut recording).await?;
            
            // 3. ファイル種別分析
            self.file_analyzer.analyze_files(&mut recording).await?;
            
            // 4. データ品質検証
            let quality_report = self.validation_engine.validate_recording(&recording).await?;
            recording.quality_info = quality_report.into();
            
            // 5. 検索インデックス更新
            self.metadata_manager.update_search_index(&recording).await?;
            
            processed_recordings.push(recording);
        }
        
        // 6. 統計情報更新
        let processing_time = start_time.elapsed();
        self.statistics_collector.record_processing_metrics(
            processed_recordings.len(),
            processing_time,
        ).await;
        
        Ok(processed_recordings)
    }
    
    async fn search_recordings(&self, criteria: FilterCriteria) -> Result<Vec<RecordingMetadata>, RecordingError> {
        // 1. フィルタ条件の検証
        self.filter_engine.validate_criteria(&criteria)?;
        
        // 2. 検索実行
        let search_results = self.filter_engine.execute_search(criteria).await?;
        
        // 3. 結果の詳細データ取得
        let mut detailed_results = Vec::new();
        for result_id in search_results {
            if let Ok(recording) = self.metadata_manager.get_recording_by_id(&result_id).await {
                detailed_results.push(recording);
            }
        }
        
        // 4. 検索履歴記録
        self.statistics_collector.record_search_metrics(&criteria, detailed_results.len()).await;
        
        Ok(detailed_results)
    }
}
```

### 内部インターフェース

#### 1. メタデータマネージャー
```rust
/// メタデータ管理インターフェース
#[async_trait]
pub trait MetadataManager: Send + Sync {
    /// 生データからメタデータ解析
    async fn parse_raw_metadata(&self, raw_data: serde_json::Value) -> Result<RecordingMetadata, MetadataError>;
    
    /// メタデータの検証・補完
    async fn validate_and_complete(&self, metadata: &mut RecordingMetadata) -> Result<(), MetadataError>;
    
    /// 検索インデックス更新
    async fn update_search_index(&self, metadata: &RecordingMetadata) -> Result<(), MetadataError>;
    
    /// ID による録画取得
    async fn get_recording_by_id(&self, id: &str) -> Result<RecordingMetadata, MetadataError>;
}

/// メタデータマネージャー実装
pub struct ZoomMetadataManager {
    /// データパーサー
    data_parser: Arc<DataParser>,
    
    /// フィールドバリデーター
    field_validators: Vec<Box<dyn FieldValidator>>,
    
    /// 検索インデックス
    search_index: Arc<SearchIndex>,
    
    /// メタデータキャッシュ
    metadata_cache: Arc<MetadataCache>,
}

impl ZoomMetadataManager {
    pub fn new(config: &RecordingConfig) -> Result<Self, MetadataError> {
        let data_parser = Arc::new(DataParser::new());
        let field_validators = vec![
            Box::new(DateTimeValidator::new()),
            Box::new(UrlValidator::new()),
            Box::new(FileSizeValidator::new()),
            Box::new(DurationValidator::new()),
            Box::new(HostInfoValidator::new()),
        ];
        let search_index = Arc::new(SearchIndex::new(config.index_config.clone())?);
        let metadata_cache = Arc::new(MetadataCache::new(config.cache_config.clone()));
        
        Ok(Self {
            data_parser,
            field_validators,
            search_index,
            metadata_cache,
        })
    }
}

#[async_trait]
impl MetadataManager for ZoomMetadataManager {
    async fn parse_raw_metadata(&self, raw_data: serde_json::Value) -> Result<RecordingMetadata, MetadataError> {
        // 1. JSON 構造検証
        self.data_parser.validate_json_structure(&raw_data)?;
        
        // 2. 基本フィールド抽出
        let mut metadata = self.data_parser.extract_base_fields(raw_data)?;
        
        // 3. フィールド個別検証
        for validator in &self.field_validators {
            validator.validate_metadata(&mut metadata)?;
        }
        
        // 4. 不足フィールド補完
        self.complete_missing_fields(&mut metadata).await?;
        
        // 5. 処理状態設定
        metadata.processing_state = ProcessingState::Parsed;
        metadata.last_updated = chrono::Utc::now();
        
        Ok(metadata)
    }
    
    async fn validate_and_complete(&self, metadata: &mut RecordingMetadata) -> Result<(), MetadataError> {
        // 1. 必須フィールド存在確認
        self.validate_required_fields(metadata)?;
        
        // 2. データ型・形式検証
        self.validate_data_formats(metadata)?;
        
        // 3. 論理的整合性確認
        self.validate_logical_consistency(metadata)?;
        
        // 4. 不足データ補完
        self.complete_missing_fields(metadata).await?;
        
        metadata.processing_state = ProcessingState::Validated;
        
        Ok(())
    }
}
```

#### 2. フィルタリングエンジン
```rust
/// フィルタリングエンジンインターフェース
#[async_trait]
pub trait FilterEngine: Send + Sync {
    /// フィルタ条件の検証
    fn validate_criteria(&self, criteria: &FilterCriteria) -> Result<(), FilterError>;
    
    /// 検索実行
    async fn execute_search(&self, criteria: FilterCriteria) -> Result<Vec<String>, FilterError>;
    
    /// 複合条件処理
    async fn apply_complex_filters(&self, recordings: Vec<RecordingMetadata>, criteria: FilterCriteria) -> Result<Vec<RecordingMetadata>, FilterError>;
    
    /// 検索結果ソート
    fn sort_results(&self, recordings: &mut Vec<RecordingMetadata>, sort_criteria: SortCriteria) -> Result<(), FilterError>;
}

/// フィルタリングエンジン実装
pub struct RustFilterEngine {
    /// インデックス検索エンジン
    index_searcher: Arc<IndexSearcher>,
    
    /// フィルタプロセッサ群
    filter_processors: HashMap<FilterType, Box<dyn FilterProcessor>>,
    
    /// ソートプロセッサ
    sort_processor: Arc<SortProcessor>,
    
    /// 性能監視
    performance_monitor: Arc<FilterPerformanceMonitor>,
}

impl RustFilterEngine {
    pub fn new(config: &FilterConfig) -> Result<Self, FilterError> {
        let index_searcher = Arc::new(IndexSearcher::new(config.index_config.clone())?);
        
        let mut filter_processors: HashMap<FilterType, Box<dyn FilterProcessor>> = HashMap::new();
        filter_processors.insert(FilterType::DateRange, Box::new(DateRangeProcessor::new()));
        filter_processors.insert(FilterType::FileType, Box::new(FileTypeProcessor::new()));
        filter_processors.insert(FilterType::TextSearch, Box::new(TextSearchProcessor::new()));
        filter_processors.insert(FilterType::FileSize, Box::new(FileSizeProcessor::new()));
        filter_processors.insert(FilterType::Duration, Box::new(DurationProcessor::new()));
        
        let sort_processor = Arc::new(SortProcessor::new());
        let performance_monitor = Arc::new(FilterPerformanceMonitor::new());
        
        Ok(Self {
            index_searcher,
            filter_processors,
            sort_processor,
            performance_monitor,
        })
    }
}

#[async_trait]
impl FilterEngine for RustFilterEngine {
    async fn execute_search(&self, criteria: FilterCriteria) -> Result<Vec<String>, FilterError> {
        let start_time = Instant::now();
        
        // 1. 基本インデックス検索
        let mut candidate_ids = self.index_searcher.search_basic_criteria(&criteria).await?;
        
        // 2. 複合フィルタ適用
        for (filter_type, processor) in &self.filter_processors {
            if criteria.has_filter(filter_type) {
                candidate_ids = processor.apply_filter(candidate_ids, &criteria).await?;
            }
        }
        
        // 3. 論理演算子適用（AND/OR/NOT）
        candidate_ids = self.apply_logic_operators(candidate_ids, &criteria).await?;
        
        // 4. 性能監視記録
        let elapsed = start_time.elapsed();
        self.performance_monitor.record_search_performance(
            &criteria,
            candidate_ids.len(),
            elapsed,
        ).await;
        
        Ok(candidate_ids)
    }
    
    async fn apply_complex_filters(&self, recordings: Vec<RecordingMetadata>, criteria: FilterCriteria) -> Result<Vec<RecordingMetadata>, FilterError> {
        let mut filtered_recordings = recordings;
        
        // 1. 各フィルタを順次適用
        for (filter_type, processor) in &self.filter_processors {
            if criteria.has_filter(filter_type) {
                filtered_recordings = processor.filter_recordings(filtered_recordings, &criteria).await?;
            }
        }
        
        // 2. カスタムフィルタ適用
        if !criteria.custom_filters.is_empty() {
            filtered_recordings = self.apply_custom_filters(filtered_recordings, &criteria.custom_filters).await?;
        }
        
        Ok(filtered_recordings)
    }
}
```

## アルゴリズム設計

### データ正規化アルゴリズム

#### 文字列正規化処理
```rust
impl DataNormalizer {
    /// 録画データの正規化
    pub async fn normalize_recording(&self, recording: &mut RecordingMetadata) -> Result<(), NormalizationError> {
        // 1. 文字列フィールド正規化
        recording.topic = self.normalize_text(&recording.topic)?;
        
        // 2. ファイル名正規化
        for file in &mut recording.files {
            file.file_name = self.normalize_filename(&file.original_name)?;
        }
        
        // 3. 日時正規化（UTC統一）
        self.normalize_timestamps(recording)?;
        
        // 4. サイズ単位正規化
        self.normalize_file_sizes(recording)?;
        
        Ok(())
    }
    
    /// ファイル名正規化（Windows対応）
    fn normalize_filename(&self, original_name: &str) -> Result<String, NormalizationError> {
        let mut normalized = original_name.to_string();
        
        // 1. Windows 禁止文字除去・置換
        let forbidden_chars = ['<', '>', ':', '"', '|', '?', '*', '\\', '/'];
        for forbidden in forbidden_chars {
            normalized = normalized.replace(forbidden, "_");
        }
        
        // 2. 制御文字除去
        normalized = normalized.chars()
            .filter(|c| !c.is_control())
            .collect();
        
        // 3. 連続する空白・アンダースコア正規化
        normalized = Self::normalize_consecutive_chars(&normalized, &[' ', '_']);
        
        // 4. パス長制限（Windows: 260文字）
        if normalized.len() > 200 {
            normalized = Self::truncate_filename(normalized, 200)?;
        }
        
        // 5. 予約名回避
        normalized = Self::avoid_reserved_names(normalized)?;
        
        Ok(normalized)
    }
    
    /// 重複ファイル名処理
    pub fn resolve_duplicate_filename(&self, base_name: &str, existing_names: &HashSet<String>) -> String {
        if !existing_names.contains(base_name) {
            return base_name.to_string();
        }
        
        // ファイル名と拡張子を分離
        let (name_part, extension) = Self::split_filename(base_name);
        
        // 連番付与による重複回避
        for i in 1..=999 {
            let candidate = format!("{}_{}{}", name_part, i, extension);
            if !existing_names.contains(&candidate) {
                return candidate;
            }
        }
        
        // 999まで使い切った場合はタイムスタンプ付与
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        format!("{}_{}{}", name_part, timestamp, extension)
    }
}
```

### フィルタリング最適化アルゴリズム

#### インデックス検索最適化
```rust
/// 高速検索のためのインデックス構造
pub struct OptimizedSearchIndex {
    /// 日付インデックス（B-Tree）
    date_index: BTreeMap<chrono::NaiveDate, HashSet<String>>,
    
    /// ファイル種別インデックス
    file_type_index: HashMap<RecordingFileType, HashSet<String>>,
    
    /// 全文検索インデックス（転置インデックス）
    text_index: InvertedIndex,
    
    /// サイズ範囲インデックス
    size_range_index: RangeIndex<u64>,
    
    /// 複合インデックス（よく使われる条件組み合わせ）
    composite_indexes: HashMap<CompositeKey, HashSet<String>>,
}

impl OptimizedSearchIndex {
    /// 複合条件の効率的検索
    pub async fn search_with_optimization(&self, criteria: &FilterCriteria) -> Result<Vec<String>, SearchError> {
        // 1. 選択性分析（最も絞り込める条件から適用）
        let filter_selectivity = self.analyze_filter_selectivity(criteria).await?;
        let ordered_filters = self.order_filters_by_selectivity(criteria, filter_selectivity);
        
        // 2. 初期候補セット取得（最も選択性の高いフィルタから）
        let mut candidates = if let Some(first_filter) = ordered_filters.first() {
            self.apply_single_filter(first_filter).await?
        } else {
            return Ok(Vec::new());
        };
        
        // 3. 残りのフィルタを順次適用（積集合演算）
        for filter in ordered_filters.iter().skip(1) {
            let filter_results = self.apply_single_filter(filter).await?;
            candidates = Self::intersect_sets(candidates, filter_results);
            
            // 早期終了判定（候補が十分絞り込まれた場合）
            if candidates.len() < 100 {
                break;
            }
        }
        
        Ok(candidates.into_iter().collect())
    }
    
    /// フィルタ選択性分析
    async fn analyze_filter_selectivity(&self, criteria: &FilterCriteria) -> Result<HashMap<FilterType, f64>, SearchError> {
        let mut selectivity = HashMap::new();
        
        // 各フィルタの推定ヒット率計算
        if let Some(date_range) = &criteria.date_range {
            let estimated_hits = self.estimate_date_range_hits(date_range).await?;
            selectivity.insert(FilterType::DateRange, estimated_hits);
        }
        
        if let Some(file_types) = &criteria.file_types {
            let estimated_hits = self.estimate_file_type_hits(file_types).await?;
            selectivity.insert(FilterType::FileType, estimated_hits);
        }
        
        if let Some(text_search) = &criteria.text_search {
            let estimated_hits = self.estimate_text_search_hits(text_search).await?;
            selectivity.insert(FilterType::TextSearch, estimated_hits);
        }
        
        Ok(selectivity)
    }
}
```

### AI要約統合アルゴリズム

#### 要約データ統合・構造化
```rust
impl AISummaryIntegrator {
    /// AI要約データの統合処理
    pub async fn integrate_summary_data(&self, recording_id: &str, raw_summary: serde_json::Value) -> Result<AISummaryData, AISummaryError> {
        // 1. 要約データ構造解析
        let summary_structure = self.analyze_summary_structure(&raw_summary)?;
        
        // 2. 基本要約情報抽出
        let mut summary_data = self.extract_basic_summary(&raw_summary, &summary_structure)?;
        
        // 3. キーポイント解析・重要度スコア計算
        summary_data.key_points = self.extract_and_score_key_points(&raw_summary).await?;
        
        // 4. アクションアイテム抽出・構造化
        summary_data.action_items = self.extract_action_items(&raw_summary).await?;
        
        // 5. 参加者インサイト分析
        summary_data.participant_insights = self.analyze_participant_insights(&raw_summary).await?;
        
        // 6. 要約品質評価
        summary_data.quality_score = self.calculate_quality_score(&summary_data).await?;
        
        // 7. メタデータ設定
        summary_data.generated_at = chrono::Utc::now();
        summary_data.model_info = self.extract_model_info(&raw_summary)?;
        
        Ok(summary_data)
    }
    
    /// キーポイント抽出・重要度計算
    async fn extract_and_score_key_points(&self, raw_summary: &serde_json::Value) -> Result<Vec<KeyPoint>, AISummaryError> {
        // 1. 要約テキストからキーポイント候補抽出
        let key_point_candidates = self.extract_key_point_candidates(raw_summary)?;
        
        // 2. 各キーポイントの重要度スコア計算
        let mut scored_key_points = Vec::new();
        for candidate in key_point_candidates {
            let importance_score = self.calculate_importance_score(&candidate).await?;
            
            let key_point = KeyPoint {
                text: candidate.text,
                importance_score,
                timestamp: candidate.timestamp,
                category: self.classify_key_point_category(&candidate)?,
                speakers: candidate.speakers,
            };
            
            scored_key_points.push(key_point);
        }
        
        // 3. 重要度順ソート・上位選択
        scored_key_points.sort_by(|a, b| b.importance_score.partial_cmp(&a.importance_score).unwrap());
        scored_key_points.truncate(10); // 上位10項目まで
        
        Ok(scored_key_points)
    }
    
    /// 重要度スコア計算アルゴリズム
    async fn calculate_importance_score(&self, candidate: &KeyPointCandidate) -> Result<f64, AISummaryError> {
        let mut score = 0.0;
        
        // 1. 長さベーススコア（適度な長さが重要）
        let length_score = Self::calculate_length_score(&candidate.text);
        score += length_score * 0.2;
        
        // 2. キーワード出現スコア
        let keyword_score = self.calculate_keyword_score(&candidate.text).await?;
        score += keyword_score * 0.3;
        
        // 3. 発言者重要度スコア
        let speaker_score = self.calculate_speaker_importance(&candidate.speakers).await?;
        score += speaker_score * 0.2;
        
        // 4. 会議内位置スコア（開始・終了付近は重要度高）
        let position_score = Self::calculate_position_score(candidate.timestamp);
        score += position_score * 0.1;
        
        // 5. 感情・トーンスコア
        let sentiment_score = self.analyze_sentiment_importance(&candidate.text).await?;
        score += sentiment_score * 0.2;
        
        // スコア正規化（0.0-1.0）
        Ok(score.min(1.0).max(0.0))
    }
}
```

## エラー処理設計

### エラー階層構造
```rust
/// 録画管理エラー定義
#[derive(Debug, thiserror::Error)]
pub enum RecordingError {
    /// メタデータ処理エラー
    #[error("Metadata processing error: {source}")]
    MetadataError {
        #[from]
        source: MetadataError,
        recording_id: Option<String>,
    },
    
    /// フィルタリングエラー
    #[error("Filtering error: {source}")]
    FilterError {
        #[from]
        source: FilterError,
        criteria: Option<FilterCriteria>,
    },
    
    /// ファイル分析エラー
    #[error("File analysis error: {source}")]
    FileAnalysisError {
        #[from]
        source: FileAnalysisError,
        file_info: Option<RecordingFileInfo>,
    },
    
    /// AI要約統合エラー
    #[error("AI summary integration error: {source}")]
    AISummaryError {
        #[from]
        source: AISummaryError,
        summary_id: Option<String>,
    },
    
    /// データ検証エラー
    #[error("Data validation error: field '{field}' - {message}")]
    ValidationError {
        field: String,
        message: String,
        expected_type: String,
        actual_value: Option<serde_json::Value>,
    },
    
    /// データ正規化エラー
    #[error("Data normalization error: {source}")]
    NormalizationError {
        #[from]
        source: NormalizationError,
        data_context: String,
    },
    
    /// 検索インデックスエラー
    #[error("Search index error: {source}")]
    SearchIndexError {
        #[from]
        source: SearchIndexError,
        operation: String,
    },
    
    /// 設定エラー
    #[error("Configuration error: {message}")]
    ConfigurationError {
        message: String,
        parameter: String,
    },
}

/// エラー回復戦略実装
pub struct RecordingErrorRecoveryStrategy {
    /// リトライ設定
    retry_config: RetryConfig,
    
    /// フォールバック処理
    fallback_processors: HashMap<RecordingErrorType, Box<dyn FallbackProcessor>>,
    
    /// エラー統計収集
    error_statistics: Arc<ErrorStatisticsCollector>,
}

impl RecordingErrorRecoveryStrategy {
    /// エラー種別に基づく自動回復
    pub async fn attempt_recovery(&self, error: &RecordingError, context: &ErrorContext) -> RecoveryResult {
        // エラー統計記録
        self.error_statistics.record_error(error, context).await;
        
        match error {
            RecordingError::MetadataError { source, recording_id } => {
                // メタデータエラー: 部分データで継続処理
                if let Some(id) = recording_id {
                    RecoveryResult::PartialRecovery {
                        recovered_data: self.create_minimal_metadata(id).await?,
                        lost_fields: source.get_affected_fields(),
                    }
                } else {
                    RecoveryResult::RequiresUserIntervention
                }
            },
            
            RecordingError::FilterError { source, criteria } => {
                // フィルタエラー: 条件簡素化による再試行
                if let Some(simplified_criteria) = self.simplify_filter_criteria(criteria) {
                    RecoveryResult::RetryWithModification {
                        modified_input: simplified_criteria,
                        modification_reason: "Simplified filter criteria for compatibility".to_string(),
                    }
                } else {
                    RecoveryResult::Unrecoverable
                }
            },
            
            RecordingError::AISummaryError { source, summary_id } => {
                // AI要約エラー: 要約なしで継続
                RecoveryResult::GracefulDegradation {
                    fallback_data: AISummaryData::create_empty_summary(),
                    degradation_level: DegradationLevel::FeatureDisabled,
                }
            },
            
            RecordingError::ValidationError { field, .. } => {
                // 検証エラー: デフォルト値適用
                if let Some(default_value) = self.get_default_value_for_field(field) {
                    RecoveryResult::AutoCorrection {
                        corrected_value: default_value,
                        correction_reason: format!("Applied default value for invalid field: {}", field),
                    }
                } else {
                    RecoveryResult::RequiresUserIntervention
                }
            },
            
            _ => RecoveryResult::Unrecoverable,
        }
    }
}
```

## テスト設計

### 単体テスト戦略
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;
    use tempfile::TempDir;
    
    // モックオブジェクト定義
    mock! {
        ApiClient {}
        
        #[async_trait]
        impl ApiClientTrait for ApiClient {
            async fn get_recording_data(&self, id: &str) -> Result<serde_json::Value, ApiError>;
            async fn get_ai_summary(&self, meeting_id: &str) -> Result<serde_json::Value, ApiError>;
        }
    }
    
    /// メタデータ処理正常系テスト
    #[tokio::test]
    async fn test_metadata_processing_success() {
        // Arrange
        let mut mock_api = MockApiClient::new();
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(temp_dir.path());
        
        mock_api
            .expect_get_recording_data()
            .returning(|_| Ok(create_mock_recording_data()));
        
        let recording_manager = ZoomRecordingManager::new(config).unwrap();
        
        // Act
        let raw_data = vec![create_mock_recording_json()];
        let result = recording_manager.process_recording_metadata(raw_data).await;
        
        // Assert
        assert!(result.is_ok());
        let recordings = result.unwrap();
        assert_eq!(recordings.len(), 1);
        assert_eq!(recordings[0].id, "test_recording_id");
        assert_eq!(recordings[0].processing_state, ProcessingState::Validated);
    }
    
    /// フィルタリング複合条件テスト
    #[tokio::test]
    async fn test_complex_filtering() {
        // Arrange
        let recording_manager = create_test_recording_manager().await;
        let test_recordings = create_test_recordings(100).await;
        
        // テストデータをマネージャーに登録
        for recording in test_recordings {
            recording_manager.register_recording(recording).await.unwrap();
        }
        
        let filter_criteria = FilterCriteria {
            date_range: Some(DateRangeFilter {
                from_date: chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                to_date: chrono::NaiveDate::from_ymd_opt(2025, 1, 31).unwrap(),
                time_range: None,
                timezone: None,
            }),
            file_types: Some(vec![
                RecordingFileType::Video { resolution: None, codec: None },
                RecordingFileType::Audio { quality: None, codec: None },
            ]),
            text_search: Some(TextSearchFilter {
                query: "project meeting".to_string(),
                fields: vec![SearchField::Topic, SearchField::Summary],
                mode: SearchMode::Partial,
                case_sensitive: false,
                regex: false,
            }),
            logic_operator: LogicOperator::And,
            ..Default::default()
        };
        
        // Act
        let result = recording_manager.search_recordings(filter_criteria).await;
        
        // Assert
        assert!(result.is_ok());
        let filtered_recordings = result.unwrap();
        
        // 全ての結果が条件を満たすことを確認
        for recording in &filtered_recordings {
            // 日付範囲確認
            assert!(recording.start_time.date_naive() >= chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
            assert!(recording.start_time.date_naive() <= chrono::NaiveDate::from_ymd_opt(2025, 1, 31).unwrap());
            
            // ファイル種別確認
            let has_video_or_audio = recording.files.iter().any(|file| {
                matches!(file.file_type, RecordingFileType::Video { .. } | RecordingFileType::Audio { .. })
            });
            assert!(has_video_or_audio);
            
            // テキスト検索確認
            assert!(recording.topic.to_lowercase().contains("project") || 
                   recording.topic.to_lowercase().contains("meeting"));
        }
    }
    
    /// AI要約統合テスト
    #[tokio::test]
    async fn test_ai_summary_integration() {
        // Arrange
        let recording_manager = create_test_recording_manager().await;
        let test_recording = create_test_recording().await;
        let ai_summary_data = create_test_ai_summary();
        
        // Act
        let result = recording_manager.integrate_ai_summary(
            &test_recording.id,
            ai_summary_data.clone()
        ).await;
        
        // Assert
        assert!(result.is_ok());
        
        let updated_recording = recording_manager.get_recording_details(&test_recording.id).await.unwrap();
        assert!(updated_recording.ai_summary.is_some());
        
        let integrated_summary = updated_recording.ai_summary.unwrap();
        assert_eq!(integrated_summary.summary_id, ai_summary_data.summary_id);
        assert_eq!(integrated_summary.key_points.len(), ai_summary_data.key_points.len());
        assert!(integrated_summary.quality_score > 0.0);
    }
}
```

### Property-basedテスト
```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        /// メタデータ正規化の冪等性テスト
        #[test]
        fn test_metadata_normalization_idempotency(
            recording in arb_recording_metadata()
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let normalizer = DataNormalizer::new();
                
                let mut recording1 = recording.clone();
                let mut recording2 = recording.clone();
                
                // 二回正規化実行
                normalizer.normalize_recording(&mut recording1).await.unwrap();
                normalizer.normalize_recording(&mut recording2).await.unwrap();
                normalizer.normalize_recording(&mut recording2).await.unwrap(); // 再度実行
                
                // Property: 正規化は冪等である
                prop_assert_eq!(recording1, recording2);
            });
        }
        
        /// フィルタリング結果の完全性テスト
        #[test]
        fn test_filtering_completeness(
            recordings in prop::collection::vec(arb_recording_metadata(), 1..100),
            criteria in arb_filter_criteria()
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let filter_engine = create_test_filter_engine().await;
                
                // フィルタ実行
                let filtered = filter_engine.apply_complex_filters(recordings.clone(), criteria.clone()).await.unwrap();
                
                // Property: フィルタ結果は元データのサブセット
                prop_assert!(filtered.len() <= recordings.len());
                
                // Property: フィルタ結果の全要素が条件を満たす
                for recording in &filtered {
                    prop_assert!(matches_filter_criteria(recording, &criteria));
                }
                
                // Property: 条件を満たす全要素がフィルタ結果に含まれる
                for recording in &recordings {
                    if matches_filter_criteria(recording, &criteria) {
                        prop_assert!(filtered.iter().any(|r| r.id == recording.id));
                    }
                }
            });
        }
        
        /// ファイル名正規化の安全性テスト
        #[test]
        fn test_filename_normalization_safety(
            filename in r"[^\x00]{1,300}"  // 任意のファイル名
        ) {
            let normalizer = DataNormalizer::new();
            
            let normalized = normalizer.normalize_filename(&filename).unwrap();
            
            // Property: 正規化後のファイル名は安全
            prop_assert!(!normalized.contains('<'));
            prop_assert!(!normalized.contains('>'));
            prop_assert!(!normalized.contains(':'));
            prop_assert!(!normalized.contains('"'));
            prop_assert!(!normalized.contains('|'));
            prop_assert!(!normalized.contains('?'));
            prop_assert!(!normalized.contains('*'));
            
            // Property: Windows パス長制限を満たす
            prop_assert!(normalized.len() <= 200);
            
            // Property: 空文字列にならない
            prop_assert!(!normalized.is_empty());
        }
        
        /// AI要約品質スコア一貫性テスト
        #[test]
        fn test_ai_summary_quality_consistency(
            summary_data in arb_ai_summary_data()
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let ai_integrator = AISummaryIntegrator::new(&create_test_config()).unwrap();
                
                let quality_score1 = ai_integrator.calculate_quality_score(&summary_data).await.unwrap();
                let quality_score2 = ai_integrator.calculate_quality_score(&summary_data).await.unwrap();
                
                // Property: 品質スコア計算は決定的
                prop_assert_eq!(quality_score1, quality_score2);
                
                // Property: 品質スコアは0.0-1.0の範囲
                prop_assert!(quality_score1 >= 0.0 && quality_score1 <= 1.0);
                
                // Property: キーポイントが多いほど品質スコアが高い傾向
                if summary_data.key_points.len() > 5 {
                    prop_assert!(quality_score1 > 0.3);
                }
            });
        }
    }
    
    /// 任意の録画メタデータ生成
    fn arb_recording_metadata() -> impl Strategy<Value = RecordingMetadata> {
        (
            "[a-zA-Z0-9]{10,20}",  // id
            "[a-zA-Z0-9]{10,20}",  // meeting_id
            "[\\w\\s]{5,100}",     // topic
            1u32..7200u32,         // duration
            prop::collection::vec(arb_recording_file_info(), 1..5),  // files
        ).prop_map(|(id, meeting_id, topic, duration, files)| {
            RecordingMetadata {
                id,
                meeting_id,
                topic,
                start_time: chrono::Utc::now() - chrono::Duration::hours(1),
                end_time: chrono::Utc::now(),
                duration_seconds: duration,
                files,
                host_info: create_default_host_info(),
                participants: vec![],
                settings: create_default_settings(),
                ai_summary: None,
                quality_info: MetadataQualityInfo::default(),
                processing_state: ProcessingState::Raw,
                last_updated: chrono::Utc::now(),
            }
        })
    }
    
    /// 任意のフィルタ条件生成
    fn arb_filter_criteria() -> impl Strategy<Value = FilterCriteria> {
        (
            prop::option::of(arb_date_range_filter()),
            prop::option::of(prop::collection::vec(arb_file_type(), 1..3)),
            prop::option::of(arb_text_search_filter()),
        ).prop_map(|(date_range, file_types, text_search)| {
            FilterCriteria {
                date_range,
                file_types,
                text_search,
                file_size: None,
                duration: None,
                hosts: None,
                ai_summary: None,
                custom_filters: HashMap::new(),
                logic_operator: LogicOperator::And,
            }
        })
    }
}
```

## 性能・セキュリティ考慮事項

### 性能最適化
1. **インデックス最適化**: B-Tree・ハッシュ・転置インデックスの最適な組み合わせ
2. **並列処理**: メタデータ解析・フィルタリングの並列実行
3. **キャッシュ戦略**: 頻繁にアクセスされるメタデータのメモリキャッシュ
4. **遅延読み込み**: 大量データの段階的読み込み・表示

### セキュリティ強化
1. **データ保護**: メタデータ内の機密情報の適切な取り扱い
2. **入力検証**: 全フィルタ条件・検索クエリの厳格な検証
3. **ログサニタイゼーション**: ログ出力時の個人情報除去
4. **アクセス制御**: 録画データへのアクセス権限管理

---

**承認**:  
**品質基準適合**: [ ] 確認済  
**ポリシー準拠**: [ ] 確認済  
**承認日**: ___________