/// DataChannelの確立要求に対し、以下の内容を実施する
/// WebRTC GW - ROS間のデータのやり取りは全てC++側に任せる
/// そのためこのモジュールでは、WebRTC GWとのJSONのやり取りのみを行う
/// 具体的な手順は以下の通り
/// 1. Dataポートを開放させ、DataChannelへのSourceとして利用する
/// 2. C++側でRos Pluginをロードさせる。
///    ロードエラーが出たら、Dataポートを閉じてエラーを返して終了。
///    ロードエラーが発生しない場合、この時点でC++側は送受信の準備ができている
/// 3. C++側で開放したポート番号を戻り値から取得し、CONNECT APIをcallし、戻り値を返す
use std::ffi::CStr;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{json, Value};
use shaku::Component;

use crate::application::dto::request::{DataRequestDto, RequestDto};
use crate::application::dto::response::{DataResponseDto, ResponseDto, ResponseDtoResult};
use crate::application::factory::Factory;
use crate::application::usecase::Service;
use crate::domain::entity::request::{DataRequest, Request};
use crate::domain::entity::response::{DataResponse, Response, ResponseResult};
use crate::domain::entity::{
    ConnectQuery, ConnectQueryOption, DataIdWrapper, PhantomId, SerializableSocket, SocketInfo,
};
use crate::domain::repository::Repository;
use crate::error;
use crate::ffi::rust_to_c_bridge::c_functions_wrapper::DataPipeInfo;
use crate::ffi::rust_to_c_bridge::state_objects::{CallbackFunctions, GlobalState};

#[derive(Component)]
#[shaku(interface = Service)]
pub(crate) struct Connect {
    #[shaku(inject)]
    repository: Arc<dyn Repository>,
    #[shaku(inject)]
    state: Arc<dyn GlobalState>,
    #[shaku(inject)]
    factory: Arc<dyn Factory>,
    #[shaku(inject)]
    callback: Arc<dyn CallbackFunctions>,
}

#[async_trait]
impl Service for Connect {
    async fn execute(&self, request: RequestDto) -> Result<ResponseDtoResult, error::Error> {
        if let RequestDto::Data(DataRequestDto::Connect {
            params: connect_params,
        }) = request
        {
            // 1.は単独で実施可能なので最初に行う
            let (data_id, address, port) = {
                let create_data_param = RequestDto::Data(DataRequestDto::Create);
                let service = self.factory.create_service(&create_data_param);
                let result = service.execute(create_data_param).await?;
                if let ResponseDtoResult::Success(ResponseDto::Data(DataResponseDto::Create(
                    socket,
                ))) = result
                {
                    (
                        socket.get_id().expect("failed to open data port"),
                        socket.ip(),
                        socket.port(),
                    )
                } else {
                    let message = format!("create data failed {:?}", result);
                    return Err(error::Error::create_local_error(&message));
                }
            };

            // 2. C++側でRos Pluginをロードさせる。
            // ここでserializeが失敗するケースはRustの型システムにより発生しないので、テストはしていない
            let plugin_params = serde_json::to_string(&connect_params.plugin_info.plugins).unwrap();

            let (flag, port, error_message) = {
                let result = self.callback.data_callback(
                    &address.to_string(),
                    port,
                    &connect_params.plugin_info.r#type,
                    &plugin_params,
                );
                (
                    result.is_success,
                    result.port,
                    unsafe { CStr::from_ptr(result.error_message) }
                        .to_str()
                        .unwrap()
                        .to_string(),
                )
            };

            if !flag {
                let delete_data_param = RequestDto::Data(DataRequestDto::Delete {
                    params: DataIdWrapper { data_id },
                });
                let _ = self.factory.create_service(&delete_data_param);
                return Err(error::Error::create_local_error(&error_message));
            }

            // 3. C++側で開放したポート番号を戻り値から取得し、CONNECT APIをcallし、戻り値を返す
            // Connect APIを呼ぶためのパラメータ生成
            // Dest ObjectのUDPソケット情報が必要なので、このタイミングで実施する
            let params = {
                let params = ConnectQuery {
                    peer_id: connect_params.peer_id,
                    token: connect_params.token,
                    options: connect_params.options,
                    target_id: connect_params.target_id,
                    params: Some(DataIdWrapper {
                        data_id: data_id.clone(),
                    }),
                    redirect_params: Some(
                        SocketInfo::<PhantomId>::try_create(None, "127.0.0.1", port).unwrap(),
                    ),
                };

                Request::Data(DataRequest::Connect { params })
            };

            let result = self.repository.register(params).await?;
            match result {
                // Connectに成功した場合
                ResponseResult::Success(Response::Data(DataResponse::Connect(params))) => {
                    // Topicの情報を保管
                    let response = DataPipeInfo {
                        data_connection_id: params.data_connection_id.clone(),
                        data_pipe_port_num: port,
                    };
                    self.state
                        .store_topic(params.data_connection_id.clone(), response);

                    return Ok(ResponseDtoResult::Success(ResponseDto::Data(
                        DataResponseDto::Connect(params),
                    )));
                }
                _ => unreachable!(),
            }
        }

        return Err(error::Error::create_local_error("invalid parameters"));
    }
}

