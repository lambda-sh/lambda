project(demo)

set(APP_SRC src/main.cpp)

add_executable(demo ${APP_SRC})

target_include_directories(demo PRIVATE ${LAMBDA_ENGINE_INCLUDE_DIR})
target_link_libraries(demo PRIVATE lambda)
