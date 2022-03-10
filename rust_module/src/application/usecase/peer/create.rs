use async_trait::async_trait;

use crate::application::dto::{
    PeerDtoResponseMessageBodyEnum, RequestDto, ResponseDto, ResponseDtoMessageBodyEnum,
};
use crate::application::usecase::Service;
use crate::application::Functions;
use crate::domain::entity::{
    PeerResponseMessageBodyEnum, Request, Response, ResponseMessageBodyEnum,
};
use crate::Repository;
use crate::{error, Logger, ProgramState};

pub(crate) struct Create {}

#[async_trait]
impl Service for Create {
    async fn execute(
        &self,
        repository: &Box<dyn Repository>,
        program_state: &ProgramState,
        logger: &Logger,
        _cb_functions: &Functions,
        message: RequestDto,
    ) -> Result<ResponseDto, error::Error> {
        if let RequestDto::Peer(ref inner) = message {
            let request = Request::Peer(inner.clone());
            let message = repository.register(program_state, logger, request).await;

            // 成功した場合はC++側にpeer_id, tokenを渡す
            if let Ok(Response::Success(ResponseMessageBodyEnum::Peer(
                PeerResponseMessageBodyEnum::Create(ref peer_info),
            ))) = message
            {
                let peer_id = peer_info.peer_id();
                let token = peer_info.token();
                crate::application::FUNCTIONS_INSTANCE
                    .get()
                    .map(|functions| {
                        functions.create_peer_callback(peer_id.as_str(), token.as_str())
                    });

                return Ok(ResponseDto::Success(ResponseDtoMessageBodyEnum::Peer(
                    PeerDtoResponseMessageBodyEnum::Create(peer_info.clone()),
                )));
            }
        }

        let error_message = format!("wrong parameter {:?}", message);
        return Err(error::Error::create_local_error(&error_message));
    }
}

#[cfg(test)]
mod create_peer_test {
    use super::*;
    use crate::application::usecase::helper;
    use crate::domain::repository::MockRepository;

    #[tokio::test]
    async fn success() {
        // CreatePeerに成功したメッセージが得られるはずである
        let answer = {
            let message = r#"{
                    "is_success":true,
                    "result":{
                        "type":"PEER",
                        "command":"CREATE",
                        "peer_id":"data_caller",
                        "token":"pt-06cf1d26-0ef0-4b03-aca6-933027d434c2"
                    }
                }"#;
            ResponseDto::from_str(message).unwrap()
        };

        // CreatePeerのパラメータ生成
        let message = r#"{
            "type": "PEER",
            "command": "CREATE",
            "params": {
                "key": "pt-9749250e-d157-4f80-9ee2-359ce8524308",
                "domain": "localhost",
                "peer_id": "peer_id",
                "turn": true
            }
        }"#;
        let dto = RequestDto::from_str(message).unwrap();

        // repositoryのMockを生成
        // 呼び出しに成功するケース
        let mut repository = MockRepository::new();
        repository.expect_register().times(1).returning(|_, _, _| {
            let message = r#"{
                    "is_success":true,
                    "result":{
                        "type":"PEER",
                        "command":"CREATE",
                        "peer_id":"data_caller",
                        "token":"pt-06cf1d26-0ef0-4b03-aca6-933027d434c2"
                    }
                }"#;
            Response::from_str(message)
        });
        let repository: Box<dyn Repository> = Box::new(repository);
        let logger = helper::create_logger();
        let program_state = helper::create_program_state();
        let function = helper::create_functions();

        // 実行
        let create_peer = Create {};
        let result = create_peer
            .execute(&repository, &program_state, &logger, &function, dto)
            .await;
        assert_eq!(result.unwrap(), answer);
    }

    #[tokio::test]
    async fn fail() {
        // APIがエラーを返してくるケース

        // CreatePeerのパラメータ生成
        let message = r#"{
            "type": "PEER",
            "command": "CREATE",
            "params": {
                "key": "pt-9749250e-d157-4f80-9ee2-359ce8524308",
                "domain": "localhost",
                "peer_id": "peer_id",
                "turn": true
            }
        }"#;
        let dto = RequestDto::from_str(message).unwrap();

        // repositoryのMockを生成
        // 呼び出しに成功するケース
        let mut repository = MockRepository::new();
        repository.expect_register().times(1).returning(|_, _, _| {
            let answer = error::Error::create_local_error("error");
            return Err(answer);
        });
        let repository: Box<dyn Repository> = Box::new(repository);
        let logger = helper::create_logger();
        let program_state = helper::create_program_state();
        let function = helper::create_functions();

        // 実行
        let create_peer = Create {};
        if let Err(error::Error::LocalError(message)) = create_peer
            .execute(&repository, &program_state, &logger, &function, dto)
            .await
        {
            assert_eq!(message, "wrong parameter Peer(Create { params: CreatePeerParams { key: \"pt-9749250e-d157-4f80-9ee2-359ce8524308\", domain: \"localhost\", peer_id: PeerId(\"peer_id\"), turn: true } })");
        }
    }

    #[tokio::test]
    async fn invalid_parameter() {
        // 間違ったパラメータの生成
        let dto = RequestDto::Test;

        // mockの生成
        // パラメータが違う場合、repositoryが呼ばれないはずである
        let mut repository = MockRepository::new();
        repository
            .expect_register()
            .times(0)
            .returning(|_, _, _| unreachable!());
        let repository: Box<dyn Repository> = Box::new(repository);

        let logger = helper::create_logger();
        let program_state = helper::create_program_state();
        let function = helper::create_functions();
        // 実行
        let create_peer = Create {};

        // 評価
        // 間違ったパラメータである旨を返してくるはずである
        if let Err(error::Error::LocalError(error_message)) = create_peer
            .execute(&repository, &program_state, &logger, &function, dto)
            .await
        {
            assert_eq!(error_message, "wrong parameter Test");
        }
    }
}
