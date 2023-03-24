use super::EventReceiveImpl;
use crate::application::dto::response::{DataConnectionEventDto, DataResponseDto};
use crate::domain::entity::response::DataResponse;
use crate::domain::entity::{DataConnectionEventEnum, DataConnectionIdWrapper};
use crate::error;

impl EventReceiveImpl {
    pub(crate) async fn process_data_event(
        &self,
        response: DataResponse,
    ) -> Result<DataResponseDto, error::Error> {
        match response {
            DataResponse::Event(DataConnectionEventEnum::OPEN(open)) => {
                if let Some(item) = self.state.find_topic(&open.data_connection_id) {
                    Ok(DataResponseDto::Event(DataConnectionEventDto::OPEN(
                        DataConnectionIdWrapper {
                            data_connection_id: item.data_connection_id,
                        },
                    )))
                } else {
                    let message = format!(
                        "no info about DataConnectionId {:?}",
                        open.data_connection_id.as_str()
                    );
                    Err(error::Error::create_local_error(&message))
                }
            }
            DataResponse::Event(DataConnectionEventEnum::CLOSE(close)) => {
                let data_info = self.state.remove_topic(&close.data_connection_id);
                data_info.map(|item| {
                    self.callback
                        .data_connection_deleted_callback(item.data_pipe_port_num);
                });

                Ok(DataResponseDto::Event(DataConnectionEventDto::CLOSE(close)))
            }
            DataResponse::Event(event) => {
                let message = format!("This event is not processed {:?}", event);
                self.logger.error(&message);
                todo!()
            }
            _ => {
                let message = format!("Non-Event object is processed in EventReceiveImpl as Data");
                self.logger.error(&message);
                unreachable!()
            }
        }
    }
}
