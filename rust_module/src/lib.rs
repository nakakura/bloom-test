// skywayプロジェクト全体をDomainモデルとして整理すると
// rust_module以下はApplicationの一部とDomain、Infra層に相当する
// skyway_webrtc_gateway_controller crate(以下SkyWay Crate)をInfra層として利用し、
// ROS側で持つべきDomain知識を定義し、サービスを提供するのが主な目的である

mod application;
mod di;
mod domain;
mod error;
mod ffi;
mod infra;
mod utils;

use std::collections::HashMap;
use std::sync::Arc;

use once_cell::sync::OnceCell;
use shaku::{Component, Interface};
use tokio::sync::Mutex;
use tokio::sync::{mpsc, oneshot};

use crate::domain::entity::{DataConnectionId, MediaConnectionId};
use crate::ffi::global_params::{
    CallbackFunctionsHolder, DataConnectionResponse, LoggerHolder, ProgramStateHolder,
};

use crate::application::dto::response::CallResponseDto;
use crate::ffi::{PluginLoadResult, TopicParameters};
#[cfg(test)]
use mockall::automock;

// C++側とオブジェクトのやり取りをする回数を最低限にするため、C++側のモジュールで本来所持するべきオブジェクトはOnceCellで保持する
//========== ↓ OnceCell ↓ ==========
// C++側に返すべきコールバックを保持する
static CALLBACK_FUNCTIONS: OnceCell<CallbackFunctionsHolder> = OnceCell::new();
// Log出力するために必要なC++側の関数を保持する
static LOGGER_INSTANCE: OnceCell<LoggerHolder> = OnceCell::new();
// Programの状態を取得・操作するために必要なC++側の関数を保持する
static PROGRAM_STATE_INSTANCE: OnceCell<ProgramStateHolder> = OnceCell::new();
// WebRTC Crate起動時に生成されたSender, Receiverを破棄すると通信できなくなるので、保持し続ける
static CHANNELS: OnceCell<Arc<dyn Channels>> = OnceCell::new();
// Event処理やDisconnect時に利用するため、DataConnection確立時に
// Source Topic とDestination Topicの情報を集めておく
static DATA_CONNECTION_STATE_INSTANCE: OnceCell<
    std::sync::Mutex<HashMap<DataConnectionId, DataConnectionResponse>>,
> = OnceCell::new();
// Event処理やDisconnect時に利用するため、MediaConnection確立時に
// メディアの転送情報を集めておく
static MEDIA_CONNECTION_STATE_INSTANCE: OnceCell<
    std::sync::Mutex<HashMap<MediaConnectionId, CallResponseDto>>,
> = OnceCell::new();

pub(crate) trait CallbackFunctions: Interface {
    fn create_peer_callback(&self, peer_id: &str, token: &str);
    fn peer_deleted_callback(&self);
    fn data_callback(&self, param: &str) -> PluginLoadResult;
    fn data_connection_deleted_callback(&self, data_connection_id: &str);
}

#[derive(Component)]
#[shaku(interface = CallbackFunctions)]
pub(crate) struct CallbackFunctionsImpl {}

impl CallbackFunctions for CallbackFunctionsImpl {
    fn create_peer_callback(&self, peer_id: &str, token: &str) {
        CallbackFunctionsHolder::global().create_peer_callback(peer_id, token)
    }

    fn peer_deleted_callback(&self) {
        CallbackFunctionsHolder::global().peer_deleted_callback()
    }

    fn data_callback(&self, param: &str) -> PluginLoadResult {
        CallbackFunctionsHolder::global().data_callback(param)
    }

    fn data_connection_deleted_callback(&self, data_connection_id: &str) {
        CallbackFunctionsHolder::global().data_connection_deleted_callback(data_connection_id)
    }
}

pub(crate) trait Logger: Interface {
    fn debug(&self, message: &str);
    fn info(&self, message: &str);
    fn warn(&self, message: &str);
    fn error(&self, message: &str);
}

#[derive(Component)]
#[shaku(interface = Logger)]
pub(crate) struct LoggerImpl {}

impl Logger for LoggerImpl {
    fn debug(&self, message: &str) {
        LoggerHolder::global().debug(message)
    }

    fn info(&self, message: &str) {
        LoggerHolder::global().info(message)
    }

    fn warn(&self, message: &str) {
        LoggerHolder::global().warn(message)
    }

    fn error(&self, message: &str) {
        LoggerHolder::global().error(message)
    }
}

pub(crate) trait ProgramState: Interface {
    fn is_running(&self) -> bool;
    fn is_shutting_down(&self) -> bool;
    fn sleep_c(&self, duration: f64);
    fn wait_for_shutdown(&self);
    fn shutdown(&self);
}

#[derive(Component)]
#[shaku(interface = ProgramState)]
pub(crate) struct ProgramStateImpl {}

impl ProgramState for ProgramStateImpl {
    fn is_running(&self) -> bool {
        ProgramStateHolder::global().is_running()
    }

