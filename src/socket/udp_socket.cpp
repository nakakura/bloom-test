//
// Created by nakakura on 22/09/08.
//

#include "udp_socket.h"

UdpSocket::UdpSocket(
    udp::endpoint target_socket,
    std::shared_ptr<std::function<void(std::vector<uint8_t>)>> callback)
    : target_socket_(target_socket), callback_(callback) {
  local_endpoint_ = getFreePort();
  send_socket_->open(udp::v4());
}

UdpSocket::~UdpSocket() {
  send_socket_->cancel();
  send_io_service_->stop();
  send_socket_->close();
}

void UdpSocket::Start() {
  if (is_running_) return;
  is_running_ = true;

  recv_thread_.reset(new std::thread([&] {
    boost::asio::io_service io_service;
    boost::asio::io_service::work work(io_service);
    std::thread io_thread([&io_service]() { io_service.run(); });

    udp::socket socket(io_service, local_endpoint_);
    udp::endpoint remote_endpoint;

    while (ros::ok() && is_running_) {
      std::vector<unsigned char> vec(1500, 0);
      std::future<std::size_t> recv_length;

      recv_length =
          socket.async_receive_from(boost::asio::buffer(vec), remote_endpoint,
                                    0, boost::asio::use_future);

      if (recv_length.wait_for(std::chrono::milliseconds(100)) ==
          std::future_status::timeout) {
        socket.cancel();
      } else {
        vec.resize(recv_length.get());
        (*callback_)(vec);
      }
    }

    io_service.stop();
    io_thread.join();
  }));
}

void UdpSocket::Stop() {
  if (!is_running_) return;

  is_running_ = false;
  recv_thread_->join();
}

unsigned short UdpSocket::Port() { return local_endpoint_.port(); }

void UdpSocket::send_handler(const boost::system::error_code &error,
                             std::size_t len) {
  if (error.value() != boost::system::errc::success) {
    ROS_WARN("fail to send data. %s", error.message().c_str());
  }
}

void UdpSocket::SendData(std::vector<uint8_t> vec) {
  send_socket_->async_send_to(
      boost::asio::buffer(vec), target_socket_,
      boost::bind(&UdpSocket::send_handler, this,
                  boost::asio::placeholders::error,
                  boost::asio::placeholders::bytes_transferred));
}

udp::endpoint getFreePort() {
  boost::asio::io_service service;
  udp::socket socket(service);
  socket.open(udp::v4());
  socket.bind(udp::endpoint(udp::v4(), 0));
  return socket.local_endpoint();
}

Component<SocketFactory> getUdpSocketComponent() {
  return createComponent().bind<Socket, UdpSocket>();
}
