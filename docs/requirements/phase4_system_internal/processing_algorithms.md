# 処理フロー図・アルゴリズム仕様書 - Zoom Video Mover

## 文書概要
**プロジェクト名**: Zoom Video Mover  
**作成日**: 2025-08-02  
  
**バージョン**: 1.0  

## 主要処理フロー図

### オーバーオール処理フロー

```plantuml
@startuml
!theme plain
title Zoom Video Mover - 全体処理フロー

start

:アプリケーション起動;

if (OAuth設定存在?) then (Yes)
  :設定読み込み;
  if (トークン有効?) then (Yes)
    :メイン画面表示;
  else (No)
    :OAuth認証実行;
    :トークン取得・保存;
    :メイン画面表示;
  endif
else (No)
  :設定画面表示;
  :OAuth設定入力;
  :OAuth認証実行;
  :トークン取得・保存;
  :メイン画面表示;
endif

repeat
  :ユーザー操作待機;
  
  switch (操作種別)
  case (録画検索)
    :検索条件取得;
    :Zoom API呼び出し;
    :録画リスト表示;
  case (ファイル選択)
    :選択状態更新;
    :統計情報計算;
  case (ダウンロード開始)
    fork
      :ダウンロードエンジン起動;
    fork again
      :進捗監視開始;
    fork again
      :AI要約取得開始;
    end fork
    :ダウンロード完了待機;
  case (設定変更)
    :設定画面表示;
    :設定保存;
  endswitch
  
repeat while (継続?)

:アプリケーション終了;

stop

@enduml
```

### OAuth認証処理フロー

```plantuml
@startuml
!theme plain
title OAuth認証処理フロー詳細

start

:OAuth設定読み込み;

note right
  config.toml から
  client_id, client_secret,
  redirect_uri を取得
end note

:認証URL生成;

note right
  - scope: recording:read user:read meeting:read
  - state: CSRF対策用ランダム文字列
  - response_type: code
end note

:ローカルサーバー起動;
note right: ポート8080でコールバック待機

:デフォルトブラウザ起動;
:認証URL表示;

:ユーザー認証操作;

note right
  ブラウザでZoomログイン
  アプリケーション認可
end note

:認証コード受信;

if (state検証) then (Valid)
  :アクセストークン要求;
  
  note right
    POST /oauth/token
    grant_type=authorization_code
    code={認証コード}
  end note
  
  if (トークン取得成功?) then (Yes)
    :トークン暗号化;
    :ファイル保存;
    
    note right
      AES-256-GCMで暗号化
      %APPDATA%/ZoomVideoMover/
      config/oauth_tokens.enc
    end note
    
    :認証成功;
  else (No)
    :エラーログ記録;
    :認証失敗;
  endif
else (Invalid)
  :CSRF攻撃検出;
  :セキュリティエラー;
endif

:ローカルサーバー停止;

stop

@enduml
```

### 録画検索処理アルゴリズム

```plantuml
@startuml
!theme plain
title 録画検索処理アルゴリズム

start

:検索条件取得;
note right
  - from_date (必須)
  - to_date (必須)  
  - file_types[] (オプション)
  - meeting_name_pattern (オプション)
end note

:トークン有効性確認;
if (トークン有効?) then (No)
  :トークン更新処理;
  if (更新成功?) then (No)
    :認証画面表示;
    stop
  endif
endif

:API呼び出し制限チェック;
note right: 10req/sec制限遵守

:ページネーション初期化;
note right
  page_size = 30
  next_page_token = null
  all_meetings = []
end note

repeat
  :API呼び出し実行;
  
  note right
    GET /v2/users/me/recordings
    ?from={from_date}&to={to_date}
    &page_size=30
    &next_page_token={token}
  end note
  
  if (API成功?) then (Yes)
    :レスポンス解析;
    :会議データ抽出;
    :ローカルリストに追加;
    
    if (next_page_token存在?) then (Yes)
      :次ページトークン設定;
    else (No)
      :ページネーション完了;
    endif
  else (No)
    if (リトライ可能?) then (Yes)
      :指数バックオフ待機;
      :リトライ実行;
    else (No)
      :エラー表示;
      stop
    endif
  endif
  
repeat while (さらにページあり?)

:データフィルタリング;

note right
  - ファイル種別フィルタ適用
  - 会議名パターンマッチ
  - サイズ・日付フィルタ適用
end note

:階層構造構築;

note right
  Meeting
  ├── RecordingFile (MP4)
  ├── RecordingFile (M4A)
  └── RecordingFile (CHAT)
end note

:UI表示データ生成;
:検索結果表示;

stop

@enduml
```

