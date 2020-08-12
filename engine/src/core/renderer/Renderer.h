/**
 * @file Renderer.h
 * @brief The rendering API.
 */
#ifndef ENGINE_SRC_CORE_RENDERER_RENDERER_H_
#define ENGINE_SRC_CORE_RENDERER_RENDERER_H_

#include "core/renderer/RendererAPI.h"

#include "core/renderer/OrthographicCamera.h"
#include "core/renderer/Shader.h"


namespace engine {
namespace core {
namespace renderer {

class Renderer {
 public:
  static void Init();

  static void BeginScene(const OrthographicCamera& camera);
  static void EndScene();

  static void OnWindowResize(uint32_t width, uint32_t height);

  // TODO(C3NZ): update this to use engine memory allocators as opposed to
  // generic smart pointers.
  static void Submit(
      const std::shared_ptr<VertexArray>& vertex_array,
      const std::shared_ptr<Shader>& shader,
      const glm::mat4& transform = glm::mat4(1.0f));

  inline static RendererAPI::API GetAPI() { return RendererAPI::GetAPI(); }

 private:
  struct SceneData {
    glm::mat4 ViewProjectionMatrix;
  };

  static SceneData* scene_data_;
};

}  // namespace renderer
}  // namespace core
}  // namespace engine

#endif  // ENGINE_SRC_CORE_RENDERER_RENDERER_H_


/**
 * @class engine::renderer::Renderer
 * @brief A lightweight rendering API implementation. Allows generalized calls
 * to be written for users
 *
 * A lightweight and not fully finished rendering API that lets you set the a
 * specific graphics context to use for rendering. This must be set externally
 * in any rendering application.
 */

/**
 * @fn engine::renderer::Renderer::BeginScene
 * @brief Begin rendering a scene
 */

/**
 * @fn engine::renderer::Renderer::EndScene
 * @brief Stop rendering a scene.
 */

/**
 * @fn engine::renderer::Renderer::Submit
 * @brief Submit a scene data to the engine.
 */
