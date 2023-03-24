//
// Created by nakakura on 22/09/04.
//

#ifndef SKYWAY_FFI_H
#define SKYWAY_FFI_H

#include <ros/ros.h>

#include "router.h"

// C++側から呼び出されるRust側関数の定義
extern "C" {
struct PluginLoadResult {
  bool is_success;
  uint16_t port;
  const char* error_message;
};

using void_char_func = void (*)(char*);
using void_uint16_func = void (*)(uint16_t);
using void_double_func = void (*)(double);
using void_void_func = void (*)();
using bool_void_func = bool (*)();
using void_char_char_func = void (*)(char*, char*);
using void_char_func = void (*)(char*);
using void_void_func = void (*)();
using plugin_topicparam_func = PluginLoadResult (*)(char*, uint16_t, char*,
                                                    char*);

struct Function {
  void_char_char_func create_peer_callback;
  void_void_func peer_deleted_callback;
  plugin_topicparam_func create_data_callback;
  void_uint16_func data_connection_deleted_callback;
  void_char_func release_string_callback;
};

struct run_response_t {
  bool flag;
  void* handler;
};

void register_callbacks(Function& functions);
char* call_service(const char* message);
char* receive_events();
void release_string(char* message);
void create_peer_callback(char* peer_id, char* token);
void peer_deleted_callback();
PluginLoadResult create_data_callback(char* parameter);
void data_connection_close_event_callback(char* data_connection_id);

void register_logger(void_char_func debug, void_char_func info,
                     void_char_func warn, void_char_func error);
void register_program_state(bool_void_func is_running_c,
                            bool_void_func is_shutting_down_c,
                            void_double_func sleep_c,
                            void_void_func wait_for_shutdown_c,
                            void_void_func shutdown_c);
run_response_t run();
void join_handler(void* handler);

void print_string(char* message);
};

// Rust側から呼び出されるC++側関数の定義
extern "C" {
// loggers
void log_debug_c(char* message);
void log_info_c(char* message);
void log_warn_c(char* message);
void log_err_c(char* message);

// ros control functions
bool is_ok_c();
bool is_shutting_down_c();
void ros_sleep_c(double dur);
void wait_for_shutdown_c();
void shutdown_c();
};

#endif  // SKYWAY_FFI_H
