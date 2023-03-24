## tips

### 映像の伝送

ROS内部だけで映像の転送を行いたい場合、 binary pluginを利用してDataConnectionで転送が可能です。
但し、ROS及びDataConnectionの性能により、広帯域な映像の伝送が必要な場合この方法は推奨しません。
gStreamer等の外部アプリケーションでRTPストリームを生成し、MediaConnectionで転送して下さい。

### WebRTC Gatewayが起動できない

WebRTC Gateway起動時に以下のようなエラーが表示されることがあります。

```shell
$ ./gateway
set port to 8000, log_level to 1
e/SkyWay++:  rest fail: bind: Address already in use
```

WebRTC Gatewayとの通信中にWebRTC Gatewayを終了した場合など、
前回起動時にWebRTC Gatewayが正常に終了できていない場合に生じます。
PEER CLOSEを確認してくから終了するようにして下さい。

WebRTC Gatewayの起動ポート番号を変更する場合、同じディレクトリに[config.toml](https://github.com/skyway/skyway-webrtc-gateway/blob/master/config.toml)ファイルを生成し、ファイル内でポート番号を指定して下さい。



