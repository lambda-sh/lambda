#ifndef ENGINE_SRC_CORE_RENDERER_RENDERERAPI_H_
#define ENGINE_SRC_CORE_RENDERER_RENDERERAPI_H_

#include <memory>

#include <glm/glm.hpp>

#include "core/renderer/VertexArray.h"

namespace engine {
namespace renderer {

/**
 * @class RendererAPI
 * @brief The Rendering API for displaying graphics through the engine api!
 */
class RendererAPI {
 public:
  /**
   * @enum API
   * @brief The APIs provided by the engine.
   */
  enum class API {
    None = 0,
    OpenGL = 1
  };

  /**
   * @fn SetClearColor
   * @brief Set the clear color to be used for the renderer.
   */
  virtual void SetClearColor(const glm::vec4& color) = 0;

  /**
   * @fn Clear
   * @brief Clear the screen.
   */
  virtual void Clear() = 0;

  /**
   * @fn DrawIndexed
   * @brief Render a vertex array.
   */
  virtual void DrawIndexed(const std::shared_ptr<VertexArray>& vertex_array)
      = 0;

  /**
   * @fn GetAPI
   * @brief Return the API being used within the engine.
   */
  static API GetAPI() { return API::OpenGL; }

 private:
  static API kAPI_;
};

}  // namespace renderer
}  // namespace engine

#endif  // ENGINE_SRC_CORE_RENDERER_RENDERERAPI_H_