## 並列ダウンロードアルゴリズム

### 並列制御アルゴリズム

```plantuml
@startuml
!theme plain
title 並列ダウンロードエンジン

start

:ダウンロード対象ファイル一覧取得;
:セッション生成;

note right
  session_id = UUID
  output_directory = 設定値
  max_concurrent = 5 (設定値)
end note

:出力フォルダ確認・作成;

note right
  - フォルダ存在確認
  - 書き込み権限確認
  - 会議別サブフォルダ作成
end note

:タスクキュー初期化;

note right
  concurrent_limit = Semaphore(5)
  task_queue = VecDeque<DownloadTask>
  progress_tracker = Arc<Mutex<Progress>>
end note

fork
  :進捗監視タスク開始;
  repeat
    :全タスク進捗収集;
    :統計計算;
    :UI更新;
    :500ms待機;
  repeat while (ダウンロード継続中?)
fork again
  :ダウンロード実行;
  
  partition "並列ダウンロード制御" {
    repeat
      if (キューに待機タスクあり?) then (Yes)
        :セマフォ取得待機;
        :タスク取得;
        
        fork
          :個別ファイルダウンロード;
        end fork
        
      else (No)
        :100ms待機;
      endif
    repeat while (すべてのタスク完了まで)
  }
end fork

:全タスク完了確認;
:最終レポート生成;
:セッション終了;

stop

@enduml
```

### 個別ファイルダウンロードアルゴリズム

```rust
/// 個別ファイルダウンロードのアルゴリズム実装
async fn download_single_file(
    file: &RecordingFile,
    output_path: &Path,
    progress_reporter: Arc<dyn ProgressReporter>
) -> Result<LocalFile, DownloadError> {
    // 事前条件: ファイル情報が有効であること
    assert!(!file.download_url.is_empty(), "Download URL must not be empty");
    assert!(file.file_size_bytes > 0, "File size must be positive");
    
    let client = create_http_client()?;
    let mut retry_count = 0;
    const MAX_RETRIES: u32 = 3;
    
    loop {
        match attempt_download(&client, file, output_path, &progress_reporter).await {
            Ok(local_file) => {
                // 事後条件: ファイルが正常にダウンロードされたことを確認
                debug_assert!(output_path.exists(), "Downloaded file must exist");
                debug_assert!(
                    std::fs::metadata(output_path)?.len() == file.file_size_bytes as u64,
                    "Downloaded file size must match expected size"
                );
                return Ok(local_file);
            },
            Err(e) if is_retryable_error(&e) && retry_count < MAX_RETRIES => {
                retry_count += 1;
                let delay = calculate_exponential_backoff(retry_count);
                
                warn!("Download failed, retrying in {:?}: {}", delay, e);
                progress_reporter.report_retry(file.file_id.clone(), retry_count).await;
                
                tokio::time::sleep(delay).await;
            },
            Err(e) => {
                error!("Download failed permanently: {}", e);
                return Err(e);
            }
        }
    }
}

/// 指数バックオフ計算
/// 
/// # 事前条件
/// - attempt は 0 より大きい値
/// 
/// # 事後条件  
/// - 返される Duration は 1秒以上 60秒以下
/// 
/// # 不変条件
/// - 計算中に attempt の値が変更されない
fn calculate_exponential_backoff(attempt: u32) -> Duration {
    debug_assert!(attempt > 0, "Attempt count must be positive");
    
    let base_delay_ms = 1000; // 1秒
    let max_delay_ms = 60000;  // 60秒
    
    let delay_ms = std::cmp::min(
        base_delay_ms * 2_u64.pow(attempt - 1),
        max_delay_ms
    );
    
    // ジッター追加（10%の変動）
    let jitter = rand::random::<f64>() * 0.1;
    let final_delay_ms = (delay_ms as f64 * (1.0 + jitter)) as u64;
    
    let result = Duration::from_millis(final_delay_ms);
    
    // 事後条件チェック
    debug_assert!(
        result >= Duration::from_secs(1) && result <= Duration::from_secs(60),
        "Backoff delay must be between 1 and 60 seconds"
    );
    
    result
}
```

