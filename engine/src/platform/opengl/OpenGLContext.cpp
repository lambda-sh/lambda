#include "platform/opengl/OpenGLContext.h"


#include <glad/glad.h>
#include <GLFW/glfw3.h>

#include "core/Assert.h"
#include "core/Core.h"

namespace engine {
namespace platform {
namespace opengl {

OpenGLContext::OpenGLContext(GLFWwindow* window_handle)
    : window_handle_(window_handle) {
  ENGINE_CORE_ASSERT(window_handle_, "The window handle is null.");
}

void OpenGLContext::Init() {
  glfwMakeContextCurrent(window_handle_);

  // Initialize glad with glfw proc address.
  int status = gladLoadGLLoader(
      reinterpret_cast<GLADloadproc>(glfwGetProcAddress));
  ENGINE_CORE_ASSERT(status, "Failed to initialize glad.");

  ENGINE_CORE_INFO(
      "OpenGL Renderer: {0} - {1} - {2}",
      glGetString(GL_VENDOR),
      glGetString(GL_RENDERER),
      glGetString(GL_VERSION));
}

void OpenGLContext::SwapBuffers() {
  int width, height;

  glfwGetFramebufferSize(window_handle_, &width, &height);
  glViewport(0, 0, width, height);

  glfwSwapBuffers(window_handle_);
}

}  // namespace opengl
}  // namespace platform
}  // namespace engine
