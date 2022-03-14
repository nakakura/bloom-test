pub(crate) mod data;
pub(crate) mod peer;

use std::net::TcpListener;

use async_trait::async_trait;

use crate::application::dto::request::RequestDto;
use crate::application::dto::response::{
    DataResponseDto, MediaResponseDto, PeerResponseDto, ResponseDto, ResponseDtoResult,
};
use crate::application::CallbackFunctions;
use crate::domain::entity::request::Request;
use crate::domain::entity::response::{Response, ResponseResult};
use crate::Repository;
use crate::{error, Logger, ProgramState};

//========== Utility Methods ==========

pub(crate) fn available_port() -> std::io::Result<u16> {
    match TcpListener::bind("0.0.0.0:0") {
        Ok(listener) => Ok(listener.local_addr().unwrap().port()),
        Err(e) => Err(e),
    }
}

//========== Service ==========
#[async_trait]
pub(crate) trait Service {
    async fn execute(
        &self,
        repository: &Box<dyn Repository>,
        program_state: &ProgramState,
        logger: &Logger,
        cb_functions: &CallbackFunctions,
        message: RequestDto,
    ) -> Result<ResponseDtoResult, error::Error>;
}

pub(crate) struct General {}

#[async_trait]
impl Service for General {
    async fn execute(
        &self,
        repository: &Box<dyn Repository>,
        program_state: &ProgramState,
        logger: &Logger,
        _cb_functions: &CallbackFunctions,
        message: RequestDto,
    ) -> Result<ResponseDtoResult, error::Error> {
        if let RequestDto::Peer(inner) = message {
            let request = Request::Peer(inner);
            let message = repository.register(program_state, logger, request).await?;
            return match message {
                ResponseResult::Success(Response::Peer(peer)) => Ok(ResponseDtoResult::Success(
                    ResponseDto::Peer(PeerResponseDto::from_entity(peer)),
                )),
                ResponseResult::Success(Response::Media(media)) => Ok(ResponseDtoResult::Success(
                    ResponseDto::Media(MediaResponseDto::from_entity(media)),
                )),
                ResponseResult::Success(Response::Data(data)) => Ok(ResponseDtoResult::Success(
                    ResponseDto::Data(DataResponseDto::from_entity(data)),
                )),
                ResponseResult::Error(error) => Ok(ResponseDtoResult::Error(error)),
            };
        }

        let error_message = format!("wrong parameter {:?}", message);
        return Err(error::Error::create_local_error(&error_message));
    }
}

#[cfg(test)]
pub(crate) mod helper {
    use std::os::raw::{c_char, c_double};

    use crate::application::{CallbackFunctions, TopicParameters};
    use crate::{Logger, ProgramState};

    extern "C" fn log(_message: *const c_char) {}

    pub(crate) fn create_logger() -> Logger {
        Logger::new(log, log, log, log)
    }

    extern "C" fn is_running() -> bool {
        return true;
    }

    extern "C" fn is_shutting_down() -> bool {
        return false;
    }

    extern "C" fn sleep_(_duration: c_double) {}

    extern "C" fn wait_for_shutdown() {}

    pub(crate) fn create_program_state() -> ProgramState {
        ProgramState::new(is_running, is_shutting_down, sleep_, wait_for_shutdown)
    }

    extern "C" fn create_peer(_peer_id: *mut c_char, _token: *mut c_char) {}

    extern "C" fn peer_delete() {}

    extern "C" fn create_data(_param: TopicParameters) {}

    extern "C" fn delete_data(_param: *mut c_char) {}

    pub(crate) fn create_functions() -> CallbackFunctions {
        CallbackFunctions {
            create_peer_callback_c: create_peer,
            peer_deleted_callback: peer_delete,
            data_callback_c: create_data,
            data_connection_deleted_callback_c: delete_data,
        }
    }
}

#[cfg(test)]
mod general_service_test {
    use crate::application::dto::request::RequestDto;
    use crate::application::dto::response::ResponseDtoResult;
    use crate::application::usecase::Service;
    use crate::application::usecase::{helper, General};
    use crate::domain::entity::response::ResponseResult;
    use crate::domain::repository::MockRepository;
    use crate::error;
    use crate::Repository;

    #[tokio::test]
    async fn success() {
        // DeletePeerに成功したメッセージが得られるはずである
        let answer = {
            let message = r#"{
                "is_success":true,
                "result":{
                    "type":"PEER",
                    "command":"DELETE",
                    "peer_id":"data_caller",
                    "token":"pt-87b54b79-643b-4c60-9c64-ead4ab902dee"
                }
            }"#;
            ResponseDtoResult::from_str(message).unwrap()
        };

        // DeletePeerのパラメータ生成
        let message = r#"{
                "type": "PEER",
                "command": "DELETE",
                "params": {
                    "peer_id": "data_caller",
                    "token": "pt-87b54b79-643b-4c60-9c64-ead4ab902dee"
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
                    "command":"DELETE",
                    "peer_id":"data_caller",
                    "token":"pt-87b54b79-643b-4c60-9c64-ead4ab902dee"
                }
            }"#;
            ResponseResult::from_str(message)
        });
        let repository: Box<dyn Repository> = Box::new(repository);

        let logger = helper::create_logger();
        let program_state = helper::create_program_state();
        let function = helper::create_functions();
        // 実行
        let general_peer_service = General {};
        let result = general_peer_service
            .execute(&repository, &program_state, &logger, &function, dto)
            .await;
        assert_eq!(result.unwrap(), answer);
    }

    #[tokio::test]
    async fn fail() {
        // APIがエラーを返してくるケース

        // DeletePeerのパラメータ生成
        let message = r#"{
                "type": "PEER",
                "command": "DELETE",
                "params": {
                    "peer_id": "data_caller",
                    "token": "pt-87b54b79-643b-4c60-9c64-ead4ab902dee"
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
        let general_peer_service = General {};
        if let Err(error::Error::LocalError(message)) = general_peer_service
            .execute(&repository, &program_state, &logger, &function, dto)
            .await
        {
            assert_eq!(message, "error");
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
        let general_peer_service = General {};

        // 評価
        // 間違ったパラメータである旨を返してくるはずである
        if let Err(error::Error::LocalError(error_message)) = general_peer_service
            .execute(&repository, &program_state, &logger, &function, dto)
            .await
        {
            assert_eq!(error_message, "wrong parameter Test");
        }
    }
}
