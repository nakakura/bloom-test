def create_peer_request(peer_id, api_key):
    return (
        "{"
        '  "request_type":"PEER",'
        '  "command":"CREATE",'
        '  "params":{'
        f'      "key":"{api_key}",'
        '      "domain":"localhost",'
        f'      "peer_id":"{peer_id}",'
        '      "turn":false'
        "  }"
        "}"
    )


def delete_peer_request(peer_id, token):
    return (
        "{"
        '   "request_type": "PEER", '
        '   "command": "DELETE", '
        '   "params": {'
        f'       "peer_id": "{peer_id}",'
        f'       "token": "{token}"'
        "   }"
        "}"
    )


def create_peer_status_request(peer_id, token):
    return (
        "{"
        '   "request_type": "PEER",'
        '   "command": "STATUS",'
        '   "params": {'
        f'       "peer_id": "{peer_id}", '
        f'       "token": "{token}"'
        "  }"
        "}"
    )
