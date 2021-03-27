/// @file OpenGLRendererAPI.h
/// @brief The OpenGL rendering implementation.
#ifndef LAMBDA_SRC_LAMBDA_PLATFORM_OPENGL_OPENGLRENDERERAPI_H_
#define LAMBDA_SRC_LAMBDA_PLATFORM_OPENGL_OPENGLRENDERERAPI_H_

#include <Lambda/core/memory/Pointers.h>
#include <Lambda/core/renderer/RendererAPI.h>
#include <Lambda/core/renderer/VertexArray.h>

namespace lambda::platform::opengl {

/// @class OpenGLRendererAPI
/// @brief The Rendering implementation for OpenGL.
class OpenGLRendererAPI : public core::renderer::RendererAPI {
 public:
  void Init() override;
  void SetClearColor(const glm::vec4& color) override;
  void SetViewport(uint32_t x, uint32_t y, uint32_t width, uint32_t height)
      override;
  void Clear() override;
  void DrawIndexed(
      core::memory::Shared<core::renderer::VertexArray> vertex_array) override;

  void DrawArrays(
      core::memory::Shared<core::renderer::VertexArray> vertex_array) override;
};

}  // namespace lambda::platform::opengl

#endif  // LAMBDA_SRC_LAMBDA_PLATFORM_OPENGL_OPENGLRENDERERAPI_H_
