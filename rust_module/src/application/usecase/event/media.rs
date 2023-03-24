use super::EventReceiveImpl;
use crate::application::dto::response::{
    CallResponseDto, MediaConnectionEventEnumDto, MediaResponseDto,
};
use crate::domain::entity::response::MediaResponse;
use crate::domain::entity::MediaConnectionEventEnum;
use crate::error;

impl EventReceiveImpl {
    pub(crate) async fn process_media_event(
        &self,
        response: MediaResponse,
    ) -> Result<MediaResponseDto, error::Error> {
        match response {
            MediaResponse::Event(MediaConnectionEventEnum::STREAM(stream)) => {
                let response = self
                    .state
                    .find_call_response(&stream.media_connection_id)
                    .expect("call response info is not stored");

                let call_response_dto = CallResponseDto {
                    send_params: response.send_params,
                    redirect_params: response.redirect_params,
                    media_connection_id: stream.media_connection_id,
                };
                Ok(MediaResponseDto::Event(
                    MediaConnectionEventEnumDto::Stream(call_response_dto),
                ))
            }
            MediaResponse::Event(MediaConnectionEventEnum::READY(stream)) => {
                let response = self
                    .state
                    .find_call_response(&stream.media_connection_id)
                    .expect("call response info is not stored");

                let call_response_dto = CallResponseDto {
                    send_params: response.send_params,
                    redirect_params: response.redirect_params,
                    media_connection_id: stream.media_connection_id,
                };
                Ok(MediaResponseDto::Event(MediaConnectionEventEnumDto::Ready(
                    call_response_dto,
                )))
            }
            MediaResponse::Event(MediaConnectionEventEnum::CLOSE(id_wrapper)) => Ok(
                MediaResponseDto::Event(MediaConnectionEventEnumDto::Close(id_wrapper)),
            ),
            MediaResponse::Event(event) => {
                let message = format!("This event is not processed {:?}", event);
                self.logger.error(&message);
                todo!()
            }
            _ => {
                let message = format!("Non-Event object is processed in EventReceiveImpl as Media");
                self.logger.error(&message);
                unreachable!()
            }
        }
    }
}
