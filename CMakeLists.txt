cmake_minimum_required(VERSION 3.0.0)

# Set CMake variables
set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)

# Set OpenGL variables (Linux specific)
set(OpenGL_GL_PREFERENCE "GLVND")

project (shaders)
add_executable(app ${CMAKE_BINARY_DIR}/src/application.cpp)

# Load resources necessary for the executable to launch (Shaders, images, etc)
file(COPY ${CMAKE_BINARY_DIR}/res DESTINATION ${CMAKE_BINARY_DIR}/bin/res)

# Add OpenGL
find_package(OpenGL REQUIRED)
target_link_libraries(app OpenGL::GL)

# Add GLEW 
find_package(GLEW REQUIRED)
target_link_libraries(app GLEW::GLEW)
target_compile_definitions(app PUBLIC GLEW_STATIC)

# Add GLFW
find_package(glfw3 3.3 REQUIRED)
target_link_libraries(app glfw)
