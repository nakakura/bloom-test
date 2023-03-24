cmake_minimum_required(VERSION 3.0.2)
project(skyway)
add_compile_options(-std=c++14)

include(ExternalProject)

SET(VENDOR_DIR ${CMAKE_CURRENT_BINARY_DIR}/${PROJECT_NAME}/vendor)

find_package(catkin REQUIRED COMPONENTS
        message_generation
        roscpp
        std_msgs
        pluginlib
        )

add_service_files(
        DIRECTORY srv
        FILES SkyWayControl.srv SkyWayEvents.srv
)

generate_messages(
        DEPENDENCIES std_msgs
)

catkin_package(
        INCLUDE_DIRS include
        LIBRARIES ${PROJECT_NAME}_plugin
        CATKIN_DEPENDS std_msgs message_runtime
)

include_directories(
        include
        ${catkin_INCLUDE_DIRS}
        ${VENDOR_DIR}/fruit/include
        ${VENDOR_DIR}/rapidjson/include
)

###########
## Build ##
###########

# add DI framework
SET(FRUIT_LIB_FILE ${VENDOR_DIR}/fruit/src/libfruit.a)
ExternalProject_Add(${PROJECT_NAME}_fruit
        URL https://github.com/google/fruit/archive/refs/tags/v3.7.1.zip
        UPDATE_COMMAND ""
        INSTALL_COMMAND ""
        SOURCE_DIR ${VENDOR_DIR}/fruit
        DOWNLOAD_DIR ${VENDOR_DIR}/fruit
        BINARY_DIR ${VENDOR_DIR}/fruit
        CMAKE_ARGS -DCMAKE_BUILD_TYPE=Release -DBUILD_SHARED_LIBS=Off
        UPDATE_DISCONNECTED 1
        )

# add json library
ExternalProject_Add(${PROJECT_NAME}_rapidjson
        URL https://github.com/Tencent/rapidjson/archive/refs/tags/v1.1.0.zip
        UPDATE_COMMAND ""
        CONFIGURE_COMMAND ""
        BUILD_COMMAND ""
        INSTALL_COMMAND ""
        SOURCE_DIR ${VENDOR_DIR}/rapidjson
        DOWNLOAD_DIR ${VENDOR_DIR}/rapidjson
        BINARY_DIR ${VENDOR_DIR}/rapidjson
        UPDATE_DISCONNECTED 1
        )

# add skyway library
if("$ENV{ROS_DISTRO}" STREQUAL "noetic")
    message(STATUS "Download library for noetic")
    SET(SKYWAY_LIB_FILE ${VENDOR_DIR}/libskyway_20.04.a)
    ExternalProject_Add(${PROJECT_NAME}_skyway
            URL https://github.com/ntt-t3/skyway_for_ros/releases/download/lib_v0.0.1/libskyway_20.04.a
            UPDATE_COMMAND ""
            CONFIGURE_COMMAND ""
            BUILD_COMMAND ""
            INSTALL_COMMAND ""
            DOWNLOAD_DIR ${VENDOR_DIR}
            BINARY_DIR ${VENDOR_DIR}
            UPDATE_DISCONNECTED 1
            DOWNLOAD_NO_EXTRACT 1
            )
endif()

if("$ENV{ROS_DISTRO}" STREQUAL "melodic")
    message(STATUS "Download library for melodic")
    SET(SKYWAY_LIB_FILE ${VENDOR_DIR}/libskyway_18.04.a)
    ExternalProject_Add(${PROJECT_NAME}_skyway
            URL https://github.com/ntt-t3/skyway_for_ros/releases/download/lib_v0.0.1/libskyway_18.04.a
            UPDATE_COMMAND ""
            CONFIGURE_COMMAND ""
            BUILD_COMMAND ""
            INSTALL_COMMAND ""
            DOWNLOAD_DIR ${VENDOR_DIR}
            BINARY_DIR ${VENDOR_DIR}
            UPDATE_DISCONNECTED 1
            DOWNLOAD_NO_EXTRACT 1
            )
