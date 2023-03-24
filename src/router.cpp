#include "router.h"

namespace {
std::function<void(int)> shutdown_handler;
void signal_handler(int signal) { shutdown_handler(signal); }
}  // namespace

RouterImpl::RouterImpl(
    ControlServiceFactory control_service_factory,
    EventsServiceFactory event_service_factory,
    std::shared_ptr<IPluginRouterFactory> plugin_router_factory)
    : control_service_factory_(control_service_factory),
      event_service_factory_(event_service_factory),
      plugin_router_factory_(std::move(plugin_router_factory)) {
  shutdown_handler =
      std::bind(&RouterImpl::shutdown, this, std::placeholders::_1);
  // 終了時にROS-nodeが落ちる前に開放処理をしなければならないので、SIGINTをhookする
  signal(SIGINT, signal_handler);

  ROS_DEBUG("start /skyway_control");
  // SkyWayControl Serviceの起動
  control_service_ = control_service_factory_(
      "skyway_control",
      std::bind(&RouterImpl::on_control_message, this, std::placeholders::_1));
  ROS_DEBUG("start /skyway_events");
  // SkyWayEvent Serviceの起動
  event_service_ = event_service_factory_(
      "skyway_events", std::bind(&RouterImpl::on_event_request, this));
}

std::string RouterImpl::on_control_message(std::string request) {
  char* message = call_service(request.c_str());
  // Rust側でCString.into_raw()しているので、開放が必要
  std::string response = message;
  release_string(message);
  return response;
}

std::string RouterImpl::on_event_request() {
  // これ以降の処理はcallbackを除き全てRust側で実装する
  char* message = receive_events();
  // Rust側でCString.into_raw()しているので、開放が必要
  std::string event = message;
  release_string(message);
  return event;
}

void RouterImpl::shutdown(int signal) {
  // ctrl-cを受けたあとの終了処理は全てここで行う

  // Peer Object生成後であれば、勝手に終了されると
  // Controllerが困るので、何もしない
  if (peer_id_ != "" && token_ != "") {
  } else {
    // Peer Object生成前であれば終了してしまう
    ros::shutdown();
  }
}

void RouterImpl::OnCreatePeer(char* peer_id, char* token) {
  // Peer Objectの生成に成功したら、peer_idとtokenを保持しておく
  // これは終了時に開放するためだけに利用する
  peer_id_ = peer_id;
  token_ = token;
}

PluginResult RouterImpl::OnConnectData(std::string target_ip,
                                       uint16_t target_port,
                                       std::string plugin_type,
                                       std::string plugin_param) {
  std::shared_ptr<rapidjson::Document> doc(new rapidjson::Document);
  doc->Parse(plugin_param.c_str());

  auto plugin_router =
      plugin_router_factory_->Create(target_ip, target_port, plugin_type, doc);
  auto result = plugin_router->TryStart();

  // pluginのロードに成功した場合のみ、実体を保管する
  if (result.is_success) {
    std::stringstream ss;
    ss << "key-" << result.port;
    std::string key = ss.str();
    plugin_map_.emplace(key, std::move(plugin_router));
  }

  return result;
}

void RouterImpl::OnDeleteData(uint16_t port_num) {
  std::stringstream ss;
  ss << "key-" << port_num;
  std::string key = ss.str();
  if (plugin_map_.find(key) != plugin_map_.end()) {
    plugin_map_.erase(key);
  } else {
    ROS_ERROR("data connection not found at onDeleteData");
  }
}

Component<Router> getRouterComponent() {
  return fruit::createComponent()
      .bind<Router, RouterImpl>()
      .install(getControlServiceComponent)
      .install(getEventsServiceComponent)
      .install(getPluginFactoryComponent);
}
