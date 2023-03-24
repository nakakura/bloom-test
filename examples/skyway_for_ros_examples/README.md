# SkyWay for ROS Examples

SkyWay for ROSを利用し、ROSをインストールしたロボット用PC(以下、ROS側PC)から別PC(以下、Web側PC)に接続するためのサンプルです。

- ROS側PCからWeb側PCへの接続
  - 音声・動画
  - データ
- Web側PCからROS側PCへの接続
  - 音声・動画
  - データ

の4通りのサンプルがあります。

## 事前準備
### Web側PC

[examples_web](./examples_web)の中にあるWeb側プログラムを起動します。

1. [SkyWay](https://skyway.ntt.com/ja/)のAPIキーを取得する
2. [examples_web/_shared/key.js](./examples_web/_shared/key.js)を編集し、APIキーを記載する
3. examples_webディレクトリ内でWebサービスを起動する(例: python3 -m http.server 8000等)
4. Google ChromeなどのPCでhttp://localhost:8000に接続する

### ROS側PC
1. [gstreamer_launcher](https://github.com/ntt-t3/gstreamer_launcher)及びこのリポジトリをcatkinのsrcディレクトリに配置し、noetic, melodicの必要なツリーにスイッチする。
2. [WebRTC Gateway](https://github.com/skyway/skyway-webrtc-gateway/releases/tag/0.4.1)をダウンロードし、起動する
3. catkin_makeを実行する
4. 依存ライブラリのインストールを行う
```
sudo apt-get install libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev libgstreamer-plugins-bad1.0-dev gstreamer1.0-plugins-base gstreamer1.0-plugins-good gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly gstreamer1.0-libav gstreamer1.0-tools gstreamer1.0-x gstreamer1.0-alsa gstreamer1.0-gl gstreamer1.0-gtk3 gstreamer1.0-pulseaudio
```

## 実行
### ROS側PCからWeb側PCへの接続
#### 音声・動画
1. Web側PCにカメラとマイクを接続する
2. p2p-media/index.htmlを開く
3. カメラとマイクの使用権限について確認ダイアログが表示されるので、承認する
4. 画面に表示されるYour IDを確認する
5. ROS側PCでAPI_KEYをexportする (例: `export API_KEY=XXXXXX`)
6. ROS側PCで、media_caller.launchを起動する(`roslaunch skyway_for_ros_examples media_caller.launch`)
7. ブラウザ上の画面に映像が表示される

#### データ
1. Web側PCでp2p-data/index.htmlを開く
2. 画面に表示されるYour IDを確認する
3. ROS側PCでAPI_KEYをexportする (例: `export API_KEY=XXXXXX`)
4. ROS側PCで、data_caller.launchを起動する(`roslaunch skyway_for_ros_examples data_caller.launch`)
5. [config/caller/data.yaml](./config/caller/data.yaml)内で `string_loopback::StringLoopback` を呼び出す設定をしているため、ブラウザ側のチャット欄から文字列を送信すると、エコーバックされる

### Web側PCからROS側PCへの接続
#### 音声・動画
1. ROS側PCでAPI_KEYをexportする (例: `export API_KEY=XXXXXX`)
2. ROS側PCで、media_callee.launchを起動する(`roslaunch skyway_for_ros_examples media_callee.launch`)
3. Web側PCにカメラとマイクを接続する
4. p2p-media/index.htmlを開く
5. カメラとマイクの使用権限について確認ダイアログが表示されるので、承認する
6. `Remote Peer ID` 欄に `media_callee` と記入しCallボタンを押す
7. ブラウザ上の画面に映像が表示される

#### データ
1. ROS側PCでAPI_KEYをexportする (例: `export API_KEY=XXXXXX`)
2. ROS側PCで、data_callee.launchを起動する(`roslaunch skyway_for_ros_examples data_callee.launch`)
3. Web側PCでp2p-data/index.htmlを開く
4. `Remote Peer ID` 欄に `data_callee` と記入しCallボタンを押す
5. [config/caller/data.yaml](./config/caller/data.yaml)内で `string_loopback::StringLoopback` を呼び出す設定をしているため、ブラウザ側のチャット欄から文字列を送信すると、エコーバックされる
