#include <ros/ros.h>

#include "ffi.h"
#include "ffi_bridge.h"
#include "presentation/control_service.h"
#include "router.h"

int main(int argc, char** argv) {
  // 日本語を出力する場合のため
  setlocale(LC_CTYPE, "ja_JP.UTF-8");
  ros::init(argc, argv, "skyway");
  ros::AsyncSpinner spinner(4);
  spinner.start();

  // Rust側からC++側の関数を呼び出すためのセッティング
  register_logger(log_debug_c, log_info_c, log_warn_c, log_err_c);
  register_program_state(is_ok_c, is_shutting_down_c, ros_sleep_c,
                         wait_for_shutdown_c, shutdown_c);
  // Rust側の処理開始
  run_response_t response = run();

  if (response.flag) {
    // Rust側の処理が正常に開始した
    ROS_DEBUG("program starts successfully");
    Injector<FfiBridge> apiInjector(getFfiComponent);
    std::shared_ptr<FfiBridge> ffi =
        apiInjector.get<std::shared_ptr<FfiBridge>>();

    //メインスレッドはここで止めてあとはSeviceとActionからの処理を待ち受け続ける
    ros::waitForShutdown();
  } else {
    // Rust側の処理が正常に開始しなかった
    // registerを忘れているケースなので通常発生しない
    ROS_ERROR("some errors occurred");
  }

  spinner.stop();
  ros::shutdown();
  // Rust側のthreadが終了するまで待機する
  if (response.flag) {
    ROS_DEBUG("Waiting for Rust side thread to finish");
    join_handler(response.handler);
  }

  return 0;
}
