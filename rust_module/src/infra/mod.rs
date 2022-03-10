use async_trait::async_trait;
use tokio::sync::Mutex;
use tokio::sync::{mpsc, oneshot};

use crate::domain::entity::{Request, Response, Stringify};
use crate::domain::repository::Repository;
use crate::error::Error;
use crate::{error, Logger, ProgramState};

#[derive(Debug)]
pub(crate) struct RepositoryImpl {
    sender: mpsc::Sender<(oneshot::Sender<String>, String)>,
    receiver: Mutex<mpsc::Receiver<String>>,
}

impl RepositoryImpl {
    pub fn new(
        sender: mpsc::Sender<(oneshot::Sender<String>, String)>,
        receiver: mpsc::Receiver<String>,
    ) -> Self {
        RepositoryImpl {
            sender,
            receiver: Mutex::new(receiver),
        }
    }
}

#[async_trait]
impl Repository for RepositoryImpl {
    async fn register(
        &self,
        _program_state: &ProgramState,
        _logger: &Logger,
        params: Request,
    ) -> Result<Response, Error> {
        // SkyWay Crateからの戻り値を得るためのoneshot channelを生成
        let (channel_message_tx, channel_message_rx) = tokio::sync::oneshot::channel();

        // Request型である時点でto_stringには失敗しない
        let message = params.to_string().unwrap();

        // SkyWay Crateへメッセージを送る
        // 失敗した場合はエラーメッセージを返す
        if let Err(_) = self.sender.send((channel_message_tx, message)).await {
            return Err(error::Error::create_local_error(
                "could not send request to skyway crate",
            ));
        }

        // SkyWay Crateからのメッセージを処理する
        match channel_message_rx.await {
            Ok(message) => Ok(Response::from_str(&message)?),
            Err(_) => Err(error::Error::create_local_error(
                "could not receive response from skyway crate",
            )),
        }
    }
    async fn receive_event(
        &self,
        program_state: &ProgramState,
        _logger: &Logger,
    ) -> Result<Response, error::Error> {
        use std::time::Duration;

        use tokio::time;
        while !program_state.is_shutting_down() {
            let mut rx = self.receiver.lock().await;

            match time::timeout(Duration::from_millis(1000), rx.recv()).await {
                Ok(Some(response_string)) => {
                    return Response::from_str(&response_string);
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
    use module::prelude::request_message::{Parameter, PeerServiceParams};

    use super::*;
    use crate::application::usecase::helper;
    use crate::domain::entity::FromStr;

    #[tokio::test]
    async fn success() {
        // 送信メッセージの生成
        let inner = PeerServiceParams::Create {
            params: Parameter(Default::default()),
        };
        let message: Request = Request::Peer(inner);

        // Repository Implの生成
        let (message_tx, mut message_rx) = mpsc::channel::<(oneshot::Sender<String>, String)>(10);
        let (_event_tx, event_rx) = mpsc::channel::<String>(1000);
        let repository_impl = RepositoryImpl::new(message_tx, event_rx);

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
                    "type":"PEER",
                    "command":"CREATE",
                    "peer_id":"hoge",
                    "token":"pt-9749250e-d157-4f80-9ee2-359ce8524308"
                }
            }"#;
            let _ = response_message_tx.send(response_str.into());
        });

        let logger = helper::create_logger();
        let program_state = helper::create_program_state();
        // 実行
        let result = repository_impl
            .register(&program_state, &logger, message)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    // responseが帰ってこないケース
    async fn error_no_response() {
        // 送信メッセージの生成
        let inner = PeerServiceParams::Create {
            params: Parameter(Default::default()),
        };
        let message: Request = Request::Peer(inner);

        // Repository Implの生成
        let (message_tx, mut message_rx) = mpsc::channel::<(oneshot::Sender<String>, String)>(10);
        let (_event_tx, event_rx) = mpsc::channel::<String>(1000);
        let repository_impl = RepositoryImpl::new(message_tx, event_rx);

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

        let logger = helper::create_logger();
        let program_state = helper::create_program_state();
        // 実行
        let result = repository_impl
            .register(&program_state, &logger, message)
            .await;
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
        let inner = PeerServiceParams::Create {
            params: Parameter(Default::default()),
        };
        let message: Request = Request::Peer(inner);

        // Repository Implの生成
        let (message_tx, mut message_rx) = mpsc::channel::<(oneshot::Sender<String>, String)>(10);
        let (_event_tx, event_rx) = mpsc::channel::<String>(1000);
        let repository_impl = RepositoryImpl::new(message_tx, event_rx);

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
                    "type":"PEER",
                    "command":"CREATE",
                    "peer_id":"hoge",
                    "token":"pt-9749250e-d157-4f80-9ee2-359ce8524308"
            }"#;
            let _ = response_message_tx.send(response_str.into());
        });

        let logger = helper::create_logger();
        let program_state = helper::create_program_state();
        // 実行
        let result = repository_impl
            .register(&program_state, &logger, message)
            .await;
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
    use super::*;
    use crate::application::usecase::helper;

    #[tokio::test]
    // eventとして異常な文字列を受信した場合
    async fn success() {
        // Repository Implの生成
        let (message_tx, _message_rx) = mpsc::channel::<(oneshot::Sender<String>, String)>(10);
        let (event_tx, event_rx) = mpsc::channel::<String>(1000);
        let repository_impl = RepositoryImpl::new(message_tx, event_rx);

        let (_close_tx, close_rx) = oneshot::channel::<()>();

        tokio::spawn(async move {
            let response_str = r#"{
                "is_success":true,
                "result":{
                    "type":"PEER",
                    "command":"CREATE",
                    "peer_id":"hoge",
                    "token":"pt-9749250e-d157-4f80-9ee2-359ce8524308"
                }
            }"#;
            let _ = event_tx.send(response_str.to_string()).await;
            let _ = close_rx.await;
        });

        let logger = helper::create_logger();
        let program_state = helper::create_program_state();
        // 実行
        let result = repository_impl.receive_event(&program_state, &logger).await;
        match result {
            Ok(_) => assert!(true),
            _ => assert!(false),
        }
    }

    #[tokio::test]
    // eventとして異常な文字列を受信した場合
    async fn recv_invalid_json() {
        // Repository Implの生成
        let (message_tx, _message_rx) = mpsc::channel::<(oneshot::Sender<String>, String)>(10);
        let (event_tx, event_rx) = mpsc::channel::<String>(1000);
        let repository_impl = RepositoryImpl::new(message_tx, event_rx);

        let (_close_tx, close_rx) = oneshot::channel::<()>();

        tokio::spawn(async move {
            let _ = event_tx.send("invalid json".to_string()).await;
            let _ = close_rx.await;
        });

        let logger = helper::create_logger();
        let program_state = helper::create_program_state();
        // 実行
        let result = repository_impl.receive_event(&program_state, &logger).await;
        match result {
            Err(error::Error::SerdeError { error: _ }) => {
                assert!(true)
            }
            _ => assert!(false),
        }
    }
}
