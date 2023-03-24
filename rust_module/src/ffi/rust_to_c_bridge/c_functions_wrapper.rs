// 当面はユニットテストは行わず、結合試験だけ行うことにする
// Fixme: Unit Test
use std::ffi::CString;
use std::os::raw::{c_char, c_double};

use serde::{Deserialize, Serialize};

use crate::domain::entity::DataConnectionId;
use crate::ffi::rust_to_c_bridge::state_objects::{
    CALLBACK_FUNCTIONS, LOGGER_INSTANCE, PROGRAM_STATE_INSTANCE,
};

// Rust側でイベントが発生した際にC++側に通知するためのコールバック関数を保持する
#[repr(C)]
pub struct CallbackFunctionsHolder {
    create_peer_callback_c: extern "C" fn(peer_id: *mut c_char, token: *mut c_char),
    peer_deleted_callback: extern "C" fn(),
    data_callback_c: extern "C" fn(
        target_ip: *mut c_char,
        target_port: u16,
        plugin_type: *mut c_char,
        plugin_param: *mut c_char,
    ) -> PluginLoadResult,
    data_connection_deleted_callback_c: extern "C" fn(data_connection_id: u16),
    release_str_c: extern "C" fn(data_connection_id: *const c_char),
}

impl CallbackFunctionsHolder {
    pub fn new(
        create_peer_callback_c: extern "C" fn(peer_id: *mut c_char, token: *mut c_char),
        peer_deleted_callback: extern "C" fn(),
        data_callback_c: extern "C" fn(
            target_ip: *mut c_char,
            target_port: u16,
            plugin_type: *mut c_char,
            plugin_param: *mut c_char,
        ) -> PluginLoadResult,
        data_connection_deleted_callback_c: extern "C" fn(data_connection_id: u16),
        release_str_c: extern "C" fn(message: *const c_char),
    ) -> Self {
        CallbackFunctionsHolder {
            create_peer_callback_c,
            peer_deleted_callback,
            data_callback_c,
            data_connection_deleted_callback_c,
            release_str_c,
        }
    }

    pub fn global() -> &'static CallbackFunctionsHolder {
        CALLBACK_FUNCTIONS
            .get()
            .expect("functions is not initialized")
    }

    pub fn create_peer_callback(&self, peer_id: &str, token: &str) {
        (self.create_peer_callback_c)(
            CString::new(peer_id).unwrap().into_raw(),
            CString::new(token).unwrap().into_raw(),
        );
    }

    pub fn peer_deleted_callback(&self) {
        (self.peer_deleted_callback)();
    }

    pub fn data_callback(
        &self,
        target_ip: &str,
        target_port: u16,
        plugin_type: &str,
        plugin_parameter: &str,
    ) -> PluginLoadResult {
        (self.data_callback_c)(
            CString::new(target_ip).unwrap().into_raw(),
            target_port,
            CString::new(plugin_type).unwrap().into_raw(),
            CString::new(plugin_parameter).unwrap().into_raw(),
        )
    }

    pub fn data_connection_deleted_callback(&self, port_num: u16) {
        (self.data_connection_deleted_callback_c)(port_num);
    }

    pub fn release_str(&self, message: *const c_char) {
        (self.release_str_c)(message);
    }
}

// Rust側でイベントが発生した際にC++側に通知するためのコールバック関数の実体をC++側から受け取る
#[no_mangle]
pub extern "C" fn register_callbacks(param: &CallbackFunctionsHolder) {
    let functions = CallbackFunctionsHolder::new(
        param.create_peer_callback_c,
        param.peer_deleted_callback,
        param.data_callback_c,
        param.data_connection_deleted_callback_c,
        param.release_str_c,
    );

    if CALLBACK_FUNCTIONS.set(functions).is_err() {
        return;
    }
}

// ROSの機能でロギングするための関数を保持する
#[derive(Debug)]
pub struct LoggerHolder {
    debug_c: extern "C" fn(*const c_char) -> (),
    info_c: extern "C" fn(*const c_char) -> (),
    warn_c: extern "C" fn(*const c_char) -> (),
    error_c: extern "C" fn(*const c_char) -> (),
}

#[allow(dead_code)]
impl LoggerHolder {
    pub fn new(
        debug_c: extern "C" fn(*const c_char) -> (),
        info_c: extern "C" fn(*const c_char) -> (),
        warn_c: extern "C" fn(*const c_char) -> (),
        error_c: extern "C" fn(*const c_char) -> (),
    ) -> Self {
        LoggerHolder {
            debug_c,
            info_c,
            warn_c,
            error_c,
        }
    }

