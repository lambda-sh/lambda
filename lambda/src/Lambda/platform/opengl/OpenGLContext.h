#ifndef LAMBDA_SRC_LAMBDA_PLATFORM_OPENGL_OPENGLCONTEXT_H_
#define LAMBDA_SRC_LAMBDA_PLATFORM_OPENGL_OPENGLCONTEXT_H_

#include <GLFW/glfw3.h>

#include "core/renderer/GraphicsContext.h"

namespace lambda {
namespace platform {
namespace opengl {

class OpenGLContext : public core::renderer::GraphicsContext {
 public:
  explicit OpenGLContext(GLFWwindow* window_handle);
  void Init() override;
  void SwapBuffers() override;

 private:
  GLFWwindow* window_handle_;
};

}  // namespace opengl
}  // namespace platform
}  // namespace lambda

#endif  // LAMBDA_SRC_LAMBDA_PLATFORM_OPENGL_OPENGLCONTEXT_H_
