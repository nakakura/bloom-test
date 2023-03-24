import rospy


class PluginManager:
    _plugin_dictionary = {}
    _connected_plugin_dictionary = {}

    def __init__(self):
        if not rospy.has_param(rospy.get_name() + "/data"):
            return

        plugin_list = rospy.get_param(rospy.get_name() + "/data")
        for plugin in plugin_list:
            if "key" in plugin:
                self._plugin_dictionary[plugin["key"]] = plugin

    def on_connect(self, data_connection_id, key):
        if key in self._plugin_dictionary:
            plugin = self._plugin_dictionary.pop(key)
            self._connected_plugin_dictionary[data_connection_id] = plugin
            return plugin
        else:
            return None

    def on_disconnect(self, key):
        if key in self._connected_plugin_dictionary:
            plugin = self._connected_plugin_dictionary.pop(key)
            self._plugin_dictionary[plugin["key"]] = plugin
