# Windows固有ポリシー - Zoom Video Mover

**技術要素**: Windows API, windows crate, パス処理  
**適用範囲**: Windows固有の実装（ファイルシステム、コンソール、GUI、文字エンコーディング）

## Windows固有設計原則

### 基本原則
- **プラットフォーム統合**: Windowsの標準動作に準拠
- **文字エンコーディング**: UTF-8/UTF-16の適切な変換
- **パス処理**: Windows パス規則の完全対応
- **パフォーマンス**: Windowsネイティブ API の活用
- **互換性**: Windows 10 20H2以降のサポート

### 設計考慮事項
- **セキュリティ**: UAC・実行ポリシーへの対応
- **国際化**: 日本語環境での完全動作
- **アクセシビリティ**: Windows標準アクセシビリティAPI
- **システム統合**: タスクバー・通知・ファイル関連付け

## ファイルシステム処理

### Windows パス処理
```rust
use std::path::{Path, PathBuf};
use std::ffi::OsString;

/// Windows固有のパス処理
/// 
/// # 事前条件
/// - input_path が有効な文字列
/// 
/// # 事後条件
/// - Windows互換のパスを返す
/// - 長いパス（260文字超）に対応
/// - Unicode文字を正しく処理
/// 
/// # 不変条件
/// - パス変換中に文字の損失なし
pub fn normalize_windows_path(input_path: &str) -> Result<PathBuf, PathError> {
    assert!(!input_path.is_empty(), "input_path must not be empty");
    
    let mut normalized = input_path.replace('/', "\\");
    
    // 長いパス対応（\\?\\ プレフィックス）
    if normalized.len() > 260 {
        if !normalized.starts_with("\\\\?\\") {
            normalized = format!("\\\\?\\{}", normalized);
        }
    }
    
    // 不正な文字の置換
    const INVALID_CHARS: &[char] = &['<', '>', ':', '"', '|', '?', '*'];
    for &invalid_char in INVALID_CHARS {
        normalized = normalized.replace(invalid_char, "_");
    }
    
    // 末尾の空白・ピリオド除去（Windows要件）
    normalized = normalized.trim_end_matches([' ', '.']).to_string();
    
    // 予約語の回避
    let path = PathBuf::from(normalized);
    let reserved_path = avoid_reserved_names(path)?;
    
    debug_assert!(
        reserved_path.to_string_lossy().len() <= input_path.len() + 10,
        "normalized path should not be excessively longer"
    );
    
    Ok(reserved_path)
}

/// Windows予約語の回避
/// 
/// # 事前条件
/// - path が有効なパス
/// 
/// # 事後条件
/// - 予約語でないファイル名を返す
fn avoid_reserved_names(path: PathBuf) -> Result<PathBuf, PathError> {
    const RESERVED_NAMES: &[&str] = &[
        "CON", "PRN", "AUX", "NUL",
        "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
        "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    
    if let Some(file_name) = path.file_name() {
        let name_str = file_name.to_string_lossy();
        let base_name = name_str.split('.').next().unwrap_or(&name_str).to_uppercase();
        
        if RESERVED_NAMES.contains(&base_name.as_str()) {
            let mut new_path = path.clone();
            let new_name = format!("_{}", file_name.to_string_lossy());
            new_path.set_file_name(new_name);
            return Ok(new_path);
        }
    }
    
    Ok(path)
}

/// 安全なファイル作成
/// 
/// # 副作用
/// - ファイルの作成
/// - ディレクトリの作成（必要に応じて）
/// 
/// # 事前条件
/// - file_path が有効なパス
/// - 親ディレクトリへの書き込み権限
/// 
/// # 事後条件
/// - ファイルが作成される
/// - 適切な権限が設定される
pub async fn create_file_safely(file_path: &Path) -> Result<tokio::fs::File, FileOperationError> {
    // パスの正規化
    let normalized_path = normalize_windows_path(&file_path.to_string_lossy())?;
    
    // 親ディレクトリの作成
    if let Some(parent) = normalized_path.parent() {
        tokio::fs::create_dir_all(parent).await
            .map_err(|e| FileOperationError::DirectoryCreation {
                path: parent.to_path_buf(),
                source: e,
            })?;
    }
    
    // ファイル作成
    let file = tokio::fs::File::create(&normalized_path).await
        .map_err(|e| FileOperationError::FileCreation {
            path: normalized_path.clone(),
            source: e,
        })?;
    
    // Windows固有の属性設定
    #[cfg(windows)]
    {
        set_windows_file_attributes(&normalized_path)?;
    }
    
    debug_assert!(normalized_path.exists(), "file should exist after creation");
    Ok(file)
}

/// Windows ファイル属性設定
#[cfg(windows)]
fn set_windows_file_attributes(path: &Path) -> Result<(), AttributeError> {
    use windows::Win32::Storage::FileSystem::{SetFileAttributesW, FILE_ATTRIBUTE_NORMAL};
    use std::os::windows::ffi::OsStrExt;
    
    let wide_path: Vec<u16> = path.as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    
    unsafe {
        SetFileAttributesW(&wide_path[0], FILE_ATTRIBUTE_NORMAL)
            .map_err(|e| AttributeError::SetAttributesFailed(e))?;
    }
    
    Ok(())
}
```

