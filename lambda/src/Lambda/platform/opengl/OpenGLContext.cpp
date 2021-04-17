#include "Lambda/platform/opengl/OpenGLContext.h"


#include <glad/glad.h>
#include <GLFW/glfw3.h>

#include "Lambda/lib/Assert.h"
#include "Lambda/core/Core.h"

namespace lambda {
namespace platform {
namespace opengl {

OpenGLContext::OpenGLContext(GLFWwindow* window_handle)
    : window_handle_(window_handle) {
  LAMBDA_CORE_ASSERT(window_handle_, "The window handle is null.", "");
}

void OpenGLContext::Init() {
  glfwMakeContextCurrent(window_handle_);

  // Initialize glad with glfw proc address.
  int status = gladLoadGLLoader(
      reinterpret_cast<GLADloadproc>(glfwGetProcAddress));
  LAMBDA_CORE_ASSERT(status, "Failed to initialize glad.", "");

  LAMBDA_CORE_INFO(
      "OpenGL Renderer: {0} - {1} - {2}",
      glGetString(GL_VENDOR),
      glGetString(GL_RENDERER),
      glGetString(GL_VERSION));
}

void OpenGLContext::SwapBuffers() {
  glfwSwapBuffers(window_handle_);
}

}  // namespace opengl
}  // namespace platform
}  // namespace lambda
