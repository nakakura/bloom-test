## DataConnection Event

**Create Peer Response**

| Field      | Type                | Description                    |
|------------|---------------------|--------------------------------|
| is_success | Boolean             | Eventの取得に成功したことを示します           |
| result     | DataConnectionEvent | DataConnectionに関するイベントの内容を示します |

**DataConnectionEvent(成功時)**

| Field              | Type   | Description                         |
|--------------------|--------|-------------------------------------|
| request_type       | String | `DATA`で固定です                         |
| command            | String | `EVENT`で固定です                        | 
| event              | String | イベントの内容を示します。 `OPEN`, `CLOSE`の2つです。 | 
| data_connection_id | String | DataConnectionを特定するためのIDです          |

**Peer Request Result(失敗時)**

resultフィールドに、エラー内容がJSONで格納されています。

例) OPENイベント
```json
{
  "is_success":true,
  "result":{
    "request_type":"DATA",
    "command":"EVENT",
    "event":"OPEN",
    "data_connection_id":"dc-cdf0eb1c-a057-4a28-8c19-d25a6926e521"
  }
}
```

例) CLOSEイベント
```json
{
  "is_success":true,
  "result":{
    "request_type":"DATA",
    "command":"EVENT",
    "event":"CLOSE",
    "data_connection_id":"dc-cdf0eb1c-a057-4a28-8c19-d25a6926e521"
  }
}
```

