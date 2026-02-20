/// 暗号化コンポーネント - AES-256-GCM + Windows DPAPI統合
/// 
/// # セキュリティ要件
/// - AES-256-GCM暗号化による機密データ保護
/// - Windows DPAPI統合によるキー管理
/// - メモリゼロ化による機密情報消去
/// - 暗号学的に安全な乱数生成

use crate::errors::{AppError, AppResult};
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce, Key
};
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};
use std::fmt;
use std::path::PathBuf;

/// 暗号化されたデータ構造
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    /// 暗号化されたデータ
    pub ciphertext: Vec<u8>,
    /// ナンス（初期化ベクトル）
    pub nonce: Vec<u8>,
    /// 暗号化アルゴリズム識別子
    pub algorithm: String,
    /// 作成日時（監査用）
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 機密データ保持構造体（自動ゼロ化対応）
#[derive(Clone, ZeroizeOnDrop)]
pub struct SecretData {
    data: Vec<u8>,
}

impl SecretData {
    /// 新しい機密データを作成
    /// 
    /// # 事前条件
    /// - data は空でない
    /// 
    /// # 事後条件
    /// - SecretDataインスタンスが作成される
    /// - 内部データが適切に保護される
    pub fn new(data: Vec<u8>) -> Self {
        assert!(!data.is_empty(), "Secret data must not be empty");
        Self { data }
    }
    
    /// 文字列から機密データを作成
    pub fn from_string(s: String) -> Self {
        Self::new(s.into_bytes())
    }
    
    /// データへの読み取り専用アクセス
    pub fn expose_secret(&self) -> &[u8] {
        &self.data
    }
    
    /// 文字列として取得（UTF-8前提）
    pub fn expose_secret_string(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.data)
    }
    
    /// データ長を取得
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    /// 空かどうか確認
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl fmt::Debug for SecretData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecretData[***REDACTED*** {} bytes]", self.data.len())
    }
}

/// 暗号化コンポーネント
#[derive(Clone)]
pub struct CryptoComponent {
    /// マスターキー（Windows DPAPI保護下）
    master_key: Option<Key<Aes256Gcm>>,
}

impl CryptoComponent {
    /// 新しい暗号化コンポーネントを作成
    pub fn new() -> Self {
        Self {
            master_key: None,
        }
    }
    
    /// マスターキーを初期化
    /// 
    /// # セキュリティ要件
    /// - Windows DPAPI使用時はユーザープロファイル保護
    /// - キーが存在しない場合は新規生成
    /// - キーは暗号学的に安全な乱数で生成
    /// 
    /// # 事前条件
    /// - Windows環境またはフォールバック環境
    /// 
    /// # 事後条件
    /// - マスターキーが初期化される
    /// - DPAPIでキーが保護される（Windows）
    pub async fn initialize_master_key(&mut self) -> AppResult<()> {
        log::info!("Initializing master encryption key");
        
        // Windows DPAPI使用時の実装
        #[cfg(target_os = "windows")]
        {
            self.initialize_master_key_windows().await
        }
        
        // 非Windows環境のフォールバック実装
        #[cfg(not(target_os = "windows"))]
        {
            self.initialize_master_key_generic().await
        }
    }
    
    /// Windows DPAPI使用のマスターキー初期化
    #[cfg(target_os = "windows")]
    async fn initialize_master_key_windows(&mut self) -> AppResult<()> {
        use std::fs;
        use std::path::PathBuf;
        
        // キー保存パス
        let key_path = Self::get_key_storage_path()?;
        
        // 既存キーの読み込み試行
        if key_path.exists() {
            match self.load_master_key_from_dpapi(&key_path).await {
                Ok(key) => {
                    self.master_key = Some(key);
                    log::info!("Master key loaded from DPAPI storage");
                    return Ok(());
                }
                Err(e) => {
                    log::warn!("Failed to load existing key from DPAPI: {:?}", e);
                    // 失敗時は新規キー生成にフォールスルー
                }
            }
        }
        
        // 新規キー生成
        let new_key = Aes256Gcm::generate_key(OsRng);
        
        // DPAPIで暗号化して保存
        self.save_master_key_with_dpapi(&key_path, &new_key).await?;
        
        self.master_key = Some(new_key);
        log::info!("New master key generated and saved with DPAPI protection");
        Ok(())
    }
    
