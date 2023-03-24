//
// Created by nakakura on 22/08/30.
//

#include <pluginlib/class_list_macros.h>
#include <ros/ros.h>
#include <skyway/binary_loopback.h>
#include <skyway/json_loopback.h>
#include <skyway/skyway_plugin.h>
#include <skyway/string_loopback.h>
#include <skyway/string_pub_sub.h>

PLUGINLIB_EXPORT_CLASS(binary_loopback::BinaryLoopback,
                       skyway_plugin::SkyWayBinaryPlugin);
PLUGINLIB_EXPORT_CLASS(json_loopback::JsonLoopback,
                       skyway_plugin::SkyWayJsonPlugin);
PLUGINLIB_EXPORT_CLASS(string_loopback::StringLoopback,
                       skyway_plugin::SkyWayStringPlugin);
PLUGINLIB_EXPORT_CLASS(string_pub_sub::StringPubSub,
                       skyway_plugin::SkyWayStringPlugin);
