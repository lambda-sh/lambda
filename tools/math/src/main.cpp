#include <Lambda/Lambda.h>
#include <Lambda/core/Entrypoint.h>
#include <Lambda/math/Precision.h>
#include <Lambda/math/Vector.h>
#include <Lambda/math/shapes/Point.h>

using lambda::core::memory::Unique;
using lambda::core::memory::CreateUnique;

int TestMath() {
  auto v = lambda::math::Vector3();

  for (auto value : v.GetRawElements()) {
    LAMBDA_CORE_INFO("Value in Vector: {0}", value);
  }

  lambda::math::shapes::Point2D<lambda::math::Real> points[2000];

  for (int i = 0; i < 2000; i += 1) {
    lambda::math::Real x = (i - 1000) / 100;
    points[i].x = x;
    points[i].y = sin(x * 10) / (1.0 + x * x);
  }

  return 0;
}

Unique<Application> lambda::core::CreateApplication() {
  TestMath();
  return memory::CreateUnique<Application>();
}
