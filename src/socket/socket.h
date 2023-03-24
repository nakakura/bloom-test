//
// Created by nakakura on 22/09/02.
//

#ifndef SKYWAY_SOCKET_H
#define SKYWAY_SOCKET_H

class Socket {
 private:
 public:
  virtual ~Socket() = default;
  // WebRTC GWからのデータ受信とPublishを開始する
  // 重複コールは許容するが、stop後の再開はサポートしない(未定義動作)
  virtual void Start() {}
  // WebRTC GWからのデータ受信とPublishを停止する
  // 重複コールは許容する
  virtual void Stop() {}
  // 内部で保持しているソケットのポート番号を取得する
  virtual unsigned short Port() { return 0; }
  // socketからデータを送信する
  virtual void SendData(std::vector<uint8_t>) {}
};

#endif  // SKYWAY_SOCKET_H
