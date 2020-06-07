/**
 * @file engine/src/core/renderer/RenderCommand.h
 * @brief The declaration file for the RenderCommand Wrapper.
 */
#ifndef ENGINE_SRC_CORE_RENDERER_RENDERCOMMAND_H_
#define ENGINE_SRC_CORE_RENDERER_RENDERCOMMAND_H_

#include "core/renderer/RendererAPI.h"

namespace engine {
namespace renderer {

/**
 * @class RenderCommand
 * @brief A static wrapper class to send commands to the Renderer.
 */
class RenderCommand {
 public:
  /**
   * @fn SetClearColor
   * @brief Sets the color to be used for clearing the screen.
   */
  inline static void SetClearColor(const glm::vec4& color) {
    renderer_API_->SetClearColor(color);
  }

  /**
   * @fn Clear
   * @brief Clear the screen.
   */
  inline static void Clear() {
    renderer_API_->Clear();
  }

  /**
   * @fn DrawIndexed
   * @brief Issues a platform specific graphics API to draw a vertex array.
   */
  static void DrawIndexed(const std::shared_ptr<VertexArray>& vertex_array) {
    renderer_API_->DrawIndexed(vertex_array);
  }

 private:
  static RendererAPI* renderer_API_;
};

}  // namespace renderer
}  // namespace engine

#endif  // ENGINE_SRC_CORE_RENDERER_RENDERCOMMAND_H_
