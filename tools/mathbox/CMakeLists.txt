project(mathbox)

file(
    GLOB_RECURSE
    APP_SRC
    src/*.h
    src/*.cpp)

file(
  COPY
  assets
  DESTINATION
  ${CMAKE_BINARY_DIR}/tools/mathbox)

add_executable(mathbox ${APP_SRC})

target_include_directories(mathbox PRIVATE ${LAMBDA_ENGINE_INCLUDE_DIR})
target_link_libraries(mathbox PRIVATE lambda)