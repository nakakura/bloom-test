//
// Created by nakakura on 22/08/29.
//
#include <pluginlib/class_list_macros.h>
#include <ros/ros.h>
#include <skyway/binary_loopback.h>
#include <skyway/skyway_plugin.h>

binary_loopback::BinaryLoopback::BinaryLoopback() {
  ROS_INFO("binary_loopback plugin loaded");
}

binary_loopback::BinaryLoopback::~BinaryLoopback() {
  ROS_INFO("binary_loopback plugin exited");
}

void binary_loopback::BinaryLoopback::Initialize(
    std::shared_ptr<rapidjson::Document> parameter,
    std::shared_ptr<std::function<void(std::vector<uint8_t>)>> callback) {
  callback_ = callback;
}

void binary_loopback::BinaryLoopback::Execute(std::vector<uint8_t> data) {
  (*callback_)(data);
}

void binary_loopback::BinaryLoopback::Shutdown() {}
