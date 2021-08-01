#ifndef LAMBDA_SRC_LAMBDA_CORE_LAYERS_GRAPHLAYER_H_
#define LAMBDA_SRC_LAMBDA_CORE_LAYERS_GRAPHLAYER_H_

#include <Lambda/concepts/Plot.h>
#include <Lambda/core/OrthographicCameraController.h>
#include <Lambda/core/layers/Layer.h>
#include <Lambda/core/renderer/Buffer.h>
#include <Lambda/core/renderer/RenderCommand.h>
#include <Lambda/core/renderer/Renderer.h>
#include <Lambda/core/renderer/Renderer2D.h>
#include <Lambda/math/plot/Graph.h>

namespace lambda::core::layers {
class GraphLayer2D : public Layer {
 public:
  explicit GraphLayer2D(math::plot::Graph2D<> graph)
      : Layer("Graph2D-Layer"),
      graph_(graph),
      camera_controller_(1280.0f / 720.0f, true) {}

  explicit GraphLayer2D(math::plot::Graph2D<>&& graph)
      : Layer("Graph2D-Layer"),
      graph_(std::move(graph)),
      camera_controller_(1280.0f / 720.0f, true) {}

  void OnAttach() override {
    for (auto& point : graph_.GetPoints()) {
      points_.push_back(point.x);
      points_.push_back(point.y);
    }
  }

  void OnDetach() override {
  }

  void OnUpdate(lib::TimeStep time_step) override {
    camera_controller_.OnUpdate(time_step);

    renderer::RenderCommand::SetClearColor({ 0.1f, 0.1f, 0.1f, 1.0f });
    renderer::RenderCommand::Clear();

    renderer::Renderer2D::BeginScene(
        camera_controller_.GetOrthographicCamera());

    for (auto& point : graph_.GetPoints()) {
      renderer::Renderer2D::DrawQuad(
        {point.x, point.y}, {0.5f, 0.5f}, {1.0f, 0.6f, 0.2f, 1.0f});
    }

    renderer::Renderer2D::EndScene();
  };

  void OnEvent(events::Event* const event) override {
    camera_controller_.OnEvent(event);
  }

  void OnImGuiRender() override {}

 private:
  math::plot::Graph2D<> graph_;
  memory::Shared<renderer::VertexArray> point_array_;
  memory::Shared<renderer::Shader> point_shader_;
  memory::Shared<renderer::VertexBuffer> point_buffer_;
  OrthographicCameraController camera_controller_;
  std::vector<float> points_;
};

}  // namespace lambda::core::layers

#endif  // LAMBDA_SRC_LAMBDA_CORE_LAYERS_GRAPHLAYER_H_
