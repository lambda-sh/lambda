#include <Lambda/core/layers/GraphLayer.h>
#include <Lambda/core/renderer/Buffer.h>

namespace lambda::core::layers {

void GraphLayer2D::OnAttach() {
  auto v = renderer::VertexBuffer::CreateFromPoints(graph_.GetPoints());
  v->Bind();
}

void GraphLayer2D::OnDetach() {}

void GraphLayer2D::OnUpdate(util::TimeStep time_step) {}

void GraphLayer2D::OnEvent(memory::Shared<events::Event> event) {}

void GraphLayer2D::OnImGuiRender() {}

void GraphLayer3D::OnAttach() {}

void GraphLayer3D::OnDetach() {}

void GraphLayer3D::OnUpdate(util::TimeStep time_step) {}

void GraphLayer3D::OnEvent(memory::Shared<events::Event> event) {}

void GraphLayer3D::OnImGuiRender() {}

}  // namespace lambda::core::layers
