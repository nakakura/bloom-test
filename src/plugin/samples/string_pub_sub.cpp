//
// Created by nakakura on 22/08/29.
//
#include <pluginlib/class_list_macros.h>
#include <ros/ros.h>
#include <skyway/skyway_plugin.h>
#include <skyway/string_pub_sub.h>

string_pub_sub::StringPubSub::StringPubSub() {
  ROS_INFO("string_pub_sub plugin loaded");
}

string_pub_sub::StringPubSub::~StringPubSub() {
  loop_thread_.join();
  ROS_INFO("camera plugin unloaded");
}

void string_pub_sub::StringPubSub::Initialize(
    std::shared_ptr<rapidjson::Document> parameter,
    std::shared_ptr<std::function<void(std::string)>> callback) {
  callback_ = callback;

  is_running_ = true;
  loop_thread_ =
      std::thread(&string_pub_sub::StringPubSub::service_thread, this);
}

void string_pub_sub::StringPubSub::Execute(std::string data) {
  std::unique_lock<std::mutex> lock(mutex_);
  parameters_.push_back(data);
}

void string_pub_sub::StringPubSub::Shutdown() { is_running_ = false; }

void string_pub_sub::StringPubSub::service_thread() {
  ros::Rate loop_rate(10);
  bool is_received = false;

  while (is_running_) {
    std::string message;
    is_received = false;

    {
      std::unique_lock<std::mutex> lock(mutex_);

      if (parameters_.size() != 0) {
        message = parameters_.front();
        parameters_.pop_front();
        is_received = true;
      }
    }

    if (is_received) {
      std_msgs::String msg;
      msg.data = message;
      pub_.publish(msg);
    } else {
      // 受信していなければ100ms待機
      loop_rate.sleep();
    }
  }
}

void string_pub_sub::StringPubSub::subscribe(
    const std_msgs::String::ConstPtr& msg) {
  (*callback_)(msg->data);
}
