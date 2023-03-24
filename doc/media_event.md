## MediaConnection Event

**MediaConnectionEvent**

| Field      | Type                       | Description                    |
|------------|----------------------------|--------------------------------|
| is_success | Boolean                    | Eventの取得に成功したことを示します           |
| result     | MediaConnectionEventResult | DataConnectionに関するイベントの内容を示します |

**MediaConnectionEventResult(成功時)**

| Field               | Type                | Description                                      |
|---------------------|---------------------|--------------------------------------------------|
| request_type        | String              | `MEDIA`で固定です                                     |
| command             | String              | `EVENT`で固定です                                     | 
| event               | String              | イベントの内容を示します。 `READY`, `CLOSE`の2つです。             | 
| send_params         | MediaSendParams     | このParamに含まれるポートにMediaを送信すると、相手側PeerにMediaが転送されます |
| redirect_params     | MediaRedirectParams | 相手側Peerから受信したMediaがこのポートに転送されます                  |
| media_connection_id | String              | MediaConnectionを特定するためのIDです                      |

**MediaConnectionEventResult(失敗時)**

resultフィールドに、エラー内容がJSONで格納されています。

**MediaSendParams**

| Field   | Type       | Description                |
|---------|------------|----------------------------|
| video   | MediaPrams | videoに関する情報を含みます           |
| audio   | MediaPrams | audioに関する情報を含みます           |

**MediaSendParams**

| Field | Type      | Description      |
|-------|-----------|------------------|
| media | MediaInfo | mediaに関する情報を含みます |
| rtcp  | RtcpInfo  | rtcpに関する情報を含みます  |

**MediaInfo**

| Field    | Type             | Description       |
|----------|------------------|-------------------|
| media_id | String           | Mediaを特定するためのIDです |
| ip_v4    | String(optional) | IPアドレスです          |
| ip_v6    | String(optional) | IPアドレスです          |
| port     | Integer          | ポート番号です     |

**RtcpInfo**

| Field   | Type             | Description      |
|---------|------------------|------------------|
| rtcp_id | String           | RTCPを特定するためのIDです |
| ip_v4   | String(optional) | IPアドレスです         |
| ip_v6   | String(optional) | IPアドレスです         |
| port    | Integer          | ポート番号です          |

**MediaRedirectParams**

| Field      | Type       | Description           |
|------------|------------|-----------------------|
| video      | SocketInfo | videoに関する情報を含みます      |
| video_rtcp | SocketInfo | video用rtcpに関する情報を含みます |
| audio      | SocketInfo | audioに関する情報を含みます      |
| audio_rtcp | SocketInfo | audio用rtcpに関する情報を含みます |

**SocketInfo**

| Field   | Type             | Description      |
|---------|------------------|------------------|
| ip_v4   | String(optional) | IPアドレスです         |
| ip_v6   | String(optional) | IPアドレスです         |
| port    | Integer          | ポート番号です          |


例) OPENイベント
```json
{
  "is_success":true,
  "result":{
    "request_type":"MEDIA",
    "command":"EVENT",
    "event":"READY",
    "send_params":{
      "video":{
        "media":{
          "media_id":"vi-83c6414f-c0e3-4223-96b4-c5145fc4cb28",
          "ip_v4":"192.168.10.123",
          "port":50001
        },
        "rtcp":{
          "rtcp_id":"rc-f285e8a3-9ac7-433f-8c0d-d0af6e1e268f",
          "ip_v4":"192.168.10.123",
          "port":50002}
      },
      "audio":{
        "media":{
          "media_id":"au-47d7e85d-daa9-4963-bfe6-3a9bd875bcca",
          "ip_v4":"192.168.10.123",
          "port":50003
        },
        "rtcp":{
          "rtcp_id":"rc-5ed37fa7-983e-43b1-99e3-c1400c4c9dfa",
          "ip_v4":"192.168.10.123",
          "port":50004
        }
      }
    },
    "redirect_params":{
      "video":{
        "ip_v4":"127.0.0.1",
        "port":35009
      },
      "video_rtcp":{
        "ip_v4":"127.0.0.1",
        "port":49979
      },
      "audio":{
        "ip_v4":"127.0.0.1",
        "port":54267
      },
      "audio_rtcp":{
        "ip_v4":"127.0.0.1",
        "port":41123
      }
    },
    "media_connection_id":"mc-499d2313-d8eb-400f-9b3c-dc3b8ec4e7bb"
  }
}
```

例) CLOSEイベント
```json
{
  "is_success":true,
  "result":{
    "request_type":"MEDIA",
    "command":"EVENT",
    "event":"CLOSE",
    "media_connection_id":"mc-499d2313-d8eb-400f-9b3c-dc3b8ec4e7bb"
  }
}
```

