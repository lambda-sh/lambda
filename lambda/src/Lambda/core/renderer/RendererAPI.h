/// @file RendererAPI.h
/// @brief the rendering API that handles all draw calls.
#ifndef LAMBDA_SRC_LAMBDA_CORE_RENDERER_RENDERERAPI_H_
#define LAMBDA_SRC_LAMBDA_CORE_RENDERER_RENDERERAPI_H_

#include <memory>

#include <glm/glm.hpp>

#include "Lambda/core/renderer/VertexArray.h"

namespace lambda {
namespace core {
namespace renderer {

/// @brief The abstract representation of
/// rendering features and functions supported by Lambda.
///
/// Platform specific APIs implement most to all of these functions.
class RendererAPI {
 public:
  /// @brief APIs supported by the engine.
  enum class API {
    None = 0,
    OpenGL = 1
  };

  /// @brief Setup the API for rendering.
  virtual void Init() = 0;

  /// @brief Setup the APIs screen clear color.
  virtual void SetClearColor(const glm::vec4& color) = 0;

  /// @brief Handle setting the viewport.
  virtual void SetViewport(
      uint32_t x, uint32_t y, uint32_t width, uint32_t height) = 0;

  /// @brief Handle clearing the screen.
  virtual void Clear() = 0;

  /// @brief Handle drawing a vertex array.
  virtual void DrawIndexed(const std::shared_ptr<VertexArray>& vertex_array)
      = 0;

  /// @brief Return the API that is being used. (Currently only supports opengl)
  static API GetAPI() { return API::OpenGL; }

 private:
  static API kAPI_;
};

}  // namespace renderer
}  // namespace core
}  // namespace lambda

#endif  // LAMBDA_SRC_LAMBDA_CORE_RENDERER_RENDERERAPI_H_
