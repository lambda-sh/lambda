/// @file RenderCommand.h
/// @brief The declaration file for the RenderCommand Wrapper.
#ifndef LAMBDA_SRC_LAMBDA_CORE_RENDERER_RENDERCOMMAND_H_
#define LAMBDA_SRC_LAMBDA_CORE_RENDERER_RENDERCOMMAND_H_

#include "Lambda/core/memory/Pointers.h"
#include "Lambda/core/renderer/RendererAPI.h"

namespace lambda::core::renderer {

/// @brief An interface for issuing commands to the current rendering API that
/// is being used with lambda.
class RenderCommand {
 public:
  /// @brief Initialize the Rendering API.
  static void Init() { renderer_API_->Init(); }

  /// @brief Set the color to clear the screen with.
  static void SetClearColor(const glm::vec4& color) {
    renderer_API_->SetClearColor(color);
  }

  /// @brief Set the viewport
  static void SetViewport(
      uint32_t x, uint32_t y, uint32_t width, uint32_t height) {
    renderer_API_->SetViewport(x, y, width, height);
  }

  /// @brief Clear the screen of anything that was previously rendered.
  static void Clear() {
    renderer_API_->Clear();
  }

  /// @brief Issues a draw call to the rendering API.
  static void DrawIndexed(memory::Shared<VertexArray> vertex_array) {
    renderer_API_->DrawIndexed(vertex_array);
  }

  static void DrawArrays(memory::Shared<VertexArray> vertex_array) {
    renderer_API_->DrawArrays(vertex_array);
  }

 private:
  static memory::Unique<RendererAPI> renderer_API_;
};

}  // namespace lambda::core::renderer

#endif  // LAMBDA_SRC_LAMBDA_CORE_RENDERER_RENDERCOMMAND_H_