### ストリーミングダウンロード実装

```plantuml
@startuml
!theme plain
title ストリーミングダウンロード処理

start

:HTTP GET リクエスト送信;

note right
  Headers:
  - Authorization: Bearer {token}
  - Accept: */*
  - User-Agent: ZoomVideoMover/1.0
end note

if (レスポンス受信?) then (200 OK)
  :Content-Length 取得;
  :出力ファイル作成;
  
  note right
    ファイル名サニタイズ:
    - 無効文字除去
    - 長さ制限 (255文字)
    - 重複時自動リネーム
  end note
  
  :ストリーミング読み書きループ;
  
  repeat
    :8KB チャンク読み取り;
    
    if (データ受信?) then (Yes)
      :ファイル書き込み;
      :進捗更新;
      :転送速度計算;
      
      note right
        progress_percent = 
        (downloaded_bytes / total_bytes) * 100
        
        transfer_rate = 
        downloaded_bytes / elapsed_time
      end note
      
    else (No)
      :読み取り完了;
    endif
    
  repeat while (データ残存?)
  
  :ファイル整合性検証;
  
  if (サイズ一致?) then (Yes)
    :ダウンロード成功;
  else (No)
    :ファイル削除;
    :サイズ不一致エラー;
  endif
  
else (Error)
  switch (ステータスコード)
  case (401 Unauthorized)
    :認証エラー;
    :トークン更新要求;
  case (404 Not Found)
    :ファイル削除済み;
    :エラー記録;
  case (429 Too Many Requests)
    :レート制限;
    :Retry-After 待機;
  case (5xx Server Error)
    :サーバーエラー;
    :リトライ対象;
  endswitch
endif

stop

@enduml
```

## データ処理アルゴリズム

### ファイル名サニタイズアルゴリズム

```rust
/// ファイル名をWindows/Linux両対応でサニタイズ
/// 
/// # 事前条件
/// - original_name は空でない文字列
/// 
/// # 事後条件
/// - 戻り値は有効なファイル名文字のみ含む
/// - 戻り値の長さは255文字以下
/// 
/// # 不変条件
/// - 元の意味を可能な限り保持する
fn sanitize_filename(original_name: &str) -> String {
    assert!(!original_name.is_empty(), "Original filename must not be empty");
    
    // Windows/Unix両方で無効な文字を除去
    let invalid_chars = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    let control_chars = (0..32).map(|i| char::from(i)).collect::<Vec<_>>();
    
    let mut sanitized = original_name
        .chars()
        .map(|c| {
            if invalid_chars.contains(&c) || control_chars.contains(&c) {
                '_'  // 無効文字を '_' に置換
            } else {
                c
            }
        })
        .collect::<String>();
    
    // 予約名の回避（Windows）
    let reserved_names = [
        "CON", "PRN", "AUX", "NUL",
        "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
        "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9"
    ];
    
    if reserved_names.contains(&sanitized.to_uppercase().as_str()) {
        sanitized = format!("_{}", sanitized);
    }
    
    // 長さ制限（拡張子込み255文字）
    if sanitized.len() > 255 {
        let extension = Path::new(&sanitized)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        
        let max_base_len = 255 - extension.len() - 1; // ドット分を引く
        let base_name = &sanitized[..max_base_len.min(sanitized.len())];
        sanitized = format!("{}.{}", base_name, extension);
    }
    
    // 末尾の空白・ピリオド除去（Windows制限）
    sanitized = sanitized.trim_end_matches([' ', '.']).to_string();
    
    // 空文字列になった場合のフォールバック
    if sanitized.is_empty() {
        sanitized = "untitled".to_string();
    }
    
    // 事後条件確認
    debug_assert!(!sanitized.is_empty(), "Sanitized filename must not be empty");
    debug_assert!(sanitized.len() <= 255, "Sanitized filename must not exceed 255 characters");
    debug_assert!(
        !sanitized.chars().any(|c| invalid_chars.contains(&c)),
        "Sanitized filename must not contain invalid characters"
    );
    
    sanitized
}
```