    /// 汎用実装のマスターキー初期化
    #[cfg(not(target_os = "windows"))]
    async fn initialize_master_key_generic(&mut self) -> AppResult<()> {
        log::warn!("Running on non-Windows platform - using generic key storage");
        
        // WARNING: この実装はWindows DPAPI相当の保護を提供しない
        // 本格運用時はプラットフォーム固有のキーストア統合が必要
        
        let new_key = Aes256Gcm::generate_key(OsRng);
        self.master_key = Some(new_key);
        
        log::info!("Master key generated (generic mode - less secure)");
        Ok(())
    }
    
    /// Windows DPAPI使用のキー読み込み
    #[cfg(target_os = "windows")]
    async fn load_master_key_from_dpapi(&self, key_path: &std::path::Path) -> AppResult<Key<Aes256Gcm>> {
        use std::fs;
        
        // DPAPI暗号化されたキーデータを読み込み
        let encrypted_key_data = fs::read(key_path)
            .map_err(|e| AppError::file_system("Failed to read encrypted key file", Some(e)))?;
        
        // Windows DPAPI復号化
        let decrypted_key_data = Self::dpapi_decrypt(&encrypted_key_data)?;
        
        // キーサイズ検証（AES-256 = 32バイト）
        if decrypted_key_data.len() != 32 {
            return Err(AppError::authentication(
                "Invalid key size after DPAPI decryption",
                None::<std::io::Error>
            ));
        }
        
        let key = Key::<Aes256Gcm>::from_slice(&decrypted_key_data);
        Ok(*key)
    }
    
