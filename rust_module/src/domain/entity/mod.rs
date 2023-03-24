pub(crate) mod request;
pub(crate) mod response;

use crate::error;

// WebRTC Crateの中のこれらのオブジェクトをentityとして再定義
// module内のオブジェクトは、この場所と、crate::errorでError定義を参照するのみで、
// その他の場所からは直接参照させない
#[allow(unused_imports)]
pub(crate) use skyway_webrtc_gateway_caller::prelude::common::*;
#[allow(unused_imports)]
pub(crate) use skyway_webrtc_gateway_caller::prelude::data::*;
#[allow(unused_imports)]
pub(crate) use skyway_webrtc_gateway_caller::prelude::media::*;
#[allow(unused_imports)]
pub(crate) use skyway_webrtc_gateway_caller::prelude::peer::*;

// メッセージを自然にStringに変換できるようにする
pub(crate) trait Stringify {
    fn to_string(&self) -> Result<String, error::Error>;
}

// 自然にStrからメッセージに変換できるようにする
pub(crate) trait FromStr: Sized {
    fn from_str(raw_message: &str) -> Result<Self, error::Error>;
}
