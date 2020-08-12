#if defined LAMBDA_PLATFORM_LINUX || defined LAMBDA_DEBUG

#include "platform/linux/Window.h"

#include <GLFW/glfw3.h>

#include "core/Core.h"
#include "core/Window.h"
#include "core/events/ApplicationEvent.h"
#include "core/events/KeyEvent.h"
#include "core/events/MouseEvent.h"
#include "core/memory/Pointers.h"
#include "core/memory/Pointers.h"
#include "core/util/Assert.h"
#include "core/util/Log.h"
#include "platform/opengl/OpenGLContext.h"

using lambda::core::memory::Shared;

namespace lambda {

#ifdef LAMBDA_PLATFORM_LINUX

// Will create a windows based implementation of the window handler.
Shared<core::Window> core::Window::Create(
    const core::WindowProperties& properties) {
  return memory::CreateShared<platform::linux::WindowImplementation>(
      properties);
}

#endif  // LAMBDA_PLATFORM_LINUX

namespace platform {
namespace linux {

// Error callback for handling GLFW specific errors
static void GLFWErrorCallback(int error, const char* description) {
  LAMBDA_CORE_ERROR("GFLW Error ({0}): {1}", error, description);
}

static bool GLFWInitialized = false;

WindowImplementation::WindowImplementation(
    const core::WindowProperties& properties) {
  Init(properties);
}

WindowImplementation::~WindowImplementation() {
  Shutdown();
}

// Initialize the windows given generic window properties to be applied to the
// current window.
void WindowImplementation::Init(const core::WindowProperties& properties) {
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
    LAMBDA_CORE_ASSERT(success, "Could not initialize GLFW!");
    glfwSetErrorCallback(GLFWErrorCallback);
    GLFWInitialized = true;
  }

  window_ = glfwCreateWindow(
      static_cast<int>(properties_.Width),
      static_cast<int>(properties_.Height),
      properties_.Title.c_str(),
      nullptr,
      nullptr);

  // TODO(C3NZ): Integrate the open gl context for the windows platform.
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

        core::memory::Shared<core::events::WindowResizeEvent> event =
            core::memory::CreateShared<core::events::WindowResizeEvent>(
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

        core::memory::Shared<core::events::WindowCloseEvent> event =
            core::memory::CreateShared<core::events::WindowCloseEvent>();
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
            core::memory::Shared<core::events::KeyPressedEvent> event =
                core::memory::CreateShared<core::events::KeyPressedEvent>(
                    key, 0);
            properties->EventCallback(event);
            break;
          }
          case GLFW_RELEASE:
          {
            core::memory::Shared<core::events::KeyReleasedEvent> event =
                core::memory::CreateShared<core::events::KeyReleasedEvent>(key);
            properties->EventCallback(event);
            break;
          }
          case GLFW_REPEAT:
          {
            core::memory::Shared<core::events::KeyPressedEvent> event =
                core::memory::CreateShared<core::events::KeyPressedEvent>(
                    key, 1);
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

            core::memory::Shared<core::events::KeyTypedEvent> event =
                core::memory::CreateShared<core::events::KeyTypedEvent>(
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
            core::memory::Shared<core::events::MouseButtonPressedEvent> event =
                core::memory::CreateShared<
                core::events::MouseButtonPressedEvent>(button);
            properties->EventCallback(event);
            break;
          }
          case GLFW_RELEASE:
          {
            core::memory::Shared<core::events::MouseButtonReleasedEvent> event =
                core::memory::CreateShared<
                    core::events::MouseButtonReleasedEvent>(button);
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

            core::memory::Shared<core::events::MouseScrolledEvent> event =
            core::memory::CreateShared<core::events::MouseScrolledEvent>(
                static_cast<float>(x_offset), static_cast<float>(y_offset));
        properties->EventCallback(event);
      });

  glfwSetCursorPosCallback(
      window_,
      [](GLFWwindow* window, double x_position, double y_position) {
        internal::Properties* properties =
            static_cast<internal::Properties*>(
                glfwGetWindowUserPointer(window));

            core::memory::Shared<core::events::MouseMovedEvent> event =
            core::memory::CreateShared<core::events::MouseMovedEvent>(
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

}  // namespace linux
}  // namespace platform
}  // namespace lambda

#endif  // LAMBDA_PLATFORM_LINUX
