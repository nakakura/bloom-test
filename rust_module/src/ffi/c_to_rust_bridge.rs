// 当面はユニットテストは行わず、結合試験だけ行うことにする
// Fixme: Unit Test
use std::ffi::{c_void, CStr, CString};
use std::os::raw::c_char;
use std::thread::JoinHandle;

use shaku::HasComponent;

use crate::application::dto::request::RequestDto;
use crate::application::usecase::Service;
use crate::di::GeneralService;
use crate::domain::entity::request::PeerRequest;
use crate::domain::entity::PeerInfo;
use crate::ffi::rust_to_c_bridge::c_functions_wrapper::*;

//========== 起動時用 ==========
// 起動に成功した場合、Rust側でWebRTC Gateawyから生じるイベントのリスナースレッドが回り続ける
// 終了時にそれを終了するため、起動に成功したというフラグとともにhandlerを一緒に返す
#[repr(C)]
pub struct RunResponse {
    flag: bool,
    handler: *mut c_void,
}

#[no_mangle]
pub extern "C" fn run() -> RunResponse {
    if !LoggerHolder::is_allocated() {
        return RunResponse {
            flag: false,
            handler: std::ptr::null_mut(),
        };
    }

    if !ProgramStateHolder::is_allocated() {
        LoggerHolder::global().error(
            "ProgramState object is not allocated. Please call the register_program_state function",
        );
        return RunResponse {
            flag: false,
            handler: std::ptr::null_mut(),
        };
    }

    // SkyWay Crateを開始する
    let handle: JoinHandle<()> = std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            crate::rust_main().await;
        });
    });

    let thread_handle = Box::into_raw(Box::new(handle)) as *mut c_void;

    return RunResponse {
        flag: true,
        handler: thread_handle,
    };
}

//========== ROS側から、WebRTC Gatewayの操作のために呼ばれる関数 ==========
#[no_mangle]
pub extern "C" fn call_service(message_char: *const c_char) -> *mut c_char {
    // C文字列とRust文字列の変換だけ行って、中身の処理はapplicationメソッドに任せる
    let rt = tokio::runtime::Runtime::new().unwrap();
    let message: String = rt.block_on(async {
        let c_str: &CStr = unsafe { CStr::from_ptr(message_char) };
        let message = c_str.to_str().unwrap().to_string();

        crate::application::call_service(message).await
    });
    return CString::new(message.as_str()).unwrap().into_raw();
}

#[no_mangle]
pub extern "C" fn receive_events() -> *mut c_char {
    // C文字列とRust文字列の変換だけ行って、中身の処理はapplicationメソッドに任せる
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async { crate::application::receive_events().await });
    return CString::new(result).unwrap().into_raw();
}

//========== 開放処理 ==========
// ros終了時にC++側から呼ばれる
// Rust側オブジェクトの開放処理と、WebRTC Gateway上のオブジェクトの開放処理を行う
#[no_mangle]
pub extern "C" fn shutdown_service(peer_id: *const c_char, token: *const c_char) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let c_str: &CStr = unsafe { CStr::from_ptr(peer_id) };
        let peer_id = c_str.to_str().unwrap().to_string();

        let c_str: &CStr = unsafe { CStr::from_ptr(token) };
        let token = c_str.to_str().unwrap().to_string();

        let param = RequestDto::Peer(PeerRequest::Delete {
            params: PeerInfo::try_create(peer_id, token).expect("peer_info is invalid"),
        });

        let module = GeneralService::builder().build();
        let service: &dyn Service = module.resolve_ref();

        if let Err(e) = service.execute(param).await {
            let error_message = format!("peer close error: {:?}", e);
            LoggerHolder::global().error(error_message);
        }

        CallbackFunctionsHolder::global().peer_deleted_callback();
    });
}

// C++側のプログラム終了時に、Rust側が全て開放されるまで待機するために呼ばれる関数
#[no_mangle]
pub extern "C" fn join_handler(handler: *mut c_void) {
    let handle = unsafe { Box::from_raw(handler as *mut JoinHandle<()>) };
    let _ = handle.join();
}

// Rust側で生成した文字列はRust側で開放するため、C++側から文字列を返す
#[no_mangle]
pub extern "C" fn release_string(message: *mut c_char) {
    unsafe {
        let _ = CString::from_raw(message);
    }
}

#[no_mangle]
pub extern "C" fn print_string(message: *const c_char) {
    let str = unsafe { CStr::from_ptr(message) }.to_str().unwrap();
    println!("{}", str);
    CallbackFunctionsHolder::global().release_str(message);
}
