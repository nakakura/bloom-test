/// 終了命令など、WebRTC Gateway自体の操作に関係ない指示がClientから来たときに呼ばれる
use std::thread::sleep;
use std::time::Duration;

use async_trait::async_trait;
use shaku::*;

use crate::application::dto::response::ResponseDto;
use crate::application::dto::response::SystemResponseDto;
use crate::application::usecase::ResponseDtoResult;
use crate::application::usecase::Service;
use crate::application::RequestDto;
use crate::di::CppObjctsModule;
use crate::error;
use crate::ffi::rust_to_c_bridge::state_objects::ProgramState;

#[derive(Component)]
#[shaku(interface = Service)]
pub(crate) struct System {}

#[async_trait]
impl Service for System {
    async fn execute(&self, request: RequestDto) -> Result<ResponseDtoResult, error::Error> {
        if let RequestDto::System(_system) = request {
            std::thread::spawn(|| {
                sleep(Duration::from_millis(100));
                let module = CppObjctsModule::builder().build();
                let state: &dyn ProgramState = module.resolve_ref();
                state.shutdown();
            });
            return Ok(ResponseDtoResult::Success(ResponseDto::System(
                SystemResponseDto { is_success: true },
            )));
        }

        return Err(error::Error::create_local_error("invalid parameters"));
    }
}
