#if defined ENGINE_PLATFORM_LINUX || defined ENGINE_DEBUG

#include "platform/linux/Input.h"

#include <utility>

#include <glad/glad.h>
#include <GLFW/glfw3.h>

#include "core/Application.h"
#include "core/Input.h"

namespace engine {

#ifdef ENGINE_PLATFORM_LINUX

Input* Input::kInput_ = new platform::linux::InputImplementation();

#endif  // ENGINE_PLATFORM_LINUX

namespace platform {
namespace linux {

bool InputImplementation::IsKeyPressedImpl(int key_code) {
  GLFWwindow* window = static_cast<GLFWwindow*>(
      Application::GetApplication().GetWindow().GetNativeWindow());

  int state = glfwGetKey(window, key_code);
  return state == GLFW_PRESS || state == GLFW_REPEAT;
}

float InputImplementation::GetMouseXImpl() {
  std::pair<float, float> position = GetMousePositionImpl();
  return position.first;
}

float InputImplementation::GetMouseYImpl() {
  std::pair<float, float> position = GetMousePositionImpl();
  return position.second;
}

std::pair<float, float> InputImplementation::GetMousePositionImpl() {
  GLFWwindow* window = static_cast<GLFWwindow*>(
      Application::GetApplication().GetWindow().GetNativeWindow());

  double x_pos, y_pos;
  glfwGetCursorPos(window, &x_pos, &y_pos);
  return std::make_pair(
      static_cast<float>(x_pos), static_cast<float>(y_pos));
}

bool InputImplementation::IsMouseButtonPressedImpl(int button) {
  GLFWwindow* window = static_cast<GLFWwindow*>(
      Application::GetApplication().GetWindow().GetNativeWindow());

  int state = glfwGetMouseButton(window, button);
  return state == GLFW_PRESS;
}

}  // namespace linux
}  // namespace platform
}  // namespace engine

#endif  // ENGINE_PLATFORM_LINUX
