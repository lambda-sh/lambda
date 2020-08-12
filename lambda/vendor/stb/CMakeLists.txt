cmake_minimum_required(VERSION 3.0.0)
project(stb)

set(
    STB_SRC
    src/stb_image.cpp
    src/stb_image.h
)

add_library(stb STATIC ${STB_SRC})

target_include_directories(
    stb
    PUBLIC src
)
