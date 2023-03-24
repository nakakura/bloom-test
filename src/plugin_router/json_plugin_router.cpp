//
// Created by nakakura on 22/08/25.
//

#include "json_plugin_router.h"

//===== private =====
void JsonPluginRouter::observe_socket(std::vector<uint8_t> data) {
  std::string message(data.begin(), data.end());
  std::shared_ptr<rapidjson::Document> doc(new rapidjson::Document);
  doc->Parse(message.c_str());

  if (doc->HasParseError()) {
    ROS_ERROR("invalid json message: %s", message.c_str());
    ROS_ERROR("error location: %ld", doc->GetErrorOffset());
    ROS_ERROR("error mesasge: %s",
              rapidjson::GetParseError_En(doc->GetParseError()));
    return;
  }

  for (auto plugin = plugins_.rbegin(); plugin != plugins_.rend(); ++plugin) {
    (*plugin)->Execute(doc);
  }
}

void JsonPluginRouter::observe_plugins(
    std::shared_ptr<rapidjson::Document> doc) {
  StringBuffer buffer;
  Writer<StringBuffer> writer(buffer);
  doc->Accept(writer);
  std::string s(buffer.GetString(), buffer.GetSize());
  std::vector<uint8_t> data(s.begin(), s.end());
  socket_->SendData(data);
}

//===== public =====
JsonPluginRouter::JsonPluginRouter(std::shared_ptr<rapidjson::Document> config,
                                   udp::endpoint target_socket,
                                   SocketFactory factory)
    : plugin_loader_("skyway", "skyway_plugin::SkyWayJsonPlugin"),
      config_(std::move(config)),
      target_socket_(target_socket) {
  // Socketからのcallbackを与えてSocketを生成`
  socket_ = factory(
      target_socket_,
      std::make_shared<std::function<void(std::vector<uint8_t>)>>(std::bind(
          &JsonPluginRouter::observe_socket, this, std::placeholders::_1)));
}

JsonPluginRouter::~JsonPluginRouter() {
  if (socket_) socket_->Stop();
  for (auto plugin = plugins_.rbegin(); plugin != plugins_.rend(); ++plugin) {
    (*plugin)->Shutdown();
    plugin->reset();
  }
}

PluginResult JsonPluginRouter::TryStart() {
  // plugin情報の配列を与えられていない場合は開始できない
  if (!config_->IsArray()) {
    return {false, 0, "invalid config parameters"};
  }

  auto callback = std::make_shared<
      std::function<void(std::shared_ptr<rapidjson::Document>)>>(std::bind(
      &JsonPluginRouter::observe_plugins, this, std::placeholders::_1));

  for (rapidjson::Value::ConstValueIterator itr = config_->Begin();
       itr != config_->End(); ++itr) {
    if (!itr->HasMember("plugin_name")) continue;
    std::string plugin_name = (*itr)["plugin_name"].GetString();

    try {
      boost::shared_ptr<skyway_plugin::SkyWayJsonPlugin> plugin =
          plugin_loader_.createInstance(plugin_name);
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

      return {false, 0, (const char*)data};
    }
  }

  // ここでsocket startするとデータが流れ始める
  socket_->Start();

  return {true, socket_->Port(), ""};
}

uint16_t JsonPluginRouter::Port() { return socket_->Port(); }

Component<fruit::Annotated<JsonAnnotation, PluginRouterFactory>>
getJsonPluginRouterComponent() {
  return createComponent()
      .bind<fruit::Annotated<JsonAnnotation, PluginRouter>, JsonPluginRouter>()
      .install(getUdpSocketComponent);
}
