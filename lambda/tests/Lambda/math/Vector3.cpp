#include <gtest/gtest.h>

#include <Lambda/math/Vector3.h>

TEST(Vector3, DefaultInitializationOfZeros) {
  lambda::math::Vector3 vec;
  ASSERT_EQ(vec.GetX(), 0);
  ASSERT_EQ(vec.GetY(), 0);
  ASSERT_EQ(vec.GetZ(), 0);
}
