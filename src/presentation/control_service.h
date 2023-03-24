// SkyWay WebRTC Gateway Caller(以下Caller)を利用して、
// WebRTC Gatewayを操作するためのクラス
// Observerとしてstd::functionを持つ
// Serviceを起動してエンドユーザプログラムからJSONを受け取り、Observerに通知する。
// Observerからの戻り値はそのままClientに返す
//
// このクラスはプログラム終了時以外に開放することを想定していないので、
// このクラス内で起動したServiceはこのクラス内では停止処理は行わない。
// ROS側の開放処理に委ねる

// TODO: unit test

#ifndef SKYWAY_CONTROL_SERVICE_H
#define SKYWAY_CONTROL_SERVICE_H

#include <fruit/fruit.h>
#include <ros/ros.h>
#include <skyway/SkyWayControl.h>

#include <functional>
#include <string>

class ControlService {
 public:
  virtual ~ControlService() = default;
  virtual void Shutdown() {}
};

class ControlServiceImpl : public ControlService {
 private:
  ros::NodeHandle nh_;
  ros::ServiceServer service_;
  std::string name_;
  std::function<std::string(std::string)> callback_;
  bool is_running_ = true;

  // エンドユーザプログラムから与えられたメッセージをCallerに与え、レスポンスをServiceのClientに返す
  bool callback(skyway::SkyWayControl::Request &req,
                skyway::SkyWayControl::Response &res);

 public:
  // コンストラクタでは、サービス名とCaller内のSender Objectを受け取る
  INJECT(ControlServiceImpl(ASSISTED(std::string) name,
                            ASSISTED(std::function<std::string(std::string)>)
                                callback));
  ~ControlServiceImpl() {}
  virtual void Shutdown() override {
    is_running_ = false;
    service_.shutdown();
  }
};

using ControlServiceFactory = std::function<std::unique_ptr<ControlService>(
    std::string, std::function<std::string(std::string)>)>;

fruit::Component<ControlServiceFactory> getControlServiceComponent();

#endif
