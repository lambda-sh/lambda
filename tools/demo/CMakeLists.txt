project(demo)

file(
	GLOB_RECURSE
	APP_SRC
	src/*.h
	src/*.cpp
)

add_executable(demo ${APP_SRC})

target_include_directories(demo PRIVATE ${LAMBDA_ENGINE_INCLUDE_DIR})
target_link_libraries(demo PRIVATE lambda)