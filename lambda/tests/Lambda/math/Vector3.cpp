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

TEST(Vector3, Addition) {
  lambda::math::Vector3 vec = {1.0, 2.0, 3.0};
  lambda::math::Vector3 other_vec = {3.0, 2.0, 1.0};
  auto result = vec + other_vec;

  ASSERT_FLOAT_EQ(result.GetX(), 4.0f);
  ASSERT_FLOAT_EQ(result.GetY(), 4.0f);
  ASSERT_FLOAT_EQ(result.GetZ(), 4.0f);

  result = other_vec + vec;

  ASSERT_FLOAT_EQ(result.GetX(), 4.0f);
  ASSERT_FLOAT_EQ(result.GetY(), 4.0f);
  ASSERT_FLOAT_EQ(result.GetZ(), 4.0f);
}

TEST(Vector3, Subtraction) {
  lambda::math::Vector3 vec = {1.0, 2.0, 3.0};
  lambda::math::Vector3 other_vec = {3.0, 2.0, 1.0};
  auto result = vec - other_vec;

  ASSERT_FLOAT_EQ(result.GetX(), -2.0f);
  ASSERT_FLOAT_EQ(result.GetY(), 0.0f);
  ASSERT_FLOAT_EQ(result.GetZ(), 2.0f);

  result = other_vec - vec;

  ASSERT_FLOAT_EQ(result.GetX(), 2.0f);
  ASSERT_FLOAT_EQ(result.GetY(), 0.0f);
  ASSERT_FLOAT_EQ(result.GetZ(), -2.0f);
}

TEST(Vector3, Multiplication) {
  lambda::math::Vector3 vec = {1.0, 2.0, 3.0};
  lambda::math::Vector3 other_vec = {3.0, 2.0, 1.0};
  auto result = vec * other_vec;

  ASSERT_FLOAT_EQ(result.GetX(), 3.0f);
  ASSERT_FLOAT_EQ(result.GetY(), 4.0f);
  ASSERT_FLOAT_EQ(result.GetZ(), 3.0f);

  result = other_vec * vec;

  ASSERT_FLOAT_EQ(result.GetX(), 3.0f);
  ASSERT_FLOAT_EQ(result.GetY(), 4.0f);
  ASSERT_FLOAT_EQ(result.GetZ(), 3.0f);
}

TEST(Vector3, Division) {
  lambda::math::Vector3 vec = {1.0, 2.0, 3.0};
  lambda::math::Vector3 other_vec = {3.0, 2.0, 1.0};
  auto result = vec / other_vec;

  ASSERT_FLOAT_EQ(result.GetX(), 1.0f / 3.0f);
  ASSERT_FLOAT_EQ(result.GetY(), 1.0f);
  ASSERT_FLOAT_EQ(result.GetZ(), 3.0f);

  result = other_vec / vec;

  ASSERT_FLOAT_EQ(result.GetX(), 3.0f);
  ASSERT_FLOAT_EQ(result.GetY(), 1.0f);
  ASSERT_FLOAT_EQ(result.GetZ(), 1.0f / 3.0f);
}

TEST(Vector3, DotProductOf) {
  lambda::math::Vector3 first_vector = {1, 2, 3};
  lambda::math::Vector3 second_vector = {3, 2, 1};
  auto result = lambda::math::DotProductOf(first_vector, second_vector);
  ASSERT_FLOAT_EQ(result, 10.0f);

  // Assert order doesn't matter
  first_vector = {3, 2, 1};
  second_vector = {1, 2, 3};
  result = lambda::math::DotProductOf(first_vector, second_vector);
  ASSERT_FLOAT_EQ(result, 10.0f);
}
