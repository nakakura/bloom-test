// このサービスでは、End-User-Programの指示を受けて、MediaConnectionの確立要求を行う
// 責務は以下の通りである
// 1. GWにMedia Portを開放させる。これはVideo, Audioともに行う
// 2. CALL APIをコールし、MediaConnectionの確立を開始する
//
// WebRTC GWの仕様により、確立は受信側でAnswerが行われたタイミングである。
// このサービスではあくまで確立要求のみを行う。
// 実際にMediaConnectionが確立されたかどうか知るために、End-User-ProgramはCONNECT Eventを監視する必要がある

use std::sync::Arc;

use async_trait::async_trait;
use shaku::Component;

use crate::application::dto::request::{ConstraintsDto, MediaRequestDto, RequestDto};
use crate::application::dto::response::{
    CallResponseDto, MediaPair, MediaResponseDto, ResponseDto, ResponseDtoResult, SendParams,
};
use crate::application::factory::Factory;
use crate::application::usecase::Service;
use crate::domain::entity::request::{IsVideo, MediaRequest, Request};
use crate::domain::entity::response::{MediaResponse, Response, ResponseResult};
use crate::domain::entity::{
    CallQuery, Constraints, MediaId, MediaParams, RedirectParameters, RtcpId, SerializableSocket,
};
use crate::domain::repository::Repository;
use crate::error;
use crate::ffi::rust_to_c_bridge::state_objects::GlobalState;

#[derive(Component)]
#[shaku(interface = Service)]
pub(crate) struct Call {
    #[shaku(inject)]
    repository: Arc<dyn Repository>,
    #[shaku(inject)]
    state: Arc<dyn GlobalState>,
    #[shaku(inject)]
    factory: Arc<dyn Factory>,
}

#[async_trait]
impl Service for Call {
    async fn execute(&self, request: RequestDto) -> Result<ResponseDtoResult, error::Error> {
        if let RequestDto::Media(MediaRequestDto::Call { params }) = request {
            let video_socket = {
                let param = RequestDto::Media(MediaRequestDto::ContentCreate {
                    params: IsVideo { is_video: true },
                });
                let service = self.factory.create_service(&param);
                let result = service.execute(param).await?;
                if let ResponseDtoResult::Success(ResponseDto::Media(
                    MediaResponseDto::ContentCreate(socket),
                )) = result
                {
                    socket
                } else {
                    let message = format!("create data failed {:?}", result);
                    return Err(error::Error::create_local_error(&message));
                }
            };

            let video_rtcp_socket = {
                let param = RequestDto::Media(MediaRequestDto::RtcpCreate { params: None });
                let service = self.factory.create_service(&param);
                let result = service.execute(param).await?;
                if let ResponseDtoResult::Success(ResponseDto::Media(
                    MediaResponseDto::RtcpCreate(socket),
                )) = result
                {
                    socket
                } else {
                    let message = format!("create data failed {:?}", result);
                    return Err(error::Error::create_local_error(&message));
                }
            };

            let audio_socket = {
                let param = RequestDto::Media(MediaRequestDto::ContentCreate {
                    params: IsVideo { is_video: false },
                });
                let service = self.factory.create_service(&param);
                let result = service.execute(param).await?;
                if let ResponseDtoResult::Success(ResponseDto::Media(
                    MediaResponseDto::ContentCreate(socket),
                )) = result
                {
                    socket
                } else {
                    let message = format!("create data failed {:?}", result);
                    return Err(error::Error::create_local_error(&message));
                }
            };

            let audio_rtcp_socket = {
                let param = RequestDto::Media(MediaRequestDto::RtcpCreate { params: None });
                let service = self.factory.create_service(&param);
                let result = service.execute(param).await?;
                if let ResponseDtoResult::Success(ResponseDto::Media(
                    MediaResponseDto::RtcpCreate(socket),
                )) = result
                {
                    socket
                } else {
                    let message = format!("create data failed {:?}", result);
                    return Err(error::Error::create_local_error(&message));
                }
            };

            // Readyイベントでユーザに返すために保持
            let send_params = SendParams {
                video: MediaPair {
                    media: video_socket.clone(),
                    rtcp: video_rtcp_socket.clone(),
                },
                audio: MediaPair {
                    media: audio_socket.clone(),
                    rtcp: audio_rtcp_socket.clone(),
                },
            };
            let redirect_params = params.redirect_params.clone();
            let constraints = create_constraint(
                video_socket.get_id().unwrap(),
                video_rtcp_socket.get_id().unwrap(),
                audio_socket.get_id().unwrap(),
                audio_rtcp_socket.get_id().unwrap(),
                &params.constraints,
                &params.redirect_params,
            );

            let params = CallQuery {
                peer_id: params.peer_id,
                token: params.token,
                target_id: params.target_id,
                constraints: Some(constraints),
                redirect_params: params.redirect_params,
            };
            let request = Request::Media(MediaRequest::Call { params });
            let result = self.repository.register(request).await?;
            match result {
                ResponseResult::Success(Response::Media(MediaResponse::Call(call_result))) => {
                    let call_response = CallResponseDto {
                        send_params,
                        redirect_params,
                        media_connection_id: call_result.media_connection_id.clone(),
                    };
                    self.state.store_call_response(
                        call_response.media_connection_id.clone(),
                        call_response,
                    );

                    return Ok(ResponseDtoResult::Success(ResponseDto::Media(
                        MediaResponseDto::Call(call_result),
                    )));
                }
                ResponseResult::Error(message) => return Ok(ResponseDtoResult::Error(message)),
                _ => {
                    unreachable!()
                }
            }
        }

        return Err(error::Error::create_local_error(
            "invalid message in call service",
        ));
    }
}

