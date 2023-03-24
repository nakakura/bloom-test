//
// Created by nakakura on 22/09/02.
//

#ifndef SKYWAY_PLUGIN_FACTORY_H
#define SKYWAY_PLUGIN_FACTORY_H

#include <fruit/fruit.h>

#include "binary_plugin_router.h"
#include "json_plugin_router.h"
#include "string_plugin_router.h"

using fruit::Annotated;
using fruit::Component;
using fruit::createComponent;
using fruit::Injector;

class IPluginRouterFactory {
 private:
 public:
  virtual ~IPluginRouterFactory() = default;
  virtual std::unique_ptr<PluginRouter> Create(
      std::string target_ip, uint16_t target_port, std::string plugin_type,
      std::shared_ptr<rapidjson::Document> config) = 0;
};

class PluginRouterFactoryImpl : public IPluginRouterFactory {
 private:
  PluginRouterFactory binary_factory_;
  PluginRouterFactory json_factory_;
  PluginRouterFactory string_factory_;

 public:
  PluginRouterFactoryImpl() = delete;
  INJECT(PluginRouterFactoryImpl(
      ANNOTATED(BinaryAnnotation, PluginRouterFactory) binary_factory,
      ANNOTATED(JsonAnnotation, PluginRouterFactory) json_factory,
      ANNOTATED(StringAnnotation, PluginRouterFactory) string_factory))
      : binary_factory_(binary_factory),
        json_factory_(json_factory),
        string_factory_(string_factory) {}
  ~PluginRouterFactoryImpl() {}

  virtual std::unique_ptr<PluginRouter> Create(
      std::string target_ip, uint16_t target_port, std::string plugin_type,
      std::shared_ptr<rapidjson::Document> config) override;
};

Component<IPluginRouterFactory> getPluginFactoryComponent();

#endif  // SKYWAY_PLUGIN_FACTORY_H
