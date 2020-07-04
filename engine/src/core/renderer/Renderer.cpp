#include "core/renderer/Renderer.h"

#include <glm/glm.hpp>

#include "core/renderer/RenderCommand.h"
#include "core/renderer/OrthographicCamera.h"
#include "platform/opengl/OpenGLShader.h"

namespace engine {
namespace renderer {

Renderer::SceneData* Renderer::scene_data_ = new Renderer::SceneData;

void Renderer::BeginScene(const OrthographicCamera& camera) {
  scene_data_->ViewProjectionMatrix = camera.GetViewProjectionMatrix();
}
void Renderer::EndScene() {}

/**
 * Binds both the shader and vertex array before issuing a draw call.
 */
void Renderer::Submit(
    const std::shared_ptr<VertexArray>& vertex_array,
    const std::shared_ptr<Shader>& shader,
    const glm::mat4& transform) {
  shader->Bind();

  // TODO(C3NZ): This is a temporary cast to an opengl specific shader and
  // should be replaced when the rendering api becomes more mature.
  const auto& cast = std::dynamic_pointer_cast<platform::opengl::OpenGLShader>
      (shader);

  cast->UploadUniformMat4(
      "u_ViewProjection", scene_data_->ViewProjectionMatrix);
  cast->UploadUniformMat4("u_Transform", transform);

  vertex_array->Bind();
  RenderCommand::DrawIndexed(vertex_array);
}

}  // namespace renderer
}  // namespace engine
