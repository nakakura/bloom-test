//
// Created by nakakura on 22/06/17.
//

#ifndef SKYWAY_PLUGIN_SKYWAY_PLUGIN_H
#define SKYWAY_PLUGIN_SKYWAY_PLUGIN_H

#include <rapidjson/document.h>
#include <rapidjson/stringbuffer.h>
#include <rapidjson/writer.h>

#include <functional>
#include <vector>

using namespace rapidjson;

namespace skyway_plugin {
class SkyWayBinaryPlugin {
 public:
  virtual void Initialize(
      std::shared_ptr<rapidjson::Document> parameter,
      std::shared_ptr<std::function<void(std::vector<uint8_t>)>> callback) = 0;
  virtual void Execute(std::vector<uint8_t> data) = 0;
  virtual void Shutdown() = 0;
  virtual ~SkyWayBinaryPlugin() {}

 protected:
  SkyWayBinaryPlugin() {}
};

class SkyWayStringPlugin {
 public:
  virtual void Initialize(
      std::shared_ptr<rapidjson::Document> parameter,
      std::shared_ptr<std::function<void(std::string)>> callback) = 0;
  virtual void Execute(std::string data) = 0;
  virtual void Shutdown() = 0;
  virtual ~SkyWayStringPlugin() {}

 protected:
  SkyWayStringPlugin() {}
};

class SkyWayJsonPlugin {
 public:
  virtual void Initialize(
      std::shared_ptr<rapidjson::Document> parameter,
      std::shared_ptr<std::function<void(std::shared_ptr<rapidjson::Document>)>>
          callback) = 0;
  virtual void Execute(std::shared_ptr<rapidjson::Document> document) = 0;
  virtual void Shutdown() = 0;
  virtual ~SkyWayJsonPlugin() {}

 protected:
  SkyWayJsonPlugin() {}
};
};  // namespace skyway_plugin

#endif  // SKYWAY_PLUGIN_SKYWAY_PLUGIN_H
