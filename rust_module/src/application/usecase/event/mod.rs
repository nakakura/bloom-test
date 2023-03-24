pub(crate) mod data;
pub(crate) mod media;
pub(crate) mod peer;

use std::sync::Arc;

use async_trait::async_trait;
use shaku::{Component, Interface};

use crate::application::dto::response::{ResponseDto, ResponseDtoResult};
use crate::domain::entity::response::{Response, ResponseResult};
use crate::domain::repository::Repository;
use crate::error;
use crate::ffi::rust_to_c_bridge::state_objects::{CallbackFunctions, GlobalState, Logger};

#[cfg(test)]
use mockall::automock;

#[async_trait]
#[cfg_attr(test, automock)]
pub(crate) trait OnEvent: Interface {
    async fn execute(&self, event: Response) -> Result<ResponseResult, error::Error>;
}

#[async_trait]
#[cfg_attr(test, automock)]
pub(crate) trait EventReceive: Interface {
    async fn execute(&self) -> Result<ResponseDtoResult, error::Error>;
}

#[derive(Component)]
#[shaku(interface = EventReceive)]
pub(crate) struct EventReceiveImpl {
    #[shaku(inject)]
    logger: Arc<dyn Logger>,
    #[shaku(inject)]
    repository: Arc<dyn Repository>,
    #[shaku(inject)]
    state: Arc<dyn GlobalState>,
    #[shaku(inject)]
    callback: Arc<dyn CallbackFunctions>,
}

#[async_trait]
impl EventReceive for EventReceiveImpl {
    async fn execute(&self) -> Result<ResponseDtoResult, error::Error> {
        let event = self.repository.receive_event().await?;
        self.process_event(event).await
    }
}

impl EventReceiveImpl {
    async fn process_event(
        &self,
        response: ResponseResult,
    ) -> Result<ResponseDtoResult, error::Error> {
        match response {
            ResponseResult::Success(Response::Peer(response)) => Ok(ResponseDtoResult::Success(
                ResponseDto::Peer(self.process_peer_event(response).await?),
            )),
            ResponseResult::Success(Response::Data(response)) => Ok(ResponseDtoResult::Success(
                ResponseDto::Data(self.process_data_event(response).await?),
            )),
            ResponseResult::Success(Response::Media(response)) => Ok(ResponseDtoResult::Success(
                ResponseDto::Media(self.process_media_event(response).await?),
            )),
            ResponseResult::Error(e) => {
                let message = format!("EventReceiveImpl receives error message {}", e);
                Err(error::Error::create_local_error(&message))
            }
        }
    }
}
