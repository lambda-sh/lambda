/**
 * @file RendererAPI.h
 * @brief the rendering API that handles all draw calls.
 */
#ifndef ENGINE_SRC_CORE_RENDERER_RENDERERAPI_H_
#define ENGINE_SRC_CORE_RENDERER_RENDERERAPI_H_

#include <memory>

#include <glm/glm.hpp>

#include "core/renderer/VertexArray.h"

namespace engine {
namespace renderer {

class RendererAPI {
 public:
  enum class API {
    None = 0,
    OpenGL = 1
  };

  virtual void Init() = 0;
  virtual void SetClearColor(const glm::vec4& color) = 0;
  virtual void Clear() = 0;
  virtual void DrawIndexed(const std::shared_ptr<VertexArray>& vertex_array)
      = 0;

  static API GetAPI() { return API::OpenGL; }

 private:
  static API kAPI_;
};

}  // namespace renderer
}  // namespace engine

#endif  // ENGINE_SRC_CORE_RENDERER_RENDERERAPI_H_

/**
 * @class engine::renderer::RendererAPI
 * @brief The Rendering API for displaying graphics through the engine api!
 */

/**
 * @enum engine::renderer::RendererAPI::API
 * @brief The APIs provided by the engine.
 */

/**
 * @fn engine::renderer::RendererAPI::SetClearColor
 * @brief Set the clear color to be used for the renderer.
 */

/**
 * @fn engine::renderer::RendererAPI::Clear
 * @brief Clear the screen.
 */

/**
 * @fn engine::renderer::RendererAPI::DrawIndexed
 * @brief Render a vertex array.
 */

/**
 * @fn engine::renderer::RendererAPI::GetAPI
 * @brief Return the API being used within the engine.
 */
