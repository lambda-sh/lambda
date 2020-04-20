#ifdef ENGINE_PLATFORM_LINUX

#include "Window.h"

#include <GLFW/glfw3.h>

#include "core/Core.h"
#include "core/Log.h"
#include "core/Window.h"

namespace engine {

// Will create a windows based implementation of the window handler.
Window* Window::Create(const engine::WindowProperties& properties) {
  return new engine::platform::linux::WindowImplementation(properties);
}

namespace platform {
namespace linux {

static bool GLFWInitialized = false;

WindowImplementation::WindowImplementation(
    const engine::WindowProperties& properties) {
  Init(properties);
}

WindowImplementation::~WindowImplementation() {
  Shutdown();
}

// Initialize the windows given generic window properties to be applied to the
// current window.
void WindowImplementation::Init(const engine::WindowProperties& properties) {
  properties_.Title = properties.Title;
  properties_.Width = properties.Width;
  properties_.Height = properties.Height;

  ENGINE_CORE_INFO(
      "Creating window {0} ({1}, {2})",
      properties_.Title,
      properties_.Width,
      properties_.Height);

  if (!GLFWInitialized) {
    int success = glfwInit();
    ENGINE_CORE_ASSERT(success, "Could not initialize GLFW!");
    GLFWInitialized = true;
  }

  window_ = glfwCreateWindow(
      static_cast<int>(properties_.Width),
      static_cast<int>(properties_.Height),
      properties_.Title.c_str(),
      nullptr,
      nullptr);

  glfwMakeContextCurrent(window_);
  glfwSetWindowUserPointer(window_, &properties_);
  SetVSync(true);
}

// Shutdown the window.
void WindowImplementation::Shutdown() {
  glfwDestroyWindow(window_);
}

// Handling updates to the screen.
void WindowImplementation::OnUpdate() {
  glfwPollEvents();
  glfwSwapBuffers(window_);
}

// Setup the current window to use or not use Vsync.
void WindowImplementation::SetVSync(bool enabled) {
  if (enabled) {
    glfwSwapInterval(1);
  } else {
    glfwSwapInterval(0);
  }

  properties_.Vsync = enabled;
}

// Check if the current window has VSync enabled.
bool WindowImplementation::IsVSync() const {
  return properties_.Vsync;
}

}  // namespace linux
}  // namespace platform
}  // namespace engine

#endif  // ENGINE_PLATFORM_LINUX
