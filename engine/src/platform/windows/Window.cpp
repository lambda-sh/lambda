#if defined ENGINE_PLATFORM_WINDOWS || defined ENGINE_DEBUG

#include "platform/windows/Window.h"

#include <glad/glad.h>
#include <GLFW/glfw3.h>

#include "core/Assert.h"
#include "core/Core.h"
#include "core/Log.h"
#include "core/Window.h"
#include "core/events/ApplicationEvent.h"
#include "core/events/KeyEvent.h"
#include "core/events/MouseEvent.h"

namespace engine {

#ifdef ENGINE_PLATFORM_WINDOWS

Window* Window::Create(const engine::WindowProperties& properties) {
  return new platform::windows::WindowImplementation(properties);
}

#endif  // ENGINE_PLATFORM_WINDOWS

namespace platform {
namespace windows {

// Error callback for handling GLFW specific errors
static void GLFWErrorCallback(int error, const char* description) {
  ENGINE_CORE_ERROR("GFLW Error ({0}): {1}", error, description);
}

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

  // Initialize GLFW
  glfwMakeContextCurrent(window_);
  glfwSetWindowUserPointer(window_, &properties_);

  int status = gladLoadGLLoader(
      reinterpret_cast<GLADloadproc>(glfwGetProcAddress));
  ENGINE_CORE_ASSERT(status, "Failed to initialize glad.");
  SetVerticalSync(true);

  glfwSetWindowSizeCallback(
      window_,
      [](GLFWwindow* window, int width, int height) {
        internal::Properties* properties =
              static_cast<internal::Properties*>(
                  glfwGetWindowUserPointer(window));

          events::WindowResizeEvent event(width, height);
          properties->Width = width;
          properties->Height = height;

          properties->EventCallback(&event);
      });

  glfwSetWindowCloseCallback(
      window_,
      [](GLFWwindow* window) {
      internal::Properties& properties =
            *static_cast<internal::Properties*>(
                glfwGetWindowUserPointer(window));

            events::WindowCloseEvent event;
            properties.EventCallback(&event);
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
            events::KeyPressedEvent event(key, 0);
            properties->EventCallback(&event);
            break;
          }
          case GLFW_RELEASE:
          {
            events::KeyReleasedEvent event(key);
            properties->EventCallback(&event);
            break;
          }
          case GLFW_REPEAT:
          {
            events::KeyPressedEvent event(key, 1);
            properties->EventCallback(&event);
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

        events::KeyTypedEvent event(character);
        properties->EventCallback(&event);
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
            events::MouseButtonPressedEvent event(button);
            properties->EventCallback(&event);
            break;
          }
          case GLFW_RELEASE:
          {
            events::MouseButtonReleasedEvent event(button);
            properties->EventCallback(&event);
            break;
          }
        }
      });

  glfwSetScrollCallback(
      window_,
      [](GLFWwindow* window, double xOffset, double yOffset) {
      internal::Properties* properties =
            static_cast<internal::Properties*>(
                glfwGetWindowUserPointer(window));

        events::MouseScrolledEvent event(
            static_cast<float>(xOffset), static_cast<float>(yOffset));
        properties->EventCallback(&event);
      });

  glfwSetCursorPosCallback(
      window_,
      [](GLFWwindow* window, double xPosition, double yPosition) {
      internal::Properties* properties =
            static_cast<internal::Properties*>(
                glfwGetWindowUserPointer(window));

        events::MouseMovedEvent event(
            static_cast<float>(xPosition), static_cast<float>(yPosition));
        properties->EventCallback(&event);
      });
}

// Shutdown the window.
void WindowImplementation::Shutdown() {
  glfwDestroyWindow(window_);
}

// Handling updates to the screen.
void WindowImplementation::OnUpdate() {
  glfwPollEvents();
  glfwSwapBuffers(window_);
  glClearColor(1.0f, 1.0f, 1.0f, 1.0f);
  glClear(GL_COLOR_BUFFER_BIT);
}

// Setup the current window to use or not use Vsync.
void WindowImplementation::SetVerticalSync(bool enabled) {
  if (enabled) {
    glfwSwapInterval(1);
  } else {
    glfwSwapInterval(0);
  }

  properties_.VerticalSync = enabled;
}

// Check if the current window has VSync enabled.
bool WindowImplementation::HasVerticalSync() const {
  return properties_.VerticalSync;
}

}  // namespace windows
}  // namespace platform
}  // namespace engine

#endif  // ENGINE_PLATFORM_WINDOWS
