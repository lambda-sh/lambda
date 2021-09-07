#include <Lambda/Lambda.h>
#include <Lambda/core/Entrypoint.h>
#include <Lambda/math/Vector.h>
#include <Lambda/math/shapes/Point.h>
#include <Lambda/math/plot/Graph.h>
#include <Lambda/core/layers/GraphLayer.h>
#include <Lambda/profiler/Profiler.h>

using lambda::core::memory::Unique;
using lambda::core::memory::Shared;
using lambda::core::memory::CreateUnique;
using lambda::core::memory::CreateShared;

using lambda::math::shapes::Point2D;
using lambda::math::plot::Graph2D;

using lambda::core::layers::GraphLayer2D;
using lambda::math::Vector2;

using lambda::core::renderer::RenderCommand;
using lambda::core::renderer::Renderer2D;

class ProfileLayer final : public lambda::core::layers::Layer {
 public:
  ProfileLayer()
      : Layer("Profiling layer"), camera_controller_(1280.0f / 720.0f, true) {}

  void OnUpdate(lambda::lib::TimeStep time_step) override {
    LAMBDA_PROFILER_MEASURE_FUNCTION();
    camera_controller_.OnUpdate(time_step);

    RenderCommand::SetClearColor({ 0.1f, 0.1f, 0.1f, 1.0f });
    RenderCommand::Clear();

    Renderer2D::BeginScene(
        camera_controller_.GetOrthographicCamera());


    z += (0.2f * time_step.InMicroseconds<float>());

    if (z > 1000) {
      z = 0;
    }

    for (Vector2& vec : vectors_) {
      auto new_pos = vec.GetX() + z;
      if (z == 0) {
        new_pos = 0;
      }

      vec.SetX(new_pos);
      vec.SetY(sin(new_pos));

      lambda::core::renderer::Renderer2D::DrawQuad(
          {vec.GetX(), vec.GetY()}, {0.5f, 0.5f}, {0.9f, 0.2f, 0.5f, 1.0f});
    }
    Renderer2D::EndScene();
  }
  void OnAttach() override {
    LAMBDA_PROFILER_MEASURE_FUNCTION();
    vectors_ = std::vector<Vector2>(200);
  };
  void OnDetach() override {}
  void OnEvent(lambda::core::events::Event* const event) override {
    camera_controller_.OnEvent(event);
  }

  void OnImGuiRender() override {}

 private:
  lambda::core::OrthographicCameraController camera_controller_;
  std::vector<Vector2> vectors_;
  float z = 0;
};

class MathBox final : public Application {
 public:
  MathBox() : Application() {
    std::vector<Point2D<>> points(200);
    std::fill(points.begin(), points.end(), Point2D<>());

    int x = 0;
    for (auto& point : points) {
      point.x = x;
      point.y = sin(x);
      x += 1;
    }

    lambda::math::Vector vec({0, 1});
    lambda::math::Vector vec2({1, 1});
    lambda::math::Vector vec3(vec + vec2);
    LAMBDA_CORE_INFO("{}", vec3.GetRawElements()[0]);

    Vector2 test = {0, 0};
    Vector2 oof = {3, 1};
    auto test_vec = test + oof;
    LAMBDA_CORE_INFO("{}", test_vec.GetX());
    LAMBDA_CORE_INFO("{}", lambda::math::PerimeterOf(
        {{1, 0}, {1, 1}, {0, 1}, {0, 0}}));

    auto cartesian = lambda::math::PolarToCartesian(
        {5.0, lambda::math::DegreeToRadians(37.0)});
    LAMBDA_CORE_INFO(
        "Cartesian coordinates of (5.0, 37.0) are: ({}, {})",
        cartesian.GetX(),
        cartesian.GetY());

    auto polar = lambda::math::CartesianToPolar({1.0, 0.0});
    LAMBDA_CORE_INFO(
        "Polar coordinates of (1.0, 0.0) are: ({}, {})",
        polar.GetX(),
        polar.GetY());

    auto polar2 = lambda::math::CartesianToPolar({-2, 3});
    LAMBDA_CORE_INFO(
        "Polar coordinates of (-2, 3) are: ({}, {})",
        polar2.GetX(),
        polar2.GetY());

    Graph2D graph(points);
    PushLayer(CreateUnique<ProfileLayer>());
    // PushLayer(CreateShared<GraphLayer2D>(graph));
  }
  ~MathBox() {}
};

Unique<Application> lambda::core::CreateApplication() {
  return memory::CreateUnique<MathBox>();
}
