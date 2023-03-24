pub(crate) use crate::domain::entity::request::PeerRequest as PeerRequestDto;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::application::dto::Command;
use crate::domain::entity::request::IsVideo;
use crate::domain::entity::{
    ConnectQueryOption, DataConnectionId, DataConnectionIdWrapper, DataIdWrapper,
    MediaConnectionId, MediaConnectionIdWrapper, MediaIdWrapper, PeerId, PhantomId,
    RedirectParameters, RtcpIdWrapper, SocketInfo, Token,
};
use crate::error;

//========== System ==========

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct SystemRequestDto {
    pub(crate) command: String,
}

//========== Peer ==========
impl Command for PeerRequestDto {
    fn command(&self) -> String {
        match self {
            PeerRequestDto::Create { params: ref _p } => "CREATE".to_string(),
            PeerRequestDto::Delete { params: ref _p } => "DELETE".to_string(),
            PeerRequestDto::Status { params: ref _p } => "STATUS".to_string(),
        }
    }
}

//========== Media ==========

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MediaParamsDto {
    /// band width between Peers
    pub band_width: usize,
    /// Codec which caller side want to use. Video: `"H264"` or `"VP8"`, Audio: `"OPUS"` or `"G711"`. It will be used in SDP.
    pub codec: String,
    /// Payload type which caller side want to use. It will be used in SDP.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload_type: Option<u16>,
    /// Sampling rate which media uses
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling_rate: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[allow(non_snake_case)]
pub struct ConstraintsDto {
    /// Parameters for sending video
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_params: Option<MediaParamsDto>,
    /// Parameters for sending audio
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_params: Option<MediaParamsDto>,
    /// metadata sent to a neighbour.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CallQueryDto {
    /// to identify which PeerObject calls to neighbour
    pub peer_id: PeerId,
    /// to show that this program has permission to control PeerObject
    pub token: Token,
    /// connect to the neighbour which has this PeerId
    pub target_id: PeerId,
    /// Parameters for MediaConnection
    /// It contains source socket. If the field is None, this MediaConnection works as RecvOnly.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints: Option<ConstraintsDto>,
    /// Shows destination socket to which received data is redirected
    /// If this field is not set, DataConnection works as SendOnly.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_params: Option<RedirectParameters>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AnswerQueryDto {
    /// Parameters for MediaConnection
    /// It contains source socket. If the field is None, this MediaConnection works as RecvOnly.
    pub constraints: ConstraintsDto,
    /// Shows destiation socket to which received data is redirected
    /// If this field is not set, DataConnection works as SendOnly.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_params: Option<RedirectParameters>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub(crate) struct AnswerParametersDto {
    pub media_connection_id: MediaConnectionId,
    pub answer_query: AnswerQueryDto,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "command")]
pub(crate) enum MediaRequestDto {
    #[serde(rename = "CONTENT_CREATE")]
    ContentCreate { params: IsVideo },
    #[serde(rename = "CONTENT_DELETE")]
    ContentDelete { params: MediaIdWrapper },
    #[serde(rename = "RTCP_CREATE")]
    RtcpCreate { params: Option<()> },
    #[serde(rename = "RTCP_CREATE")]
    RtcpDelete { params: RtcpIdWrapper },
    #[serde(rename = "CALL")]
    Call { params: CallQueryDto },
    #[serde(rename = "STATUS")]
    Status { params: MediaConnectionIdWrapper },
    #[serde(rename = "ANSWER")]
    Answer { params: AnswerParametersDto },
    #[serde(rename = "DISCONNECT")]
    Disconnect { params: MediaConnectionIdWrapper },
}

impl Command for MediaRequestDto {
    fn command(&self) -> String {
        match self {
            MediaRequestDto::ContentCreate { .. } => "CONTENT_CREATE".to_string(),
            MediaRequestDto::ContentDelete { .. } => "CONTENT_DELETE".to_string(),
            MediaRequestDto::RtcpCreate { .. } => "RTCP_CREATE".to_string(),
            MediaRequestDto::RtcpDelete { .. } => "RTCP_DELETE".to_string(),
            MediaRequestDto::Call { .. } => "CALL".to_string(),
            MediaRequestDto::Status { .. } => "STATUS".to_string(),
            MediaRequestDto::Answer { .. } => "ANSWER".to_string(),
            MediaRequestDto::Disconnect { .. } => "DISCONNECT".to_string(),
        }
    }
}

//========== Data ==========
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct ConnectDtoParams {
    pub peer_id: PeerId,
    pub token: Token,
    pub target_id: PeerId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ConnectQueryOption>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<DataIdWrapper>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_params: Option<SocketInfo<PhantomId>>,
    pub plugin_info: PluginInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct PluginInfo {
    pub r#type: String,
    pub plugins: Vec<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct RedirectDtoParams {
    pub data_connection_id: DataConnectionId,
    pub plugin_info: PluginInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "command")]
pub(crate) enum DataRequestDto {
    #[serde(rename = "CREATE")]
    Create,
    #[serde(rename = "DELETE")]
    Delete { params: DataIdWrapper },
    #[serde(rename = "CONNECT")]
    Connect { params: ConnectDtoParams },
    #[serde(rename = "REDIRECT")]
    Redirect { params: RedirectDtoParams },
    #[serde(rename = "DISCONNECT")]
    Disconnect { params: DataConnectionIdWrapper },
    #[serde(rename = "STATUS")]
    Status { params: DataConnectionIdWrapper },
}

impl Command for DataRequestDto {
    fn command(&self) -> String {
        match self {
            DataRequestDto::Create => "CREATE".to_string(),
            DataRequestDto::Delete { .. } => "DELETE".to_string(),
            DataRequestDto::Connect { .. } => "CONNECT".to_string(),
            DataRequestDto::Redirect { .. } => "REDIRECT".to_string(),
            DataRequestDto::Disconnect { .. } => "DISCONNECT".to_string(),
            DataRequestDto::Status { .. } => "STATUS".to_string(),
        }
    }
}

//========== General ==========
// JSONでクライアントから受け取るメッセージ
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "request_type")]
pub(crate) enum RequestDto {
    #[serde(rename = "PEER")]
    Peer(PeerRequestDto),
    #[serde(rename = "DATA")]
    Data(DataRequestDto),
    #[serde(rename = "MEDIA")]
    Media(MediaRequestDto),
    #[serde(rename = "SYSTEM")]
    System(SystemRequestDto),
    #[cfg(test)]
    Test,
}

impl RequestDto {
    pub fn from_str(json: &str) -> Result<Self, error::Error> {
        serde_json::from_str::<RequestDto>(json).map_err(|e| error::Error::SerdeError { error: e })
    }

    pub fn dto_type(&self) -> String {
        match self {
            RequestDto::Peer(ref _p) => "PEER".to_string(),
            RequestDto::Data(ref _d) => "DATA".to_string(),
            RequestDto::Media(ref _m) => "MEDIA".to_string(),
            RequestDto::System(ref _m) => "SYSTEM".to_string(),
            #[cfg(test)]
            _ => "TEST".to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn to_string(&self) -> Result<String, error::Error> {
        serde_json::to_string(self).map_err(|e| error::Error::SerdeError { error: e })
    }
}

impl Command for RequestDto {
    fn command(&self) -> String {
        match self {
            RequestDto::Peer(ref peer) => peer.command(),
            RequestDto::Data(ref data) => data.command(),
            RequestDto::Media(ref media) => media.command(),
            RequestDto::System(_) => "SYSTEM".to_string(),
            #[cfg(test)]
            RequestDto::Test => {
                unreachable!()
            }
        }
    }
}
