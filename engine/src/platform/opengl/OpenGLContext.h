#ifndef ENGINE_SRC_PLATFORM_OPENGL_OPENGLCONTEXT_H_
#define ENGINE_SRC_PLATFORM_OPENGL_OPENGLCONTEXT_H_

#include <GLFW/glfw3.h>

#include "core/renderer/GraphicsContext.h"

namespace engine {
namespace platform {
namespace opengl {


class OpenGLContext : public renderer::GraphicsContext {
 public:
  explicit OpenGLContext(GLFWwindow* window_handle);
  void Init() override;
  void SwapBuffers() override;

 private:
  GLFWwindow* window_handle_;
};

}  // namespace opengl
}  // namespace platform
}  // namespace engine

#endif  // ENGINE_SRC_PLATFORM_OPENGL_OPENGLCONTEXT_H_
