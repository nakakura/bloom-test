//
// Created by nakakura on 22/09/02.
//

#include "plugin_router_factory.h"

std::unique_ptr<PluginRouter> PluginRouterFactoryImpl::Create(
    std::string target_ip, uint16_t port, std::string plugin_type,
    std::shared_ptr<rapidjson::Document> config) {
  if (plugin_type == "binary") {
    return binary_factory_(
        config, udp::endpoint(address::from_string(target_ip), port));
  } else if (plugin_type == "json") {
    return json_factory_(config,
                         udp::endpoint(address::from_string(target_ip), port));
  } else {
    return string_factory_(
        config, udp::endpoint(address::from_string(target_ip), port));
  }
}

Component<IPluginRouterFactory> getPluginFactoryComponent() {
  return createComponent()
      .bind<IPluginRouterFactory, PluginRouterFactoryImpl>()
      .install(getBinaryPluginRouterComponent)
      .install(getJsonPluginRouterComponent)
      .install(getStringPluginRouterComponent);
}
