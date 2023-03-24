#include "../../src/plugin_router/plugin_router_factory.h"

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

#include "../../src/plugin_router/binary_plugin_router.h"
#include "../../src/plugin_router/json_plugin_router.h"
#include "../../src/plugin_router/string_plugin_router.h"

using fruit::Component;
using fruit::createComponent;
using fruit::Injector;

// Binary Loopback Pluginを使うケース
TEST(TestSuite, create_binary_plugin_with_factory) {
  std::shared_ptr<rapidjson::Document> config(new rapidjson::Document);
  config->Parse(
      "[{\"plugin_name\":\"binary_loopback::BinaryLoopback\",\"param\":"
      "\"Parameter\"}]");

  // objectを作成し、受信スレッドを開始
  Injector<IPluginRouterFactory> injector(getPluginFactoryComponent);
  auto pluginFactory = injector.get<IPluginRouterFactory*>();

  // データは送信しないのでportは何でも良い
  auto source = pluginFactory->Create("127.0.0.1", 50000, "binary", config);
  auto result = source->TryStart();
  // TryStartに成功する
  // データの送信と評価はMockBinarySocket内でやっている
  ASSERT_TRUE(result.is_success);
}

// Json Loopback Pluginを使うケース
TEST(TestSuite, create_json_plugin_with_factory) {
  std::shared_ptr<rapidjson::Document> config(new rapidjson::Document);
  config->Parse(
      "[{\"plugin_name\":\"json_loopback::JsonLoopback\",\"param\":"
      "\"Parameter\"}]");

  // objectを作成し、受信スレッドを開始
  Injector<IPluginRouterFactory> injector(getPluginFactoryComponent);
  auto pluginFactory = injector.get<IPluginRouterFactory*>();

  // データは送信しないのでportは何でも良い
  auto source = pluginFactory->Create("127.0.0.1", 50000, "json", config);
  auto result = source->TryStart();
  // TryStartに成功する
  // データの送信と評価はMockBinarySocket内でやっている
  ASSERT_TRUE(result.is_success);
}

// String Loopback Pluginを使うケース
TEST(TestSuite, create_string_plugin_with_factory) {
  std::shared_ptr<rapidjson::Document> config(new rapidjson::Document);
  config->Parse(
      "[{\"plugin_name\":\"string_loopback::StringLoopback\",\"param\":"
      "\"Parameter\"}]");

  // objectを作成し、受信スレッドを開始
  Injector<IPluginRouterFactory> injector(getPluginFactoryComponent);
  auto pluginFactory = injector.get<IPluginRouterFactory*>();

  // データは送信しないのでportは何でも良い
  auto source = pluginFactory->Create("127.0.0.1", 50000, "string", config);
  auto result = source->TryStart();
  // TryStartに成功する
  // データの送信と評価はMockBinarySocket内でやっている
  ASSERT_TRUE(result.is_success);
}