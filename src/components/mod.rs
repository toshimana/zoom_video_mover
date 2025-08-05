/// コンポーネント基盤モジュール
/// 
/// # 設計方針
/// - レイヤードアーキテクチャに基づくコンポーネント構造
/// - 単一責任原則に基づく責任分離
/// - 依存関係の明確化

pub mod auth;
pub mod config;
pub mod api;
pub mod recording;
pub mod download;
pub mod ui;

// 共通トレイトとタイプ定義
use crate::errors::{AppError, AppResult};
use async_trait::async_trait;

/// コンポーネントの基本ライフサイクル管理
#[async_trait]
pub trait ComponentLifecycle {
    /// コンポーネントの初期化
    /// 
    /// # 事前条件
    /// - 必要な依存関係が利用可能である
    /// 
    /// # 事後条件
    /// - コンポーネントが使用可能状態になる
    /// - 初期化が失敗した場合は適切なエラーが返される
    async fn initialize(&mut self) -> AppResult<()>;
    
    /// コンポーネントの終了処理
    /// 
    /// # 事前条件
    /// - コンポーネントが初期化済みである
    /// 
    /// # 事後条件
    /// - リソースが適切に解放される
    /// - 終了処理が完了する
    async fn shutdown(&mut self) -> AppResult<()>;
    
    /// コンポーネントの健全性チェック
    /// 
    /// # 事前条件
    /// - コンポーネントが初期化済みである
    /// 
    /// # 事後条件
    /// - コンポーネントの状態が正常な場合 true を返す
    /// - 問題がある場合は false を返す
    async fn health_check(&self) -> bool;
}

/// 設定可能なコンポーネント
pub trait Configurable<T> {
    /// 設定を更新する
    /// 
    /// # 事前条件
    /// - config は有効な設定オブジェクトである
    /// 
    /// # 事後条件
    /// - 設定が正常に適用される
    /// - 無効な設定の場合はエラーが返される
    fn update_config(&mut self, config: T) -> AppResult<()>;
    
    /// 現在の設定を取得する
    /// 
    /// # 事前条件
    /// - コンポーネントが初期化済みである
    /// 
    /// # 事後条件
    /// - 現在の設定オブジェクトが返される
    fn get_config(&self) -> &T;
}

/// イベント発行可能なコンポーネント
#[async_trait]
pub trait EventEmitter<T> {
    /// イベントを発行する
    /// 
    /// # 副作用
    /// - 登録されたリスナーにイベントが送信される
    /// 
    /// # 事前条件
    /// - event は有効なイベントオブジェクトである
    /// 
    /// # 事後条件
    /// - イベントが正常に発行される
    /// - 発行に失敗した場合はエラーが返される
    async fn emit_event(&self, event: T) -> AppResult<()>;
}

/// 共通のコンポーネント設定
#[derive(Debug, Clone)]
pub struct ComponentConfig {
    /// コンポーネント名
    pub name: String,
    /// 有効/無効フラグ
    pub enabled: bool,
    /// タイムアウト設定（秒）
    pub timeout_seconds: u64,
    /// リトライ回数
    pub max_retries: u32,
}