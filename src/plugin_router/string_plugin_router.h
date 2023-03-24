//
// Created by nakakura on 22/08/31.
//

#ifndef SKYWAY_PLUGIN_UDP_PIPE_STRING_PLUGIN_ROUTER_H
#define SKYWAY_PLUGIN_UDP_PIPE_STRING_PLUGIN_ROUTER_H

#include <fruit/fruit.h>
#include <pluginlib/class_loader.h>
#include <ros/ros.h>

#include "../socket/udp_socket.h"
#include "plugin_router.h"
#include "skyway/skyway_plugin.h"

using fruit::Component;
using fruit::createComponent;
using fruit::Injector;

class StringPluginRouter : public PluginRouter {
 private:
  pluginlib::ClassLoader<skyway_plugin::SkyWayStringPlugin> plugin_loader_;
  std::vector<boost::shared_ptr<skyway_plugin::SkyWayStringPlugin>> plugins_;
  udp::endpoint target_socket_;
  std::unique_ptr<Socket> socket_;
  std::shared_ptr<rapidjson::Document> config_;

  void observe_socket(std::vector<uint8_t> data);
  void observe_plugins(std::string message);

 public:
  StringPluginRouter() = delete;
  INJECT(StringPluginRouter(ASSISTED(std::shared_ptr<rapidjson::Document>)
                                config,
                            ASSISTED(udp::endpoint) target_socket,
                            SocketFactory factory));
  ~StringPluginRouter();

  virtual PluginResult TryStart() override;
  virtual uint16_t Port() override;
};

struct StringAnnotation {};

Component<fruit::Annotated<StringAnnotation, PluginRouterFactory>>
getStringPluginRouterComponent();

#endif  // SKYWAY_PLUGIN_UDP_PIPE_STRING_PLUGIN_ROUTER_H
