cmake_minimum_required(VERSION 3.0.0)
project(glad)

# Not needed anymore since the engine is now being compiled as a static
# lib.
# set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -fPIC")

set(
    GLAD_SRC
    include/glad/glad.h
    include/KHR/khrplatform.h
    src/glad.c
)

add_library(glad STATIC ${GLAD_SRC})

target_include_directories(
    glad
    PUBLIC include
)
