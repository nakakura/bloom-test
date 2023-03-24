//
// Created by nakakura on 22/09/08.
//

#ifndef SKYWAY_UDP_SOCKET_FUTURE_H
#define SKYWAY_UDP_SOCKET_FUTURE_H

#include <fruit/fruit.h>
#include <ros/ros.h>

#include <boost/asio.hpp>
#include <boost/asio/use_future.hpp>
#include <future>

#include "socket.h"

using boost::asio::ip::address;
using boost::asio::ip::udp;
using fruit::Component;
using fruit::createComponent;
using fruit::Injector;

class UdpSocket : public Socket {
 public:
  UdpSocket() = delete;
  INJECT(UdpSocket(
      ASSISTED(udp::endpoint) target_socket,
      ASSISTED(std::shared_ptr<std::function<void(std::vector<uint8_t>)>>)
          callback));
  ~UdpSocket();

  virtual void Start() override;
  virtual void Stop() override;
  virtual unsigned short Port() override;
  virtual void SendData(std::vector<uint8_t> vec) override;

 private:
  // 受信側パラメータ
  // 受信を継続することを示すフラグ。StartメソッドとStopメソッドで変更される
  bool is_running_ = false;
  // io_serviceが生成されたスレッドでなければ正しく動作しないため、
  // ソケットやその他の受信に必要なリソースは、基本的に全てこのスレッド内で生成され、
  // スレッド終了時に開放される
  std::unique_ptr<std::thread> recv_thread_{nullptr};
  // 受信したパケットのリダイレクト先
  std::shared_ptr<std::function<void(std::vector<uint8_t>)>> callback_;
  // socket本体はStartメソッド実行中にrecv_thread内で生成されるが、
  // Portメソッドでポート番号を返すため、インスタンス変数として保持し、
  // コンストラクタ内でgetFreePortを使って、0.0.0.0とランダムポート番号でアサインする
  udp::endpoint local_endpoint_{};

  // 送信側パラメータ
  // 送信はスレッド起動の必要がないので、コンストラクタ内で生成する
  // Start, Stopメソッドでの操作は受け付けず、デストラクタで開放される
  std::unique_ptr<boost::asio::io_service> send_io_service_{
      new boost::asio::io_service()};
  std::unique_ptr<udp::socket> send_socket_{
      std::make_unique<udp::socket>(*send_io_service_.get())};
  // このソケットの送信先はWebRTC Gatewayで固定なので、
  // コンストラクタでソケット情報を受けとり、そのまま保持する
  udp::endpoint target_socket_;
  // 送信結果のコールバック
  void send_handler(const boost::system::error_code &error, std::size_t len);
};

udp::endpoint getFreePort();

using SocketFactory = std::function<std::unique_ptr<Socket>(
    udp::endpoint, std::shared_ptr<std::function<void(std::vector<uint8_t>)>>)>;

fruit::Component<SocketFactory> getUdpSocketComponent();

#endif  // SKYWAY_UDP_SOCKET_FUTURE_H
