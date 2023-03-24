#[allow(unused_imports)]
pub(crate) use skyway_webrtc_gateway_caller::prelude::response_parser::ResponseMessage as Response;
#[allow(unused_imports)]
pub(crate) use skyway_webrtc_gateway_caller::prelude::response_parser::{
    DataResponse, MediaResponse, PeerResponse, ResponseResult,
};

use crate::domain::entity::{FromStr, Stringify};
use crate::error;

#[allow(dead_code)]
impl Stringify for ResponseResult {
    fn to_string(&self) -> Result<String, error::Error> {
        return serde_json::to_string(self).map_err(|e| error::Error::SerdeError { error: e });
    }
}

#[allow(dead_code)]
impl FromStr for ResponseResult {
    fn from_str(raw_message: &str) -> Result<Self, error::Error> {
        serde_json::from_str(raw_message).map_err(|e| error::Error::SerdeError { error: e })
    }
}