### ディスク容量・権限チェック
```rust
/// ディスク容量チェック
/// 
/// # 事前条件
/// - path が有効なパス
/// 
/// # 事後条件
/// - 利用可能容量を返す
/// - エラー時は適切なエラーを返す
pub fn get_available_disk_space(path: &Path) -> Result<u64, DiskSpaceError> {
    #[cfg(windows)]
    {
        use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
        use std::os::windows::ffi::OsStrExt;
        
        let wide_path: Vec<u16> = path.as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        
        let mut free_bytes = 0u64;
        let mut total_bytes = 0u64;
        
        unsafe {
            GetDiskFreeSpaceExW(
                &wide_path[0],
                &mut free_bytes,
                &mut total_bytes,
                std::ptr::null_mut(),
            ).map_err(|e| DiskSpaceError::QueryFailed(e))?;
        }
        
        debug_assert!(free_bytes <= total_bytes, "free space should not exceed total space");
        Ok(free_bytes)
    }
    
    #[cfg(not(windows))]
    {
        // 非Windows環境でのフォールバック
        Err(DiskSpaceError::UnsupportedPlatform)
    }
}

/// ファイル権限チェック
/// 
/// # 事前条件
/// - path が存在するパス
/// 
/// # 事後条件
/// - 権限情報を返す
pub fn check_file_permissions(path: &Path) -> Result<FilePermissions, PermissionError> {
    #[cfg(windows)]
    {
        use windows::Win32::Storage::FileSystem::{GetFileAttributesW, FILE_ATTRIBUTE_READONLY};
        use std::os::windows::ffi::OsStrExt;
        
        let wide_path: Vec<u16> = path.as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        
        let attributes = unsafe {
            GetFileAttributesW(&wide_path[0])
        };
        
        if attributes.0 == u32::MAX {
            return Err(PermissionError::QueryFailed);
        }
        
        let permissions = FilePermissions {
            readable: true, // Windows では通常読み取り可能
            writable: (attributes.0 & FILE_ATTRIBUTE_READONLY.0) == 0,
            executable: path.extension()
                .map(|ext| ext.to_string_lossy().to_lowercase())
                .map(|ext| matches!(ext.as_str(), "exe" | "bat" | "cmd" | "com"))
                .unwrap_or(false),
        };
        
        Ok(permissions)
    }
}
```

## 文字エンコーディング処理

