/// UI制御コンポーネント（スタブ実装）
/// 
/// # 責任
/// - eGUI 状態管理
/// - イベント処理
/// - 画面遷移制御

use crate::errors::AppResult;
use crate::components::ComponentLifecycle;
use async_trait::async_trait;

pub struct UiComponent {
    // TODO: 実装
}

impl UiComponent {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl ComponentLifecycle for UiComponent {
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