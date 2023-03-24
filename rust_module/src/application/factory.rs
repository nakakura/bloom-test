use std::sync::Arc;

use async_trait::async_trait;
use shaku::{Component, HasComponent, Interface};

use crate::application::dto::request::{
    DataRequestDto, MediaRequestDto, PeerRequestDto, RequestDto,
};
use crate::application::usecase::Service;
use crate::di::*;

#[cfg(test)]
use mockall::automock;

#[async_trait]
#[cfg_attr(test, automock)]
pub(crate) trait Factory: Interface {
    fn create_service(&self, params: &RequestDto) -> Arc<dyn Service>;
}

#[derive(Component)]
#[shaku(interface = Factory)]
pub(crate) struct FactoryImpl {}

/// 各UseCaseのインスタンスを生成し、サービスとして返す
impl Factory for FactoryImpl {
    fn create_service(&self, request: &RequestDto) -> Arc<dyn Service> {
        match request {
            RequestDto::Peer(PeerRequestDto::Create { params: _ }) => {
                let module = PeerCreateService::builder().build();
                module.resolve()
            }
            RequestDto::Data(DataRequestDto::Connect { params: _ }) => {
                let module = DataConnectService::builder().build();
                module.resolve()
            }
            RequestDto::Data(DataRequestDto::Redirect { params: _ }) => {
                let module = DataRedirectService::builder().build();
                module.resolve()
            }
            RequestDto::Data(DataRequestDto::Create) => {
                let module = GeneralService::builder().build();
                module.resolve()
            }
            RequestDto::Data(DataRequestDto::Status { params: _ }) => {
                let module = GeneralService::builder().build();
                module.resolve()
            }
            RequestDto::Media(MediaRequestDto::Call { params: _ }) => {
                let module = MediaCallService::builder().build();
                module.resolve()
            }
            RequestDto::Media(MediaRequestDto::Answer { params: _ }) => {
                let module = MediaAnswerService::builder().build();
                module.resolve()
            }
            RequestDto::System(_) => {
                let module = SystemService::builder().build();
                module.resolve()
            }
            _ => {
                let module = GeneralService::builder().build();
                module.resolve()
            }
        }
    }
}