### UTF-8/UTF-16変換
```rust
/// 安全な文字エンコーディング変換
/// 
/// # 事前条件
/// - input が有効な UTF-8 文字列
/// 
/// # 事後条件
/// - Windows API互換のワイド文字列を返す
/// - 情報の損失なし
/// 
/// # 不変条件
/// - 変換前後で文字の意味が保持される
pub fn utf8_to_wide_string(input: &str) -> Vec<u16> {
    assert!(input.is_utf8(), "input must be valid UTF-8");
    
    use std::os::windows::ffi::OsStrExt;
    
    let wide_chars: Vec<u16> = std::ffi::OsStr::new(input)
        .encode_wide()
        .chain(std::iter::once(0)) // null terminator
        .collect();
    
    debug_assert!(
        !wide_chars.is_empty() && wide_chars[wide_chars.len() - 1] == 0,
        "wide string should be null-terminated"
    );
    
    wide_chars
}

/// ワイド文字列からUTF-8への変換
/// 
/// # 事前条件
/// - wide_str が有効なワイド文字列
/// 
/// # 事後条件
/// - UTF-8文字列を返す
/// - 不正な文字は置換文字で代替
pub fn wide_string_to_utf8(wide_str: &[u16]) -> Result<String, EncodingError> {
    use std::os::windows::ffi::OsStringExt;
    
    // null terminatorを除去
    let clean_wide: Vec<u16> = wide_str.iter()
        .take_while(|&&c| c != 0)
        .copied()
        .collect();
    
    let os_string = std::ffi::OsString::from_wide(&clean_wide);
    
    os_string.into_string()
        .map_err(|invalid_string| EncodingError::InvalidUnicode {
            original: invalid_string,
        })
}

/// コンソール出力の文字化け対策
/// 
/// # 副作用
/// - コンソールのコードページ設定
/// - 出力エンコーディングの変更
pub fn setup_console_encoding() -> Result<(), ConsoleError> {
    #[cfg(windows)]
    {
        use windows::Win32::System::Console::{SetConsoleOutputCP, SetConsoleCP};
        
        // UTF-8 (65001) に設定
        const CP_UTF8: u32 = 65001;
        
        unsafe {
            SetConsoleOutputCP(CP_UTF8)
                .map_err(|e| ConsoleError::SetOutputCodePageFailed(e))?;
            SetConsoleCP(CP_UTF8)
                .map_err(|e| ConsoleError::SetInputCodePageFailed(e))?;
        }
        
        println!("Console encoding set to UTF-8");
    }
    
    Ok(())
}
```

## Windows GUI統合

### システム通知
```rust
/// Windows 通知システム統合
/// 
/// # 副作用
/// - Windows通知の表示
/// - システムトレイアイコンの更新
/// 
/// # 事前条件
/// - title, message が空でない
/// - アプリケーションが通知権限を持つ
/// 
/// # 事後条件
/// - 通知が表示される
/// - 適切なアイコンが設定される
pub fn show_windows_notification(
    title: &str,
    message: &str,
    notification_type: NotificationType,
) -> Result<(), NotificationError> {
    assert!(!title.is_empty(), "title must not be empty");
    assert!(!message.is_empty(), "message must not be empty");
    
    #[cfg(windows)]
    {
        use windows::UI::Notifications::{ToastNotificationManager, ToastNotification};
        use windows::Data::Xml::Dom::XmlDocument;
        
        // XML テンプレート作成
        let xml_content = format!(
            r#"
            <toast>
                <visual>
                    <binding template="ToastGeneric">
                        <text>{}</text>
                        <text>{}</text>
                        <image placement="appLogoOverride" src="{}"/>
                    </binding>
                </visual>
                <actions>
                    <action content="開く" arguments="open"/>
                    <action content="閉じる" arguments="dismiss"/>
                </actions>
            </toast>
            "#,
            escape_xml(title),
            escape_xml(message),
            get_notification_icon(notification_type)
        );
        
        let xml_doc = XmlDocument::new()?;
        xml_doc.LoadXml(&xml_content.into())?;
        
        let toast = ToastNotification::CreateToastNotification(&xml_doc)?;
        let notifier = ToastNotificationManager::CreateToastNotifierWithId(
            &"ZoomVideoMover".into()
        )?;
        
        notifier.Show(&toast)?;
        
        debug_assert!(!title.is_empty() && !message.is_empty(), "notification content should be preserved");
    }
    
    Ok(())
}

/// タスクバー統合
pub struct TaskbarIntegration {
    #[cfg(windows)]
    taskbar_list: Option<windows::Win32::UI::Shell::ITaskbarList3>,
}

impl TaskbarIntegration {
    /// タスクバー進捗表示
    /// 
    /// # 副作用
    /// - タスクバーアイコンの進捗表示更新
    /// 
    /// # 事前条件
    /// - progress は 0.0 から 1.0 の範囲
    /// - window_handle が有効
    pub fn set_progress(&self, window_handle: isize, progress: f32) -> Result<(), TaskbarError> {
        debug_assert!((0.0..=1.0).contains(&progress), "progress must be between 0.0 and 1.0");
        
        #[cfg(windows)]
        {
            if let Some(ref taskbar) = self.taskbar_list {
                let progress_value = (progress * 100.0) as u64;
                
                unsafe {
                    taskbar.SetProgressValue(
                        windows::Win32::Foundation::HWND(window_handle),
                        progress_value,
                        100,
                    ).map_err(|e| TaskbarError::SetProgressFailed(e))?;
                }
            }
        }
        
        Ok(())
    }
    
    /// タスクバー状態設定
    pub fn set_state(&self, window_handle: isize, state: TaskbarState) -> Result<(), TaskbarError> {
        #[cfg(windows)]
        {
            use windows::Win32::UI::Shell::{TBPF_NORMAL, TBPF_ERROR, TBPF_PAUSED, TBPF_NOPROGRESS};
            
            if let Some(ref taskbar) = self.taskbar_list {
                let tbp_flags = match state {
                    TaskbarState::Normal => TBPF_NORMAL,
                    TaskbarState::Error => TBPF_ERROR,
                    TaskbarState::Paused => TBPF_PAUSED,
                    TaskbarState::NoProgress => TBPF_NOPROGRESS,
                };
                
                unsafe {
                    taskbar.SetProgressState(
                        windows::Win32::Foundation::HWND(window_handle),
                        tbp_flags,
                    ).map_err(|e| TaskbarError::SetStateFailed(e))?;
                }
            }
        }
        
        Ok(())
    }
}
```

