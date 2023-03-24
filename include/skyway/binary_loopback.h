//
// Created by nakakura on 22/08/29.
//

#ifndef SKYWAY_PLUGIN_LOOPBACK_BINARY_PLUGIN_H
#define SKYWAY_PLUGIN_LOOPBACK_BINARY_PLUGIN_H

#include <skyway/skyway_plugin.h>

namespace binary_loopback {
class BinaryLoopback : public skyway_plugin::SkyWayBinaryPlugin {
 private:
  std::shared_ptr<std::function<void(std::vector<uint8_t>)>> callback_;

 public:
  BinaryLoopback();
  virtual ~BinaryLoopback() override;
  virtual void Initialize(
      std::shared_ptr<rapidjson::Document> parameter,
      std::shared_ptr<std::function<void(std::vector<uint8_t>)>> callback)
      override;
  virtual void Execute(std::vector<uint8_t> data) override;
  virtual void Shutdown() override;
};
};  // namespace binary_loopback

#endif  // SKYWAY_PLUGIN_LOOPBACK_BINARY_PLUGIN_H
