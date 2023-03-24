## Eventの取得

### Event Request
`skyway_events`サービスをコールすることでイベントを取得できます。
サービスをコールする際にパラメータは不要です。

### Event Response

**EventResponse**

| Field      | Type        | Description                                           |
|------------|-------------|-------------------------------------------------------|
| is_success | Boolean     | Eventが正常に取得できたかどうかを示します。                              |
| result     | EventResult | Peer, Data, Media 3種類のイベントが含まれます。イベントの詳細は各ページを参照して下さい |

- Peer
  - PeerObjectに関するイベントが格納されます
- [Data](./data_event.md)
  - DataConnectionに関するイベントが格納されます
- [Media](./media_event.md)
  - MediaConnectionに関するイベントが格納されます
