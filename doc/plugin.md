## Pluginの構成

PluginはBinary, String, JSONの3種類を定義しています。
基本的な構成は同じで、SkyWay for ROS本体から渡されるデータのフォーマットのみが異なります。

1つのDataConnectionに対して、Binary, String, JSONいずれかの種類を決定した上で、同種のPluginは複数ロードすることができます。
そのため例えば、センサデータをTopicで受信するPluginと、移動制御モジュールをActionで操作するPluginをロードし、
同じDataConnectionを介してPeerに対して複数種類のデータ送受信を行うことができます。

一方で、1つのPeerObjectが複数のDataConnectionを確立させることも可能なため、別々のDataConnectionにそれぞれ1つずつのPluginをロードすることも可能です。

具体的なPluginの定義は以下の通りです。

```C++
class SkyWayBinaryPlugin {
 public:
  virtual void Initialize(
      std::shared_ptr<rapidjson::Document> parameter,
      std::shared_ptr<std::function<void(std::vector<uint8_t>)>> callback) = 0;
  virtual void Execute(std::vector<uint8_t> data) = 0;
  virtual void Shutdown() = 0;
  virtual ~SkyWayBinaryPlugin() {}

 protected:
  SkyWayBinaryPlugin() {}
};

class SkyWayStringPlugin {
 public:
  virtual void Initialize(
      std::shared_ptr<rapidjson::Document> parameter,
      std::shared_ptr<std::function<void(std::string)>> callback) = 0;
  virtual void Execute(std::string data) = 0;
  virtual void Shutdown() = 0;
  virtual ~SkyWayStringPlugin() {}

 protected:
  SkyWayStringPlugin() {}
};

class SkyWayJsonPlugin {
 public:
  virtual void Initialize(
      std::shared_ptr<rapidjson::Document> parameter,
      std::shared_ptr<std::function<void(std::shared_ptr<rapidjson::Document>)>>
          callback) = 0;
  virtual void Execute(std::shared_ptr<rapidjson::Document> document) = 0;
  virtual void Shutdown() = 0;
  virtual ~SkyWayJsonPlugin() {}

 protected:
  SkyWayJsonPlugin() {}
};
```

それぞれのPluginのサンプルは[pluginディレクトリ](../src/plugin/samples)にあります。

### Initializeメソッド
Pluginのロード時に呼ばれる初期化用メソッドです。
引数として、parameter, callbackをとります。

parameterは[DataConnection](./data_connect.md), [DataRedirect](./data_redirect.md)の際にplugin_infoとして`skyway_control`に渡したJSONです。

callbackは相手側にデータを転送するためのコールバック関数です。BinaryPluginでは`std::vector<uint8_t>`, StringPluginでは`std::string`, JsonPluginでは`rapidjson::Document`を渡すことで、Peerに対してデータが送信されます。

### Executeメソッド
PeerからDataConnectionを経由してデータを受信した際、Pluginに対して受け渡すために呼ばれるメソッドです。
BinaryPluginでは`std::vector<uint8_t>`, StringPluginでは`std::string`, JsonPluginでは`rapidjson::Document`が引数として与えられます。

このメソッドは同期的に呼ばれるため、このメソッド内で負荷のかかる処理は行わないでください。
別のPlugin呼び出しや新規のデータ受信に遅れが生じることになります。

### Shutdownメソッド
Pluginが削除される際に呼ばれます。
Plugin内で起動したスレッドやServiceなどの停止処理を行うために利用できます。

