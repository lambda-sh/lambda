project(sandbox)

file(
    GLOB_RECURSE
    APP_SRC
    src/*.h
    src/*.cpp)

add_executable(sandbox ${APP_SRC})

file(
  COPY
  assets
  DESTINATION
  ${CMAKE_BINARY_DIR}/tools/sandbox)

if (WIN32)
    message (STATUS ">>> Setting windows compile definitions for sandbox.")
    target_compile_definitions(
        sandbox
        PUBLIC LAMBDA_DEBUG
        PUBLIC LAMBDA_PLATFORM_WINDOWS
        PUBLIC LAMBDA_INCLUDE_PROFILER)
elseif (UNIX)
    message (STATUS ">>> Setting linux compile definitions for sandbox.")
    target_compile_definitions(
        sandbox
        PUBLIC LAMBDA_DEBUG
        PUBLIC LAMBDA_PLATFORM_LINUX
        PUBLIC LAMBDA_INCLUDE_PROFILER)
endif()

target_include_directories(sandbox PRIVATE "${LAMBDA_ENGINE_INCLUDE_DIR}")
target_link_libraries(sandbox PRIVATE lambda)