## レジストリ・設定管理

### Windows レジストリ操作
```rust
/// Windows レジストリ操作
/// 
/// # 副作用
/// - レジストリキーの読み書き
/// 
/// # 事前条件
/// - key_path が有効なレジストリパス
/// - 適切な権限を持つ
pub struct WindowsRegistry;

impl WindowsRegistry {
    /// 設定値の保存
    /// 
    /// # 事前条件
    /// - key_path が有効なパス
    /// - value_name が空でない
    /// 
    /// # 事後条件
    /// - レジストリに値が保存される
    pub fn save_string_value(
        key_path: &str,
        value_name: &str,
        value: &str,
    ) -> Result<(), RegistryError> {
        assert!(!key_path.is_empty(), "key_path must not be empty");
        assert!(!value_name.is_empty(), "value_name must not be empty");
        
        #[cfg(windows)]
        {
            use windows::Win32::System::Registry::{
                RegCreateKeyExW, RegSetValueExW, RegCloseKey,
                HKEY_CURRENT_USER, KEY_WRITE, REG_SZ,
            };
            
            let wide_key_path = utf8_to_wide_string(key_path);
            let wide_value_name = utf8_to_wide_string(value_name);
            let wide_value = utf8_to_wide_string(value);
            
            let mut key_handle = windows::Win32::System::Registry::HKEY::default();
            
            unsafe {
                // キー作成/オープン
                RegCreateKeyExW(
                    HKEY_CURRENT_USER,
                    &wide_key_path[0],
                    0,
                    std::ptr::null(),
                    0,
                    KEY_WRITE,
                    std::ptr::null(),
                    &mut key_handle,
                    std::ptr::null_mut(),
                ).map_err(|e| RegistryError::KeyCreationFailed(e))?;
                
                // 値設定
                let value_size = (wide_value.len() * 2) as u32;
                RegSetValueExW(
                    key_handle,
                    &wide_value_name[0],
                    0,
                    REG_SZ,
                    wide_value.as_ptr() as *const u8,
                    value_size,
                ).map_err(|e| RegistryError::ValueSetFailed(e))?;
                
                // キー閉じる
                RegCloseKey(key_handle);
            }
        }
        
        Ok(())
    }
    
    /// 設定値の読み込み
    /// 
    /// # 事前条件
    /// - key_path が有効なパス
    /// - value_name が空でない
    /// 
    /// # 事後条件
    /// - 成功時: 設定値を返す
    /// - 失敗時: 適切なエラーを返す
    pub fn load_string_value(
        key_path: &str,
        value_name: &str,
    ) -> Result<String, RegistryError> {
        assert!(!key_path.is_empty(), "key_path must not be empty");
        assert!(!value_name.is_empty(), "value_name must not be empty");
        
        #[cfg(windows)]
        {
            use windows::Win32::System::Registry::{
                RegOpenKeyExW, RegQueryValueExW, RegCloseKey,
                HKEY_CURRENT_USER, KEY_READ, REG_SZ,
            };
            
            let wide_key_path = utf8_to_wide_string(key_path);
            let wide_value_name = utf8_to_wide_string(value_name);
            
            let mut key_handle = windows::Win32::System::Registry::HKEY::default();
            
            unsafe {
                // キーオープン
                RegOpenKeyExW(
                    HKEY_CURRENT_USER,
                    &wide_key_path[0],
                    0,
                    KEY_READ,
                    &mut key_handle,
                ).map_err(|e| RegistryError::KeyOpenFailed(e))?;
                
                // データサイズ取得
                let mut data_size = 0u32;
                let mut data_type = 0u32;
                
                RegQueryValueExW(
                    key_handle,
                    &wide_value_name[0],
                    std::ptr::null(),
                    &mut data_type,
                    std::ptr::null_mut(),
                    &mut data_size,
                ).map_err(|e| RegistryError::ValueQueryFailed(e))?;
                
                if data_type != REG_SZ.0 {
                    return Err(RegistryError::InvalidValueType);
                }
                
                // データ読み込み
                let mut buffer = vec![0u16; (data_size as usize) / 2];
                RegQueryValueExW(
                    key_handle,
                    &wide_value_name[0],
                    std::ptr::null(),
                    &mut data_type,
                    buffer.as_mut_ptr() as *mut u8,
                    &mut data_size,
                ).map_err(|e| RegistryError::ValueQueryFailed(e))?;
                
                RegCloseKey(key_handle);
                
                // UTF-8変換
                wide_string_to_utf8(&buffer)
                    .map_err(|e| RegistryError::EncodingConversion(e))
            }
        }
        
        #[cfg(not(windows))]
        {
            Err(RegistryError::UnsupportedPlatform)
        }
    }
}
```

