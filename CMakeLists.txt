cmake_minimum_required(VERSION 3.0.0)

set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)

# ----------------------------------- ENGINE ----------------------------------

set(OpenGL_GL_PREFERENCE "GLVND")

file(
    GLOB_RECURSE 
    ENGINE_SRC 
    ${CMAKE_BINARY_DIR}/engine/src/*.h 
    ${CMAKE_BINARY_DIR}/engine/src/*.cpp
)
add_library(engine SHARED ${ENGINE_SRC})

if (WIN32)
    target_compile_definitions(engine PRIVATE ENGINE_PLATFORM_WINDOWS)
    target_compile_definitions(engine PRIVATE ENGINE_BUILD_DLL)
elseif (UNIX)
    target_compile_definitions(engine PRIVATE ENGINE_PLATFORM_LINUX)
    target_compile_definitions(engine PRIVATE ENGINE_BUILD_DLL)
endif()

target_compile_definitions(engine PUBLIC GLEW_STATIC)

# Add Dependencies
find_package(OpenGL REQUIRED)
target_link_libraries(engine OpenGL::GL)

find_package(GLEW REQUIRED)
target_link_libraries(engine GLEW::GLEW)

find_package(glfw3 3.3 REQUIRED)
target_link_libraries(engine glfw)

# ----------------------------------- SANDBOX ----------------------------------

file(
    GLOB_RECURSE
    APP_SRC
    ${CMAKE_BINARY_DIR}/sandbox/src/*.h
    ${CMAKE_BINARY_DIR}/sandbox/src/*.cpp
)
add_executable(app ${APP_SRC})

if (WIN32)
    target_compile_definitions(app PRIVATE ENGINE_PLATFORM_WINDOWS)
elseif (UNIX)
    target_compile_definitions(app PRIVATE ENGINE_PLATFORM_LINUX)
endif()

target_link_libraries(app engine)

# Load resources necessary for the executable to launch (Shaders, images, etc)
# file(COPY ${CMAKE_BINARY_DIR}/res DESTINATION ${CMAKE_BINARY_DIR}/bin/res)
