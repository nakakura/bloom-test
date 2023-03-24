#include "events_service.h"

using fruit::Component;
using fruit::createComponent;
using fruit::Injector;

// エンドユーザプログラムから与えられたメッセージをCallerに与え、レスポンスをServiceのClientに返す
bool EventsServiceImpl::callback(skyway::SkyWayEvents::Request &req,
                                 skyway::SkyWayEvents::Response &res) {
  res.response = callback_();
  return true;
}

// コンストラクタでは、サービス名とCaller内のSender Objectを受け取る
EventsServiceImpl::EventsServiceImpl(ASSISTED(std::string) name,
                                     ASSISTED(std::function<std::string()>)
                                         callback)
    : name_(name), callback_(callback) {
  service_ = nh_.advertiseService(name, &EventsServiceImpl::callback, this);
}

Component<EventsServiceFactory> getEventsServiceComponent() {
  return createComponent().bind<EventsService, EventsServiceImpl>();
}
