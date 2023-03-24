// skyway_webrtc_gateway_callerをInfra層として利用するための薄いラッパー
use std::sync::Arc;

use async_trait::async_trait;
use shaku::Component;

use crate::domain::entity::request::Request;
use crate::domain::entity::response::ResponseResult;
use crate::domain::entity::Stringify;
use crate::domain::repository::Repository;
use crate::error;
use crate::ffi::rust_to_c_bridge::state_objects::GlobalState;

#[derive(Component)]
#[shaku(interface = Repository)]
pub(crate) struct RepositoryImpl {
    #[shaku(inject)]
    state: Arc<dyn GlobalState>,
}

#[async_trait]
impl Repository for RepositoryImpl {
    async fn register(&self, params: Request) -> Result<ResponseResult, error::Error> {
        // SkyWay Crateからの戻り値を得るためのoneshot channelを生成
        let (channel_message_tx, channel_message_rx) = tokio::sync::oneshot::channel();

        // Request型である時点でto_stringには失敗しない
        let message = params.to_string().unwrap();

        let sender = self.state.channels().sender();

        // SkyWay Crateへメッセージを送る
        // 失敗した場合はエラーメッセージを返す
        if let Err(_) = sender.send((channel_message_tx, message)).await {
            return Err(error::Error::create_local_error(
                "could not send request to skyway crate",
            ));
        }

        // SkyWay Crateからのメッセージを処理する
        match channel_message_rx.await {
            Ok(message) => Ok(ResponseResult::from_str(&message)?),
            Err(_) => Err(error::Error::create_local_error(
                "could not receive response from skyway crate",
            )),
        }
    }

    async fn receive_event(&self) -> Result<ResponseResult, error::Error> {
        use std::time::Duration;

        use tokio::time;
        let state = self.state.program_state();
        let channels = self.state.channels();
        let receiver = channels.receiver();

        while !state.is_shutting_down() {
            let mut rx = receiver.lock().await;
            match time::timeout(Duration::from_millis(1000), rx.recv()).await {
                Ok(Some(response_string)) => {
                    return ResponseResult::from_str(&response_string);
                }
                Ok(None) => {
                    // closed
                    return Err(error::Error::create_local_error("receiver is closed"));
                }
                Err(_) => {
                    //timeout
                    continue;
                }
            }
        }

        return Err(error::Error::create_local_error("ros has been shut down"));
    }
}

#[cfg(test)]
mod infra_send_message_test {
    use once_cell::sync::OnceCell;
    use shaku::HasComponent;
    use tokio::sync::{mpsc, oneshot, Mutex};

    use super::*;
    use crate::di::RepositoryModule;
    use crate::domain::entity::request::PeerRequest;
    use crate::domain::entity::{CreatePeerParams, FromStr, PeerId};
    use crate::ffi::rust_to_c_bridge::state_objects::{Channels, ChannelsImpl, MockGlobalState};

    fn create_request() -> Request {
        let inner = PeerRequest::Create {
            params: CreatePeerParams {
                key: "API_KEY".to_string(),
                domain: "localhost".to_string(),
                peer_id: PeerId::new("peer_id"),
                turn: false,
            },
        };
        return Request::Peer(inner);
    }

    #[tokio::test]
    async fn success() {
        // 送信メッセージの生成
        let message = create_request();

        let (message_tx, mut message_rx) = mpsc::channel::<(oneshot::Sender<String>, String)>(10);
        // eventのtestは他でやるので、txは使わない
        let (_event_tx, event_rx) = mpsc::channel::<String>(1000);
        static CHANNELS: OnceCell<Arc<dyn Channels>> = OnceCell::new();
        let _ = CHANNELS.set(Arc::new(ChannelsImpl::new(
            message_tx,
            Mutex::new(event_rx),
        )));

        // GlobalStateのMockを生成
        // message_txを返すために使う
        let mut state = MockGlobalState::new();
        state
            .expect_channels()
            .times(1)
            .returning(move || CHANNELS.get().unwrap());

        // サービスを生成
        let module = RepositoryModule::builder()
            .with_component_override::<dyn GlobalState>(Box::new(state))
            .build();
        let repository_impl: &dyn Repository = module.resolve_ref();

        // WebRTC Gatewayから値を返してくる動きのmock
        // 成功して正常な値を返してくる
        tokio::spawn(async move {
            let (response_message_tx, request_message) = message_rx.recv().await.unwrap();

            let request = Request::from_str(&request_message);
            match request {
                Ok(Request::Peer(_)) => {
                    assert!(true)
                }
                _ => {
                    assert!(false)
                }
            }

            let response_str = r#"{
                "is_success":true,
                "result":{
                    "request_type":"PEER",
                    "command":"CREATE",
                    "peer_id":"hoge",
                    "token":"pt-9749250e-d157-4f80-9ee2-359ce8524308"
                }
            }"#;
            let _ = response_message_tx.send(response_str.into());
        });

