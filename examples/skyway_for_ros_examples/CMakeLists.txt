cmake_minimum_required(VERSION 3.0.2)
project(skyway_for_ros_examples)

catkin_package()

find_package(catkin REQUIRED COMPONENTS
  roscpp
  rospy
  skyway
  std_msgs
)
