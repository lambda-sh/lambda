#include "core/renderer/Renderer.h"

#include "core/renderer/RenderCommand.h"
#include "core/renderer/OrthographicCamera.h"

namespace engine {
namespace renderer {

Renderer::SceneData* Renderer::scene_data_ = new Renderer::SceneData;

void Renderer::BeginScene(const OrthographicCamera& camera) {
  scene_data_->ViewProjectionMatrix = camera.GetViewProjectionMatrix();
}
void Renderer::EndScene() {}

void Renderer::Submit(
    const std::shared_ptr<VertexArray>& vertex_array,
    const std::shared_ptr<Shader>& shader) {
  shader->Bind();
  shader->UploadUniformMat4(
      "u_ViewProjection", scene_data_->ViewProjectionMatrix);

  vertex_array->Bind();
  RenderCommand::DrawIndexed(vertex_array);
}

}  // namespace renderer
}  // namespace engine
