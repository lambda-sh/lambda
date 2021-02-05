#if defined LAMBDA_PLATFORM_WINDOWS || defined LAMBDA_DEBUG

#include "Lambda/platform/windows/Input.h"

#include <utility>

#include <glad/glad.h>
#include <GLFW/glfw3.h>

#include "Lambda/core/Application.h"
#include "Lambda/core/input/Input.h"

namespace lambda {

#ifdef LAMBDA_PLATFORM_WINDOWS

core::input::Input* core::input::Input::kInput_ = new platform::windows::InputImplementation();

#endif  // LAMBDA_PLATFORM_WINDOWS

namespace platform {
namespace windows {


bool InputImplementation::IsKeyPressedImpl(int key_code) {
  GLFWwindow* window = static_cast<GLFWwindow*>(
      core::Application::GetApplication().GetWindow().GetNativeWindow());

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
      core::Application::GetApplication().GetWindow().GetNativeWindow());

  double x_pos, y_pos;
  glfwGetCursorPos(window, &x_pos, &y_pos);
  return std::make_pair(
      static_cast<float>(x_pos), static_cast<float>(y_pos));
}

bool InputImplementation::IsMouseButtonPressedImpl(int button) {
  GLFWwindow* window = static_cast<GLFWwindow*>(
      core::Application::GetApplication().GetWindow().GetNativeWindow());

  int state = glfwGetMouseButton(window, button);
  return state == GLFW_PRESS;
}

}  // namespace windows
}  // namespace platform
}  // namespace lambda

#endif  // LAMBDA_PLATFORM_WINDOWS