endif()


set_directory_properties(PROPERTIES ADDITIONAL_MAKE_CLEAN_FILES ${VENDOR_DIR})

add_executable(${PROJECT_NAME}
        src/main.cpp
        src/ffi.cpp
        src/ffi_bridge.cpp
        src/router.cpp
        src/plugin_router/binary_plugin_router.cpp
        src/plugin_router/json_plugin_router.cpp
        src/plugin_router/plugin_router_factory.cpp
        src/plugin_router/string_plugin_router.cpp
        src/presentation/control_service.cpp
        src/presentation/events_service.cpp
        src/socket/udp_socket.cpp
        )

add_dependencies(${PROJECT_NAME}
        ${${PROJECT_NAME}_EXPORTED_TARGETS}
        ${catkin_EXPORTED_TARGETS}
        ${PROJECT_NAME}_fruit
        ${PROJECT_NAME}_rapidjson
        ${PROJECT_NAME}_skyway)

target_link_libraries(${PROJECT_NAME}
        ${catkin_LIBRARIES}
        ${SKYWAY_LIB_FILE}
        ${FRUIT_LIB_FILE}
        crypto
        ssl
        pthread
        dl
        rt
        )

add_library(${PROJECT_NAME}_plugin
        src/plugin/samples/binary_loopback.cpp
        src/plugin/samples/json_loopback.cpp
        src/plugin/samples/string_loopback.cpp
        src/plugin/samples/string_pub_sub.cpp
        src/plugin/skyway_plugin.cpp)

add_dependencies(${PROJECT_NAME}_plugin
        ${${PROJECT_NAME}_EXPORTED_TARGETS}
        ${catkin_EXPORTED_TARGETS}
        ${PROJECT_NAME}
        ${PROJECT_NAME}_rapidjson
        ${PROJECT_NAME}_skyway)

install(TARGETS ${PROJECT_NAME} ${PROJECT_NAME}_plugin
  ARCHIVE DESTINATION ${CATKIN_PACKAGE_LIB_DESTINATION}
  LIBRARY DESTINATION ${CATKIN_PACKAGE_LIB_DESTINATION}
  RUNTIME DESTINATION ${CATKIN_PACKAGE_BIN_DESTINATION}
)

install(DIRECTORY include/${PROJECT_NAME}/
        DESTINATION ${CATKIN_PACKAGE_INCLUDE_DESTINATION})

install(FILES ${PROJECT_NAME}_plugin.xml
        DESTINATION ${CATKIN_PACKAGE_SHARE_DESTINATION})

if (CATKIN_ENABLE_TESTING)
    find_package(rostest REQUIRED)
    add_rostest_gtest(${PROJECT_NAME}_test_node test/unit_test.test
            test/test_main.cpp
            test/plugin_router/binary_plugin_router_test.cpp
            test/plugin_router/json_plugin_router_test.cpp
            test/plugin_router/plugin_router_factory_test.cpp
            test/plugin_router/string_plugin_router_test.cpp
            test/socket/udp_socket_test.cpp
            src/plugin_router/binary_plugin_router.cpp
            src/plugin_router/json_plugin_router.cpp
            src/plugin_router/plugin_router_factory.cpp
            src/plugin_router/string_plugin_router.cpp
            src/presentation/control_service.cpp
            src/presentation/events_service.cpp
            src/socket/udp_socket.cpp
            )
    add_dependencies(${PROJECT_NAME}_test_node
            ${${PROJECT_NAME}_EXPORTED_TARGETS}
            ${catkin_EXPORTED_TARGETS}
            ${PROJECT_NAME}_fruit
            ${PROJECT_NAME}_rapidjson
            ${PROJECT_NAME}_skyway)
    target_link_libraries(${PROJECT_NAME}_test_node
            ${catkin_LIBRARIES}
            ${SKYWAY_LIB_FILE}
            ${FRUIT_LIB_FILE}
            crypto
            ssl
            pthread
            dl
            rt
            )
endif ()
