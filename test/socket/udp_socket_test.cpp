#include "../../src/socket/udp_socket.h"

#include <fruit/fruit.h>
#include <gtest/gtest.h>

#include <algorithm>
#include <boost/array.hpp>
#include <boost/asio.hpp>
#include <functional>
#include <iostream>
#include <iterator>
#include <memory>
#include <unordered_map>

#include "std_msgs/UInt8MultiArray.h"

using boost::asio::ip::address;
using boost::asio::ip::udp;
using fruit::Component;
using fruit::createComponent;
using fruit::Injector;

// UDPでデータを受信したらcallbackが発火する部分のテスト
TEST(TestSuite, socket_callback) {
  ros::NodeHandle nh;

  // callbackが発火したことを示すflag
  bool is_received = false;
  // 最終的に評価に使う、受信データを格納するvector
  std::vector<uint8_t> received_vec;

  // socketに渡すcallback
  auto func_ptr = std::make_shared<std::function<void(std::vector<uint8_t>)>>(
      [&](std::vector<uint8_t> vec) {
        // 受信データを格納し、フラグを立てる
        copy(vec.begin(), vec.end(), back_inserter(received_vec));
        is_received = true;
      });

  // objectを作成し、受信スレッドを開始
  Injector<SocketFactory> injector(getUdpSocketComponent);
  SocketFactory socketFactory(injector);
  // データは送信しないのでportは何でも良い
  auto source = socketFactory(udp::endpoint(udp::v4(), 0), func_ptr);
  source->Start();

  // 受信スレッドに対してデータを送信
  unsigned short port = source->Port();
  std::thread sending_thread([&] {
    ros::Rate wait_rate(10);
    wait_rate.sleep();
    boost::asio::io_service io_service;
    udp::resolver resolver(io_service);
    udp::endpoint receiver_endpoint(udp::v4(), port);

    udp::socket socket(io_service);
    socket.open(udp::v4());

    std::vector<uint8_t> cvec = {0, 1, 2, 3, 4};
    auto buf = boost::asio::buffer(cvec);
    socket.send_to(buf, receiver_endpoint);
  });

  // データの受け渡しが完了するまでspinする
  ros::Rate loop_rate(100);
  while (!ros::isShuttingDown() && !is_received) {
    ros::spinOnce();
    loop_rate.sleep();
  }

  // 受信スレッドの停止
  source->Stop();

  sending_thread.join();

  // 送信データはindexとvalueが一致した数列
  for (int i = 0; i < received_vec.size(); i++) {
    ASSERT_EQ(received_vec[i], i);
  }
}

// UDPでデータを受信したらcallbackが発火する部分のテスト
// 複数回受信できることを確認
TEST(TestSuite, socket_callback_twice) {
  ros::NodeHandle nh;

  // callbackが発火したことを示すflag
  bool is_received = false;
  // 最終的に評価に使う、受信データを格納するvector
  std::vector<uint8_t> received_vec;
  // 2回目でループを抜けるためのcounter
  int counter = 0;

  // socketに渡すcallback
  auto func_ptr = std::make_shared<std::function<void(std::vector<uint8_t>)>>(
      [&](std::vector<uint8_t> vec) {
        // 受信データを格納し、フラグを立てる
        counter += 1;
        if (counter == 2) {
          copy(vec.begin(), vec.end(), back_inserter(received_vec));
          is_received = true;
        }
      });

  // objectを作成し、受信スレッドを開始
  Injector<SocketFactory> injector(getUdpSocketComponent);
  SocketFactory socketFactory(injector);
  // データは送信しないのでportは何でも良い
  auto source = socketFactory(udp::endpoint(udp::v4(), 0), func_ptr);
  source->Start();

  // 受信スレッドに対してデータを送信
  unsigned short port = source->Port();
  std::thread sending_thread([&] {
    ros::Rate wait_rate(10);
    wait_rate.sleep();
    boost::asio::io_service io_service;
    udp::resolver resolver(io_service);
    udp::endpoint receiver_endpoint(udp::v4(), port);

    udp::socket socket(io_service);
    socket.open(udp::v4());

    std::vector<uint8_t> cvec = {0, 1, 2, 3, 4};
    auto buf = boost::asio::buffer(cvec);
    // 2回送信
    socket.send_to(buf, receiver_endpoint);
    socket.send_to(buf, receiver_endpoint);
  });

  // データの受け渡しが完了するまでspinする
  ros::Rate loop_rate(100);
  while (!ros::isShuttingDown() && !is_received) {
    ros::spinOnce();
    loop_rate.sleep();
  }

  // 受信スレッドの停止
  source->Stop();

  sending_thread.join();

  // 送信データはindexとvalueが一致した数列
  for (int i = 0; i < received_vec.size(); i++) {
    ASSERT_EQ(received_vec[i], i);
  }
}

