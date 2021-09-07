project(lambda_tests)

file(
  GLOB_RECURSE
  LAMBDA_TEST_SRC
  Lambda/*.cpp
)

add_executable(lambda_tests ${LAMBDA_TEST_SRC})
add_test(NAME lambda_tests COMMAND lambda_tests)

target_link_libraries(
  lambda_tests
  PRIVATE lambda
  PRIVATE gtest
)

target_include_directories(
  lambda_tests
  PRIVATE ${LAMBDA_ENGINE_INCLUDE_DIR}
)
