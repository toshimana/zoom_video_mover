/// API統合コンポーネント（スタブ実装）
/// 
/// # 責任
/// - Zoom API との通信
/// - レート制限管理
/// - リクエスト/レスポンス処理

use crate::errors::{AppError, AppResult};
use crate::components::ComponentLifecycle;
use async_trait::async_trait;

pub struct ApiComponent {
    // TODO: 実装
}

impl ApiComponent {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl ComponentLifecycle for ApiComponent {
    async fn initialize(&mut self) -> AppResult<()> {
        Ok(())
    }
    
    async fn shutdown(&mut self) -> AppResult<()> {
        Ok(())
    }
    
    async fn health_check(&self) -> bool {
        true
    }
}