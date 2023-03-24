//
// Created by nakakura on 22/08/31.
//

#ifndef SKYWAY_PLUGIN_STRING_LOOPBACK_H
#define SKYWAY_PLUGIN_STRING_LOOPBACK_H

#include <skyway/skyway_plugin.h>

namespace string_loopback {
class StringLoopback : public skyway_plugin::SkyWayStringPlugin {
 private:
  std::shared_ptr<std::function<void(std::string)>> callback_;

 public:
  StringLoopback();
  virtual void Initialize(
      std::shared_ptr<rapidjson::Document> parameter,
      std::shared_ptr<std::function<void(std::string)>> callback) override;
  virtual void Execute(std::string data) override;
  virtual void Shutdown() override;
};
};  // namespace string_loopback

#endif  // SKYWAY_PLUGIN_STRING_LOOPBACK_H
