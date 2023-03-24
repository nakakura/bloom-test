//
// Created by nakakura on 22/08/25.
//

#ifndef SKYWAY_PLUGIN_UDP_PIPE_PLUGINROUTER_H
#define SKYWAY_PLUGIN_UDP_PIPE_PLUGINROUTER_H

#include <rapidjson/document.h>
#include <rapidjson/stringbuffer.h>
#include <rapidjson/writer.h>

using namespace rapidjson;

struct PluginResult {
  bool is_success;
  uint16_t port;
  const char* error_message;
};

class PluginRouter {
 private:
 public:
  virtual ~PluginRouter() = default;
  virtual PluginResult TryStart() { return {true, 0, ""}; }
  virtual uint16_t Port() = 0;
};

using PluginRouterFactory = std::function<std::unique_ptr<PluginRouter>(
    std::shared_ptr<rapidjson::Document>, udp::endpoint)>;

#endif  // SKYWAY_PLUGIN_UDP_PIPE_PLUGINROUTER_H
