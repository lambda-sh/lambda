project(math)

file(
    GLOB_RECURSE
    APP_SRC
    src/*.h
    src/*.cpp)

add_executable(mathbox ${APP_SRC})

if (WIN32)
    message (STATUS ">>> Setting windows compile definitions for Mathbox.")
    target_compile_definitions(
        mathbox
        PUBLIC LAMBDA_DEBUG
        PUBLIC LAMBDA_PLATFORM_WINDOWS
        PUBLIC LAMBDA_INCLUDE_PROFILER)
elseif (UNIX)
    message (STATUS ">>> Setting linux compile definitions for Mathbox.")
    target_compile_definitions(
        mathbox
        PUBLIC LAMBDA_DEBUG
        PUBLIC LAMBDA_PLATFORM_LINUX
        PUBLIC LAMBDA_INCLUDE_PROFILER)
endif()

target_link_libraries(mathbox PRIVATE lambda)
