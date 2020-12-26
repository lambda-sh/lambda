cmake_minimum_required(VERSION 3.0.0)

if (${DISTRIBUTION_BUILD})
    set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/Dist/lib)
    set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/Dist/lib)
    set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/Dist/bin)
elseif (${CMAKE_BUILD_TYPE} STREQUAL "Release" OR ${CMAKE_BUILD_TYPE} STREQUAL "Debug")
    set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY
        ${CMAKE_BINARY_DIR}/${CMAKE_BUILD_TYPE}/lib)
    set(CMAKE_LIBRARY_OUTPUT_DIRECTORY
        ${CMAKE_BINARY_DIR}/${CMAKE_BUILD_TYPE}/lib)
    set(CMAKE_RUNTIME_OUTPUT_DIRECTORY
        ${CMAKE_BINARY_DIR}/${CMAKE_BUILD_TYPE}/bin)
endif()

# ----------------------------------- SETTINGS ---------------------------------

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

set(OpenGL_GL_PREFERENCE "GLVND")

# ----------------------------------- LAMBDA -----------------------------------

project(lambda)

file(
    GLOB_RECURSE
    LAMBDA_SRC
    ${CMAKE_SOURCE_DIR}/lambda/src/Lambda/*.h ${CMAKE_SOURCE_DIR}/lambda/src/Lambda/*.cpp)

add_library(lambda STATIC ${LAMBDA_SRC})

set_target_properties(
    lambda
    PROPERTIES PUBLIC_HEADER ${CMAKE_SOURCE_DIR}/lambda/src/Lambda/Lambda.h)

set_target_properties(lambda PROPERTIES PREFIX "")
set_target_properties(lambda PROPERTIES OUTPUT_NAME "lambda")

target_include_directories(
    lambda
    PUBLIC ${CMAKE_SOURCE_DIR}/lambda/src
    PUBLIC ${CMAKE_SOURCE_DIR}/lambda/vendor/spdlog/include
    PUBLIC ${CMAKE_SOURCE_DIR}/lambda/vendor/glad/include
    PUBLIC ${CMAKE_SOURCE_DIR}/lambda/vendor/glm/glm
    PUBLIC ${CMAKE_SOURCE_DIR}/lambda/vendor/glfw/include
    PUBLIC ${CMAKE_SOURCE_DIR}/lambda/vendor/imgui
    PUBLIC ${CMAKE_SOURCE_DIR}/lambda/vendor/stb_image/src
    PUBLIC ${CMAKE_SOURCE_DIR}/lambda/vendor/readerwriterqueue
    PUBLIC ${CMAKE_SOURCE_DIR}/lambda/vendor/concurrentqueue)

if (WIN32)
    message (STATUS ">>> Setting windows compile definitions for the lambda.")
    target_compile_definitions(
        lambda
        PUBLIC GLFW_INCLUDE_NONE
        PUBLIC IMGUI_IMPL_OPENGL_LOADER_GLAD
        PRIVATE LAMBDA_PLATFORM_WINDOWS
        PRIVATE LAMBDA_BUILD_DLL
        PRIVATE LAMBDA_DEBUG
        PRIVATE LAMBDA_INCLUDE_PROFILER)

    # Use a Multi threaded specific and DLL specific runtime for windows.
    set(CMAKE_CXX_FLAGS_RELEASE "${CMAKE_CXX_FLAGS_RELEASE} /MD")
    set(CMAKE_CXX_FLAGS_DEBUG "${CMAKE_CXX_FLAGS_DEBUG} /MDd")
elseif (UNIX)
    message (STATUS ">>> Setting linux compile definitions for the lambda.")
    target_compile_definitions(
        lambda
        PUBLIC GLFW_INCLUDE_NONE
        PUBLIC IMGUI_IMPL_OPENGL_LOADER_GLAD
        PRIVATE LAMBDA_PLATFORM_LINUX
        PRIVATE LAMBDA_BUILD_DLL
        PRIVATE LAMBDA_DEBUG
        PRIVATE LAMBDA_INCLUDE_PROFILER)
elseif(APPLE)
    message (STATUS ">>> Setting linux compile definitions for the lambda.")
    target_compile_definitions(
        lambda
        PUBLIC GLFW_INCLUDE_NONE
        PUBLIC IMGUI_IMPL_OPENGL_LOADER_GLAD
        PRIVATE LAMBDA_PLATFORM_LINUX
        PRIVATE LAMBDA_BUILD_DLL
        PRIVATE LAMBDA_DEBUG
        PRIVATE LAMBDA_INCLUDE_PROFILER)

    target_link_libraries(
        lambda
        "-framework Cocoa"
        "-framework AppKit"
        "-framework CoreData"
        "-framework Foundation")
    set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} -stdlib=libstdc++ -lc++abi")
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -stdlib=libstdc++")
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

if (${WITH_TOOL} STREQUAL "sandbox")
  project(sandbox)

  file(
      GLOB_RECURSE
      APP_SRC
      ${CMAKE_SOURCE_DIR}/tools/sandbox/src/*.h
      ${CMAKE_SOURCE_DIR}/tools/sandbox/src/*.cpp)

  add_executable(sandbox ${APP_SRC})

  file(
    COPY
    ${CMAKE_SOURCE_DIR}/tools/sandbox/assets
    DESTINATION
    ${CMAKE_RUNTIME_OUTPUT_DIRECTORY})

  if (WIN32)
      message (STATUS ">>> Setting windows compile definitions for sandbox.")
      target_compile_definitions(
          sandbox
          PUBLIC LAMBDA_DEBUG
          PUBLIC LAMBDA_PLATFORM_WINDOWS
          PUBLIC LAMBDA_WITH_PROFILING)
  elseif (UNIX)
      message (STATUS ">>> Setting linux compile definitions for sandbox.")
      target_compile_definitions(
          sandbox
          PUBLIC LAMBDA_DEBUG
          PUBLIC LAMBDA_PLATFORM_LINUX
          PUBLIC LAMBDA_INCLUDE_PROFILER)
  endif()
  target_link_libraries(sandbox PRIVATE lambda)

endif()

# Load resources necessary for the executable to launch (Shaders, images, etc)
# file(COPY ${CMAKE_BINARY_DIR}/res DESTINATION ${CMAKE_BINARY_DIR}/bin/res)