        // 実行
        let result = repository_impl.register(message).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    // responseが帰ってこないケース
    async fn error_no_response() {
        // 送信メッセージの生成
        let message = create_request();

        // WebRTC Gatewayが生成するSenderとReceiver相当のものを作成
        let (message_tx, mut message_rx) = mpsc::channel::<(oneshot::Sender<String>, String)>(10);
        // eventのtestは他でやるので、txは使わない
        let (_event_tx, event_rx) = mpsc::channel::<String>(1000);
        static CHANNELS: OnceCell<Arc<dyn Channels>> = OnceCell::new();
        let _ = CHANNELS.set(Arc::new(ChannelsImpl::new(
            message_tx,
            Mutex::new(event_rx),
        )));

        // GlobalStateのMockを生成
        // message_txを返すために使う
        let mut state = MockGlobalState::new();
        state
            .expect_channels()
            .times(1)
            .returning(move || CHANNELS.get().unwrap());

        // サービスを生成
        let module = RepositoryModule::builder()
            .with_component_override::<dyn GlobalState>(Box::new(state))
            .build();
        let repository_impl: &dyn Repository = module.resolve_ref();

        // WebRTC Gatewayが値を返さないケース
        tokio::spawn(async move {
            let (_response_message_tx, request_message) = message_rx.recv().await.unwrap();

            let request = Request::from_str(&request_message);
            match request {
                Ok(Request::Peer(_)) => {
                    assert!(true)
                }
                _ => {
                    assert!(false)
                }
            }
        });

        // 実行
        let result = repository_impl.register(message).await;
        match result {
            Err(error::Error::LocalError(message)) => {
                assert_eq!(message, "could not receive response from skyway crate");
            }
            _ => assert!(false),
        }
    }

    #[tokio::test]
    // responseがinvalidなJSONでパースできないケース
    async fn error_recv_invalid_message() {
        // 送信メッセージの生成
        let message = create_request();

        // WebRTC Gatewayが生成するSenderとReceiver相当のものを作成
        let (message_tx, mut message_rx) = mpsc::channel::<(oneshot::Sender<String>, String)>(10);
        // eventのtestは他でやるので、txは使わない
        let (_event_tx, event_rx) = mpsc::channel::<String>(1000);
        static CHANNELS: OnceCell<Arc<dyn Channels>> = OnceCell::new();
        let _ = CHANNELS.set(Arc::new(ChannelsImpl::new(
            message_tx,
            Mutex::new(event_rx),
        )));

        // GlobalStateのMockを生成
        // message_txを返すために使う
        let mut state = MockGlobalState::new();
        state
            .expect_channels()
            .times(1)
            .returning(move || CHANNELS.get().unwrap());

        // サービスを生成
        let module = RepositoryModule::builder()
            .with_component_override::<dyn GlobalState>(Box::new(state))
            .build();
        let repository_impl: &dyn Repository = module.resolve_ref();

        tokio::spawn(async move {
            let (response_message_tx, request_message) = message_rx.recv().await.unwrap();

            let request = Request::from_str(&request_message);
            match request {
                Ok(Request::Peer(_)) => {
                    assert!(true)
                }
                _ => {
                    assert!(false)
                }
            }

            let response_str = r#"{
                "is_success":true,
                "result":{
                    "request_type":"PEER",
                    "command":"CREATE",
                    "peer_id":"hoge",
                    "token":"pt-9749250e-d157-4f80-9ee2-359ce8524308"
            }"#;
            let _ = response_message_tx.send(response_str.into());
        });

        // 実行
        let result = repository_impl.register(message).await;
        match result {
            Err(error::Error::SerdeError { error: _ }) => {
                assert!(true)
            }
            _ => assert!(false),
        }
    }
}

