/**
 * @file RendererAPI.h
 * @brief the rendering API that handles all draw calls.
 */
#ifndef LAMBDA_SRC_CORE_RENDERER_RENDERERAPI_H_
#define LAMBDA_SRC_CORE_RENDERER_RENDERERAPI_H_

#include <memory>

#include <glm/glm.hpp>

#include "core/renderer/VertexArray.h"

namespace lambda {
namespace core {
namespace renderer {

class RendererAPI {
 public:
  enum class API {
    None = 0,
    OpenGL = 1
  };

  virtual void Init() = 0;
  virtual void SetClearColor(const glm::vec4& color) = 0;
  virtual void SetViewport(
      uint32_t x, uint32_t y, uint32_t width, uint32_t height) = 0;
  virtual void Clear() = 0;
  virtual void DrawIndexed(const std::shared_ptr<VertexArray>& vertex_array)
      = 0;

  static API GetAPI() { return API::OpenGL; }

 private:
  static API kAPI_;
};

}  // namespace renderer
}  // namespace core
}  // namespace lambda

#endif  // LAMBDA_SRC_CORE_RENDERER_RENDERERAPI_H_

/**
 * @class lambda::renderer::RendererAPI
 * @brief The Rendering API for displaying graphics through the engine api!
 */

/**
 * @enum lambda::renderer::RendererAPI::API
 * @brief The APIs provided by the engine.
 */

/**
 * @fn lambda::renderer::RendererAPI::SetClearColor
 * @brief Set the clear color to be used for the renderer.
 */

/**
 * @fn lambda::renderer::RendererAPI::Clear
 * @brief Clear the screen.
 */

/**
 * @fn lambda::renderer::RendererAPI::DrawIndexed
 * @brief Render a vertex array.
 */

/**
 * @fn lambda::renderer::RendererAPI::GetAPI
 * @brief Return the API being used within the engine.
 */
