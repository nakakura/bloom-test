//
// Created by nakakura on 22/09/04.
//

#include "ffi_bridge.h"

#include <signal.h>

namespace {
std::function<void(int)> shutdown_handler;
std::function<void(char*, char*)> create_peer_callback_handler;
std::function<PluginLoadResult(char*)> create_data_callback_handler;
std::function<void(char*)> data_connection_close_event_callback_handler;
}  // namespace

extern "C" {
void create_peer_callback_ffi(char* peer_id, char* token) {
  ROS_ERROR("create peer callback");
  create_peer_callback_handler(peer_id, token);
}

// Peer Closeイベントが発火したときにプログラム全体を終了する
void peer_deleted_callback_ffi() { ros::shutdown(); }

PluginLoadResult create_data_callback_ffi(char* message) {
  return create_data_callback_handler(message);
}

void data_connection_close_event_callback_ffi(char* data_connection_id) {
  // data_connection_close_event_callback_handler(data_connection_id);
}
}

FfiBridgeImpl::FfiBridgeImpl(std::shared_ptr<Router> router)
    : router_(std::move(router)) {
  create_peer_callback_handler =
      std::bind(&FfiBridgeImpl::create_peer_callback, this,
                std::placeholders::_1, std::placeholders::_2);

  create_data_callback_handler =
      std::bind(&FfiBridgeImpl::create_data_connection_callback, this,
                std::placeholders::_1);

  data_connection_close_event_callback_handler =
      std::bind(&FfiBridgeImpl::delete_data_connection_callback, this,
                std::placeholders::_1);

  Function functions{create_peer_callback_ffi, peer_deleted_callback_ffi,
                     create_data_callback_ffi,
                     data_connection_close_event_callback_ffi};
  register_callbacks(functions);
}

void FfiBridgeImpl::create_peer_callback(char* peer_id, char* token) {
  router_->OnCreatePeer(peer_id, token);

  release_string(peer_id);
  release_string(token);
}

PluginLoadResult FfiBridgeImpl::create_data_connection_callback(char* message) {
  release_string(message);
  // Todo: impl
  return {.is_success = true, .port = 51111, .error_message = ""};
  /*
  auto source = source_factory_(
      parameter.source_parameters.source_topic_name,
      udp::endpoint(boost::asio::ip::address::from_string(
                        parameter.source_parameters.destination_address),
                    parameter.source_parameters.destination_port));
  auto destination = destination_factory_(
      parameter.destination_parameters.destination_topic_name,
      udp::endpoint(udp::v4(), parameter.destination_parameters.source_port));
  data_topic_container_->CreateData(parameter.data_connection_id,
                                    std::move(source),
                                    std::move(destination));
                                    */
}

void FfiBridgeImpl::delete_data_connection_callback(char* data_connection_id) {
  // Todo: impl
  // data_topic_container_->DeleteData(data_connection_id);
  release_string(data_connection_id);
}

Component<FfiBridge> getFfiComponent() {
  return fruit::createComponent().bind<FfiBridge, FfiBridgeImpl>().install(
      getRouterComponent);
}