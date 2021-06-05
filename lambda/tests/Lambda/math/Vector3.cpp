#include <array>

#include <gtest/gtest.h>

#include <Lambda/math/Precision.h>
#include <Lambda/math/Vector3.h>

TEST(Vector3, DefaultInitialization) {
  lambda::math::Vector3 vec;
  ASSERT_EQ(vec.GetX(), 0);
  ASSERT_EQ(vec.GetY(), 0);
  ASSERT_EQ(vec.GetZ(), 0);
}

TEST(Vector3, InitializationFromArray) {
  lambda::math::Vector3 vec(std::array<lambda::math::Real, 3>({0, 0, 0}));
  ASSERT_EQ(vec.GetX(), 0);
  ASSERT_EQ(vec.GetY(), 0);
  ASSERT_EQ(vec.GetZ(), 0);
}

TEST(Vector3, InitializationFromFloats) {
  lambda::math::Vector3 vec(1.0f, 2.0f, 3.0f);
  ASSERT_EQ(vec.GetX(), 1.0f);
  ASSERT_EQ(vec.GetY(), 2.0f);
  ASSERT_EQ(vec.GetZ(), 3.0f);
}
