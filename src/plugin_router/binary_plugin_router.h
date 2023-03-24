//
// Created by nakakura on 22/08/25.
//

#ifndef SKYWAY_PLUGIN_UDP_PIPE_BINARY_PLUGIN_ROUTER_H
#define SKYWAY_PLUGIN_UDP_PIPE_BINARY_PLUGIN_ROUTER_H

#include <fruit/fruit.h>
#include <pluginlib/class_loader.h>
#include <ros/ros.h>

#include "../socket/udp_socket.h"
#include "plugin_router.h"
#include "skyway/skyway_plugin.h"

using fruit::Component;
using fruit::createComponent;
using fruit::Injector;

class BinaryPluginRouter : public PluginRouter {
 private:
  ros::NodeHandle nh_;
  pluginlib::ClassLoader<skyway_plugin::SkyWayBinaryPlugin> plugin_loader_;
  std::vector<boost::shared_ptr<skyway_plugin::SkyWayBinaryPlugin>> plugins_;
  udp::endpoint target_socket_;
  std::unique_ptr<Socket> socket_;
  std::shared_ptr<rapidjson::Document> config_;

  void observe_socket(std::vector<uint8_t> data);
  void observe_plugins(std::vector<uint8_t> data);

 public:
  BinaryPluginRouter() = delete;
  INJECT(BinaryPluginRouter(ASSISTED(std::shared_ptr<rapidjson::Document>)
                                config,
                            ASSISTED(udp::endpoint) target_socket,
                            SocketFactory factory));
  ~BinaryPluginRouter();

  virtual PluginResult TryStart() override;
  virtual uint16_t Port() override;
};

struct BinaryAnnotation {};

Component<fruit::Annotated<BinaryAnnotation, PluginRouterFactory>>
getBinaryPluginRouterComponent();

#endif  // SKYWAY_PLUGIN_UDP_PIPE_BINARY_PLUGIN_ROUTER_H