// UDPでデータを受信したらcallbackが発火する部分のテスト
// 実際に利用する際にはunordered_mapに入れて使うので、入れた状態でテストする
TEST(TestSuite, socket_callback_in_map) {
  ros::NodeHandle nh;

  // callbackが発火したことを示すflag
  bool is_received = false;
  // 最終的に評価に使う、受信データを格納するvector
  std::vector<uint8_t> received_vec;

  // socketに渡すcallback
  auto func_ptr = std::make_shared<std::function<void(std::vector<uint8_t>)>>(
      [&](std::vector<uint8_t> vec) {
        // 受信データを格納し、フラグを立てる
        copy(vec.begin(), vec.end(), back_inserter(received_vec));
        is_received = true;
      });

  // objectを作成し、source objectをmapで管理
  // 受信スレッドを開始
  Injector<SocketFactory> injector(getUdpSocketComponent);
  SocketFactory socketFactory(injector);
  // データは送信しないのでportは何でも良い
  auto source = socketFactory(udp::endpoint(udp::v4(), 0), func_ptr);
  std::unordered_map<std::string, std::unique_ptr<Socket>> map;
  map.emplace("socket", std::move(source));
  map.at("socket")->Start();

  // 受信スレッドに対してデータを送信
  unsigned short port = map.at("socket")->Port();
  std::thread sending_thread([&] {
    ros::Rate wait_rate(10);
    wait_rate.sleep();
    boost::asio::io_service io_service;
    udp::resolver resolver(io_service);
    udp::endpoint receiver_endpoint(udp::v4(), port);

    udp::socket socket(io_service);
    socket.open(udp::v4());

    std::vector<uint8_t> cvec = {0, 1, 2, 3, 4};
    auto buf = boost::asio::buffer(cvec);
    socket.send_to(buf, receiver_endpoint);
  });

  // データの受け渡しが完了するまでspinする
  ros::Rate loop_rate(100);
  while (!ros::isShuttingDown() && !is_received) {
    ros::spinOnce();
    loop_rate.sleep();
  }

  // 受信スレッドの停止
  map.at("socket")->Stop();

  sending_thread.join();

  // 送信データはindexとvalueが一致した数列
  for (int i = 0; i < received_vec.size(); i++) {
    ASSERT_EQ(received_vec[i], i);
  }
}

