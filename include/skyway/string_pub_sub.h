//
// Created by nakakura on 22/08/31.
//

#ifndef SKYWAY_PLUGIN_STRING_PUB_SUB_H
#define SKYWAY_PLUGIN_STRING_PUB_SUB_H

#include <skyway/skyway_plugin.h>

#include <mutex>
#include <thread>

#include "std_msgs/String.h"

namespace string_pub_sub {
class StringPubSub : public skyway_plugin::SkyWayStringPlugin {
 private:
  ros::NodeHandle nh_;
  ros::Publisher pub_ =
      nh_.advertise<std_msgs::String>("skyway_string_publisher", 1000);
  ros::Subscriber sub_ = nh_.subscribe("skyway_string_subscriber", 1000,
                                       &StringPubSub::subscribe, this);
  std::shared_ptr<std::function<void(std::string)>> callback_;
  std::list<std::string> parameters_{};
  std::mutex mutex_;
  std::thread loop_thread_;
  bool is_running_;

  void subscribe(const std_msgs::String::ConstPtr& msg);
  void service_thread();

 public:
  StringPubSub();
  ~StringPubSub();
  virtual void Initialize(
      std::shared_ptr<rapidjson::Document> parameter,
      std::shared_ptr<std::function<void(std::string)>> callback) override;
  virtual void Execute(std::string data) override;
  virtual void Shutdown() override;
};
};  // namespace string_pub_sub

#endif  // SKYWAY_PLUGIN_STRING_PUB_SUB_H
