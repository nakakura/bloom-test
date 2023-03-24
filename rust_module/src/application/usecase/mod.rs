pub(crate) mod data;
pub(crate) mod event;
pub(crate) mod general;
pub(crate) mod media;
pub(crate) mod peer;
pub(crate) mod system;

use async_trait::async_trait;
use shaku::Interface;

use crate::application::dto::request::RequestDto;
use crate::application::dto::response::ResponseDtoResult;
use crate::error;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait]
pub(crate) trait Service: Interface {
    async fn execute(&self, request: RequestDto) -> Result<ResponseDtoResult, error::Error>;
}