### 重複ファイル処理アルゴリズム

```plantuml
@startuml
!theme plain
title 重複ファイル処理アルゴリズム

start

:ファイル保存パス生成;
note right: base_path = output_dir + meeting_folder + sanitized_name

if (ファイル存在?) then (No)
  :そのまま保存;
  stop
else (Yes)
  :設定確認;
  if (自動リネーム有効?) then (Yes)
    :連番生成ループ;
    
    note right
      pattern: "filename (1).ext"
      pattern: "filename (2).ext"
      ...
      pattern: "filename (99).ext"
    end note
    
    repeat
      :連番付きファイル名生成;
      
      note right
        stem = filename without extension
        ext = file extension
        new_name = "{stem} ({counter}).{ext}"
      end note
      
      if (連番付きファイル存在?) then (No)
        :連番付きパスで保存;
        stop
      else (Yes)
        :カウンター increment;
        if (カウンター > 99?) then (Yes)
          :エラー: 重複解決不可;
          stop
        endif
      endif
    repeat while (重複解決まで)
    
  else (No)
    :ユーザー確認ダイアログ;
    
    switch (ユーザー選択)
    case (上書き)
      :既存ファイル削除;
      :新ファイル保存;
    case (スキップ)
      :ダウンロードスキップ;
      :次ファイルへ;
    case (リネーム)
      :手動ファイル名入力;
      :カスタム名で保存;
    case (キャンセル)
      :ダウンロード中止;
    endswitch
  endif
endif

stop

@enduml
```

## 進捗計算アルゴリズム

### リアルタイム進捗監視

```rust
/// 進捗監視とメトリクス計算
/// 
/// # 副作用
/// - UI要素の更新
/// - ログファイルへの書き込み
/// 
/// # 事前条件
/// - download_tasks が空でない
/// - progress_reporter が初期化済み
/// 
/// # 事後条件
/// - 全タスクの進捗が正確に反映される
/// - 推定時間が合理的な範囲内
/// 
/// # 不変条件
/// - 全体進捗は個別タスク進捗の加重平均と一致
async fn monitor_download_progress(
    download_tasks: Arc<Mutex<Vec<DownloadTask>>>,
    progress_reporter: Arc<dyn ProgressReporter>
) -> Result<(), ProgressError> {
    assert!(!download_tasks.lock().unwrap().is_empty(), "Download tasks must not be empty");
    
    let start_time = Instant::now();
    let mut last_update_time = start_time;
    let mut last_total_bytes = 0u64;
    
    loop {
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        let tasks = download_tasks.lock().unwrap();
        
        // 全体統計計算
        let total_files = tasks.len();
        let completed_files = tasks.iter().filter(|t| t.status == TaskStatus::Completed).count();
        let failed_files = tasks.iter().filter(|t| t.status == TaskStatus::Failed).count();
        let running_files = tasks.iter().filter(|t| t.status == TaskStatus::Downloading).count();
        
        // データ量統計
        let total_bytes: u64 = tasks.iter().map(|t| t.total_bytes).sum();
        let downloaded_bytes: u64 = tasks.iter().map(|t| t.downloaded_bytes).sum();
        
        // 進捗率計算
        let progress_percent = if total_bytes > 0 {
            (downloaded_bytes as f64 / total_bytes as f64 * 100.0) as u32
        } else {
            0
        };
        
        // 転送速度計算（直近の測定値を使用）
        let current_time = Instant::now();
        let time_delta = current_time.duration_since(last_update_time).as_secs_f64();
        let bytes_delta = downloaded_bytes.saturating_sub(last_total_bytes);
        
        let transfer_rate = if time_delta > 0.0 {
            bytes_delta as f64 / time_delta
        } else {
            0.0
        };
        
        // 残り時間推定
        let remaining_bytes = total_bytes.saturating_sub(downloaded_bytes);
        let estimated_time_remaining = if transfer_rate > 0.0 && remaining_bytes > 0 {
            Some(Duration::from_secs_f64(remaining_bytes as f64 / transfer_rate))
        } else {
            None
        };
        
        // 進捗情報構築
        let progress = DownloadProgress {
            total_files,
            completed_files,
            failed_files,
            running_files,
            total_bytes,
            downloaded_bytes,
            progress_percent,
            transfer_rate,
            estimated_time_remaining,
            elapsed_time: current_time.duration_since(start_time),
        };
        
        // 事後条件確認
        debug_assert!(
            progress_percent <= 100,
            "Progress percentage must not exceed 100%"
        );
        debug_assert!(
            downloaded_bytes <= total_bytes,
            "Downloaded bytes must not exceed total bytes"
        );
        debug_assert!(
            completed_files + failed_files + running_files <= total_files,
            "Sum of file statuses must not exceed total files"
        );
        
        // UI更新
        progress_reporter.update_progress(progress).await?;
        
        // 完了判定
        if completed_files + failed_files == total_files {
            break;
        }
        
        // 次回計算用の値を保存
        last_update_time = current_time;
        last_total_bytes = downloaded_bytes;
    }
    
    Ok(())
}
```

