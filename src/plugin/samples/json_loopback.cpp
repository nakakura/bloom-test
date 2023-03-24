//
// Created by nakakura on 22/08/29.
//
#include <pluginlib/class_list_macros.h>
#include <ros/ros.h>
#include <skyway/json_loopback.h>
#include <skyway/skyway_plugin.h>

json_loopback::JsonLoopback::JsonLoopback() {
  ROS_INFO("json_loopback plugin loaded");
}

void json_loopback::JsonLoopback::Initialize(
    std::shared_ptr<rapidjson::Document> parameter,
    std::shared_ptr<std::function<void(std::shared_ptr<rapidjson::Document>)>>
        callback) {
  callback_ = callback;
}

void json_loopback::JsonLoopback::Execute(
    std::shared_ptr<rapidjson::Document> data) {
  (*callback_)(data);
}

void json_loopback::JsonLoopback::Shutdown() {}
