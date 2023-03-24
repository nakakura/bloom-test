#!/usr/bin/env python3
# -*- coding: utf-8 -*-
import rospy
import signal
import subprocess
import threading
from subprocess import PIPE
import os
import psutil

from gstreamer_launcher.srv import *

exiting_thread = None

def exiting():
    rospy.sleep(0.2)
    rospy.signal_shutdown("finish")



def launcher(req):
    global exiting_thread

    if req.message_type == "LAUNCH":
        proc = subprocess.Popen(req.command, shell=True, restore_signals=True, stdout=PIPE, stderr=PIPE)
        try:
            res = proc.communicate(timeout=0.1)
        except subprocess.TimeoutExpired:
            return GStreamerLauncherResponse(True, proc.pid)

        return GStreamerLauncherResponse(False, -1)
    elif req.message_type == "EXIT":
        try:
            os.killpg(os.getpgid(req.pid), signal.SIGTERM)
        except ProcessLookupError:
            return GStreamerLauncherResponse(False, -1)

        return GStreamerLauncherResponse(True, 0)
    elif req.message_type == "SYSTEM_EXIT":
        # メッセージだけ返したらexitする
        exiting_thread = threading.Thread(target=exiting)
        exiting_thread.start()
        return GStreamerLauncherResponse(True, 0)

    return GStreamerLauncherResponse(False, -1)


def gst_launch_server():
    s = rospy.Service('gst_launch', GStreamerLauncher, launcher)
    rospy.spin()


def sigterm_handler(signal, frame):
    # SYSTEM_EXITを受け取らない限り何があっても落とさない
    # 単独で起動することはせず、roslaunch内でclientと一緒に起動すべき
    pass


def main():
    global exiting_thread
    gst_launch_server()
    if not exiting_thread is None:
        exiting_thread.join()


if __name__ == "__main__":
    rospy.init_node('gstreamer_launcher', log_level=rospy.INFO)
    signal.signal(signal.SIGTERM, sigterm_handler)
    signal.signal(signal.SIGINT, sigterm_handler)
    main()