## AI要約処理アルゴリズム

### AI要約データ処理フロー

```plantuml
@startuml
!theme plain
title AI要約データ処理フロー

start

:会議リストからAI要約対応会議を抽出;

note right
  条件:
  - 会議時間が10分以上
  - 録画ファイルが存在
  - AI要約が生成済み
end note

repeat :会議を順次処理;

  :AI要約API呼び出し;
  
  note right
    GET /v2/meetings/{meetingId}/summary
    Authorization: Bearer {access_token}
  end note
  
  if (API成功?) then (Yes)
    :レスポンスJSON解析;
    
    :データ構造化処理;
    
    note right
      - 要約テキスト抽出
      - キーポイント配列処理
      - アクションアイテム構造化
      - 参加者情報整理
    end note
    
    :ファイル名生成;
    
    note right
      形式: "{meeting_date}_{sanitized_topic}_summary.json"
      例: "2025-08-01_週次会議_summary.json"
    end note
    
    :JSON形式で保存;
    
    note right
      保存先: output_dir/meeting_folder/
      エンコーディング: UTF-8
      フォーマット: 読みやすい改行・インデント付き
    end note
    
  else (No)
    if (404 Not Found?) then (Yes)
      :AI要約未生成をログ記録;
      note right: 会議にAI要約機能が使用されていない
    else (Other Error)
      :エラーログ記録;
      :リトライ判定;
      
      if (リトライ可能?) then (Yes)
        :指数バックオフ待機;
      else (No)
        :該当会議スキップ;
      endif
    endif
  endif

repeat while (処理対象会議が残存?)

:AI要約処理完了;

stop

@enduml
```

### JSON構造化アルゴリズム

```rust
/// AI要約レスポンスの構造化処理
/// 
/// # 事前条件
/// - raw_response は有効なJSON文字列
/// - meeting_info は会議基本情報を含む
/// 
/// # 事後条件
/// - 構造化されたAISummaryオブジェクトが返される
/// - 必須フィールドがすべて存在する
/// 
/// # 不変条件
/// - 元のデータの意味的内容が保持される
fn structure_ai_summary(
    raw_response: &str,
    meeting_info: &Meeting
) -> Result<AISummary, ParseError> {
    assert!(!raw_response.is_empty(), "Raw response must not be empty");
    
    let response_json: serde_json::Value = serde_json::from_str(raw_response)?;
    
    // 必須フィールドの存在確認
    let summary_obj = response_json.get("summary")
        .ok_or(ParseError::MissingField("summary"))?;
    
    // 要約テキスト抽出
    let overview = summary_obj.get("overview")
        .and_then(|v| v.as_str())
        .unwrap_or("要約情報なし")
        .to_string();
    
    // キーポイント配列処理
    let key_points = summary_obj.get("key_points")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item.as_str())
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
        })
        .unwrap_or_default();
    
    // アクションアイテム構造化
    let action_items = summary_obj.get("action_items")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| parse_action_item(item))
                .collect::<Vec<ActionItem>>()
        })
        .unwrap_or_default();
    
    // 参加者情報処理
    let participants = summary_obj.get("participants_summary")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| parse_participant(item, &meeting_info.meeting_id))
                .collect::<Vec<Participant>>()
        })
        .unwrap_or_default();
    
    // 構造化オブジェクト生成
    let ai_summary = AISummary {
        summary_id: uuid::Uuid::new_v4().to_string(),
        meeting_id: meeting_info.meeting_id.clone(),
        summary_text: overview.clone(),
        overview,
        key_points,
        created_at: chrono::Utc::now(),
        source_api: "zoom_ai_companion".to_string(),
        confidence_score: extract_confidence_score(&response_json),
        language: detect_language(&overview),
    };
    
    // 事後条件確認
    debug_assert!(!ai_summary.summary_id.is_empty(), "Summary ID must not be empty");
    debug_assert!(!ai_summary.meeting_id.is_empty(), "Meeting ID must not be empty");
    debug_assert!(
        ai_summary.confidence_score.map_or(true, |score| score >= 0.0 && score <= 1.0),
        "Confidence score must be between 0.0 and 1.0"
    );
    
    Ok(ai_summary)
}

/// アクションアイテムのパース処理
fn parse_action_item(item_json: &serde_json::Value) -> Option<ActionItem> {
    let description = item_json.get("description")?.as_str()?.to_string();
    let assignee = item_json.get("assignee").and_then(|v| v.as_str()).map(|s| s.to_string());
    let due_date = item_json.get("due_date")
        .and_then(|v| v.as_str())
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
    
    Some(ActionItem {
        action_item_id: uuid::Uuid::new_v4().to_string(),
        summary_id: String::new(), // 後で設定される
        description,
        assignee,
        due_date,
        priority: Priority::Medium, // デフォルト値
        status: ActionItemStatus::Pending,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    })
}
```

