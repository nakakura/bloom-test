use async_trait::async_trait;
use shaku::Interface;

use crate::domain::entity::request::Request;
use crate::domain::entity::response::ResponseResult;
use crate::error;

#[cfg(test)]
use mockall::automock;

/// skyway_webrtc_gateway_callerを利用するためのtrait定義
#[cfg_attr(test, automock)]
#[async_trait]
pub(crate) trait Repository: Interface {
    /// APIを能動的に呼ぶためのメソッド
    async fn register(&self, params: Request) -> Result<ResponseResult, error::Error>;
    /// イベントを監視するためのメソッド
    async fn receive_event(&self) -> Result<ResponseResult, error::Error>;
}
