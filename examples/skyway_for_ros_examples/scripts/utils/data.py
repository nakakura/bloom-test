import json


def create_connect_request(
    peer_id,
    token,
    target_id,
    plugin_info,
    metadata,
    data_id,
    redirect_ip,
    redirect_port,
):
    message = json.loads(
        "{"
        '   "request_type":"DATA",'
        '   "command":"CONNECT",'
        '   "params": {'
        f'      "peer_id": "{peer_id}", '
        f'      "token": "{token}", '
        f'      "target_id": "{target_id}", '
        f'      "plugin_info": {json.dumps(plugin_info)}, '
        '       "options": {'
        '           "serialization": "NONE", '
        '           "dcInit": {'
        '               "ordered": false, '  # パケットの順番をソートさせない場合
        '               "maxRetransmits": 0 '  # リアルタイム通信を阻害しないよう再送させない場合
        "           }"
        "       },"
        '       "params": { '
        f'          "data_id": "{data_id}" '
        "       }, "
        '       "redirect_params": { '
        f'          "ip_v4": "{redirect_ip}", '
        f'          "port": {redirect_port}'
        "       }"
        "   }"
        "}"
    )

    message["params"]["options"]["metadata"] = json.dumps(metadata)
    return json.dumps(message)


def redirect_request(data_connection_id, plugin_info):
    return (
        "{"
        '   "request_type":"DATA",'
        '   "command":"REDIRECT",'
        '   "params":{'
        f'     "data_connection_id":"{data_connection_id}",'
        f'     "plugin_info":{json.dumps(plugin_info)}'
        "   }"
        "}"
    )


def create_data_status_request(data_connection_id):
    return (
        "{"
        '    "request_type":"DATA",'
        '    "command":"STATUS",'
        '    "params":{'
        f'       "data_connection_id":"{data_connection_id}"'
        "    }"
        "}"
    )
