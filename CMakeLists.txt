cmake_minimum_required(VERSION 3.0.0)

if (${DISTRIBUTION_BUILD})
    set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/Dist/lib)
    set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/Dist/lib)
    set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/Dist/bin)
elseif (${CMAKE_BUILD_TYPE} STREQUAL "Release" OR ${CMAKE_BUILD_TYPE} STREQUAL "Debug")
    set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/${CMAKE_BUILD_TYPE}/lib)
    set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/${CMAKE_BUILD_TYPE}/lib)
    set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/${CMAKE_BUILD_TYPE}/bin)
endif()


# For vim setup.
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# ----------------------------------- ENGINE ----------------------------------

project(engine)

set(OpenGL_GL_PREFERENCE "GLVND")

file(
    GLOB_RECURSE
    ENGINE_SRC
    ${CMAKE_SOURCE_DIR}/engine/src/*.h
    ${CMAKE_SOURCE_DIR}/engine/src/*.cpp
)

add_library(engine SHARED ${ENGINE_SRC})

set_target_properties(
    engine
    PROPERTIES PUBLIC_HEADER ${CMAKE_SOURCE_DIR}/engine/src/Engine.h
)

target_include_directories(
    engine
    PUBLIC ${CMAKE_SOURCE_DIR}/engine/src
    PUBLIC ${CMAKE_SOURCE_DIR}/engine/vendor/spdlog/include
    PUBLIC ${CMAKE_SOURCE_DIR}/engine/vendor/glfw/include
    PUBLIC ${CMAKE_SOURCE_DIR}/engine/vendor/glad/include
    PUBLIC ${CMAKE_SOURCE_DIR}/engine/vendor/imgui
)

if (WIN32)
    message (STATUS ">>> Setting windows compile definitions for the engine.")
    target_compile_definitions(
        engine
        PUBLIC GLFW_INCLUDE_NONE
        PUBLIC IMGUI_IMPL_OPENGL_LOADER_GLAD
        PRIVATE ENGINE_PLATFORM_WINDOWS
        PRIVATE ENGINE_BUILD_DLL
        PRIVATE ENGINE_DEBUG
    )

    # Use a Multi threaded specific and DLL specific runtime for windows.
    set(CMAKE_CXX_FLAGS_RELEASE "${CMAKE_CXX_FLAGS_RELEASE} /MD")
    set(CMAKE_CXX_FLAGS_DEBUG "${CMAKE_CXX_FLAGS_DEBUG} /MDd")
elseif (UNIX)
    message (STATUS ">>> Setting linux compile definitions for the engine.")
    target_compile_definitions(
        engine
        PUBLIC GLFW_INCLUDE_NONE
        PUBLIC IMGUI_IMPL_OPENGL_LOADER_GLAD
        PRIVATE ENGINE_PLATFORM_LINUX
        PRIVATE ENGINE_BUILD_DLL
        PRIVATE ENGINE_DEBUG
    )
endif()

add_subdirectory(${CMAKE_SOURCE_DIR}/engine/vendor/spdlog)
target_link_libraries(engine spdlog::spdlog)

add_subdirectory(${CMAKE_SOURCE_DIR}/engine/vendor/glfw)
target_link_libraries(engine glfw ${GLFW_LIBRARIES})

add_subdirectory(${CMAKE_SOURCE_DIR}/engine/vendor/glad)
target_link_libraries(engine glad)

add_subdirectory(${CMAKE_SOURCE_DIR}/engine/vendor/imgui)
target_link_libraries(engine imgui)

#find_package(OpenGL REQUIRED)
#target_link_libraries(engine OpenGL::GL)

# ----------------------------------- SANDBOX ----------------------------------

project(sandbox)

file(
    GLOB_RECURSE
    APP_SRC
    ${CMAKE_SOURCE_DIR}/sandbox/src/*.h
    ${CMAKE_SOURCE_DIR}/sandbox/src/*.cpp
)
add_executable(app ${APP_SRC})

if (WIN32)
    message (STATUS ">>> Setting windows compile definitions for sandbox.")
    target_compile_definitions(
        app
        PRIVATE ENGINE_PLATFORM_WINDOWS
    )
elseif (UNIX)
    message (STATUS ">>> Setting linux compile definitions for sandbox.")
    target_compile_definitions(
        app
        PRIVATE ENGINE_PLATFORM_LINUX
    )
endif()

target_link_libraries(app PRIVATE engine)
# Load resources necessary for the executable to launch (Shaders, images, etc)
# file(COPY ${CMAKE_BINARY_DIR}/res DESTINATION ${CMAKE_BINARY_DIR}/bin/res)