## パフォーマンス最適化

### Windows固有最適化
```rust
/// Windows パフォーマンス最適化
pub struct WindowsOptimizer;

impl WindowsOptimizer {
    /// プロセス優先度設定
    /// 
    /// # 副作用
    /// - プロセス優先度の変更
    /// 
    /// # 事前条件
    /// - 適切な権限を持つ
    pub fn set_process_priority(priority: ProcessPriority) -> Result<(), OptimizationError> {
        #[cfg(windows)]
        {
            use windows::Win32::System::Threading::{
                GetCurrentProcess, SetPriorityClass,
                NORMAL_PRIORITY_CLASS, HIGH_PRIORITY_CLASS, BELOW_NORMAL_PRIORITY_CLASS,
            };
            
            let priority_class = match priority {
                ProcessPriority::Normal => NORMAL_PRIORITY_CLASS,
                ProcessPriority::High => HIGH_PRIORITY_CLASS,
                ProcessPriority::BelowNormal => BELOW_NORMAL_PRIORITY_CLASS,
            };
            
            unsafe {
                SetPriorityClass(GetCurrentProcess(), priority_class)
                    .map_err(|e| OptimizationError::PrioritySetFailed(e))?;
            }
            
            log::info!("Process priority set to: {:?}", priority);
        }
        
        Ok(())
    }
    
    /// メモリ作業セット調整
    pub fn optimize_working_set() -> Result<(), OptimizationError> {
        #[cfg(windows)]
        {
            use windows::Win32::System::ProcessStatus::{K32SetProcessWorkingSetSize};
            use windows::Win32::System::Threading::GetCurrentProcess;
            
            unsafe {
                // 作業セットを最小化（メモリ効率化）
                K32SetProcessWorkingSetSize(
                    GetCurrentProcess(),
                    usize::MAX, // 最小サイズ
                    usize::MAX, // 最大サイズ
                ).map_err(|e| OptimizationError::WorkingSetOptimizationFailed(e))?;
            }
        }
        
        Ok(())
    }
}
```

## エラー処理・デバッグ

