/// @file OpenGLContext.h
/// @brief The OpenGL graphics context implementation.
#ifndef LAMBDA_SRC_LAMBDA_PLATFORM_OPENGL_OPENGLCONTEXT_H_
#define LAMBDA_SRC_LAMBDA_PLATFORM_OPENGL_OPENGLCONTEXT_H_


#include "Lambda/core/renderer/GraphicsContext.h"
#include <Lambda/platform/glfw/GLFW.h>

namespace lambda::platform::opengl {

/// @brief The graphics context for
class OpenGLContext : public core::renderer::GraphicsContext {
 public:
  explicit OpenGLContext(GLFWwindow* window_handle);
  void Init() override;
  void SwapBuffers() override;

 private:
  GLFWwindow* window_handle_;
};

}  // namespace lambda::platform::opengl

#endif  // LAMBDA_SRC_LAMBDA_PLATFORM_OPENGL_OPENGLCONTEXT_H_