## エラー処理・回復アルゴリズム

### 包括的エラー分類処理

```plantuml
@startuml
!theme plain
title エラー分類・回復処理フロー

start

:エラー発生検出;

switch (エラーソース)
case (HTTP通信)
  :HTTPステータス分析;
  
  switch (ステータスコード)
  case (401 Unauthorized)
    :認証エラー処理;
    :トークンリフレッシュ試行;
    if (リフレッシュ成功?) then (Yes)
      :処理再実行;
    else (No)
      :再認証要求;
    endif
    
  case (429 Rate Limit)
    :レート制限エラー;
    :Retry-Afterヘッダー確認;
    :指定時間待機;
    :自動リトライ;
    
  case (5xx Server Error)
    :サーバーエラー;
    :指数バックオフリトライ;
    
  case (4xx Client Error)
    :クライアントエラー;
    :ユーザー対応要求;
    
  endswitch

case (ファイルシステム)
  :ファイルシステムエラー分析;
  
  switch (エラー種別)
  case (容量不足)
    :ディスク容量不足;
    :容量確認・警告表示;
    :出力先変更促進;
    
  case (権限不足)
    :アクセス権限エラー;
    :権限確認・指示表示;
    
  case (パス不正)
    :パス解決エラー;
    :パス修正促進;
    
  endswitch

case (ネットワーク)
  :ネットワークエラー分析;
  
  switch (エラー種別)
  case (接続タイムアウト)
    :接続タイムアウト;
    :ネットワーク状態確認;
    :リトライ実行;
    
  case (DNS解決失敗)
    :DNS解決エラー;
    :DNS設定確認促進;
    
  case (プロキシエラー)
    :プロキシ設定エラー;
    :プロキシ設定確認促進;
    
  endswitch

case (データ破損)
  :データ整合性エラー;
  :ファイル削除;
  :再ダウンロード実行;

endswitch

:エラーログ記録;

note right
  ログ形式:
  {
    "timestamp": "2025-08-02T10:30:15Z",
    "level": "ERROR",
    "module": "download_engine",
    "error_type": "network_timeout",
    "error_code": "TIMEOUT_001",
    "message": "Download timeout after 30 seconds",
    "context": {
      "file_id": "...",
      "url": "...",
      "retry_count": 2
    },
    "user_action_required": true,
    "suggested_actions": [
      "Check internet connection",
      "Verify proxy settings"
    ]
  }
end note

:ユーザー通知;

note right
  通知内容:
  - エラー概要（技術的詳細は隠蔽）
  - 推定原因
  - 具体的対処方法
  - 回復可能性の表示
end note

stop

@enduml
```

---

**承認**:  
**品質基準適合**: [ ] 確認済  
**ポリシー準拠**: [ ] 確認済  
**承認日**: ___________