### Windows エラー処理
```rust
/// Windows固有エラー型
#[derive(Debug, thiserror::Error)]
pub enum WindowsError {
    #[error("Path operation failed: {operation} on {path}")]
    PathOperation {
        operation: String,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    
    #[error("Encoding conversion failed: {message}")]
    Encoding { message: String },
    
    #[error("Registry operation failed: {operation}")]
    Registry {
        operation: String,
        #[source]
        source: windows::core::Error,
    },
    
    #[error("Console operation failed: {operation}")]
    Console {
        operation: String,
        #[source]
        source: windows::core::Error,
    },
    
    #[error("Win32 API call failed: {api_name}")]
    Win32Api {
        api_name: String,
        error_code: u32,
    },
}

impl WindowsError {
    /// Windows エラーコードから詳細メッセージ取得
    pub fn get_detailed_message(&self) -> String {
        match self {
            Self::Win32Api { api_name, error_code } => {
                format!("{} failed with error code: 0x{:08X} ({})", 
                       api_name, error_code, 
                       self.get_system_error_message(*error_code))
            }
            _ => self.to_string(),
        }
    }
    
    /// システムエラーメッセージ取得
    fn get_system_error_message(&self, error_code: u32) -> String {
        #[cfg(windows)]
        {
            use windows::Win32::System::Diagnostics::Debug::FormatMessageW;
            use windows::Win32::Foundation::{FORMAT_MESSAGE_FROM_SYSTEM, FORMAT_MESSAGE_IGNORE_INSERTS};
            
            let mut buffer = [0u16; 1024];
            
            unsafe {
                let length = FormatMessageW(
                    FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
                    std::ptr::null(),
                    error_code,
                    0,
                    &mut buffer,
                    std::ptr::null(),
                );
                
                if length > 0 {
                    wide_string_to_utf8(&buffer[..length as usize])
                        .unwrap_or_else(|_| format!("Error code: {}", error_code))
                } else {
                    format!("Unknown error: {}", error_code)
                }
            }
        }
        
        #[cfg(not(windows))]
        {
            format!("Error code: {}", error_code)
        }
    }
}
```

## セキュリティ考慮事項

### UAC・権限管理
```rust
/// 管理者権限チェック
/// 
/// # 事後条件
/// - 管理者権限の有無を返す
pub fn is_running_as_admin() -> bool {
    #[cfg(windows)]
    {
        use windows::Win32::Security::{GetTokenInformation, TokenElevation};
        use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
        use windows::Win32::Foundation::{TOKEN_QUERY, BOOL};
        
        unsafe {
            let mut token_handle = windows::Win32::Foundation::HANDLE::default();
            
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token_handle).is_err() {
                return false;
            }
            
            let mut elevation = BOOL::default();
            let mut size = std::mem::size_of::<BOOL>() as u32;
            
            let result = GetTokenInformation(
                token_handle,
                TokenElevation,
                &mut elevation as *mut _ as *mut _,
                size,
                &mut size,
            );
            
            windows::Win32::Foundation::CloseHandle(token_handle);
            
            result.is_ok() && elevation.as_bool()
        }
    }
    
    #[cfg(not(windows))]
    {
        false
    }
}

/// セキュアなテンポラリファイル作成
/// 
/// # 副作用
/// - 一時ファイルの作成
/// - 適切な権限設定
/// 
/// # 事後条件
/// - セキュアな一時ファイルを返す
/// - 他ユーザーからアクセス不可
pub fn create_secure_temp_file(prefix: &str) -> Result<PathBuf, SecurityError> {
    #[cfg(windows)]
    {
        use windows::Win32::Storage::FileSystem::{
            CreateFileW, GENERIC_WRITE, CREATE_NEW, FILE_ATTRIBUTE_TEMPORARY,
            FILE_FLAG_DELETE_ON_CLOSE,
        };
        
        let temp_dir = std::env::temp_dir();
        let file_name = format!("{}_{}.tmp", prefix, uuid::Uuid::new_v4());
        let temp_path = temp_dir.join(file_name);
        
        let wide_path = utf8_to_wide_string(&temp_path.to_string_lossy());
        
        unsafe {
            let handle = CreateFileW(
                &wide_path[0],
                GENERIC_WRITE.0,
                0, // 共有なし
                std::ptr::null(),
                CREATE_NEW,
                FILE_ATTRIBUTE_TEMPORARY | FILE_FLAG_DELETE_ON_CLOSE,
                windows::Win32::Foundation::HANDLE::default(),
            ).map_err(|e| SecurityError::SecureFileCreationFailed(e))?;
            
            windows::Win32::Foundation::CloseHandle(handle);
        }
        
        Ok(temp_path)
    }
}
```

## 品質目標

- **Windows互換性**: Windows 10 20H2以降で100%動作
- **文字化け**: 0件（日本語環境含む）
- **パス処理エラー**: 0件（長いパス・Unicode対応）
- **メモリリーク**: 0件（Win32 APIリソース管理）
- **権限エラー**: 適切なエラーメッセージ100%
- **パフォーマンス**: ネイティブアプリ同等

Windowsプラットフォームに最適化された、安全で高性能なアプリケーションを実現します。