#if defined LAMBDA_PLATFORM_WINDOWS || defined LAMBDA_DEBUG

#include "Lambda/platform/windows/Window.h"

#include <GLFW/glfw3.h>
#include "Lambda/core/Core.h"
#include "Lambda/core/Window.h"
#include "Lambda/core/events/ApplicationEvent.h"
#include "Lambda/core/events/KeyEvent.h"
#include "Lambda/core/events/MouseEvent.h"
#include "Lambda/core/memory/Pointers.h"
#include "Lambda/core/util/Assert.h"
#include "Lambda/core/util/Log.h"
#include "Lambda/platform/opengl/OpenGLContext.h"

namespace lambda {


#ifdef LAMBDA_PLATFORM_WINDOWS

// Will create a windows based implementation of the window handler.
core::memory::Shared<core::Window> core::Window::Create(
    WindowProperties properties) {
  return memory::CreateShared<platform::windows::WindowImplementation>(
      std::move(properties));
}

#endif  // LAMBDA_PLATFORM_WINDOWS

namespace platform {
namespace windows {

using core::events::KeyPressedEvent;
using core::events::KeyReleasedEvent;
using core::events::KeyTypedEvent;
using core::events::MouseButtonPressedEvent;
using core::events::MouseButtonReleasedEvent;
using core::events::MouseMovedEvent;
using core::events::MouseScrolledEvent;
using core::events::WindowCloseEvent;
using core::events::WindowResizeEvent;
using core::memory::CreateShared;
using core::memory::Shared;

// Error callback for handling GLFW specific errors
static void GLFWErrorCallback(int error, const char* description) {
  LAMBDA_CORE_ERROR("GFLW Error ({0}): {1}", error, description);
}

static bool GLFWInitialized = false;

WindowImplementation::WindowImplementation(
    core::WindowProperties properties) {
  Init(std::move(properties));
}

WindowImplementation::~WindowImplementation() {
  Shutdown();
}

// Initialize the windows given generic window properties to be applied to the
// current window.
void WindowImplementation::Init(core::WindowProperties properties) {
  properties_.Title = properties.Title;
  properties_.Width = properties.Width;
  properties_.Height = properties.Height;

  LAMBDA_CORE_INFO(
      "Creating window {0} ({1}, {2})",
      properties_.Title,
      properties_.Width,
      properties_.Height);

  if (!GLFWInitialized) {
    int success = glfwInit();
    LAMBDA_CORE_ASSERT(success, "Could not initialize GLFW!", "");
    glfwSetErrorCallback(GLFWErrorCallback);
    GLFWInitialized = true;
  }

  window_ = glfwCreateWindow(
      static_cast<int>(properties_.Width),
      static_cast<int>(properties_.Height),
      properties_.Title.c_str(),
      nullptr,
      nullptr);

  context_ = new opengl::OpenGLContext(window_);
  context_->Init();

  glfwSetWindowUserPointer(window_, &properties_);
  SetVerticalSync(true);

  glfwSetWindowSizeCallback(
      window_,
      [](GLFWwindow* window, int new_width, int new_height) {
      internal::Properties* properties =
              static_cast<internal::Properties*>(
                  glfwGetWindowUserPointer(window));

          Shared<WindowResizeEvent> event = CreateShared<WindowResizeEvent>(
              new_width, new_height);
          properties->Width = new_width;
          properties->Height = new_height;

          properties->EventCallback(event);
      });

  glfwSetWindowCloseCallback(
      window_,
      [](GLFWwindow* window) {
      internal::Properties* properties =
            static_cast<internal::Properties*>(
                glfwGetWindowUserPointer(window));

            Shared<WindowCloseEvent> event = CreateShared<WindowCloseEvent>();
            properties->EventCallback(event);
      });

  glfwSetKeyCallback(
      window_,
      [](GLFWwindow* window, int key, int scancode, int action, int mods) {
      internal::Properties* properties =
            static_cast<internal::Properties*>(
                glfwGetWindowUserPointer(window));

        switch (action) {
          case GLFW_PRESS:
          {
            Shared<KeyPressedEvent> event = CreateShared<KeyPressedEvent>(
                key, 0);
            properties->EventCallback(event);
            break;
          }
          case GLFW_RELEASE:
          {
            Shared<KeyReleasedEvent> event = CreateShared<KeyReleasedEvent>(
                key);
            properties->EventCallback(event);
            break;
          }
          case GLFW_REPEAT:
          {
            Shared<KeyPressedEvent> event = CreateShared<KeyPressedEvent>(
                key, 0);
            properties->EventCallback(event);
            break;
          }
        }
      });

  glfwSetCharCallback(
      window_,
      [](GLFWwindow* window, unsigned int character) {
      internal::Properties* properties =
            static_cast<internal::Properties*>(
                glfwGetWindowUserPointer(window));

            Shared<KeyTypedEvent> event = CreateShared<KeyTypedEvent>(
                character);
            properties->EventCallback(event);
      });

  glfwSetMouseButtonCallback(
      window_,
      [](GLFWwindow* window, int button, int action, int mods) {
      internal::Properties* properties =
            static_cast<internal::Properties*>(
                glfwGetWindowUserPointer(window));

        switch (action) {
          case GLFW_PRESS:
          {
            Shared<MouseButtonPressedEvent> event = CreateShared<
                MouseButtonPressedEvent>(button);
            properties->EventCallback(event);
            break;
          }
          case GLFW_RELEASE:
          {
            Shared<MouseButtonReleasedEvent> event = CreateShared<
                MouseButtonReleasedEvent>(button);
            properties->EventCallback(event);
            break;
          }
        }
      });

  glfwSetScrollCallback(
      window_,
      [](GLFWwindow* window, double x_offset, double y_offset) {
      internal::Properties* properties =
            static_cast<internal::Properties*>(
                glfwGetWindowUserPointer(window));

        Shared<MouseScrolledEvent> event = CreateShared<MouseScrolledEvent>(
            static_cast<float>(x_offset), static_cast<float>(y_offset));
        properties->EventCallback(event);
      });

  glfwSetCursorPosCallback(
      window_,
      [](GLFWwindow* window, double x_position, double y_position) {
      internal::Properties* properties =
            static_cast<internal::Properties*>(
                glfwGetWindowUserPointer(window));

        Shared<MouseMovedEvent> event = CreateShared<MouseMovedEvent>(
            static_cast<float>(x_position), static_cast<float>(y_position));
        properties->EventCallback(event);
      });
}

// Shutdown the window.
void WindowImplementation::Shutdown() {
  glfwDestroyWindow(window_);
}

// Handling updates to the screen.
void WindowImplementation::OnUpdate() {
  glfwPollEvents();
  context_->SwapBuffers();
}

// Setup the current window to use or not use Vertical sync.
void WindowImplementation::SetVerticalSync(bool enabled) {
  glfwSwapInterval(enabled ? 1 : 0);
  properties_.VerticalSync = enabled;
}

// Check if the current window has VSync enabled.
bool WindowImplementation::HasVerticalSync() const {
  return properties_.VerticalSync;
}

}  // namespace windows
}  // namespace platform
}  // namespace lambda

#endif  // LAMBDA_PLATFORM_WINDOWS
