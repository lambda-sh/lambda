cmake_minimum_required(VERSION 3.0.0)

set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)

# For vim setup.
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# ----------------------------------- ENGINE ----------------------------------

set(OpenGL_GL_PREFERENCE "GLVND")
project(engine)

file(
    GLOB_RECURSE 
    ENGINE_SRC 
    ${CMAKE_SOURCE_DIR}/engine/src/*.h 
    ${CMAKE_SOURCE_DIR}/engine/src/*.cpp
)

message(STATUS ${ENGINE_SRC})
add_library(engine SHARED ${ENGINE_SRC})

set_target_properties(
    engine 
    PROPERTIES 
    PUBLIC_HEADER 
    ${CMAKE_SOURCE_DIR}/engine/src/Engine.h
)

target_include_directories(
    engine
    PUBLIC
    ${CMAKE_SOURCE_DIR}/engine/src
)

if (WIN32)
    message (STATUS ">>> Setting windows compile definitions")
    target_compile_definitions(engine PRIVATE ENGINE_PLATFORM_WINDOWS)
    target_compile_definitions(engine PRIVATE ENGINE_BUILD_DLL)
elseif (UNIX)
    message (STATUS ">>> Setting linux compile definitions")
    target_compile_definitions(
        engine 
        PRIVATE ENGINE_PLATFORM_LINUX
        PRIVATE ENGINE_BUILD_DLL
        INTERFACE GLEW_STATIC
    )
endif()

# Add Dependencies
find_package(OpenGL REQUIRED)
target_link_libraries(engine OpenGL::GL)

find_package(GLEW REQUIRED)
target_link_libraries(engine GLEW::GLEW)

find_package(glfw3 3.3 REQUIRED)
target_link_libraries(engine glfw)

# ----------------------------------- SANDBOX ----------------------------------

project(sandbox)

file(
    GLOB_RECURSE
    APP_SRC
    ${CMAKE_SOURCE_DIR}/sandbox/src/*.h
    ${CMAKE_SOURCE_DIR}/sandbox/src/*.cpp
)
add_executable(app ${APP_SRC})

if (WIN32)
    target_compile_definitions(app PRIVATE ENGINE_PLATFORM_WINDOWS)
elseif (UNIX)
    target_compile_definitions(app PRIVATE ENGINE_PLATFORM_LINUX)
endif()

target_link_libraries(app PUBLIC engine)
# Load resources necessary for the executable to launch (Shaders, images, etc)
# file(COPY ${CMAKE_BINARY_DIR}/res DESTINATION ${CMAKE_BINARY_DIR}/bin/res)
