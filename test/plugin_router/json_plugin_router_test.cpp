#include "../../src/plugin_router/json_plugin_router.h"

#include <fruit/fruit.h>
#include <gtest/gtest.h>

#include <algorithm>
#include <boost/array.hpp>
#include <boost/asio.hpp>
#include <functional>
#include <iostream>
#include <iterator>
#include <memory>
#include <unordered_map>

#include "../../src/socket/udp_socket.h"
#include "std_msgs/UInt8MultiArray.h"

using boost::asio::ip::udp;
using fruit::Annotated;
using fruit::Component;
using fruit::createComponent;
using fruit::Injector;

class MockJsonSocket : public Socket {
 private:
  std::shared_ptr<std::function<void(std::vector<uint8_t>)>> callback_;
  int counter = 0;

 public:
  // デフォルトコンストラクタは削除
  MockJsonSocket() = delete;
  INJECT(MockJsonSocket(
      ASSISTED(udp::endpoint) target_socket,
      ASSISTED(std::shared_ptr<std::function<void(std::vector<uint8_t>)>>)
          callback))
      : callback_(callback) {}

  virtual void Start() override {
    std::string first_message =
        "{\"key\": \"value\", \"bool\": true, \"num\": 10}";
    std::vector<uint8_t> first_data(first_message.begin(), first_message.end());
    (*callback_)(first_data);

    std::string second_message =
        "{\"key\": \"value\", \"bool\": false, \"num\": 0}";
    std::vector<uint8_t> second_data(second_message.begin(),
                                     second_message.end());
    (*callback_)(second_data);
  }

  virtual void SendData(std::vector<uint8_t> data) override {
    counter += 1;
    std::string message(data.begin(), data.end());
    if (counter == 1) {
      ASSERT_STREQ(message.c_str(),
                   "{\"key\":\"value\",\"bool\":true,\"num\":10}");
    } else {
      ASSERT_STREQ(message.c_str(),
                   "{\"key\":\"value\",\"bool\":false,\"num\":0}");
    }
  }
};

// Mockを入れるもの
Component<SocketFactory> getMockJsonUdpSourceComponent() {
  return createComponent().bind<Socket, MockJsonSocket>();
}

Component<Annotated<JsonAnnotation, PluginRouterFactory>>
getMockJsonPluginRouterComponent() {
  return createComponent()
      .replace(getUdpSocketComponent)
      .with(getMockJsonUdpSourceComponent)
      .install(getJsonPluginRouterComponent);
}

// XmlRpcValueが不正なケース
TEST(TestSuite, json_plugin_try_start_with_invalid_xml) {
  std::shared_ptr<rapidjson::Document> config(new rapidjson::Document);
  config->Parse("{}");

  // objectを作成し、受信スレッドを開始
  Injector<Annotated<JsonAnnotation, PluginRouterFactory>> injector(
      getMockJsonPluginRouterComponent);
  PluginRouterFactory pluginRouterFactory =
      injector.get<Annotated<JsonAnnotation, PluginRouterFactory>>();

  // データは送信しないのでportは何でも良い
  auto source = pluginRouterFactory(config, udp::endpoint(udp::v4(), 0));
  auto result = source->TryStart();
  // TryStartに失敗して、config間違いのエラーメッセージを受け取る
  ASSERT_FALSE(result.is_success);
  ASSERT_STREQ(result.error_message, "invalid config parameters");
}

// pluginが見つからないケース
TEST(TestSuite, json_plugin_try_start_not_found_plugin) {
  std::shared_ptr<rapidjson::Document> config(new rapidjson::Document);
  config->Parse(
      "[{\"plugin_name\":\"json_loopback::JsonLoopback\",\"param\":"
      "\"Parameter\"},{"
      "\"plugin_name\":\"not_found_plugin::NotFoundPlugin\"}]");

  // objectを作成し、受信スレッドを開始
  Injector<Annotated<JsonAnnotation, PluginRouterFactory>> injector(
      getMockJsonPluginRouterComponent);
  PluginRouterFactory pluginRouterFactory =
      injector.get<Annotated<JsonAnnotation, PluginRouterFactory>>();

  // データは送信しないのでportは何でも良い
  auto source = pluginRouterFactory(config, udp::endpoint(udp::v4(), 0));
  auto result = source->TryStart();
  // TryStartに失敗して、pluginがない旨のメッセージを受け取る
  ASSERT_FALSE(result.is_success);
  std::string error_message = result.error_message;
  ASSERT_EQ(error_message.rfind("Failed to load"), 0);
}

// Loopback Pluginを使うケース
TEST(TestSuite, json_plugin_try_start_with_loopback_plugin) {
  std::shared_ptr<rapidjson::Document> config(new rapidjson::Document);
  config->Parse(
      "[{\"plugin_name\":\"json_loopback::JsonLoopback\",\"param\":"
      "\"Parameter\"}]");

  // objectを作成し、受信スレッドを開始
  Injector<Annotated<JsonAnnotation, PluginRouterFactory>> injector(
      getMockJsonPluginRouterComponent);
  PluginRouterFactory pluginRouterFactory =
      injector.get<Annotated<JsonAnnotation, PluginRouterFactory>>();

  // データは送信しないのでportは何でも良い
  auto source = pluginRouterFactory(config, udp::endpoint(udp::v4(), 0));
  auto result = source->TryStart();
  // TryStartに成功する
  // データの送信と評価はMockJsonSocket内でやっている
  ASSERT_TRUE(result.is_success);
}
