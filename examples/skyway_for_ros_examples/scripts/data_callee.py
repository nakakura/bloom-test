#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import rospy
import threading
import signal
import os
from utils.peer import *
from utils.service_client import *
from utils.plugin import *
from utils.data import *


def signal_handler(peer_id, token):
    def internal(signal, frame):
        # delete the peer object to exit
        request = delete_peer_request(peer_id, token)
        response = skyway_control(request)

    return internal


class EventListener(threading.Thread):
    _peer_id = ""
    _token = ""
    _plugin_manager = None
    _is_running = True
    _pid_map = {}

    def __init__(self, peer_id, token):
        super().__init__()
        self._peer_id = peer_id
        self._token = token
        self._plugin_manager = PluginManager()

    def run(self):
        has_data_config = rospy.has_param(rospy.get_name() + "/data")
        if not has_data_config:
            rospy.logdebug("no data config. ignore data events")

        has_media_config = rospy.has_param(rospy.get_name() + "/media")

        while self._is_running:
            event = skyway_event(self._peer_id, self._token)

            if event is None:
                continue
            if "result" not in event:
                continue
            if "event" not in event["result"]:
                continue

            request_type = event["result"]["request_type"]
            if request_type == "PEER":
                self._peer_event_react(
                    event, has_data_config, has_media_config
                )
            elif request_type == "DATA":
                if has_data_config:
                    self._data_event_react(event)
            elif request_type == "MEDIA":
                if has_media_config:
                    self._media_event_react(event)

    def _peer_event_react(self, event, has_data_config, has_media_config):
        # このイベント発火後、データの転送を行っても良い
        if event["result"]["event"] == "OPEN":
            rospy.loginfo("you can send data now")
        elif event["result"]["event"] == "CLOSE":
            # Terminate this program when the release of the peer object is confirmed.
            self._is_running = False
            # SkyWay for ROSを落とす
            response = skyway_control(
                '{"request_type": "SYSTEM", "command": "SHUTDOWN"}'
            )
            rospy.signal_shutdown("finish")
        elif event["result"]["event"] == "CONNECTION":
            data_connection_id = event["result"]["data_params"][
                "data_connection_id"
            ]

            # DataConnectionのstatusを取得する
            # data_calleeではOPENイベントの中で行っているが、
            # SkyWayではCONNECTIONイベント発火時に既にDataConnectionが確立されているので、
            # この時点でstatusの取得が可能である
            data_connection_status_request = create_data_status_request(
                data_connection_id
            )
            data_connection_status_response = skyway_control(
                data_connection_status_request
            )
            rospy.loginfo("DataConnection Status")
            rospy.loginfo(data_connection_status_response)

            # データの受信設定を行う
            # configファイルで与えられていない場合はスキップする
            if not has_data_config:
                return

            if "metadata" not in event["result"]["status"]:
                return

            metadata = json.loads(event["result"]["status"]["metadata"])
            if metadata and "connection_id" in metadata:
                plugin_id = metadata["connection_id"]
                plugin_info = self._plugin_manager.on_connect(
                    data_connection_id, plugin_id
                )
                if plugin_info:
                    message = redirect_request(data_connection_id, plugin_info)
                    response = skyway_control(message)

    def _data_event_react(self, event):
        if not event["is_success"]:
            return

        if event["result"]["event"] == "CLOSE":
            self._plugin_manager.on_disconnect(
                event["result"]["data_connection_id"]
            )

    def _media_event_react(self, event):
        # data系のサンプルなので省略
        pass


def main():
    if not rospy.has_param(rospy.get_name() + "/peer_id"):
        rospy.logerr("no peer_id")
        exit(0)
    peer_id = rospy.get_param(rospy.get_name() + "/peer_id")

    if "API_KEY" not in os.environ:
        rospy.logerr("no API_KEY")
        rospy.logerr("exiting")
        exit(0)

    key = os.environ["API_KEY"]
    request = create_peer_request(peer_id, key)
    peer_create_response = skyway_control(request)

    if not peer_create_response["is_success"]:
        return
    else:
        # succeed to create a peer object
        peer_id = peer_create_response["result"]["peer_id"]
        token = peer_create_response["result"]["token"]

        # Peer Statusのチェックをする場合
        # 接続済みなので、disconnectedはFalseになっているのが正しい
        status_request = create_peer_status_request(peer_id, token)
        status_response = skyway_control(status_request)
        rospy.loginfo("Peer Object has been created")
        rospy.loginfo(status_response)

        # hook ctrl-c to delete the peer object when exiting
        signal.signal(signal.SIGINT, signal_handler(peer_id, token))

        # this program only reacts to events
        event_thread = EventListener(peer_id, token)
        event_thread.start()

        rospy.spin()
        event_thread.join()


if __name__ == "__main__":
    rospy.init_node("my_node", log_level=rospy.INFO)
    main()