    pub fn global() -> &'static LoggerHolder {
        LOGGER_INSTANCE.get().expect("logger is not initialized")
    }

    pub fn is_allocated() -> bool {
        LOGGER_INSTANCE.get().is_some()
    }

    pub fn debug(&self, message: impl Into<String>) {
        let message_raw = CString::new(message.into()).unwrap().into_raw();
        (self.debug_c)(message_raw);
    }

    pub fn info(&self, message: impl Into<String>) {
        let message_raw = CString::new(message.into()).unwrap().into_raw();
        (self.info_c)(message_raw);
    }

    pub fn warn(&self, message: impl Into<String>) {
        let message_raw = CString::new(message.into()).unwrap().into_raw();
        (self.warn_c)(message_raw);
    }

    pub fn error(&self, message: impl Into<String>) {
        let message_raw = CString::new(message.into()).unwrap().into_raw();
        (self.error_c)(message_raw);
    }
}

// ROSの機能でロギングするための関数の実体を受け取るための関数
#[no_mangle]
pub extern "C" fn register_logger(
    debug_c: extern "C" fn(*const c_char),
    info_c: extern "C" fn(*const c_char),
    warn_c: extern "C" fn(*const c_char),
    error_c: extern "C" fn(*const c_char),
) {
    LOGGER_INSTANCE
        .set(LoggerHolder {
            debug_c,
            info_c,
            warn_c,
            error_c,
        })
        .unwrap();
}

// ROSの機能を制御するための関数を保持する
#[derive(Debug)]
pub struct ProgramStateHolder {
    is_running_c: extern "C" fn() -> bool,
    is_shutting_down_c: extern "C" fn() -> bool,
    sleep_c: extern "C" fn(c_double) -> (),
    wait_for_shutdown_c: extern "C" fn() -> (),
    shutdown_c: extern "C" fn() -> (),
}

#[allow(dead_code)]
impl ProgramStateHolder {
    pub fn new(
        is_running_c: extern "C" fn() -> bool,
        is_shutting_down_c: extern "C" fn() -> bool,
        sleep_c: extern "C" fn(c_double) -> (),
        wait_for_shutdown_c: extern "C" fn() -> (),
        shutdown_c: extern "C" fn() -> (),
    ) -> Self {
        ProgramStateHolder {
            is_running_c,
            is_shutting_down_c,
            sleep_c,
            wait_for_shutdown_c,
            shutdown_c,
        }
    }

    pub fn global() -> &'static ProgramStateHolder {
        PROGRAM_STATE_INSTANCE
            .get()
            .expect("ProgramState is not initialized")
    }

    pub fn is_allocated() -> bool {
        PROGRAM_STATE_INSTANCE.get().is_some()
    }

    pub fn is_running(&self) -> bool {
        (self.is_running_c)()
    }

    pub fn is_shutting_down(&self) -> bool {
        (self.is_shutting_down_c)()
    }

    pub fn sleep_c(&self, duration: f64) {
        (self.sleep_c)(duration as c_double);
    }

    pub fn wait_for_shutdown(&self) {
        (self.wait_for_shutdown_c)();
    }

    pub fn shutdown(&self) {
        (self.shutdown_c)();
    }
}

// ROSの機能を制御するための関数の実体を受け取るための関数
#[no_mangle]
pub extern "C" fn register_program_state(
    is_running_c: extern "C" fn() -> bool,
    is_shutting_down_c: extern "C" fn() -> bool,
    sleep_c: extern "C" fn(c_double) -> (),
    wait_for_shutdown_c: extern "C" fn() -> (),
    shutdown_c: extern "C" fn() -> (),
) {
    PROGRAM_STATE_INSTANCE
        .set(ProgramStateHolder {
            is_running_c,
            is_shutting_down_c,
            sleep_c,
            wait_for_shutdown_c,
            shutdown_c,
        })
        .unwrap();
}

// Pluginがロードされた場合、この構造体に格納して使用中のPluginを管理する
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct DataPipeInfo {
    pub data_connection_id: DataConnectionId,
    pub data_pipe_port_num: u16,
}

// DataChannel <-> ROS間のデータのやり取りはC++側のPluginでハンドリングする
// Pluginが正常にロードされたかどうかを返す
#[repr(C)]
pub struct PluginLoadResult {
    pub(crate) is_success: bool,
    pub(crate) port: u16,
    pub(crate) error_message: *mut c_char,
}

#[cfg(test)]
pub(crate) mod helper {
    use std::os::raw::c_double;

    pub extern "C" fn is_running() -> bool {
        true
    }

    pub extern "C" fn is_shutting_down() -> bool {
        false
    }

    pub extern "C" fn sleep(_param: c_double) {}

    pub extern "C" fn wait_for_shutdown() {}

    pub extern "C" fn shutdown() {}
}
