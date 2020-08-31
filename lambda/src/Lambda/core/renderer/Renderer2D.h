/// @file Renderer2D.h
/// @brief The 2D rendering implementation.
#ifndef LAMBDA_SRC_LAMBDA_CORE_RENDERER_RENDERER2D_H_
#define LAMBDA_SRC_LAMBDA_CORE_RENDERER_RENDERER2D_H_

#include "Lambda/core/memory/Pointers.h"
#include "Lambda/core/renderer/OrthographicCamera.h"
#include "Lambda/core/renderer/Texture.h"

namespace lambda {
namespace core {
namespace renderer {

/// @brief The 2D rendering API.
///
/// The entire 2D Rendering API is exposed entirely via static function calls.
class Renderer2D {
 public:
  /// @brief Initialize the 2D rendering system.
  static void Init();

  /// @brief Shutdown the 2D rendering system.
  static void Shutdown();

  /// @brief Render a scene for a given camera.
  static void BeginScene(const OrthographicCamera& camera);

  /// @brief End the scene.
  static void EndScene();

  /// @brief Draw a quad given it's position, size, and color.
  static void DrawQuad(
      const glm::vec2& position,
      const glm::vec2& size,
      const glm::vec4& color);

  /// @brief Draw a quad given it's position, size, and color.
  static void DrawQuad(
      const glm::vec3& position,
      const glm::vec2& size,
      const glm::vec4& color);

  /// @brief Draw a quad given it's position, size, and Texture.
  static void DrawQuad(
      const glm::vec2& position,
      const glm::vec2& size,
      core::memory::Shared<Texture2D> texture);

  /// @brief Draw a quad given it's position, size, and Texture.
  static void DrawQuad(
      const glm::vec3& position,
      const glm::vec2& size,
      core::memory::Shared<Texture2D> texture);
};

}  // namespace renderer
}  // namespace core
}  // namespace lambda

#endif  // LAMBDA_SRC_LAMBDA_CORE_RENDERER_RENDERER2D_H_
