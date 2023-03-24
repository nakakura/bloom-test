// SkyWay WebRTC Gateway Caller(以下Caller)を利用して、
// WebRTC Gateway内で発生したイベントを取得するためのクラス
// Callerから帰ってきたイベントを通知するためのROS Serviceを提供する
//
// クライアントから要求がくると、まず終了処理中ではないか確認を行う。
// 終了処理中でなければ、オブザーバに対して通知を行う。
// 通知を受けたオブザーバーは、Callerからイベントを取得して戻り値として返す。
// 戻り値を受け取ったらServiceとして返す。

// TODO: unit test

#ifndef SKYWAY_EVENT_SERVICE_H
#define SKYWAY_EVENT_SERVICE_H

#include <fruit/fruit.h>
#include <ros/ros.h>
#include <skyway/SkyWayEvents.h>

#include <functional>
#include <string>

class EventsService {
 public:
  virtual ~EventsService() = default;
  virtual void Shutdown() {}
};

class EventsServiceImpl : public EventsService {
 private:
  ros::NodeHandle nh_;
  ros::ServiceServer service_;
  std::string name_;
  std::function<std::string()> callback_;
  bool is_running_ = true;

  // エンドユーザプログラムから与えられたメッセージをCallerに与え、レスポンスをServiceのClientに返す
  bool callback(skyway::SkyWayEvents::Request &req,
                skyway::SkyWayEvents::Response &res);

 public:
  // コンストラクタでは、サービス名とCaller内のSender Objectを受け取る
  INJECT(EventsServiceImpl(ASSISTED(std::string) name,
                           ASSISTED(std::function<std::string()>) callback));
  ~EventsServiceImpl() {}
  virtual void Shutdown() override {
    is_running_ = false;
    service_.shutdown();
  }
};

using EventsServiceFactory = std::function<std::unique_ptr<EventsService>(
    std::string, std::function<std::string()>)>;

fruit::Component<EventsServiceFactory> getEventsServiceComponent();

#endif