    fn is_shutting_down(&self) -> bool {
        ProgramStateHolder::global().is_shutting_down()
    }

    fn sleep_c(&self, duration: f64) {
        ProgramStateHolder::global().sleep_c(duration)
    }

    fn wait_for_shutdown(&self) {
        ProgramStateHolder::global().wait_for_shutdown()
    }

    fn shutdown(&self) {
        ProgramStateHolder::global().shutdown()
    }
}

pub(crate) trait Channels: Interface {
    fn sender(&self) -> &mpsc::Sender<(oneshot::Sender<String>, String)>;
    fn receiver(&self) -> &Mutex<mpsc::Receiver<String>>;
}

pub(crate) struct ChannelsImpl {
    sender: mpsc::Sender<(oneshot::Sender<String>, String)>,
    receiver: Mutex<mpsc::Receiver<String>>,
}

impl Channels for ChannelsImpl {
    fn sender(&self) -> &mpsc::Sender<(oneshot::Sender<String>, String)> {
        &self.sender
    }

    fn receiver(&self) -> &Mutex<mpsc::Receiver<String>> {
        &self.receiver
    }
}

#[cfg_attr(test, automock)]
pub(crate) trait GlobalState: Interface {
    fn channels(&self) -> &'static Arc<dyn Channels>;
    fn program_state(&self) -> &'static ProgramStateHolder;
    fn store_topic(&self, data_connection_id: DataConnectionId, response: DataConnectionResponse);
    fn find_topic(&self, data_connection_id: &DataConnectionId) -> Option<DataConnectionResponse>;
    fn remove_topic(&self, data_connection_id: &DataConnectionId);
    fn store_call_response(
        &self,
        media_connection_id: MediaConnectionId,
        response: CallResponseDto,
    );
    fn find_call_response(
        &self,
        media_connection_id: &MediaConnectionId,
    ) -> Option<CallResponseDto>;
}

#[derive(Component)]
#[shaku(interface = GlobalState)]
pub(crate) struct GlobalStateImpl {}

impl GlobalState for GlobalStateImpl {
    fn channels(&self) -> &'static Arc<dyn Channels> {
        CHANNELS.get().expect("CHANNELS is not initialized")
    }

    fn program_state(&self) -> &'static ProgramStateHolder {
        PROGRAM_STATE_INSTANCE
            .get()
            .expect("PROGRAM_STATE is not initialized")
    }

    fn store_topic(&self, data_connection_id: DataConnectionId, response: DataConnectionResponse) {
        let hash = DATA_CONNECTION_STATE_INSTANCE.get().unwrap();
        hash.lock().unwrap().insert(data_connection_id, response);
    }

    fn find_topic(&self, data_connection_id: &DataConnectionId) -> Option<DataConnectionResponse> {
        let hash = DATA_CONNECTION_STATE_INSTANCE
            .get()
            .unwrap()
            .lock()
            .unwrap();
        let item = hash.get(data_connection_id);
        item.map(|item| item.clone())
    }

    fn remove_topic(&self, data_connection_id: &DataConnectionId) {
        let mut hash = DATA_CONNECTION_STATE_INSTANCE
            .get()
            .unwrap()
            .lock()
            .unwrap();
        let _ = hash.remove(data_connection_id);
    }

    fn store_call_response(
        &self,
        media_connection_id: MediaConnectionId,
        response: CallResponseDto,
    ) {
        let hash = MEDIA_CONNECTION_STATE_INSTANCE.get().unwrap();
        hash.lock().unwrap().insert(media_connection_id, response);
    }

    fn find_call_response(
        &self,
        media_connection_id: &MediaConnectionId,
    ) -> Option<CallResponseDto> {
        let hash = MEDIA_CONNECTION_STATE_INSTANCE
            .get()
            .unwrap()
            .lock()
            .unwrap();
        let item = hash.get(media_connection_id);
        item.map(|item| item.clone())
    }
}

//========== ↑ OnceCell ↑ ==========

pub(crate) async fn rust_main() {
    let _ = DATA_CONNECTION_STATE_INSTANCE.set(std::sync::Mutex::new(HashMap::new()));
    let _ = MEDIA_CONNECTION_STATE_INSTANCE.set(std::sync::Mutex::new(HashMap::new()));

    let (sender, receiver) = skyway_webrtc_gateway_caller::run("http://localhost:8000").await;
    // SkyWay Crateにアクセスするためのsender, receiverを保持する
    // Channels objectに入れた上でOnceCellで保持する
    let channels = ChannelsImpl {
        sender,
        receiver: tokio::sync::Mutex::new(receiver),
    };
    let result = CHANNELS.set(Arc::new(channels));
    if result.is_err() {
        LoggerHolder::global().error("CHANNELS set error");
        ProgramStateHolder::global().shutdown();
    }

    // ROS Serviceからの操作を別スレッドで受け付ける。
    // ROSが終了するまで待機する
    ProgramStateHolder::global().wait_for_shutdown();
}
