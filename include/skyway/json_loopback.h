//
// Created by nakakura on 2022/09/03.
//

#ifndef SKYWAY_PLUGIN_JSON_LOOPBACK_H
#define SKYWAY_PLUGIN_JSON_LOOPBACK_H

#include <skyway/skyway_plugin.h>

namespace json_loopback {
class JsonLoopback : public skyway_plugin::SkyWayJsonPlugin {
 private:
  std::shared_ptr<std::function<void(std::shared_ptr<rapidjson::Document>)>>
      callback_;

 public:
  JsonLoopback();
  virtual void Initialize(
      std::shared_ptr<rapidjson::Document> parameter,
      std::shared_ptr<std::function<void(std::shared_ptr<rapidjson::Document>)>>
          callback) override;
  virtual void Execute(std::shared_ptr<rapidjson::Document> data) override;
  virtual void Shutdown() override;
};
};  // namespace json_loopback

#endif  // SKYWAY_PLUGIN_JSON_LOOPBACK_H
