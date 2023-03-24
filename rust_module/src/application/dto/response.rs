use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};

use crate::domain::entity::response::{DataResponse, MediaResponse, PeerResponse};
use crate::domain::entity::{
    AnswerResult, DataConnectionId, DataConnectionIdWrapper, DataConnectionStatus, DataId,
    DataIdWrapper, MediaConnectionId, MediaConnectionIdWrapper, MediaConnectionStatus, MediaId,
    MediaIdWrapper, PeerCallEvent, PeerCloseEvent, PeerErrorEvent, PeerInfo, PeerOpenEvent,
    PeerStatusMessage, RedirectParameters, RtcpId, RtcpIdWrapper, SerializableId, SocketInfo,
};
use crate::error;

//========== System ==========
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SystemResponseDto {
    pub(crate) is_success: bool,
}

//========== Peer ==========

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PeerCallEventDto {
    /// Pair of PeerId and Token. Indicate which Peer Object is regarded.
    pub params: PeerInfo,
    /// Id to identify the DataConnection
    pub call_params: MediaConnectionIdWrapper,
    /// status of the DataConnection
    pub status: MediaConnectionStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PeerConnectionEventDto {
    /// Pair of PeerId and Token. Indicate which Peer Object is regarded.
    pub params: PeerInfo,
    /// Id to identify the DataConnection
    pub data_params: DataConnectionIdWrapper,
    /// status of the DataConnection
    pub status: DataConnectionStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "event")]
pub enum PeerEventEnumDto {
    OPEN(PeerOpenEvent),
    CLOSE(PeerCloseEvent),
    CONNECTION(PeerConnectionEventDto),
    CALL(PeerCallEventDto),
    ERROR(PeerErrorEvent),
    TIMEOUT,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "command")]
pub(crate) enum PeerResponseDto {
    #[serde(rename = "CREATE")]
    Create(PeerInfo),
    #[serde(rename = "STATUS")]
    Status(PeerStatusMessage),
    #[serde(rename = "DELETE")]
    Delete(PeerInfo),
    #[serde(rename = "EVENT")]
    Event(PeerEventEnumDto),
}

impl PeerResponseDto {
    #[allow(dead_code)]
    pub(crate) fn from_entity(entity: PeerResponse) -> Self {
        match entity {
            PeerResponse::Create(item) => PeerResponseDto::Create(item),
            PeerResponse::Delete(item) => PeerResponseDto::Delete(item),
            PeerResponse::Status(item) => PeerResponseDto::Status(item),
            PeerResponse::Event(item) => unreachable!(),
        }
    }
}

//========== Media ==========

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "event")]
pub(crate) enum MediaConnectionEventEnumDto {
    #[serde(rename = "READY")]
    Ready(CallResponseDto),
    #[serde(rename = "STREAM")]
    Stream(CallResponseDto),
    #[serde(rename = "CLOSE")]
    Close(MediaConnectionIdWrapper),
    #[serde(rename = "ERROR")]
    Error((MediaConnectionId, String)),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct CallResponseDto {
    pub send_params: SendParams,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_params: Option<RedirectParameters>,
    pub media_connection_id: MediaConnectionId,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct MediaPair<M: SerializableId, R: SerializableId> {
    pub media: SocketInfo<M>,
    pub rtcp: SocketInfo<R>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct SendParams {
    pub video: MediaPair<MediaId, RtcpId>,
    pub audio: MediaPair<MediaId, RtcpId>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "command")]
pub(crate) enum MediaResponseDto {
    #[serde(rename = "CONTENT_CREATE")]
    ContentCreate(SocketInfo<MediaId>),
    #[serde(rename = "CONTENT_DELETE")]
    ContentDelete(MediaIdWrapper),
    #[serde(rename = "RTCP_CREATE")]
    RtcpCreate(SocketInfo<RtcpId>),
    #[serde(rename = "RTCP_DELETE")]
    RtcpDelete(RtcpIdWrapper),
    #[serde(rename = "CALL")]
    Call(MediaConnectionIdWrapper),
    #[serde(rename = "ANSWER")]
    Answer(AnswerResult),
    #[serde(rename = "EVENT")]
    Event(MediaConnectionEventEnumDto),
    #[serde(rename = "DISCONNECT")]
    Disconnect(Option<()>),
    #[serde(rename = "STATUS")]
    Status(MediaConnectionStatus),
}

impl MediaResponseDto {
    #[allow(dead_code)]
    pub(crate) fn from_entity(entity: MediaResponse) -> Self {
        match entity {
            MediaResponse::ContentCreate(item) => MediaResponseDto::ContentCreate(item),
            MediaResponse::ContentDelete(item) => MediaResponseDto::ContentDelete(item),
            MediaResponse::RtcpCreate(item) => MediaResponseDto::RtcpCreate(item),
            MediaResponse::RtcpDelete(item) => MediaResponseDto::RtcpDelete(item),
            MediaResponse::Call(item) => MediaResponseDto::Call(item),
            MediaResponse::Answer(item) => MediaResponseDto::Answer(item),
            MediaResponse::Disconnect(item) => MediaResponseDto::Disconnect(item),
            MediaResponse::Event(_item) => unreachable!(),
            MediaResponse::Status(item) => MediaResponseDto::Status(item),
        }
    }
}

//========== Data ==========

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "event")]
pub(crate) enum DataConnectionEventDto {
    OPEN(DataConnectionIdWrapper),
    CLOSE(DataConnectionIdWrapper),
    ERROR((DataConnectionId, String)),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "command")]
