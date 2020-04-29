#ifndef ENGINE_SRC_CORE_IMGUI_IMGUIBUILD_H_
#define ENGINE_SRC_CORE_IMGUI_IMGUIBUILD_H_
// Currently the build file that allows the imgui layer to bind to our setup.

// TODO(C3NZ): Investigate into if there is a better way of handling imgui
// integrations into our engine such that we can integrate imgui into multiple
// different types of windows and rendering backends while also allowing imgui
// to be completely stripped from the engine when compiled.

#include "examples/imgui_impl_glfw.h"
#include "examples/imgui_impl_opengl3.h"

#include "examples/imgui_impl_glfw.cpp"
#include "examples/imgui_impl_opengl3.cpp"

#endif  // ENGINE_SRC_CORE_IMGUI_IMGUIBUILD_H_
