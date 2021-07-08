# ----------------------------------- SETTINGS ---------------------------------

cmake_minimum_required(VERSION 3.20.0)


set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

set(OpenGL_GL_PREFERENCE "GLVND")

# ----------------------------------- LAMBDA -----------------------------------

project(lambda)

option(LAMBDA_TOOLS_BUILD_SANDBOX "Sandbox for testing lambda" OFF)
option(LAMBDA_TOOLS_BUILD_MATHBOX "Sandbox for testing math functions" OFF)
option(LAMBDA_ENGINE_BUILD_TESTS "Build tests for lambda" OFF)

file(
    GLOB_RECURSE
    LAMBDA_SRC
    ${CMAKE_SOURCE_DIR}/lambda/src/Lambda/*.h
    ${CMAKE_SOURCE_DIR}/lambda/src/Lambda/*.cpp)

add_library(lambda STATIC ${LAMBDA_SRC})

set_target_properties(
    lambda
    PROPERTIES PUBLIC_HEADER ${CMAKE_SOURCE_DIR}/lambda/src/Lambda/Lambda.h)

set_target_properties(lambda PROPERTIES PREFIX "")
set_target_properties(lambda PROPERTIES OUTPUT_NAME "lambda")

# Directory to include Lambda src.
set(LAMBDA_ENGINE_INCLUDE_DIR "${CMAKE_SOURCE_DIR}/lambda/src")

# Root directory to provide sub directories a reliable absolute path
set(LAMBDA_ENGINE_ROOT_DIR "${CMAKE_SOURCE_DIR}")


# todo(C3NZ): Make all include directories private once lambdas API is
# complete.
target_include_directories(
    lambda
    PRIVATE ${LAMBDA_ENGINE_INCLUDE_DIR}
    PRIVATE ${CMAKE_SOURCE_DIR}/lambda/vendor/spdlog/include
    PRIVATE ${CMAKE_SOURCE_DIR}/lambda/vendor/glad/include
    PRIVATE ${CMAKE_SOURCE_DIR}/lambda/vendor/glfw/include
    PRIVATE ${CMAKE_SOURCE_DIR}/lambda/vendor/stb_image/src
    PRIVATE ${CMAKE_SOURCE_DIR}/lambda/vendor/readerwriterqueue
    PUBLIC ${CMAKE_SOURCE_DIR}/lambda/vendor/concurrentqueue
    PUBLIC ${CMAKE_SOURCE_DIR}/lambda/vendor/imgui
    PUBLIC ${CMAKE_SOURCE_DIR}/lambda/vendor/glm/glm)

if (WIN32)
    message (STATUS ">>> Setting windows compile definitions for the lambda.")
    target_compile_definitions(
        lambda
        PUBLIC LAMBDA_PLATFORM_WINDOWS
        PUBLIC LAMBDA_BUILD_DLL
        PUBLIC LAMBDA_DEBUG
        PUBLIC LAMBDA_INCLUDE_PROFILER)
elseif (UNIX AND NOT APPLE)
    message (STATUS ">>> Setting linux compile definitions for the lambda.")
    target_compile_definitions(
        lambda
        PUBLIC LAMBDA_PLATFORM_LINUX
        PUBLIC LAMBDA_BUILD_DLL
        PUBLIC LAMBDA_DEBUG
        PUBLIC LAMBDA_INCLUDE_PROFILER)
elseif(APPLE)
    message (STATUS ">>> Setting macos compile definitions for the lambda.")
    target_compile_definitions(
        lambda
        PRIVATE LAMBDA_PLATFORM_MACOS
        PRIVATE LAMBDA_PLATFORM_LINUX
        PRIVATE LAMBDA_BUILD_DLL
        PRIVATE LAMBDA_DEBUG
        PRIVATE LAMBDA_INCLUDE_PROFILER)

    target_link_libraries(
        lambda
        "-framework OpenGL")
endif()

# ----------------------------- ENGINE DEPENDENCIES ----------------------------

add_subdirectory(${CMAKE_SOURCE_DIR}/lambda/vendor/spdlog)
target_link_libraries(lambda spdlog::spdlog)

add_subdirectory(${CMAKE_SOURCE_DIR}/lambda/vendor/glad)
target_link_libraries(lambda glad)

add_subdirectory(${CMAKE_SOURCE_DIR}/lambda/vendor/glm)
target_link_libraries(lambda glm)

add_subdirectory(${CMAKE_SOURCE_DIR}/lambda/vendor/glfw)
target_link_libraries(lambda glfw ${GLFW_LIBRARIES})

add_subdirectory(${CMAKE_SOURCE_DIR}/lambda/vendor/imgui)
target_link_libraries(lambda imgui)

add_subdirectory(${CMAKE_SOURCE_DIR}/lambda/vendor/stb)
target_link_libraries(lambda stb)

add_subdirectory(${CMAKE_SOURCE_DIR}/lambda/vendor/readerwriterqueue)
target_link_libraries(lambda readerwriterqueue)

# ----------------------------------- SANDBOX ----------------------------------

# Update how tests are implemented.
if (${LAMBDA_ENGINE_BUILD_TESTS})
    add_subdirectory(${CMAKE_SOURCE_DIR}/lambda/vendor/googletest)
    add_subdirectory(${CMAKE_SOURCE_DIR}/lambda/tests)
endif()

if (${LAMBDA_TOOLS_BUILD_SANDBOX})
  add_subdirectory(${CMAKE_SOURCE_DIR}/tools/sandbox)
endif()

if (${LAMBDA_TOOLS_BUILD_MATHBOX})
  add_subdirectory(${CMAKE_SOURCE_DIR}/tools/mathbox)
endif()
