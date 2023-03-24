#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import rospy
import threading
import signal
import os
from utils.peer import *
from utils.data import *
from utils.service_client import *


def signal_handler(peer_id, token):
    def internal(signal, frame):
        # delete the peer object to exit
        request = delete_peer_request(peer_id, token)
        response = skyway_control(request)

    return internal


class EventListener(threading.Thread):
    _peer_id = ""
    _token = ""
    _is_running = True
    _pid_map = {}

    def __init__(self, peer_id, token):
        super().__init__()
        self._peer_id = peer_id
        self._token = token

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
        if event["result"]["event"] == "CLOSE":
            # Terminate this program when the release of the peer object is confirmed.
            self._is_running = False
            # SkyWay for ROSを落とす
            response = skyway_control(
                '{"request_type": "SYSTEM", "command": "SHUTDOWN"}'
            )
            # gStreamer Controllerを落とす
            rospy.signal_shutdown("finish")

    def _data_event_react(self, event):
        if not event["is_success"]:
            return

        # このイベント発火後、データの転送を行っても良い
        if event["result"]["event"] == "OPEN":
            # DataConnectionのstatus取得
            data_connection_status_request = create_data_status_request(
                event["result"]["data_connection_id"]
            )
            data_connection_status_response = skyway_control(
                data_connection_status_request
            )
            rospy.loginfo("DataConnection Status")
            rospy.loginfo(data_connection_status_response)

            rospy.loginfo("you can send data now")

        if event["result"]["event"] == "CLOSE":
            rospy.loginfo(
                f"DataConnection {event['result']['data_connection_id']} disconnected"
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

        # DataConnectionの確立を開始する
        message = create_connect_request(
            peer_id,
            token,
            "target_id",
            {
                "type": "string",
                "plugins": [
                    {"plugin_name": "string_loopback::StringLoopback"}
                ],
            },
            json.loads('{"foo": "bar"}'),
            "da-50a32bab-b3d9-4913-8e20-f79c90a6a211",
            "127.0.0.1",
            10000,
        )
        response = skyway_control(message)

        # hook ctrl-c to delete the peer object when exiting
        signal.signal(signal.SIGINT, signal_handler(peer_id, token))

        event_thread = EventListener(peer_id, token)
        event_thread.start()

        rospy.spin()
        event_thread.join()


if __name__ == "__main__":
    rospy.init_node("my_node", log_level=rospy.INFO)
    main()
