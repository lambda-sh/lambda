#include <Lambda/Lambda.h>
#include <Lambda/core/Entrypoint.h>
#include <Lambda/math/Vector.h>

using lambda::core::memory::Unique;
using lambda::core::memory::CreateUnique;

int TestMath() {
  auto v = lambda::math::Vector3();

  for (auto value : v.GetRawElements()) {
    LAMBDA_CORE_INFO("Value in Vector: {0}", value);
  }

  return 0;
}

Unique<Application> lambda::core::CreateApplication() {
  TestMath();
  return memory::CreateUnique<Application>();
}
