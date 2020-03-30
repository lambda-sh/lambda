cmake_minimum_required(VERSION 3.0.0)

# Set CMake variables
set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)

# Set OpenGL variables (Linux specific)
set(OpenGL_GL_PREFERENCE "GLVND")

# Compile our game engine library to be used as a dynamic lib inside of our
# sandbox application.
add_library(engine SHARED ${CMAKE_BINARY_DIR}/engine/engine.cpp)

# Add OpenGL
find_package(OpenGL REQUIRED)
target_link_libraries(engine OpenGL::GL)

# Add GLEW 
find_package(GLEW REQUIRED)
target_link_libraries(engine GLEW::GLEW)
target_compile_definitions(engine PUBLIC GLEW_STATIC)

# Add GLFW
find_package(glfw3 3.3 REQUIRED)
target_link_libraries(engine glfw)

# Compile our sandbox application, which will be used to test our game engine.
add_executable(app ${CMAKE_BINARY_DIR}/sandbox/application.cpp)
target_link_libraries(app engine)


# Load resources necessary for the executable to launch (Shaders, images, etc)
file(COPY ${CMAKE_BINARY_DIR}/res DESTINATION ${CMAKE_BINARY_DIR}/bin/res)


