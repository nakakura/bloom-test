//
// Created by nakakura on 22/09/04.
//

/// Rust側から呼ばれた処理をハンドリングするためのクラス
#ifndef SKYWAY_FFI_BRIDGE_H
#define SKYWAY_FFI_BRIDGE_H

#include <fruit/fruit.h>
#include <ros/ros.h>

#include "ffi.h"

using fruit::Component;
using fruit::Injector;

class FfiBridge {};

class FfiBridgeImpl : public FfiBridge {
 private:
  void create_peer_callback(char* peer_id, char* token);
  PluginLoadResult create_data_connection_callback(char*, uint16_t, char*,
                                                   char*);
  void delete_data_connection_callback(uint16_t);

  std::shared_ptr<Router> router_;

 public:
  FfiBridgeImpl() = delete;
  INJECT(FfiBridgeImpl(std::shared_ptr<Router> router));
  virtual ~FfiBridgeImpl() {}
};

Component<FfiBridge> getFfiComponent();

#endif  // SKYWAY_FFI_BRIDGE_H
