#include <Lambda/Lambda.h>
#include <Lambda/core/Entrypoint.h>
#include <Lambda/math/Precision.h>
#include <Lambda/math/Vector.h>
#include <Lambda/math/shapes/Point.h>
#include <Lambda/math/plot/Graph.h>

using lambda::core::memory::Unique;
using lambda::core::memory::CreateUnique;

using lambda::math::shapes::Point2D;
using lambda::math::plot::Graph2D;

int TestMath() {
  auto v = lambda::math::Vector3();

  for (auto value : v.GetRawElements()) {
    LAMBDA_CORE_INFO("Value in Vector: {0}", value);
  }

  std::vector<Point2D<>> points(2000);
  std::fill(points.begin(), points.end(), Point2D<>());

  int counter = 0;
  for (auto& point : points) {
    lambda::math::Real x = (counter - 1000) / 100;
    point.x = x;
    point.y = sin(x * 10) / (1.0 + x * x);
    counter += 1;
  }

  Graph2D graph(std::move(points));
  graph = graph
    .StartFrom(0)
    .EndAt(100);

  return 0;
}

Unique<Application> lambda::core::CreateApplication() {
  TestMath();
  return memory::CreateUnique<Application>();
}
