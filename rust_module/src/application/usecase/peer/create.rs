/// Create Peer APIは、あくまでPeer Objectの生成要求に過ぎず、
/// Openイベントが発火するまではPeer Objectの生成に失敗する可能性がある
/// ユーザにとってはPeer Objectは生成に完了して然るべきもので、Eventの監視をする積極的理由がないので、
/// このUseCase内でEventの監視まで自動的に行い、Open完了時に結果を返す
use std::sync::Arc;

use async_trait::async_trait;
use shaku::Component;

use crate::application::dto::request::RequestDto;
use crate::application::dto::response::{PeerResponseDto, ResponseDto, ResponseDtoResult};
use crate::application::usecase::Service;
use crate::domain::entity::request::Request;
use crate::domain::entity::response::{PeerResponse, Response, ResponseResult};
use crate::domain::repository::Repository;
use crate::error;
use crate::ffi::rust_to_c_bridge::state_objects::{CallbackFunctions, GlobalState};

#[allow(unused)]
#[derive(Component)]
#[shaku(interface = Service)]
pub(crate) struct Create {
    #[shaku(inject)]
    repository: Arc<dyn Repository>,
    #[shaku(inject)]
    state: Arc<dyn GlobalState>,
    #[shaku(inject)]
    callback: Arc<dyn CallbackFunctions>,
}

#[async_trait]
impl Service for Create {
    async fn execute(&self, request: RequestDto) -> Result<ResponseDtoResult, error::Error> {
        if let RequestDto::Peer(ref inner) = request {
            let request = Request::Peer(inner.clone());
            let result = self.repository.register(request).await?;

            // 成功した場合はC++側にpeer_id, tokenを渡す
            match result {
                ResponseResult::Success(Response::Peer(PeerResponse::Create(ref peer_info))) => {
                    let peer_id = peer_info.peer_id();
                    let token = peer_info.token();
                    // shutdown処理のためにpeer_id, tokenをC++側に通知
                    self.callback
                        .create_peer_callback(peer_id.as_str(), token.as_str());

                    return Ok(ResponseDtoResult::Success(ResponseDto::Peer(
                        PeerResponseDto::Create(peer_info.clone()),
                    )));
                }
                // API Callには成功したが、内部処理に失敗したケース
                ResponseResult::Error(message) => {
                    return Ok(ResponseDtoResult::Error(message));
                }
                _ => {
                    unreachable!()
                }
            }
        }

        let error_message = format!("wrong parameter {:?}", request);
        return Err(error::Error::create_local_error(&error_message));
    }
}

#[cfg(test)]
mod create_peer_test {
    use shaku::HasComponent;

    use super::*;
    use crate::application::dto::request::RequestDto;
    use crate::application::dto::response::ResponseDtoResult;
    use crate::di::PeerCreateService;
    use crate::domain::entity::response::ResponseResult;
    use crate::domain::repository::MockRepository;
    use crate::ffi::rust_to_c_bridge::state_objects::MockCallbackFunctions;

    #[tokio::test]
    async fn success() {
        // CreatePeerに成功したメッセージが得られるはずである
        let expected = {
            let message = r#"{
                    "is_success":true,
                    "result":{
                        "request_type":"PEER",
                        "command":"CREATE",
                        "peer_id":"peer_id",
                        "token":"pt-06cf1d26-0ef0-4b03-aca6-933027d434c2"
                    }
                }"#;
            ResponseDtoResult::from_str(message).unwrap()
        };

        // CreatePeerのパラメータ生成
        let dto = {
            let message = r#"{
                "request_type": "PEER",
                "command": "CREATE",
                "params": {
                    "key": "API_KEY",
                    "domain": "localhost",
                    "peer_id": "peer_id",
                    "turn": true
                }
            }"#;
            RequestDto::from_str(message).unwrap()
        };

        // repositoryのMockを生成
        // 呼び出しに成功するケース
        let mut repository = MockRepository::new();
        repository.expect_register().times(1).returning(|_| {
            let message = r#"{
                    "is_success":true,
                    "result":{
                        "request_type":"PEER",
                        "command":"CREATE",
                        "peer_id":"peer_id",
                        "token":"pt-06cf1d26-0ef0-4b03-aca6-933027d434c2"
                    }
                }"#;
            ResponseResult::from_str(message)
        });

        let mut caller = MockCallbackFunctions::new();
        caller
            .expect_create_peer_callback()
            .times(1)
            .returning(|_, _| ());

        // サービスの生成
        let module = PeerCreateService::builder()
            .with_component_override::<dyn Repository>(Box::new(repository))
            .with_component_override::<dyn CallbackFunctions>(Box::new(caller))
            .build();
        let service: &dyn Service = module.resolve_ref();

        // 実行
        let result = service.execute(dto).await;
        assert_eq!(result.unwrap(), expected);
    }

    #[tokio::test]
    async fn fail() {
        // APIがエラーを返してくるケース

        // CreatePeerのパラメータ生成
        let dto = {
            let message = r#"{
                "request_type": "PEER",
                "command": "CREATE",
                "params": {
                    "key": "API_KEY",
                    "domain": "localhost",
                    "peer_id": "peer_id",
                    "turn": true
                }
            }"#;
            RequestDto::from_str(message).unwrap()
        };

        // repositoryのMockを生成
        // errorを返してくるケース
        let mut repository = MockRepository::new();
        repository.expect_register().times(1).returning(|_| {
            let answer = error::Error::create_local_error("error");
            return Err(answer);
        });

        // サービスの生成
        let module = PeerCreateService::builder()
            .with_component_override::<dyn Repository>(Box::new(repository))
            .build();
        let service: &dyn Service = module.resolve_ref();

        // 実行
        let result = service.execute(dto).await;
        if let Err(error::Error::LocalError(message)) = result {
            assert_eq!(message, "error");
        }
    }

    #[tokio::test]
    async fn invalid_parameter() {
        // 間違ったパラメータの生成
        let dto = RequestDto::Test;

        // repositoryのMockを生成
        // 呼ばれないはずである
        let mut repository = MockRepository::new();
        repository
            .expect_register()
            .times(0)
            .returning(|_| unreachable!());

        // サービスの生成
        let module = PeerCreateService::builder()
            .with_component_override::<dyn Repository>(Box::new(repository))
            .build();
        let service: &dyn Service = module.resolve_ref();

        // 実行
        let result = service.execute(dto).await;
        // 評価
        // 間違ったパラメータである旨を返してくるはずである
        if let Err(error::Error::LocalError(error_message)) = result {
            assert_eq!(error_message, "wrong parameter Test");
        }
    }
}
