#include "core/renderer/Renderer.h"
#include "core/renderer/RenderCommand.h"

namespace engine {
namespace renderer {

void Renderer::BeginScene() {}
void Renderer::EndScene() {}

void Renderer::Submit(const std::shared_ptr<VertexArray> &vertex_array) {
  vertex_array->Bind();
  RenderCommand::DrawIndexed(vertex_array);
}

}  // namespace renderer
}  // namespace engine
