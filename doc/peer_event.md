## PeerEvent

**PeerEvent Response**

| Field        | Type                                             | Description                       |
|--------------|--------------------------------------------------|-----------------------------------|
| is_success   | Boolean                                          | Eventの取得に成功したことを示します              |
| result       | PeerConnectionEvent/PeerCallEvent/PeerCloseEvent | PeerObjectに関するイベントの内容を示します        |

**PeerConnectionEvent(成功時)**

| Field        | Type       | Description                |
|--------------|------------|----------------------------|
| request_type | String     | `PEER`で固定です                |
| command      | String     | `EVENT`で固定です               | 
| event        | String     | `CONNECTION`で固定です          | 
| params       | PeerInfo   | 対象のPeerObjectを特定するための情報です  |
| data_params  | DataParams | DataConnectionを特定するための情報です |
| status       | PeerStatus | DataConnectionのステータスを示します  |

**PeerCallEvent(成功時)**

| Field        | Type       | Description                 |
|--------------|------------|-----------------------------|
| request_type | String     | `PEER`で固定です                 |
| command      | String     | `EVENT`で固定です                | 
| event        | String     | `CALL`で固定です                 | 
| params       | PeerInfo   | 対象のPeerObjectを特定するための情報です   |
| call_params  | CallParams | MediaConnectionを特定するための情報です |

**PeerCloseEvent(成功時)**

| Field        | Type       | Description                 |
|--------------|------------|-----------------------------|
| request_type | String     | `PEER`で固定です                 |
| command      | String     | `EVENT`で固定です                | 
| event        | String     | `CLOSE`で固定です                | 
| params       | PeerInfo   | 対象のPeerObjectを特定するための情報です   |

PeerObjectが削除されたことを示します。
PeerObjectが削除される時点で、そのPeerが利用していたDataConnectionやMediaConnectionなどのリソースも開放されているため、
この時点でプログラムの終了が可能です。

**PeerInfo**

| Field   | Type    | Description                                    |
|---------|---------|------------------------------------------------|
| peer_id      | String | PeerObjectとして登録されたPeerIdです              |
| token        | String | PeerObjectを利用するための識別キーとして利用するためのTokenです |


**DataParams**

| Field              | Type    | Description                     |
|--------------------|---------|---------------------------------|
| data_connection_id | String | 確立されたDataConnectionを特定するためのIDです |

SkyWayではDataConnectionは確立要求があった時点で確立されるため、既に接続済みの状態です。

**CallParams**

| Field               | Type    | Description                         |
|---------------------|---------|-------------------------------------|
| media_connection_id | String | 確立要求のあったMediaConnectionを特定するためのIDです |

SkyWayでMediaConnectionは確立要求に対して応答してから確立されるため、
イベント受信時点ではまだ確立されていません。

**PeerStatus**

| Field         | Type    | Description                                                                                             |
|---------------|---------|---------------------------------------------------------------------------------------------------------|
| remote_id     | String  | 接続要求を送信してきたPeerのIDです                                                                                    |
| buffersize    | String  | WebRTC側で設定されているデータのバッファサイズです                                                                            |
| label         | String  | DataConnection確立時に、WebRTC実装が設定したラベルです                                                                   
| metadata      | String  | DataConnection確立要求を出す際に、metadataとして指定した値が取得できます                                                         |
| open          | Boolean | DataConnectionが通信可能な状態かどうかを示します                                                                         |
| serialization | String  | `BINARY`, `JSON`, `NONE`の3種類です。SkyWay for ROSの場合`NONE`であるべきですが、接続要求側で指定した値になります。`NONE`でない場合は正常に通信できません。 |
| reliable      | Boolean | 通信経路上でパケットロスがあった場合、再送されるかどうかを示します                                                                       |
| type          | String  | `DATA`で固定です                                                                                             |

[DataConnection Event](./data_event.md)内の`OPEN`イベントが発火するまで実際に通信可能ではないため、
この時点では正確に取得できていない値があります。

**PeerEvent(失敗時)**

resultフィールドに、エラー内容がJSONで格納されています。


例) CONNECTイベント
```json
{
  "is_success":true,
  "result":{
    "request_type":"PEER",
    "command":"EVENT",
    "event":"CONNECTION",
    "params":{
      "peer_id":"foo",
      "token":"pt-16c821b6-1799-4fdf-ad91-7dcdb81b2a0e"
    },
    "data_params":{
      "data_connection_id":"dc-c083180f-71e1-4f2b-9615-f52ef4fe91f4"
    },
    "status":{
      "remote_id":"target_id",
      "buffersize":0,
      "label":"dc_gr9oitwazkg",
      "metadata":"{\n    \"connection_id\": \"string_connections\"\n}",
      "open":false,
      "reliable":false,
      "serialization":"NONE",
      "type":"DATA"
    }
  }
}
```

例) CALLイベント
```json
{
  "is_success":true,
  "result":{
    "request_type":"PEER",
    "command":"EVENT",
    "event":"CALL",
    "params":{
      "peer_id":"media_callee",
      "token":"pt-f8b793b9-c5e0-45d2-987a-627494b67758"
    },
    "call_params":{
      "media_connection_id":"mc-c2313f1e-1530-4018-8768-13a6415ad81c"
    }
  }
}
```

例) CLOSEイベント

```json
{
  "is_success":true,
  "result":{
    "request_type":"PEER",
    "command":"EVENT",
    "event":"CLOSE",
    "params":{
      "peer_id":"media_callee",
      "token":"pt-f8b793b9-c5e0-45d2-987a-627494b67758"
    }
  }
}
```