#[cfg(test)]
mod connect_data_test {
    use std::ffi::CString;

    use shaku::HasComponent;

    use super::*;
    use crate::application::factory::MockFactory;
    use crate::application::usecase::MockService;
    use crate::di::*;
    use crate::domain::entity::response::{DataResponse, ResponseResult};
    use crate::domain::entity::{DataConnectionId, DataConnectionIdWrapper, DataId, SocketInfo};
    use crate::domain::repository::MockRepository;
    use crate::ffi::rust_to_c_bridge::c_functions_wrapper::PluginLoadResult;
    use crate::ffi::rust_to_c_bridge::state_objects::{MockCallbackFunctions, MockGlobalState};

    #[tokio::test]
    // Dataポートの開放に失敗した場合はエラーを返す
    async fn create_data_port_fail() {
        // mockのsetup
        let mut factory = MockFactory::new();
        factory.expect_create_service().times(1).returning(|_| {
            // Dataポートの開放に失敗
            let mut mock_service = MockService::new();
            mock_service
                .expect_execute()
                .returning(|_| Err(error::Error::create_local_error("failed to open data port")));
            Arc::new(mock_service)
        });

        // 以下のMockはこのテストでは呼ばれない
        let mut repository = MockRepository::new();
        repository
            .expect_register()
            .times(0)
            .returning(|_| unreachable!());
        let mut caller = MockCallbackFunctions::new();
        caller
            .expect_data_callback()
            .times(0)
            .returning(|_, _, _, _| unreachable!());
        let mut state = MockGlobalState::new();
        state
            .expect_store_topic()
            .times(0)
            .returning(|_, _| unreachable!());

        // サービスの生成
        let module = DataConnectService::builder()
            .with_component_override::<dyn Factory>(Box::new(factory))
            .with_component_override::<dyn Repository>(Box::new(repository))
            .with_component_override::<dyn CallbackFunctions>(Box::new(caller))
            .with_component_override::<dyn GlobalState>(Box::new(state))
            .build();
        let service: &dyn Service = module.resolve_ref();

        let request = {
            let message = r#"{
                   "request_type":"DATA",
                   "command":"CONNECT",
                   "params":{
                       "peer_id": "peer_id",
                       "token": "pt-06cf1d26-0ef0-4b03-aca6-933027d434c2",
                       "target_id":"target_id",
                       "plugin_info": {
                            "type": "binary",
                            "plugins": []
                       }
                   }
               }"#;

            RequestDto::from_str(&message).unwrap()
        };

