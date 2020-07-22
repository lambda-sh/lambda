/**
 * @file RenderCommand.h
 * @brief The declaration file for the RenderCommand Wrapper.
 */
#ifndef ENGINE_SRC_CORE_RENDERER_RENDERCOMMAND_H_
#define ENGINE_SRC_CORE_RENDERER_RENDERCOMMAND_H_

#include "core/renderer/RendererAPI.h"
#include "core/memory/Pointers.h"

namespace engine {
namespace renderer {

class RenderCommand {
 public:
  inline static void Init() { renderer_API_->Init(); }

  inline static void SetClearColor(const glm::vec4& color) {
    renderer_API_->SetClearColor(color);
  }

  inline static void Clear() {
    renderer_API_->Clear();
  }

  static void DrawIndexed(const memory::Shared<VertexArray>& vertex_array) {
    renderer_API_->DrawIndexed(vertex_array);
  }

 private:
  static memory::Unique<RendererAPI> renderer_API_;
};

}  // namespace renderer
}  // namespace engine

#endif  // ENGINE_SRC_CORE_RENDERER_RENDERCOMMAND_H_

/**
 * @class engine::renderer::RenderCommand
 * @brief A static wrapper class to send commands to the Renderer.
 */

/**
 * @fn engine::renderer::RenderCommand::SetClearColor
 * @brief Sets the color to be used for clearing the screen.
 */

/**
 * @fn engine::renderer::RenderCommand::Clear
 * @brief Clear the screen.
 */

/**
 * @fn engine::renderer::RenderCommand::DrawIndexed
 * @brief Issues a platform specific graphics API to draw a vertex array.
 */
