#include <Lambda/Lambda.h>
#include <Lambda/core/Entrypoint.h>
#include <Lambda/math/Vector.h>
#include <Lambda/math/shapes/Point.h>
#include <Lambda/math/plot/Graph.h>
#include <Lambda/core/layers/GraphLayer.h>

using lambda::core::memory::Unique;
using lambda::core::memory::CreateUnique;
using lambda::core::memory::CreateShared;

using lambda::math::shapes::Point2D;
using lambda::math::plot::Graph2D;

using lambda::core::layers::GraphLayer2D;

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

    Graph2D graph(points);
    PushLayer(CreateShared<GraphLayer2D>(graph));
  }
  ~MathBox() {}
};

Unique<Application> lambda::core::CreateApplication() {
  return memory::CreateUnique<MathBox>();
}
