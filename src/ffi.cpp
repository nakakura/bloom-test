//
// Created by nakakura on 22/09/04.
//

#include "ffi.h"

// Rust側から呼び出されるC++側関数の実体
extern "C" {
// loggers
void log_debug_c(char* message) {
  ROS_DEBUG("%s", message);
  release_string(message);
}
void log_info_c(char* message) {
  ROS_INFO("%s", message);
  release_string(message);
}
void log_warn_c(char* message) {
  ROS_WARN("%s", message);
  release_string(message);
}
void log_err_c(char* message) {
  ROS_ERROR("%s", message);
  release_string(message);
}

// ros control functions
bool is_ok_c() { return ros::ok(); }
bool is_shutting_down_c() { return ros::isShuttingDown(); }
void ros_sleep_c(double dur) { ros::Duration(dur).sleep(); }
void wait_for_shutdown_c() { ros::waitForShutdown(); }
void shutdown_c() { ros::shutdown(); }
}
