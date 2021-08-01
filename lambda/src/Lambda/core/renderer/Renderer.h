/// @file Renderer.h
/// @brief The rendering API.
#ifndef LAMBDA_SRC_LAMBDA_CORE_RENDERER_RENDERER_H_
#define LAMBDA_SRC_LAMBDA_CORE_RENDERER_RENDERER_H_

#include <Lambda/core/memory/Pointers.h>
#include <Lambda/core/renderer/OrthographicCamera.h>
#include <Lambda/core/renderer/RendererAPI.h>
#include <Lambda/core/renderer/Shader.h>

namespace lambda::core::renderer {

/// @brief The primary interface used for managing
class Renderer {
 public:
  /// @brief Initialize the Renderer.
  static void Init();

  /// @brief Begin a scene to be renderer.
  ///
  /// Uses a single orthograhpic camera to determine where things should be
  /// renderered on screen.
  static void BeginScene(const OrthographicCamera& camera);

  /// @begin Ends the rendering scene
  static void EndScene();

  /// @brief Updates the renderer with the new screen width and height.
  static void OnWindowResize(uint32_t width, uint32_t height);

  /// @brief Submit a vertex array, shader, and transform matrix for rendering.
  ///
  /// Must be associated with a specific scene. (used in between BeginScene and
  /// EndScene calls)
  static void Submit(
      memory::Shared<VertexArray> vertex_array,
      memory::Shared<Shader> shader,
      const glm::mat4& transform = glm::mat4(1.0f));

  /// @brief Get the API being used by the current renderer.
  static RendererAPI::API GetAPI() { return RendererAPI::GetAPI(); }

 private:
  struct SceneData {
    glm::mat4 ViewProjectionMatrix;
  };

  static SceneData* scene_data_;
};

}  // namespace lambda::core::renderer

#endif  // LAMBDA_SRC_LAMBDA_CORE_RENDERER_RENDERER_H_
