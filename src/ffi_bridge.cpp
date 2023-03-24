//
// Created by nakakura on 22/09/04.
//

#include "ffi_bridge.h"

#include <signal.h>

namespace {
std::function<void(int)> shutdown_handler;
std::function<void(char*, char*)> create_peer_callback_handler;
std::function<PluginLoadResult(char*, uint16_t, char*, char*)>
    create_data_callback_handler;
std::function<void(uint16_t)> data_connection_close_event_callback_handler;
}  // namespace

extern "C" {
void create_peer_callback_ffi(char* peer_id, char* token) {
  create_peer_callback_handler(peer_id, token);
}

// Peer Closeイベントが発火したときにプログラム全体を終了する
void peer_deleted_callback_ffi() { ros::shutdown(); }

PluginLoadResult create_data_callback_ffi(char* target_ip, uint16_t target_port,
                                          char* plugin_type,
                                          char* plugin_param) {
  return create_data_callback_handler(target_ip, target_port, plugin_type,
                                      plugin_param);
}

void data_connection_close_event_callback_ffi(uint16_t port_num) {
  data_connection_close_event_callback_handler(port_num);
}

// リソース開放処理
void release_string_ffi(char* str) { free(str); }
}

FfiBridgeImpl::FfiBridgeImpl(std::shared_ptr<Router> router)
    : router_(std::move(router)) {
  create_peer_callback_handler =
      std::bind(&FfiBridgeImpl::create_peer_callback, this,
                std::placeholders::_1, std::placeholders::_2);

  create_data_callback_handler =
      std::bind(&FfiBridgeImpl::create_data_connection_callback, this,
                std::placeholders::_1, std::placeholders::_2,
                std::placeholders::_3, std::placeholders::_4);

  data_connection_close_event_callback_handler =
      std::bind(&FfiBridgeImpl::delete_data_connection_callback, this,
                std::placeholders::_1);

  Function functions{create_peer_callback_ffi, peer_deleted_callback_ffi,
                     create_data_callback_ffi,
                     data_connection_close_event_callback_ffi,
                     release_string_ffi};
  register_callbacks(functions);
}

void FfiBridgeImpl::create_peer_callback(char* peer_id, char* token) {
  router_->OnCreatePeer(peer_id, token);

  release_string(peer_id);
  release_string(token);
}

PluginLoadResult FfiBridgeImpl::create_data_connection_callback(
    char* target_ip, uint16_t port, char* plugin_type, char* plugin_param) {
  auto result =
      router_->OnConnectData(target_ip, port, plugin_type, plugin_param);
  release_string(target_ip);
  release_string(plugin_type);
  release_string(plugin_param);

  struct PluginLoadResult response = {.is_success = result.is_success,
                                      .port = result.port,
                                      .error_message = result.error_message};
  return response;
}

void FfiBridgeImpl::delete_data_connection_callback(uint16_t port_num) {
  router_->OnDeleteData(port_num);
}

Component<FfiBridge> getFfiComponent() {
  return fruit::createComponent().bind<FfiBridge, FfiBridgeImpl>().install(
      getRouterComponent);
}