    /// Windows DPAPI使用のキー保存
    #[cfg(target_os = "windows")]
    async fn save_master_key_with_dpapi(&self, key_path: &std::path::Path, key: &Key<Aes256Gcm>) -> AppResult<()> {
        use std::fs;
        
        // DPAPIで暗号化
        let encrypted_key_data = Self::dpapi_encrypt(key.as_slice())?;
        
        // ディレクトリ作成
        if let Some(parent) = key_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| AppError::file_system("Failed to create key directory", Some(e)))?;
        }
        
        // 暗号化されたキーを保存
        fs::write(key_path, encrypted_key_data)
            .map_err(|e| AppError::file_system("Failed to save encrypted key", Some(e)))?;
        
        log::info!("Master key saved with DPAPI protection: {:?}", key_path);
        Ok(())
    }
    
    /// キー保存パスを取得
    fn get_key_storage_path() -> AppResult<PathBuf> {
        use std::path::PathBuf;
        
        // Windows: %APPDATA%\ZoomVideoMover\encryption.key
        #[cfg(target_os = "windows")]
        {
            let mut path = dirs::config_dir()
                .ok_or_else(|| AppError::file_system("Could not determine config directory", None::<std::io::Error>))?;
            path.push("ZoomVideoMover");
            path.push("encryption.key");
            Ok(path)
        }
        
        // Unix-like: ~/.config/zoom-video-mover/encryption.key
        #[cfg(not(target_os = "windows"))]
        {
            let mut path = dirs::config_dir()
                .ok_or_else(|| AppError::file_system("Could not determine config directory", None::<std::io::Error>))?;
            path.push("zoom-video-mover");
            path.push("encryption.key");
            Ok(path)
        }
    }
    
    /// Windows DPAPI暗号化
    #[cfg(target_os = "windows")]
    fn dpapi_encrypt(data: &[u8]) -> AppResult<Vec<u8>> {
        use winapi::um::dpapi::CryptProtectData;
        use winapi::um::wincrypt::DATA_BLOB;
        use std::ptr;

        const CRYPTPROTECT_UI_FORBIDDEN: u32 = 0x1;

        let mut data_in = DATA_BLOB {
            cbData: data.len() as u32,
            pbData: data.as_ptr() as *mut u8,
        };
        
        let mut data_out = DATA_BLOB {
            cbData: 0,
            pbData: ptr::null_mut(),
        };

        let success = unsafe {
            CryptProtectData(
                &mut data_in,
                ptr::null_mut(), // 説明文字列なし
                ptr::null_mut(), // 追加エントロピーなし
                ptr::null_mut(), // 予約済み
                ptr::null_mut(), // プロンプト構造体なし
                CRYPTPROTECT_UI_FORBIDDEN, // UIプロンプトを禁止
                &mut data_out,
            )
        };
        
        if success == 0 {
            return Err(AppError::authentication("DPAPI encryption failed", None::<std::io::Error>));
        }
        
        let encrypted_data = unsafe {
            std::slice::from_raw_parts(data_out.pbData, data_out.cbData as usize).to_vec()
        };
        
        // メモリ解放
        unsafe {
            winapi::um::winbase::LocalFree(data_out.pbData as *mut _);
        }
        
        Ok(encrypted_data)
    }
    
    /// Windows DPAPI復号化
    #[cfg(target_os = "windows")]
    fn dpapi_decrypt(encrypted_data: &[u8]) -> AppResult<Vec<u8>> {
        use winapi::um::dpapi::CryptUnprotectData;
        use winapi::um::wincrypt::DATA_BLOB;
        use std::ptr;

        let mut data_in = DATA_BLOB {
            cbData: encrypted_data.len() as u32,
            pbData: encrypted_data.as_ptr() as *mut u8,
        };

        let mut data_out = DATA_BLOB {
            cbData: 0,
            pbData: ptr::null_mut(),
        };
        
        let success = unsafe {
            CryptUnprotectData(
                &mut data_in,
                ptr::null_mut(), // 説明文字列出力なし
                ptr::null_mut(), // 追加エントロピーなし
                ptr::null_mut(), // 予約済み
                ptr::null_mut(), // プロンプト構造体なし
                0, // フラグなし
                &mut data_out,
            )
        };
        
        if success == 0 {
            return Err(AppError::authentication("DPAPI decryption failed", None::<std::io::Error>));
        }
        
        let decrypted_data = unsafe {
            std::slice::from_raw_parts(data_out.pbData, data_out.cbData as usize).to_vec()
        };
        
        // メモリ解放
        unsafe {
            winapi::um::winbase::LocalFree(data_out.pbData as *mut _);
        }
        
        Ok(decrypted_data)
    }
    
    /// データを暗号化
    /// 
    /// # セキュリティ要件
    /// - AES-256-GCM使用
    /// - 暗号学的に安全なナンス生成
    /// - 認証付き暗号化による改ざん検出
    /// 
    /// # 事前条件
    /// - マスターキーが初期化済み
    /// - data は空でない
    /// 
    /// # 事後条件
    /// - 暗号化されたデータが返される
    /// - ナンスが含まれる
    pub fn encrypt(&self, data: &SecretData) -> AppResult<EncryptedData> {
        let master_key = self.master_key.as_ref()
            .ok_or_else(|| AppError::authentication("Master key not initialized", None::<std::io::Error>))?;
        
        // AES-GCMクライアント作成
        let cipher = Aes256Gcm::new(master_key);
        
        // ナンス生成（96ビット）
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        // 暗号化実行
        let ciphertext = cipher.encrypt(&nonce, data.expose_secret())
            .map_err(|_| AppError::authentication("Encryption failed", None::<std::io::Error>))?;
        
        let encrypted_data = EncryptedData {
            ciphertext,
            nonce: nonce.to_vec(),
            algorithm: "AES-256-GCM".to_string(),
            created_at: chrono::Utc::now(),
        };
        
        log::debug!("Data encrypted successfully ({} bytes -> {} bytes)", 
                   data.len(), encrypted_data.ciphertext.len());
        
        Ok(encrypted_data)
    }
    
    /// データを復号化
    /// 
    /// # セキュリティ要件
    /// - 認証タグ検証による改ざん検出
    /// - 復号化後のデータを自動ゼロ化対象に
    /// 
    /// # 事前条件
    /// - マスターキーが初期化済み
    /// - encrypted_data は有効なEncryptedData
    /// 
    /// # 事後条件
    /// - 復号化されたデータが返される
    /// - 改ざんが検出された場合はエラー
    pub fn decrypt(&self, encrypted_data: &EncryptedData) -> AppResult<SecretData> {
        let master_key = self.master_key.as_ref()
            .ok_or_else(|| AppError::authentication("Master key not initialized", None::<std::io::Error>))?;
        
        // アルゴリズム確認
        if encrypted_data.algorithm != "AES-256-GCM" {
            return Err(AppError::authentication("Unsupported encryption algorithm", None::<std::io::Error>));
        }
        
        // ナンス長確認（AES-GCM: 96ビット = 12バイト）
        if encrypted_data.nonce.len() != 12 {
            return Err(AppError::authentication("Invalid nonce length", None::<std::io::Error>));
        }
        
        let cipher = Aes256Gcm::new(master_key);
        let nonce = Nonce::from_slice(&encrypted_data.nonce);
        
        // 復号化実行（認証タグ検証込み）
        let plaintext = cipher.decrypt(nonce, encrypted_data.ciphertext.as_ref())
            .map_err(|_| AppError::authentication("Decryption failed (data may be corrupted)", None::<std::io::Error>))?;
        
        log::debug!("Data decrypted successfully ({} bytes -> {} bytes)", 
                   encrypted_data.ciphertext.len(), plaintext.len());
        
        Ok(SecretData::new(plaintext))
    }
    
    /// 暗号化データをJSONとしてシリアライズ
    pub fn encrypt_to_json(&self, data: &SecretData) -> AppResult<String> {
        let encrypted_data = self.encrypt(data)?;
        let json = serde_json::to_string(&encrypted_data)
            .map_err(|e| AppError::serialization("Failed to serialize encrypted data", Some(e)))?;
        Ok(json)
    }
    
    /// JSONから暗号化データをデシリアライズして復号化
    pub fn decrypt_from_json(&self, json: &str) -> AppResult<SecretData> {
        let encrypted_data: EncryptedData = serde_json::from_str(json)
            .map_err(|e| AppError::serialization("Failed to deserialize encrypted data", Some(e)))?;
        self.decrypt(&encrypted_data)
    }
    
    /// コンポーネントの安全な初期化チェック
    pub fn is_initialized(&self) -> bool {
        self.master_key.is_some()
    }
    
    /// セキュアなメモリクリア
    pub fn clear_master_key(&mut self) {
        if let Some(mut key) = self.master_key.take() {
            key.zeroize();
        }
        log::info!("Master key cleared from memory");
    }
}

