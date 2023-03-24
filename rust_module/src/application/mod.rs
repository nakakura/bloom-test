/// Rust側の処理の大元となるモジュール
/// 全ての処理はcall_serviceとreceive_eventの2つを経由してC++側と連携される
pub(crate) mod dto;
pub(crate) mod factory;
pub(crate) mod usecase;

use serde::{Deserialize, Serialize};
use shaku::HasComponent;

use crate::application::dto::request::RequestDto;
use crate::application::dto::Command;
use crate::application::factory::Factory;
use crate::application::usecase::event::EventReceive;
use crate::di::*;
use crate::domain::entity::Stringify;
use crate::error;
use crate::ffi::rust_to_c_bridge::c_functions_wrapper::LoggerHolder;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct ErrorMessage {
    is_success: bool,
    result: ErrorMessageInternal,
}

impl Stringify for ErrorMessage {
    fn to_string(&self) -> Result<String, error::Error> {
        serde_json::to_string(self).map_err(|e| error::Error::SerdeError { error: e })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct ErrorMessageInternal {
    request_type: Option<String>,
    command: Option<String>,
    error: String,
}

/// called from ffi::call_service
/// 能動的にWebRTC GatewayのAPIを呼ぶために使用される
/// 取得した結果は、そのままの形ではなく、C++側/End Userが必要とする形に変換される。
/// また、処理によってはRust側のEventListenerが内部的な処理を行うものもある
/// 特別な処理を行うものは、usecase内のdata, media, peer moduleの中で実装される。
/// その他のものはgeneral moduleの中で処理される。
pub(crate) async fn call_service(message: String) -> String {
    // 正常にparseできなかった場合に、request_typeとcommandをユーザに返すために取得を試みる
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub(crate) struct RequestTypeAndCommand {
        request_type: String,
        command: String,
    }

    match RequestDto::from_str(&message) {
        Ok(dto) => {
            let module = GeneralFactory::builder().build();
            let factory: &dyn Factory = module.resolve_ref();
            let service = factory.create_service(&dto);

            // errorメッセージを生成する際に必要なので確保しておく
            let command = dto.command();
            let dto_type = dto.dto_type();

            match service.execute(dto).await {
                Ok(response) => {
                    // ResponseMessageはto_stringでエラーを出すことはない
                    response.to_string().unwrap()
                }
                Err(e) => {
                    let internal = ErrorMessageInternal {
                        request_type: Some(dto_type),
                        command: Some(command),
                        error: format!("{:?}", e),
                    };
                    let error_message = ErrorMessage {
                        is_success: false,
                        result: internal,
                    };
                    error_message.to_string().unwrap()
                }
            }
        }
        Err(_e) => {
            let type_and_command = match serde_json::from_str::<RequestTypeAndCommand>(&message) {
                Ok(request_type_and_command) => (
                    Some(request_type_and_command.request_type),
                    Some(request_type_and_command.command),
                ),
                Err(_e) => (None, None),
            };

            let internal = ErrorMessageInternal {
                request_type: type_and_command.0,
                command: type_and_command.1,
                error: format!("invalid message in call_service: {}", message),
            };
            let error_message = ErrorMessage {
                is_success: false,
                result: internal,
            };
            let message = error_message.to_string().unwrap();
            LoggerHolder::global().error(message.as_str());
            message
        }
    }
}

/// called from ffi::receive_events
/// 起動時に開始されたEventListenerが常時WebRTC Gatewayのイベントを監視している。
/// この関数を通してC++側のプログラムがイベントを取得する。
/// 取得したイベントはそのままの形ではなく、C++側/End Userが必要とする形に変換される。
/// また、イベントによってはRust側のEventListenerが受信時に処理を行うものもある
pub async fn receive_events() -> String {
    let module = EventReceiveService::builder().build();
    let service: &dyn EventReceive = module.resolve_ref();
    let event = service.execute().await;
    match event {
        Ok(event) => serde_json::to_string(&event).unwrap(),
        Err(error) => {
            let internal = ErrorMessageInternal {
                request_type: None,
                command: None,
                error: format!("invalid message in receive_events: {:?}", error),
            };
            let error_message = ErrorMessage {
                is_success: false,
                result: internal,
            };
            let message = error_message.to_string().unwrap();
            LoggerHolder::global().error(message.as_str());
            return message;
        }
    }
}