#[cfg(test)]
mod infra_receive_event_test {
    use once_cell::sync::OnceCell;
    use shaku::HasComponent;
    use tokio::sync::{mpsc, oneshot, Mutex};

    use super::*;
    use crate::di::RepositoryModule;
    use crate::ffi::rust_to_c_bridge::c_functions_wrapper::{helper, ProgramStateHolder};
    use crate::ffi::rust_to_c_bridge::state_objects::{Channels, ChannelsImpl, MockGlobalState};

    #[tokio::test]
    // eventを正常に受信するケース
    async fn success() {
        // WebRTC Gatewayが生成するSenderとReceiver相当のものを作成
        let (message_tx, _message_rx) = mpsc::channel::<(oneshot::Sender<String>, String)>(10);
        // eventのtestは他でやるので、txは使わない
        let (event_tx, event_rx) = mpsc::channel::<String>(1000);
        static CHANNELS: OnceCell<Arc<dyn Channels>> = OnceCell::new();
        let _ = CHANNELS.set(Arc::new(ChannelsImpl::new(
            message_tx,
            Mutex::new(event_rx),
        )));

        static PROGRAM_STATE_INSTANCE: OnceCell<ProgramStateHolder> = OnceCell::new();
        let _ = PROGRAM_STATE_INSTANCE.set(ProgramStateHolder::new(
            helper::is_running,
            helper::is_shutting_down,
            helper::sleep,
            helper::wait_for_shutdown,
            helper::shutdown,
        ));

        // GlobalStateのMockを生成
        let mut state = MockGlobalState::new();
        state
            .expect_channels()
            .times(1)
            .returning(move || CHANNELS.get().unwrap());
        state
            .expect_program_state()
            .times(1)
            .returning(move || PROGRAM_STATE_INSTANCE.get().unwrap());

        // サービスを生成
        let module = RepositoryModule::builder()
            .with_component_override::<dyn GlobalState>(Box::new(state))
            .build();
        let repository_impl: &dyn Repository = module.resolve_ref();

        tokio::spawn(async move {
            let response_str = r#"{
                "is_success":true,
                "result":{
                    "request_type":"PEER",
                    "command":"CREATE",
                    "peer_id":"hoge",
                    "token":"pt-9749250e-d157-4f80-9ee2-359ce8524308"
                }
            }"#;
            let _ = event_tx.send(response_str.to_string()).await;
        });

        // 実行
        let result = repository_impl.receive_event().await;
        match result {
            Ok(_) => assert!(true),
            _ => assert!(false),
        }
    }

    #[tokio::test]
    // eventとして異常な文字列を受信した場合
    async fn recv_invalid_json() {
        // WebRTC Gatewayが生成するSenderとReceiver相当のものを作成
        let (message_tx, _message_rx) = mpsc::channel::<(oneshot::Sender<String>, String)>(10);
        // eventのtestは他でやるので、txは使わない
        let (event_tx, event_rx) = mpsc::channel::<String>(1000);
        static CHANNELS: OnceCell<Arc<dyn Channels>> = OnceCell::new();
        let _ = CHANNELS.set(Arc::new(ChannelsImpl::new(
            message_tx,
            Mutex::new(event_rx),
        )));

        static PROGRAM_STATE_INSTANCE: OnceCell<ProgramStateHolder> = OnceCell::new();
        let _ = PROGRAM_STATE_INSTANCE.set(ProgramStateHolder::new(
            helper::is_running,
            helper::is_shutting_down,
            helper::sleep,
            helper::wait_for_shutdown,
            helper::shutdown,
        ));

        // GlobalStateのMockを生成
        let mut state = MockGlobalState::new();
        state
            .expect_channels()
            .times(1)
            .returning(move || CHANNELS.get().unwrap());
        state
            .expect_program_state()
            .times(1)
            .returning(move || PROGRAM_STATE_INSTANCE.get().unwrap());

        // サービスを生成
        let module = RepositoryModule::builder()
            .with_component_override::<dyn GlobalState>(Box::new(state))
            .build();
        let repository_impl: &dyn Repository = module.resolve_ref();

        tokio::spawn(async move {
            let _ = event_tx.send("invalid json".to_string()).await;
        });

        // 実行
        let result = repository_impl.receive_event().await;
        match result {
            Err(error::Error::SerdeError { error: _ }) => {
                assert!(true)
            }
            _ => assert!(false),
        }
    }
}