impl Default for CryptoComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for CryptoComponent {
    fn drop(&mut self) {
        self.clear_master_key();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_encryption_roundtrip() {
        let mut crypto = CryptoComponent::new();
        assert!(crypto.initialize_master_key().await.is_ok());
        
        let test_data = SecretData::from_string("sensitive test data".to_string());
        
        // 暗号化
        let encrypted = crypto.encrypt(&test_data).unwrap();
        assert_eq!(encrypted.algorithm, "AES-256-GCM");
        assert_eq!(encrypted.nonce.len(), 12);
        
        // 復号化
        let decrypted = crypto.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted.expose_secret_string().unwrap(), "sensitive test data");
    }
    
    #[tokio::test]
    async fn test_json_roundtrip() {
        let mut crypto = CryptoComponent::new();
        assert!(crypto.initialize_master_key().await.is_ok());
        
        let test_data = SecretData::from_string("JSON serialization test".to_string());
        
        // JSON暗号化
        let encrypted_json = crypto.encrypt_to_json(&test_data).unwrap();
        assert!(encrypted_json.contains("AES-256-GCM"));
        
        // JSON復号化
        let decrypted = crypto.decrypt_from_json(&encrypted_json).unwrap();
        assert_eq!(decrypted.expose_secret_string().unwrap(), "JSON serialization test");
    }
    
    #[tokio::test]
    async fn test_tampered_data_detection() {
        let mut crypto = CryptoComponent::new();
        assert!(crypto.initialize_master_key().await.is_ok());
        
        let test_data = SecretData::from_string("tamper detection test".to_string());
        let mut encrypted = crypto.encrypt(&test_data).unwrap();
        
        // 暗号文を改ざん
        if !encrypted.ciphertext.is_empty() {
            encrypted.ciphertext[0] ^= 0xFF;
        }
        
        // 復号化失敗を確認（認証タグ検証失敗）
        let result = crypto.decrypt(&encrypted);
        assert!(result.is_err());
    }
}