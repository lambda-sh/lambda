#ifndef LAMBDA_SRC_LAMBDA_CORE_LAYERS_GRAPHLAYER_H_
#define LAMBDA_SRC_LAMBDA_CORE_LAYERS_GRAPHLAYER_H_

#include <Lambda/concepts/Plot.h>
#include <Lambda/core/layers/Layer.h>
#include <Lambda/core/renderer/Buffer.h>

namespace lambda::core::layers {

template <concepts::Graph Plot>
class GraphLayer2D : public Layer {
 public:
  explicit GraphLayer2D(
    Plot graph)
        : Layer("Graph2D-Layer"), graph_(std::move(graph)) {}

  explicit GraphLayer2D(Plot&& graph)
      : Layer("Graph2D-Layer"), graph_(std::move(graph)) {}

  void OnAttach() override;
  void OnDetach() override;
  void OnUpdate(util::TimeStep time_step) override;
  void OnEvent(memory::Shared<events::Event> event) override;
  void OnImGuiRender() override;
 private:
  Plot graph_;
};

template<concepts::Graph Plot>
class GraphLayer3D : public Layer {
 public:
  explicit GraphLayer3D(Plot graph)
          : Layer("Graph3D-Layer"), graph_(std::move(graph)) {}

  explicit GraphLayer3D(Plot&& graph)
          : Layer("Graph3D-Layer"), graph_(std::move(graph)) {}

  void OnAttach() override;
  void OnDetach() override;
  void OnUpdate(util::TimeStep time_step) override;
  void OnEvent(memory::Shared<events::Event> event) override;
  void OnImGuiRender() override;
 private:
  Plot graph_;
};

}  // namespace lambda::core::layers

#endif  // LAMBDA_SRC_LAMBDA_CORE_LAYERS_GRAPHLAYER_H_
