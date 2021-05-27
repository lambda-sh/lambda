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

target_include_directories(sandbox PRIVATE "${LAMBDA_ENGINE_INCLUDE_DIR}")
target_link_libraries(sandbox PRIVATE lambda)
