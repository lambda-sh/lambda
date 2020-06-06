/**
 * @file engine/src/core/renderer/Renderer.h
 * @brief The rendering API.
 */
#ifndef ENGINE_SRC_CORE_RENDERER_RENDERER_H_
#define ENGINE_SRC_CORE_RENDERER_RENDERER_H_

#include "core/renderer/RendererAPI.h"

namespace engine {
namespace renderer {

/**
 * @class Renderer
 * @brief A lightweight rendering API implementation. Allows generalized calls
 * to be written for users
 *
 * A lightweight and not fully finished rendering API that lets you set the a
 * specific graphics context to use for rendering. This must be set externally
 * in any rendering application.
 */
class Renderer {
 public:
  /**
   * @fn BeginScene
   * @brief Begin rendering a scene
   */
  static void BeginScene();

  /**
   * Stop rendering a scene.
   */
  static void EndScene();

  static void Submit(const std::shared_ptr<VertexArray>& vertex_array);

  inline static RendererAPI::API GetAPI() { return RendererAPI::GetAPI(); }
 private:
};

}  // namespace renderer
}  // namespace engine

#endif  // ENGINE_SRC_CORE_RENDERER_RENDERER_H_
