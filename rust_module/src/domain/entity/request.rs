use serde::{Deserialize, Serialize};

use super::{
    AnswerQuery, ConnectQuery, CreatePeerParams, DataConnectionIdWrapper, DataIdWrapper, FromStr,
    MediaConnectionId, MediaIdWrapper, PeerInfo, RedirectParams, Stringify,
};
use crate::domain::entity::{CallQuery, MediaConnectionIdWrapper, RtcpIdWrapper};
use crate::error;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "command")]
pub(crate) enum PeerRequest {
    #[serde(rename = "CREATE")]
    Create { params: CreatePeerParams },
    #[serde(rename = "STATUS")]
    Status { params: PeerInfo },
    #[serde(rename = "DELETE")]
    Delete { params: PeerInfo },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "command")]
pub(crate) enum DataRequest {
    #[serde(rename = "CREATE")]
    Create { params: bool },
    #[serde(rename = "DELETE")]
    Delete { params: DataIdWrapper },
    #[serde(rename = "CONNECT")]
    Connect { params: ConnectQuery },
    #[serde(rename = "REDIRECT")]
    Redirect { params: RedirectParams },
    #[serde(rename = "DISCONNECT")]
    Disconnect { params: DataConnectionIdWrapper },
    #[serde(rename = "STATUS")]
    Status { params: DataConnectionIdWrapper },
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub(crate) struct IsVideo {
    pub(crate) is_video: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub(crate) struct AnswerParameters {
    pub media_connection_id: MediaConnectionId,
    pub answer_query: AnswerQuery,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "command")]
pub(crate) enum MediaRequest {
    #[serde(rename = "CONTENT_CREATE")]
    ContentCreate { params: IsVideo },
    #[serde(rename = "CONTENT_DELETE")]
    ContentDelete { params: MediaIdWrapper },
    #[serde(rename = "RTCP_CREATE")]
    RtcpCreate { params: Option<()> },
    #[serde(rename = "RTCP_DELETE")]
    RtcpDelete { params: RtcpIdWrapper },
    #[serde(rename = "CALL")]
    Call { params: CallQuery },
    #[serde(rename = "ANSWER")]
    Answer { params: AnswerParameters },
    #[serde(rename = "STATUS")]
    Status { params: MediaConnectionIdWrapper },
    #[serde(rename = "DISCONNECT")]
    Disconnect { params: MediaConnectionIdWrapper },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")]
pub(crate) enum Request {
    #[serde(rename = "PEER")]
    Peer(PeerRequest),
    #[serde(rename = "MEDIA")]
    Media(MediaRequest),
    #[serde(rename = "DATA")]
    Data(DataRequest),
}

#[allow(dead_code)]
impl Stringify for Request {
    fn to_string(&self) -> Result<String, error::Error> {
        return serde_json::to_string(self).map_err(|e| error::Error::SerdeError { error: e });
    }
}

#[allow(dead_code)]
impl FromStr for Request {
    fn from_str(raw_message: &str) -> Result<Self, error::Error> {
        serde_json::from_str(raw_message).map_err(|e| error::Error::SerdeError { error: e })
    }
}
