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
use crate::ffi::rust_to_c_bridge::c_functions_wrapper::*;

use crate::application::dto::response::CallResponseDto;
use ffi::rust_to_c_bridge::state_objects::{
    ChannelsImpl, CHANNELS, DATA_CONNECTION_STATE_INSTANCE, MEDIA_CONNECTION_STATE_INSTANCE,
};
#[cfg(test)]
use mockall::automock;

pub(crate) async fn rust_main() {
    let _ = DATA_CONNECTION_STATE_INSTANCE.set(std::sync::Mutex::new(HashMap::new()));
    let _ = MEDIA_CONNECTION_STATE_INSTANCE.set(std::sync::Mutex::new(HashMap::new()));

    let (sender, receiver) = skyway_webrtc_gateway_caller::run("http://localhost:8000").await;
    // SkyWay Crateにアクセスするためのsender, receiverを保持する
    // Channels objectに入れた上でOnceCellで保持する
    let channels = ChannelsImpl::new(sender, tokio::sync::Mutex::new(receiver));
    let result = CHANNELS.set(Arc::new(channels));
    if result.is_err() {
        LoggerHolder::global().error("CHANNELS set error");
        ProgramStateHolder::global().shutdown();
    }

    // ROS Serviceからの操作を別スレッドで受け付ける。
    // ROSが終了するまで待機する
    ProgramStateHolder::global().wait_for_shutdown();
}
