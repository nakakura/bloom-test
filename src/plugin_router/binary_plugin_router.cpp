//
// Created by nakakura on 22/08/25.
//

#include "binary_plugin_router.h"

//===== private =====
void BinaryPluginRouter::observe_socket(std::vector<uint8_t> data) {
  for (auto plugin = plugins_.rbegin(); plugin != plugins_.rend(); ++plugin) {
    (*plugin)->Execute(data);
  }
}

void BinaryPluginRouter::observe_plugins(std::vector<uint8_t> data) {
  socket_->SendData(data);
}

//===== public =====
BinaryPluginRouter::BinaryPluginRouter(
    std::shared_ptr<rapidjson::Document> config, udp::endpoint target_socket,
    SocketFactory factory)
    : plugin_loader_("skyway", "skyway_plugin::SkyWayBinaryPlugin"),
      config_(std::move(config)),
      target_socket_(target_socket) {
  // Socketからのcallbackを与えてSocketを生成`
  socket_ = factory(
      target_socket_,
      std::make_shared<std::function<void(std::vector<uint8_t>)>>(std::bind(
          &BinaryPluginRouter::observe_socket, this, std::placeholders::_1)));
}

BinaryPluginRouter::~BinaryPluginRouter() {
  if (socket_) socket_->Stop();
  for (auto plugin = plugins_.rbegin(); plugin != plugins_.rend(); ++plugin) {
    (*plugin)->Shutdown();
    plugin->reset();
  }
}

PluginResult BinaryPluginRouter::TryStart() {
  // plugin情報の配列を与えられていない場合は開始できない
  if (!config_->IsArray()) {
    return {false, 0, "invalid config parameters"};
  }

  // try startにして、errorを返せるようにする
  auto callback =
      std::make_shared<std::function<void(std::vector<uint8_t>)>>(std::bind(
          &BinaryPluginRouter::observe_plugins, this, std::placeholders::_1));

  for (rapidjson::Value::ConstValueIterator itr = config_->Begin();
       itr != config_->End(); ++itr) {
    if (!itr->HasMember("plugin_name")) continue;
    std::string plugin_name = (*itr)["plugin_name"].GetString();

    try {
      boost::shared_ptr<skyway_plugin::SkyWayBinaryPlugin> plugin =
          plugin_loader_.createInstance(plugin_name.c_str());
      std::shared_ptr<rapidjson::Document> parameter(new rapidjson::Document);
      parameter->CopyFrom(*itr, parameter->GetAllocator());
      plugin->Initialize(std::move(parameter), callback);
      plugins_.push_back(plugin);
    } catch (pluginlib::PluginlibException& ex) {
      // pluginがopenできなかったらここでreturnする
      std::ostringstream stream;
      stream << "Failed to load " << plugin_name << ex.what();
      std::string message = stream.str();
      char* data = (char*)malloc(strlen(message.c_str()) + 1);
      strcpy(data, message.c_str());
      return {.is_success = false, 0, .error_message = (const char*)data};
    }
  }

  // ここでsocket startするとデータが流れ始める
  socket_->Start();

  return {true, socket_->Port(), ""};
}

uint16_t BinaryPluginRouter::Port() { return socket_->Port(); }

Component<fruit::Annotated<BinaryAnnotation, PluginRouterFactory>>
getBinaryPluginRouterComponent() {
  return createComponent()
      .bind<fruit::Annotated<BinaryAnnotation, PluginRouter>,
            BinaryPluginRouter>()
      .install(getUdpSocketComponent);
}
