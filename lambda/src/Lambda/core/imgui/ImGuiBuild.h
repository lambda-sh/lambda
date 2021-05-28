/// @file engine/src/core/imgui/ImGuiBuild.h
/// @brief The Imgui build header for compiling platform specific
/// imgui implementations.
#ifndef LAMBDA_SRC_LAMBDA_CORE_IMGUI_IMGUIBUILD_H_
#define LAMBDA_SRC_LAMBDA_CORE_IMGUI_IMGUIBUILD_H_

/// TODO(C3NZ): Investigate into if there is a better way of handling imgui
// integrations into our engine such that we can integrate imgui into multiple
// different types of windows and rendering backends while also allowing imgui
// to be completely stripped from the engine when compiled.

#include <Lambda/platform/glad/Glad.h>

#define IMGUI_IMPL_OPENGL_LOADER_GLAD

#include <examples/imgui_impl_glfw.h>
#include <examples/imgui_impl_glfw.cpp>

#include <examples/imgui_impl_opengl3.h>
#include <examples/imgui_impl_opengl3.cpp>

#endif  // LAMBDA_SRC_LAMBDA_CORE_IMGUI_IMGUIBUILD_H_