pub(crate) fn create_constraint(
    video_id: MediaId,
    video_rtcp_id: RtcpId,
    audio_id: MediaId,
    audio_rtcp_id: RtcpId,
    constraint_dto: &Option<ConstraintsDto>,
    redirect_params: &Option<RedirectParameters>,
) -> Constraints {
    let video_receive_enabled = if let Some(RedirectParameters {
        video: Some(ref _video),
        ..
    }) = redirect_params
    {
        Some(true)
    } else {
        None
    };

    let audio_receive_enabled = if let Some(RedirectParameters {
        audio: Some(ref _audio),
        ..
    }) = redirect_params
    {
        Some(true)
    } else {
        None
    };

    let video_params = if let Some(ConstraintsDto {
        video_params: Some(ref params),
        ..
    }) = constraint_dto
    {
        Some(MediaParams {
            band_width: params.band_width,
            codec: params.codec.clone(),
            media_id: video_id,
            rtcp_id: Some(video_rtcp_id),
            payload_type: params.payload_type,
            sampling_rate: params.sampling_rate,
        })
    } else {
        None
    };

    let audio_params = if let Some(ConstraintsDto {
        audio_params: Some(ref params),
        ..
    }) = constraint_dto
    {
        Some(MediaParams {
            band_width: params.band_width,
            codec: params.codec.clone(),
            media_id: audio_id,
            rtcp_id: Some(audio_rtcp_id),
            payload_type: params.payload_type,
            sampling_rate: params.sampling_rate,
        })
    } else {
        None
    };

    let metadata = if let Some(ConstraintsDto {
        metadata: Some(ref metadata),
        ..
    }) = constraint_dto
    {
        Some(metadata.clone())
    } else {
        None
    };

    Constraints {
        video: true,
        videoReceiveEnabled: video_receive_enabled,
        audio: true,
        audioReceiveEnabled: audio_receive_enabled,
        video_params,
        audio_params,
        metadata: metadata,
    }
}

#[cfg(test)]
mod call_media_test {
    use shaku::HasComponent;

    use super::*;
    use crate::application::dto::request::{CallQueryDto, MediaRequestDto};
    use crate::application::dto::response::CallResponseDto;
    use crate::application::factory::MockFactory;
    use crate::application::usecase::MockService;
    use crate::di::MediaCallService;
    use crate::domain::entity::request::{MediaRequest, Request};
    use crate::domain::entity::response::ResponseResult;
    use crate::domain::entity::{
        MediaConnectionId, MediaConnectionIdWrapper, PeerId, SocketInfo, Token,
    };
    use crate::domain::repository::MockRepository;
    use crate::ffi::rust_to_c_bridge::state_objects::MockGlobalState;

