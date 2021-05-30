#if defined LAMBDA_PLATFORM_WINDOWS || defined LAMBDA_DEBUG

#include <Lambda/platform/windows/Window.h>

#include <Lambda/core/Core.h>
#include <Lambda/core/Window.h>
#include <Lambda/core/events/ApplicationEvent.h>
#include <Lambda/core/events/KeyEvent.h>
#include <Lambda/core/events/MouseEvent.h>
#include <Lambda/core/memory/Pointers.h>

#include <Lambda/lib/Assert.h>
#include <Lambda/lib/Log.h>

#include <Lambda/platform/glfw/GLFW.h>
#include <Lambda/platform/opengl/OpenGLContext.h>

namespace lambda {

#ifdef LAMBDA_PLATFORM_WINDOWS

namespace {

using lambda::core::memory::Unique;

}  // namespace

// Will create a windows based implementation of the window handler.
core::memory::Unique<core::Window> core::Window::Create(
    WindowProperties properties) {
  return core::memory::CreateUnique<platform::windows::Window>(
      std::move(properties));
}

#endif  // LAMBDA_PLATFORM_WINDOWS

namespace platform::windows {

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

Window::Window(
    core::WindowProperties properties) {
  Init(std::move(properties));
}

Window::~Window() {
  Shutdown();
}

// Initialize the windows given generic window properties to be applied to the
// current window.
void Window::Init(core::WindowProperties properties) {
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

        core::memory::Unique<core::events::WindowResizeEvent> event =
            core::memory::CreateUnique<core::events::WindowResizeEvent>(
                new_width, new_height);

        properties->Width = new_width;
        properties->Height = new_height;

        properties->EventCallback(std::move(event));
      });

  glfwSetWindowCloseCallback(
      window_,
      [](GLFWwindow* window) {
        internal::Properties* properties =
            static_cast<internal::Properties*>(
                glfwGetWindowUserPointer(window));

        core::memory::Unique<core::events::WindowCloseEvent> event =
            core::memory::CreateUnique<core::events::WindowCloseEvent>();
        properties->EventCallback(std::move(event));
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
            core::memory::Unique<core::events::KeyPressedEvent> event =
                core::memory::CreateUnique<core::events::KeyPressedEvent>(
                    key, 0);
            properties->EventCallback(std::move(event));
            break;
          }
          case GLFW_RELEASE:
          {
            core::memory::Unique<core::events::KeyReleasedEvent> event =
                core::memory::CreateUnique<core::events::KeyReleasedEvent>(key);
            properties->EventCallback(std::move(event));
            break;
          }
          case GLFW_REPEAT:
          {
            core::memory::Unique<core::events::KeyPressedEvent> event =
                core::memory::CreateUnique<core::events::KeyPressedEvent>(
                    key, 1);
            properties->EventCallback(std::move(event));
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

            core::memory::Unique<core::events::KeyTypedEvent> event =
                core::memory::CreateUnique<core::events::KeyTypedEvent>(
                    character);
            properties->EventCallback(std::move(event));
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
            core::memory::Unique<core::events::MouseButtonPressedEvent> event =
                core::memory::CreateUnique<
                core::events::MouseButtonPressedEvent>(button);
            properties->EventCallback(std::move(event));
            break;
          }
          case GLFW_RELEASE:
          {
            core::memory::Unique<core::events::MouseButtonReleasedEvent> event =
                core::memory::CreateUnique<
                    core::events::MouseButtonReleasedEvent>(button);
            properties->EventCallback(std::move(event));
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

            core::memory::Unique<core::events::MouseScrolledEvent> event =
            core::memory::CreateUnique<core::events::MouseScrolledEvent>(
                static_cast<float>(x_offset), static_cast<float>(y_offset));
        properties->EventCallback(std::move(event));
      });

  glfwSetCursorPosCallback(
      window_,
      [](GLFWwindow* window, double x_position, double y_position) {
        internal::Properties* properties =
            static_cast<internal::Properties*>(
                glfwGetWindowUserPointer(window));

            core::memory::Unique<core::events::MouseMovedEvent> event =
            core::memory::CreateUnique<core::events::MouseMovedEvent>(
                static_cast<float>(x_position), static_cast<float>(y_position));
        properties->EventCallback(std::move(event));
      });
}

// Shutdown the window.
void Window::Shutdown() {
  glfwDestroyWindow(window_);
}

// Handling updates to the screen.
void Window::OnUpdate() {
  glfwPollEvents();
  context_->SwapBuffers();
}

// Setup the current window to use or not use Vertical sync.
void Window::SetVerticalSync(bool enabled) {
  glfwSwapInterval(enabled ? 1 : 0);
  properties_.VerticalSync = enabled;
}

// Check if the current window has VSync enabled.
bool Window::HasVerticalSync() const {
  return properties_.VerticalSync;
}

}  // namespace platform::windows
}  // namespace lambda

#endif  // LAMBDA_PLATFORM_WINDOWS
