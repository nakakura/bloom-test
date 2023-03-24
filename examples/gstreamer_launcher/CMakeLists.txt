cmake_minimum_required(VERSION 3.0.2)
project(gstreamer_launcher)

find_package(catkin REQUIRED COMPONENTS
  roscpp
  rospy
  std_msgs
  message_generation
)

################################################
## Declare ROS messages, services and actions ##
################################################

add_service_files(
   FILES
   GStreamerLauncher.srv
)

generate_messages(
  DEPENDENCIES
  std_msgs
)

###################################
## catkin specific configuration ##
###################################

catkin_package(
#  INCLUDE_DIRS include
#  LIBRARIES gstreamer_launcher
#  CATKIN_DEPENDS rospy std_msgs
#  DEPENDS system_lib
)

