#ifndef LAMBDA_SRC_LAMBDA_CORE_LAYERS_GRAPHLAYER_H_
#define LAMBDA_SRC_LAMBDA_CORE_LAYERS_GRAPHLAYER_H_

#include <vector>

#include <Lambda/core/layers/Layer.h>
#include <Lambda/core/renderer/Buffer.h>
#include <Lambda/math/shapes/Point.h>
#include <Lambda/math/plot/Graph.h>

namespace lambda::core::layers {

class GraphLayer2D : public Layer {
 public:
  explicit GraphLayer2D(
    math::plot::Graph2D<> graph)
        : Layer("Graph-Layer"), graph_(std::move(graph)) {}
  void OnAttach() override;
  void OnDetach() override;
  void OnUpdate(util::TimeStep time_step) override;
  void OnEvent(memory::Shared<events::Event> event) override;
  void OnImGuiRender() override;
 private:
  math::plot::Graph2D<> graph_;
};

class GraphLayer3D : public Layer {
 public:
  GraphLayer3D() : Layer("Graph-Layer") {}

  explicit GraphLayer3D(
      std::vector<lambda::math::shapes::Point3D<>> points)
          : Layer("Graph-Layer"), points_(points) {}

  explicit GraphLayer3D(
      std::vector<lambda::math::shapes::Point3D<>>&& points)
          : Layer("Graph-Layer"), points_(std::move(points)) {}

  void OnAttach() override;
  void OnDetach() override;
  void OnUpdate(util::TimeStep time_step) override;
  void OnEvent(memory::Shared<events::Event> event) override;
  void OnImGuiRender() override;
 private:
  std::vector<lambda::math::shapes::Point3D<>> points_;
};

}  // namespace lambda::core::layers

#endif  // LAMBDA_SRC_LAMBDA_CORE_LAYERS_GRAPHLAYER_H_
