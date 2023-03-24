#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import sys
import json
import rospy
import threading
import signal
import subprocess
from subprocess import PIPE
import os
from utils.peer import *
from utils.service_client import *
from utils.media import *


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
        if not has_media_config:
            rospy.logdebug("no media config. ignore media events")

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
            (result, pid) = gst_launch("SYSTEM_EXIT", "", 0)
            rospy.signal_shutdown("finish")
        elif event["result"]["event"] == "CALL":
            if not has_media_config:
                return

            media_connection_id = event["result"]["call_params"][
                "media_connection_id"
            ]
            answer_query = create_answer_query(media_connection_id)
            response = skyway_control(answer_query)

    def _data_event_react(self, event):
        # media系のサンプルなのでdata系の処理は省略
        pass

    def _media_event_react(self, event):
        if not event["is_success"]:
            return

        if event["result"]["event"] == "STREAM":
            # media connectionのstatusを取得
            media_status_request = create_media_status_request(
                event["result"]["media_connection_id"]
            )
            media_status_response = skyway_control(media_status_request)
            rospy.loginfo("MediaConnection Status")
            rospy.loginfo(media_status_response)

            # STREAMイベントが発火したあとはメディアの転送を行って良い
            # メディアの送信先情報に関しては、eventパラメータから取得できる。
            # このサンプルでは、取得した情報を用いて、config内のスクリプトを置き換え、gStreamerを起動している

            # スクリプトの取得
            script = rospy.get_param(rospy.get_name() + "/media/gst_script")

            # videoの送信先情報の取得
            video_send_params = parse_media_info(
                event["result"]["send_params"]["video"]["media"]
            )
            # video RTPの送信先ポートの情報をセット
            script = script.replace(
                "DEST_VIDEO_RTP_PORT", str(video_send_params[1])
            )
            # video RTCPの送信先ポートの情報をセット
            script = script.replace(
                "DEST_VIDEO_RTCP_PORT", str(video_send_params[3])
            )
            # videoの送信先はaudioと同じであるため最後に一回置換する

            # audioの送信先情報の取得
            audio_send_params = parse_media_info(
                event["result"]["send_params"]["audio"]["media"]
            )
            # audio RTPの送信先ポートの情報をセット
            script = script.replace(
                "DEST_AUDIO_RTP_PORT", str(audio_send_params[1])
            )
            # audio RTCPの送信先ポートの情報をセット
            script = script.replace(
                "DEST_AUDIO_RTCP_PORT", str(audio_send_params[3])
            )
            # audioの送信先はvideoと同じであるため最後に一回置換する

            # videoの受信ポートの取得
            # videoストリームはこのIPアドレスとポート番号に転送される
            video_recv_params = parse_media_info(
                event["result"]["redirect_params"]["video"]
            )
            # video RTPの受信ポートをセット
            script = script.replace(
                "SRC_VIDEO_RTP_PORT", str(video_recv_params[1])
            )
            # video RTCPの受信ポートをセット
            script = script.replace(
                "SRC_VIDEO_RTCP_PORT", str(video_recv_params[3])
            )

            # audioの受信ポートの取得
            # audioストリームはこのIPアドレスとポート番号に転送される
            audio_recv_params = parse_media_info(
                event["result"]["redirect_params"]["audio"]
            )
            # audio RTPの受信ポートをセット
            script = script.replace(
                "SRC_AUDIO_RTP_PORT", str(audio_recv_params[1])
            )
            # audio RTCPの受信ポートをセット
            script = script.replace(
                "SRC_AUDIO_RTCP_PORT", str(audio_recv_params[3])
            )

            # video, audioの送信先IPアドレスのセット
            # RTCPの送信先IPアドレスも取得している(Tupleの3つめ)が、今回の構成だと同じなので、スクリプト上で省略している
            script = script.replace("DEST", str(video_send_params[0]))

            # gStreamer起動サービスのコール
            (result, pid) = gst_launch("LAUNCH", script, 0)
            if result:
                self._pid_map[event["result"]["media_connection_id"]] = pid
        elif event["result"]["event"] == "CLOSE":
            media_connection_id = event["result"]["media_connection_id"]
            if media_connection_id in self._pid_map:
                pid = self._pid_map[media_connection_id]
                (result, pid) = gst_launch("EXIT", "", pid)
                self._pid_map.pop(media_connection_id)


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