        let result = service.execute(request).await;
        if let Err(error::Error::LocalError(e)) = result {
            assert_eq!(e, "failed to open data port");
        }
    }

    #[tokio::test]
    // Pluginのロードに失敗した場合は、Dataポートを閉じたあとエラーを返す
    async fn plugin_load_failed() {
        // mockのsetup
        let mut factory = MockFactory::new();
        factory.expect_create_service().times(2).returning(|_| {
            let mut mock_service = MockService::new();
            mock_service
                .expect_execute()
                .returning(|request| match request {
                    RequestDto::Data(DataRequestDto::Create) => {
                        let socket = SocketInfo::<DataId>::try_create(
                            Some("da-06cf1d26-0ef0-4b03-aca6-933027d434c2".to_string()),
                            "127.0.0.1",
                            10000,
                        )
                        .unwrap();
                        Ok(ResponseDtoResult::Success(ResponseDto::Data(
                            DataResponseDto::Create(socket),
                        )))
                    }
                    RequestDto::Data(DataRequestDto::Delete { params }) => {
                        Ok(ResponseDtoResult::Success(ResponseDto::Data(
                            DataResponseDto::Delete(params),
                        )))
                    }
                    _ => unreachable!(),
                });
            Arc::new(mock_service)
        });

        let mut caller = MockCallbackFunctions::new();
        caller
            .expect_data_callback()
            .times(1)
            .returning(|_, _, _, _| PluginLoadResult {
                is_success: false,
                port: 0,
                error_message: CString::new("plugin_router load error").unwrap().into_raw(),
            });

        // 以下のMockはこのテストでは呼ばれない
        let mut repository = MockRepository::new();
        repository
            .expect_register()
            .times(0)
            .returning(|_| unreachable!());
        let mut state = MockGlobalState::new();
        state
            .expect_store_topic()
            .times(0)
            .returning(|_, _| unreachable!());

        // サービスの生成
        let module = DataConnectService::builder()
            .with_component_override::<dyn Factory>(Box::new(factory))
            .with_component_override::<dyn Repository>(Box::new(repository))
            .with_component_override::<dyn CallbackFunctions>(Box::new(caller))
            .with_component_override::<dyn GlobalState>(Box::new(state))
            .build();
        let service: &dyn Service = module.resolve_ref();

        let request = {
            let message = r#"{
                   "request_type":"DATA",
                   "command":"CONNECT",
                   "params":{
                       "peer_id": "peer_id",
                       "token": "pt-06cf1d26-0ef0-4b03-aca6-933027d434c2",
                       "target_id":"target_id",
                       "plugin_info": {
                            "type": "binary",
                            "plugins": []
                       }
                   }
               }"#;

            RequestDto::from_str(&message).unwrap()
        };

        let result = service.execute(request).await;
        if let Err(error::Error::LocalError(e)) = result {
            assert_eq!(e, "plugin_router load error");
        }
    }

    #[tokio::test]
    // eventとして異常な文字列を受信した場合
    async fn success() {
        // 待値の生成
        // DataConnectionResponseを含むConnect パラメータを受け取れるはずである
        let expected = {
            let value = DataConnectionIdWrapper {
                data_connection_id: DataConnectionId::try_create(
                    "dc-8bdef7a1-65c8-46be-a82e-37d51c776309",
                )
                .unwrap(),
            };

            ResponseDtoResult::Success(ResponseDto::Data(DataResponseDto::Connect(value)))
        };

        let mut factory = MockFactory::new();
        factory.expect_create_service().times(1).returning(|_| {
            let mut mock_service = MockService::new();
            mock_service.expect_execute().returning(|_| {
                let socket = SocketInfo::<DataId>::try_create(
                    Some("da-06cf1d26-0ef0-4b03-aca6-933027d434c2".to_string()),
                    "127.0.0.1",
                    10000,
                )
                .unwrap();
                Ok(ResponseDtoResult::Success(ResponseDto::Data(
                    DataResponseDto::Create(socket),
                )))
            });
            Arc::new(mock_service)
        });

        let mut repository = MockRepository::new();
        repository.expect_register().times(1).returning(|_| {
            // redirectのmock
            // 成功し、DataConnectionIdを返すケース
            Ok(ResponseResult::Success(Response::Data(
                DataResponse::Connect(DataConnectionIdWrapper {
                    data_connection_id: DataConnectionId::try_create(
                        "dc-8bdef7a1-65c8-46be-a82e-37d51c776309",
                    )
                    .unwrap(),
                }),
            )))
        });

        let mut caller = MockCallbackFunctions::new();
        caller
            .expect_data_callback()
            .times(1)
            .returning(|_, _, _, _| PluginLoadResult {
                is_success: true,
                port: 60000,
                error_message: CString::new("").unwrap().into_raw(),
            });

        let mut state = MockGlobalState::new();
        state.expect_store_topic().times(1).returning(
            |data_connection_id: DataConnectionId, info: DataPipeInfo| {
                assert_eq!(
                    data_connection_id.as_str(),
                    "dc-8bdef7a1-65c8-46be-a82e-37d51c776309"
                );
                assert_eq!(info.data_pipe_port_num, 60000);
            },
        );

        // サービスの生成
        let module = DataConnectService::builder()
            .with_component_override::<dyn Factory>(Box::new(factory))
            .with_component_override::<dyn Repository>(Box::new(repository))
            .with_component_override::<dyn CallbackFunctions>(Box::new(caller))
            .with_component_override::<dyn GlobalState>(Box::new(state))
            .build();
        let service: &dyn Service = module.resolve_ref();

        let request = {
            let message = r#"{
                   "request_type":"DATA",
                   "command":"CONNECT",
                   "params":{
                       "peer_id": "peer_id",
                       "token": "pt-06cf1d26-0ef0-4b03-aca6-933027d434c2",
                       "target_id":"target_id",
                       "plugin_info": {
                            "type": "binary",
                            "plugins": []
                       }
                   }
               }"#;

            RequestDto::from_str(&message).unwrap()
        };

        let result = service.execute(request).await;
        assert_eq!(result.unwrap(), expected);
    }
}
