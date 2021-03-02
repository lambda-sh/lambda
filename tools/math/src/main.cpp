#include <Lambda/Lambda.h>
#include <Lambda/core/Entrypoint.h>
#include <Lambda/math/Vector.h>

using lambda::core::memory::Unique;
using lambda::core::memory::CreateUnique;

int TestMath() {
  lambda::math::Vector<float> v({ 3.0, 3.0 });

  auto values = v.GetRawElements();

  for (auto value : values) {
    LAMBDA_CORE_INFO("Value in Vector: {0}", value);
  }

  return 0;
}

Unique<Application> lambda::core::CreateApplication() {
  TestMath();
  return memory::CreateUnique<Application>();
}