// 外部から受信したデータの送信
TEST(TestSuite, socket_send_data) {
  ros::NodeHandle nh;

  // callbackが発火したことを示すflag
  bool is_received = false;
  // 最終的に評価に使う、受信データを格納するvector
  std::vector<uint8_t> received_vec;

  // 受信socketをrandomなポート番号でbind
  boost::asio::io_service io_service;
  udp::socket socket(io_service, udp::endpoint(udp::v4(), 0));
  // 受信用ポートの情報を取得
  auto endpoint = socket.local_endpoint();
  // 受信スレッド開始
  std::thread receiving_thread([&] {
    boost::array<uint8_t, 2048> recv_buf;
    udp::endpoint remote_endpoint;
    boost::system::error_code error;
    size_t len = socket.receive_from(boost::asio::buffer(recv_buf),
                                     remote_endpoint, 0, error);
    received_vec.clear();
    received_vec.insert(received_vec.end(), &recv_buf[0], &recv_buf[len]);

    is_received = true;
  });

  // 受信スレッド開始完了するまで待機
  // 100ms程度待てば十分
  ros::Rate loop_rate(100);
  loop_rate.sleep();

  // socketに渡すcallback
  // このテストでは呼ばれない
  auto func_ptr = std::make_shared<std::function<void(std::vector<uint8_t>)>>(
      [&](std::vector<uint8_t> vec) { ASSERT_TRUE(false); });

  // objectを作成し、受信スレッドを開始
  Injector<SocketFactory> injector(getUdpSocketComponent);
  SocketFactory socketFactory(injector);
  // 受信スレッドのportを与える
  auto source = socketFactory(endpoint, func_ptr);
  source->Start();
  std::vector<uint8_t> data{0, 1, 2, 3};
  source->SendData(data);

  // 非同期実行なので、データが確実に届くまで待機が必要
  loop_rate.sleep();
  source->Stop();

  receiving_thread.join();

  // 送信データはindexとvalueが一致した数列
  for (int i = 0; i < received_vec.size(); i++) {
    ASSERT_EQ(received_vec[i], i);
  }

  ASSERT_TRUE(is_received);
}

// 外部から受信したデータの送信
// 実際に利用する際にはunordered_mapに入れて使うので、入れた状態でテストする
TEST(TestSuite, socket_send_data_in_map) {
  ros::NodeHandle nh;

  // callbackが発火したことを示すflag
  bool is_received = false;
  // 最終的に評価に使う、受信データを格納するvector
  std::vector<uint8_t> received_vec;

  // 受信socketをrandomなポート番号でbind
  boost::asio::io_service io_service;
  udp::socket socket(io_service, udp::endpoint(udp::v4(), 0));
  // 受信用ポートの情報を取得
  auto endpoint = socket.local_endpoint();
  // 受信スレッド開始
  std::thread receiving_thread([&] {
    boost::array<uint8_t, 2048> recv_buf;
    udp::endpoint remote_endpoint;
    boost::system::error_code error;
    size_t len = socket.receive_from(boost::asio::buffer(recv_buf),
                                     remote_endpoint, 0, error);
    received_vec.clear();
    received_vec.insert(received_vec.end(), &recv_buf[0], &recv_buf[len]);

    is_received = true;
  });

  // 受信スレッド開始完了するまで待機
  // 100ms程度待てば十分
  ros::Rate loop_rate(100);
  loop_rate.sleep();

  // socketに渡すcallback
  // このテストでは呼ばれない
  auto func_ptr = std::make_shared<std::function<void(std::vector<uint8_t>)>>(
      [&](std::vector<uint8_t> vec) { ASSERT_TRUE(false); });

  // objectを作成し、受信スレッドを開始
  Injector<SocketFactory> injector(getUdpSocketComponent);
  SocketFactory socketFactory(injector);
  // 受信スレッドのportを与える
  auto source = socketFactory(endpoint, func_ptr);
  std::unordered_map<std::string, std::unique_ptr<Socket>> map;
  map.emplace("socket", std::move(source));
  map.at("socket")->Start();
  std::vector<uint8_t> data{0, 1, 2, 3};
  map.at("socket")->SendData(data);

  // 非同期実行なので、データが確実に届くまで待機が必要
  loop_rate.sleep();
  map.at("socket")->Stop();

  receiving_thread.join();

  // 送信データはindexとvalueが一致した数列
  for (int i = 0; i < received_vec.size(); i++) {
    ASSERT_EQ(received_vec[i], i);
  }

  ASSERT_TRUE(is_received);
}
