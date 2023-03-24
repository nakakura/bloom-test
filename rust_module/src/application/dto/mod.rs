pub(crate) mod request;
pub(crate) mod response;

use crate::application::dto::request::{DataRequestDto, MediaRequestDto, RequestDto};
use crate::application::dto::response::{
    DataResponseDto, MediaResponseDto, PeerResponseDto, ResponseDto, ResponseDtoResult,
};
use crate::domain::entity::request::{DataRequest, MediaRequest, Request};
use crate::domain::entity::response::{
    DataResponse, MediaResponse, PeerResponse, Response, ResponseResult,
};
use crate::domain::entity::ConnectQuery;
use crate::error;

pub(crate) trait Command {
    fn command(&self) -> String;
}

/// Dto objectからDomain objectへの変換
pub(crate) fn dto_to_request(dto: RequestDto) -> Result<Request, error::Error> {
    match dto {
        RequestDto::Peer(parameter) => Ok(Request::Peer(parameter)),
        RequestDto::Data(DataRequestDto::Create) => {
            Ok(Request::Data(DataRequest::Create { params: true }))
        }
        RequestDto::Data(DataRequestDto::Connect { params }) => {
            let query = ConnectQuery {
                peer_id: params.peer_id,
                token: params.token,
                options: params.options,
                target_id: params.target_id,
                params: params.params,
                redirect_params: params.redirect_params,
            };
            Ok(Request::Data(DataRequest::Connect { params: query }))
        }
        RequestDto::Data(DataRequestDto::Delete { params }) => {
            Ok(Request::Data(DataRequest::Delete { params }))
        }
        RequestDto::Data(DataRequestDto::Disconnect { params }) => {
            Ok(Request::Data(DataRequest::Disconnect { params }))
        }
        RequestDto::Data(DataRequestDto::Status { params }) => {
            Ok(Request::Data(DataRequest::Status { params }))
        }
        RequestDto::Media(MediaRequestDto::ContentCreate { params }) => {
            Ok(Request::Media(MediaRequest::ContentCreate { params }))
        }
        RequestDto::Media(MediaRequestDto::ContentDelete { params }) => {
            Ok(Request::Media(MediaRequest::ContentDelete { params }))
        }
        RequestDto::Media(MediaRequestDto::RtcpCreate { params }) => {
            Ok(Request::Media(MediaRequest::RtcpCreate { params }))
        }
        RequestDto::Media(MediaRequestDto::RtcpDelete { params }) => {
            Ok(Request::Media(MediaRequest::RtcpDelete { params }))
        }
        RequestDto::Media(MediaRequestDto::Status { params }) => {
            Ok(Request::Media(MediaRequest::Status { params }))
        }
        RequestDto::Media(MediaRequestDto::Disconnect { params }) => {
            Ok(Request::Media(MediaRequest::Disconnect { params }))
        }
        param => {
            let message = format!("invalid parameter for GeneralService {:?}", param);
            Err(error::Error::create_local_error(&message))
        }
    }
}

/// Domain objectからDto objectへの変換
pub(crate) fn result_to_dto(response: ResponseResult) -> Result<ResponseDtoResult, error::Error> {
    match response {
        ResponseResult::Success(Response::Peer(PeerResponse::Create(params))) => Ok(
            ResponseDtoResult::Success(ResponseDto::Peer(PeerResponseDto::Create(params))),
        ),
        ResponseResult::Success(Response::Peer(PeerResponse::Delete(params))) => Ok(
            ResponseDtoResult::Success(ResponseDto::Peer(PeerResponseDto::Delete(params))),
        ),
        ResponseResult::Success(Response::Peer(PeerResponse::Status(params))) => Ok(
            ResponseDtoResult::Success(ResponseDto::Peer(PeerResponseDto::Status(params))),
        ),
        ResponseResult::Success(Response::Data(DataResponse::Create(params))) => Ok(
            ResponseDtoResult::Success(ResponseDto::Data(DataResponseDto::Create(params))),
        ),
        ResponseResult::Success(Response::Data(DataResponse::Delete(params))) => Ok(
            ResponseDtoResult::Success(ResponseDto::Data(DataResponseDto::Delete(params))),
        ),
        ResponseResult::Success(Response::Data(DataResponse::Disconnect(params))) => Ok(
            ResponseDtoResult::Success(ResponseDto::Data(DataResponseDto::Disconnect(params))),
        ),
        ResponseResult::Success(Response::Data(DataResponse::Status(params))) => Ok(
            ResponseDtoResult::Success(ResponseDto::Data(DataResponseDto::Status(params))),
        ),
        ResponseResult::Success(Response::Media(MediaResponse::ContentCreate(params))) => Ok(
            ResponseDtoResult::Success(ResponseDto::Media(MediaResponseDto::ContentCreate(params))),
        ),
        ResponseResult::Success(Response::Media(MediaResponse::ContentDelete(params))) => Ok(
            ResponseDtoResult::Success(ResponseDto::Media(MediaResponseDto::ContentDelete(params))),
        ),
        ResponseResult::Success(Response::Media(MediaResponse::RtcpCreate(params))) => Ok(
            ResponseDtoResult::Success(ResponseDto::Media(MediaResponseDto::RtcpCreate(params))),
        ),
        ResponseResult::Success(Response::Media(MediaResponse::RtcpDelete(params))) => Ok(
            ResponseDtoResult::Success(ResponseDto::Media(MediaResponseDto::RtcpDelete(params))),
        ),
        ResponseResult::Success(Response::Media(MediaResponse::Status(params))) => Ok(
            ResponseDtoResult::Success(ResponseDto::Media(MediaResponseDto::Status(params))),
        ),
        ResponseResult::Success(Response::Media(MediaResponse::Disconnect(params))) => Ok(
            ResponseDtoResult::Success(ResponseDto::Media(MediaResponseDto::Disconnect(params))),
        ),
        param => {
            let message = format!("invalid response for GeneralService {:?}", param);
            Err(error::Error::create_local_error(&message))
        }
    }
}