    #[tokio::test]
    async fn success() {
        // 正解データの生成
        let dto = {
            //CallResponseDtoはVideo, Audio, MediaConnectionIdの情報が必要なので生成する
            let send = {
                let video = {
                    let media = SocketInfo::<MediaId>::try_create(
                        Some("vi-4d053831-5dc2-461b-a358-d062d6115216".to_string()),
                        "127.0.0.1",
                        10000,
                    )
                    .unwrap();
                    let rtcp = SocketInfo::<RtcpId>::try_create(
                        Some("rc-4d053831-5dc2-461b-a358-d062d6115216".to_string()),
                        "127.0.0.1",
                        10001,
                    )
                    .unwrap();
                    MediaPair { media, rtcp }
                };

                let audio = {
                    let media = SocketInfo::<MediaId>::try_create(
                        Some("au-4d053831-5dc2-461b-a358-d062d6115216".to_string()),
                        "127.0.0.1",
                        10010,
                    )
                    .unwrap();
                    let rtcp = SocketInfo::<RtcpId>::try_create(
                        Some("rc-5d053831-5dc2-461b-a358-d062d6115216".to_string()),
                        "127.0.0.1",
                        10011,
                    )
                    .unwrap();
                    MediaPair { media, rtcp }
                };

                SendParams { video, audio }
            };

            let media_connection_id =
                MediaConnectionId::try_create("mc-102127d9-30de-413b-93f7-41a33e39d82b").unwrap();

            CallResponseDto {
                send_params: send,
                redirect_params: None,
                media_connection_id,
            }
        };

        // repositoryのMockを生成
        // callに成功するケース
        let mut repository = MockRepository::new();
        repository
            .expect_register()
            // create dataで失敗した場合は1回しか呼ばれない
            .returning(move |request| match request {
                Request::Media(MediaRequest::Call { params: _ }) => {
                    let call = ResponseResult::Success(Response::Media(MediaResponse::Call(
                        MediaConnectionIdWrapper {
                            media_connection_id: MediaConnectionId::try_create(
                                "mc-102127d9-30de-413b-93f7-41a33e39d82b",
                            )
                            .unwrap(),
                        },
                    )));
                    Ok(call)
                }
                _ => todo!(),
            });

        let params = CallQueryDto {
            peer_id: PeerId::new("peer_id"),
            token: Token::try_create("pt-06cf1d26-0ef0-4b03-aca6-933027d434c2").unwrap(),
            target_id: PeerId::new("target_id"),
            constraints: None,
            redirect_params: None,
        };

        let mut state = MockGlobalState::new();
        state
            .expect_store_call_response()
            .times(1)
            .returning(|_, _| ());

        let mut factory = MockFactory::new();
        factory.expect_create_service().times(4).returning(|_| {
            let mut mock_service = MockService::new();
            mock_service
                .expect_execute()
                .returning(|request| match request {
                    RequestDto::Media(MediaRequestDto::ContentCreate { params }) => {
                        if params.is_video {
                            let socket = SocketInfo::<MediaId>::try_create(
                                Some("vi-06cf1d26-0ef0-4b03-aca6-933027d434c2".to_string()),
                                "127.0.0.1",
                                10000,
                            )
                            .unwrap();
                            Ok(ResponseDtoResult::Success(ResponseDto::Media(
                                MediaResponseDto::ContentCreate(socket),
                            )))
                        } else {
                            let socket = SocketInfo::<MediaId>::try_create(
                                Some("au-06cf1d26-0ef0-4b03-aca6-933027d434c2".to_string()),
                                "127.0.0.1",
                                10001,
                            )
                            .unwrap();
                            Ok(ResponseDtoResult::Success(ResponseDto::Media(
                                MediaResponseDto::ContentCreate(socket),
                            )))
                        }
                    }
                    RequestDto::Media(MediaRequestDto::RtcpCreate { params: _ }) => {
                        let socket = SocketInfo::<RtcpId>::try_create(
                            Some("rc-06cf1d26-0ef0-4b03-aca6-933027d434c2".to_string()),
                            "127.0.0.1",
                            10010,
                        )
                        .unwrap();
                        Ok(ResponseDtoResult::Success(ResponseDto::Media(
                            MediaResponseDto::RtcpCreate(socket),
                        )))
                    }
                    _ => {
                        unreachable!()
                    }
                });
            Arc::new(mock_service)
        });

        let module = MediaCallService::builder()
            .with_component_override::<dyn Factory>(Box::new(factory))
            .with_component_override::<dyn Repository>(Box::new(repository))
            .with_component_override::<dyn GlobalState>(Box::new(state))
            .build();
        let service: &dyn Service = module.resolve_ref();
        let result = service
            .execute(RequestDto::Media(MediaRequestDto::Call { params }))
            .await;

        let expected = ResponseDtoResult::Success(ResponseDto::Media(MediaResponseDto::Call(
            MediaConnectionIdWrapper {
                media_connection_id: dto.media_connection_id,
            },
        )));

        assert_eq!(result.unwrap(), expected);
    }
}