pub(crate) enum DataResponseDto {
    #[serde(rename = "CREATE")]
    Create(SocketInfo<DataId>),
    #[serde(rename = "CONNECT")]
    Connect(DataConnectionIdWrapper),
    #[serde(rename = "DELETE")]
    Delete(DataIdWrapper),
    #[serde(rename = "DISCONNECT")]
    Disconnect(DataConnectionIdWrapper),
    #[serde(rename = "REDIRECT")]
    Redirect(DataConnectionIdWrapper),
    #[serde(rename = "EVENT")]
    Event(DataConnectionEventDto),
    #[serde(rename = "STATUS")]
    Status(DataConnectionStatus),
}

impl DataResponseDto {
    #[allow(dead_code)]
    pub(crate) fn from_entity(entity: DataResponse) -> Self {
        match entity {
            DataResponse::Create(item) => DataResponseDto::Create(item),
            DataResponse::Connect(item) => DataResponseDto::Connect(item),
            DataResponse::Delete(item) => DataResponseDto::Delete(item),
            DataResponse::Disconnect(item) => DataResponseDto::Disconnect(item),
            DataResponse::Redirect(item) => DataResponseDto::Redirect(item),
            DataResponse::Event(_) => unreachable!(),
            DataResponse::Status(item) => DataResponseDto::Status(item),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "request_type")]
pub(crate) enum ResponseDto {
    #[serde(rename = "PEER")]
    Peer(PeerResponseDto),
    #[serde(rename = "MEDIA")]
    Media(MediaResponseDto),
    #[serde(rename = "DATA")]
    Data(DataResponseDto),
    #[serde(rename = "SYSTEM")]
    System(SystemResponseDto),
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub(crate) enum ResponseDtoResult {
    Success(ResponseDto),
    Error(String),
}

#[allow(dead_code)]
impl ResponseDtoResult {
    pub(crate) fn from_str(json: &str) -> Result<ResponseDtoResult, error::Error> {
        #[allow(dead_code)]
        #[derive(Deserialize)]
        struct ResponseMessageStruct {
            is_success: bool,
            result: serde_json::Value,
        }
        let value = serde_json::from_str::<ResponseMessageStruct>(json)
            .map_err(|e| error::Error::SerdeError { error: e })?;
        match value.is_success {
            true => {
                let content: ResponseDto = serde_json::from_value(value.result)
                    .map_err(|e| error::Error::SerdeError { error: e })?;
                Ok(ResponseDtoResult::Success(content))
            }
            _ => {
                let content: String = serde_json::from_value(value.result)
                    .map_err(|e| error::Error::SerdeError { error: e })?;
                Ok(ResponseDtoResult::Error(content))
            }
        }
    }

    pub(crate) fn to_string(&self) -> Result<String, error::Error> {
        serde_json::to_string(self).map_err(|e| error::Error::SerdeError { error: e })
    }
}

#[allow(dead_code)]
impl Serialize for ResponseDtoResult {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Person", 2)?;
        match self {
            ResponseDtoResult::Success(value) => {
                state.serialize_field("is_success", &true)?;
                state.serialize_field("result", &value)?;
            }
            ResponseDtoResult::Error(value) => {
                state.serialize_field("is_success", &false)?;
                state.serialize_field("result", &value)?;
            }
        }
        state.end()
    }
}
