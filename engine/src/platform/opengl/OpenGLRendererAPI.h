/**
 * @file engine/src/platform/opengl/OpenGLRendererAPI.h
 * @brief The OpenGL rendering implementation.
 */
#ifndef ENGINE_SRC_PLATFORM_OPENGL_OPENGLRENDERERAPI_H_
#define ENGINE_SRC_PLATFORM_OPENGL_OPENGLRENDERERAPI_H_

#include "core/renderer/RendererAPI.h"
#include "core/renderer/VertexArray.h"

namespace engine {
namespace platform {
namespace opengl {

/**
 * @class OpenGLRendererAPI
 * @brief The Rendering implementation for OpenGL.
 */
class OpenGLRendererAPI : public renderer::RendererAPI {
 public:
  void Init() override;
  void SetClearColor(const glm::vec4& color) override;
  void SetViewport(uint32_t x, uint32_t y, uint32_t width, uint32_t height) override;
  void Clear() override;

  void DrawIndexed(const std::shared_ptr<renderer::VertexArray>& vertex_array)
      override;

};


}  // namespace opengl
}  // namespace platform
}  // namespace engine

#endif  // ENGINE_SRC_PLATFORM_OPENGL_OPENGLRENDERERAPI_H_
