#include "Lambda/core/renderer/Renderer.h"

#include <glm/glm.hpp>

#include "Lambda/core/renderer/RenderCommand.h"
#include "Lambda/core/renderer/OrthographicCamera.h"
#include "Lambda/platform/opengl/OpenGLShader.h"

namespace lambda {
namespace core {
namespace renderer {

Renderer::SceneData* Renderer::scene_data_ = new Renderer::SceneData;

void Renderer::Init() {
  RenderCommand::Init();
}

void Renderer::BeginScene(const OrthographicCamera& camera) {
  scene_data_->ViewProjectionMatrix = camera.GetViewProjectionMatrix();
}

void Renderer::EndScene() {}

void Renderer::OnWindowResize(uint32_t width, uint32_t height) {
  RenderCommand::SetViewport(0, 0, width, height);
}

/// Binds both the shader and vertex array before issuing a draw call.
void Renderer::Submit(
      core::memory::Shared<VertexArray> vertex_array,
      core::memory::Shared<Shader> shader,
      const glm::mat4& transform) {
  shader->Bind();

  /// @todo (C3NZ): This is a temporary cast to an opengl specific shader and
  /// should be replaced when the rendering api becomes more mature.
  const auto& cast = std::dynamic_pointer_cast<platform::opengl::OpenGLShader>
      (shader);

  cast->UploadUniformMat4(
      "u_ViewProjection", scene_data_->ViewProjectionMatrix);
  cast->UploadUniformMat4("u_Transform", transform);

  vertex_array->Bind();
  RenderCommand::DrawIndexed(vertex_array);
}

}  // namespace renderer
}  // namespace core
}  // namespace lambda
