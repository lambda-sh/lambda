cmake_minimum_required(VERSION 3.0.0)
project(glad)

set(
    GLAD_SRC
    include/glad/glad.h
    include/KHR/khrplatform.h
    src/glad.c
)

add_library(glad SHARED ${GLAD_SRC})

target_include_directories(
    glad
    PUBLIC include
)
