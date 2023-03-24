//
// Created by nakakura on 22/08/29.
//
#include <pluginlib/class_list_macros.h>
#include <ros/ros.h>
#include <skyway/skyway_plugin.h>
#include <skyway/string_loopback.h>

string_loopback::StringLoopback::StringLoopback() {
  ROS_INFO("string_loopback plugin loaded");
}

void string_loopback::StringLoopback::Initialize(
    std::shared_ptr<rapidjson::Document> parameter,
    std::shared_ptr<std::function<void(std::string)>> callback) {
  callback_ = callback;
}

void string_loopback::StringLoopback::Execute(std::string data) {
  (*callback_)(data);
}

void string_loopback::StringLoopback::Shutdown() {}
