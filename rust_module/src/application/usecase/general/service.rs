use std::sync::Arc;

use async_trait::async_trait;
use shaku::Component;

use crate::application::dto;
use crate::application::dto::request::RequestDto;
use crate::application::dto::response::ResponseDtoResult;
use crate::application::usecase::Service;
use crate::domain::repository::Repository;
use crate::error;

/// 一般的なWebRTC GatewayのAPIアクセス全てを処理するService
#[derive(Component)]
#[shaku(interface = Service)]
pub(crate) struct General {
    #[shaku(inject)]
    repository: Arc<dyn Repository>,
}

#[async_trait]
impl Service for General {
    async fn execute(&self, dto: RequestDto) -> Result<ResponseDtoResult, error::Error> {
        let request = dto::dto_to_request(dto)?;
        let result = self.repository.register(request).await?;
        dto::result_to_dto(result)
    }
}

#[cfg(test)]
mod create_peer_test {
    use shaku::HasComponent;

    use crate::application::dto::request::RequestDto;
    use crate::application::dto::response::ResponseDtoResult;
    use crate::application::usecase::*;
    use crate::di::GeneralService;
    use crate::domain::entity::response::ResponseResult;
    use crate::domain::repository::MockRepository;
    use crate::domain::repository::Repository;

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

        // サービスの生成
        let module = GeneralService::builder()
            .with_component_override::<dyn Repository>(Box::new(repository))
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
        let module = GeneralService::builder()
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
        let module = GeneralService::builder()
            .with_component_override::<dyn Repository>(Box::new(repository))
            .build();
        let service: &dyn Service = module.resolve_ref();

        // 実行
        let result = service.execute(dto).await;
        // 評価
        // 間違ったパラメータである旨を返してくるはずである
        if let Err(error::Error::LocalError(error_message)) = result {
            assert_eq!(error_message, "invalid parameter for GeneralService Test");
        }
    }
}

#[cfg(test)]
mod create_data_test {
    use shaku::HasComponent;

    use crate::application::dto::request::{DataRequestDto, RequestDto};
    use crate::application::dto::response::{DataResponseDto, ResponseDto, ResponseDtoResult};
    use crate::application::usecase::*;
    use crate::di::GeneralService;
    use crate::domain::entity::response::{DataResponse, Response, ResponseResult};
    use crate::domain::entity::{DataId, SerializableSocket, SocketInfo};
    use crate::domain::repository::{MockRepository, Repository};

    #[tokio::test]
    async fn success() {
        // CreateDataに成功したメッセージが得られるはずである
        let expected = {
            let socket = SocketInfo::<DataId>::try_create(
                Some("da-06cf1d26-0ef0-4b03-aca6-933027d434c2".to_string()),
                "127.0.0.1",
                10000,
            )
            .unwrap();
            ResponseDtoResult::Success(ResponseDto::Data(DataResponseDto::Create(socket.clone())))
        };

        // CreateDataのパラメータ生成
        let dto = RequestDto::Data(DataRequestDto::Create);

        // repositoryのMockを生成
        // 呼び出しに成功するケース
        let mut repository = MockRepository::new();
        repository.expect_register().times(1).returning(|_| {
            let socket = SocketInfo::<DataId>::try_create(
                Some("da-06cf1d26-0ef0-4b03-aca6-933027d434c2".to_string()),
                "127.0.0.1",
                10000,
            )
            .unwrap();
            Ok(ResponseResult::Success(Response::Data(
                DataResponse::Create(socket),
            )))
        });

        // サービスの生成
        let module = GeneralService::builder()
            .with_component_override::<dyn Repository>(Box::new(repository))
            .build();
        let service: &dyn Service = module.resolve_ref();

        // 実行
        let result = service.execute(dto).await;
        assert_eq!(result.unwrap(), expected);
    }

    #[tokio::test]
    async fn fail() {
        // APIがエラーを返してくるケース

        // CreateDataのパラメータ生成
        let dto = RequestDto::Data(DataRequestDto::Create);

        // repositoryのMockを生成
        // 呼び出しに成功するケース
        let mut repository = MockRepository::new();
        repository.expect_register().times(1).returning(|_| {
            let answer = error::Error::create_local_error("error");
            return Err(answer);
        });

        // サービスの生成
        let module = GeneralService::builder()
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
        repository.expect_register().times(0).returning(|_| {
            let answer = error::Error::create_local_error("error");
            return Err(answer);
        });

        // サービスの生成
        let module = GeneralService::builder()
            .with_component_override::<dyn Repository>(Box::new(repository))
            .build();
        let service: &dyn Service = module.resolve_ref();

        // 実行
        let result = service.execute(dto).await;
        // 間違ったパラメータである旨を返してくるはずである
        if let Err(error::Error::LocalError(error_message)) = result {
            assert_eq!(error_message, "invalid parameter for GeneralService Test");
        }
    }
}
