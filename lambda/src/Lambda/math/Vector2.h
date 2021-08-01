#ifndef LAMBDA_SRC_LAMBDA_MATH_VECTOR2_H_
#define LAMBDA_SRC_LAMBDA_MATH_VECTOR2_H_

#include <Lambda/math/Vector.h>

namespace lambda::math {

// ----------------------------------- VECTOR2 ---------------------------------
class Vector2;
Real LengthOf(const Vector2& vector) noexcept;

class Vector2 final : public Vector<Real, std::array<Real, 2>> {
 public:
  Vector2() noexcept : Vector({0, 0}) {}
  Vector2(const Real x, const Real y) noexcept : Vector({x, y}) {}
  ~Vector2() = default;

  explicit Vector2(const std::array<Real, 2> elements) noexcept
      : Vector(elements) {}
  explicit Vector2(const Vector2& vec) noexcept = default;
  explicit Vector2(Vector2&& vec) noexcept = default;

  Vector2& operator=(const Vector2& vec) noexcept = default;
  Vector2& operator=(Vector2&& vec) noexcept = default;

  /// @brief Set the x component of the current vector.
  void SetX(const Real x) noexcept {
    elements_[0] = x;
  }

  [[nodiscard]] Real GetX() const noexcept {
    return elements_[0];
  }

  /// @brief Set the y component of the current vector.
  void SetY(const Real y) noexcept {
      elements_[1] = y;
    }

  [[nodiscard]] Real GetY() const noexcept {
    return elements_[1];
  }

  [[nodiscard]] Real GetLength() const noexcept {
    return LengthOf(*this);
  }

  void operator+=(const Vector2& other_vector) noexcept {
    elements_[0] += other_vector.elements_[0];
    elements_[1] += other_vector.elements_[1];
  }

  Vector2 operator+(const Vector2& other_vector) const noexcept {
    return Vector2(
        elements_[0] + other_vector.elements_[0],
        elements_[1] + other_vector.elements_[1]);
  }

  void operator-=(const Vector2& other_vector) noexcept {
    elements_[0] -= other_vector.elements_[0];
    elements_[1] -= other_vector.elements_[1];
  }

  Vector2 operator-(const Vector2& other_vector) const noexcept {
    return Vector2(
        elements_[0] - other_vector.elements_[0],
        elements_[1] - other_vector.elements_[1]);
  }

  void operator*=(const Vector2& other_vector) noexcept {
    elements_[0] *= other_vector.elements_[0];
    elements_[1] *= other_vector.elements_[1];
  }

  Vector2 operator*(const Vector2& other_vector) const noexcept {
    return Vector2(
        elements_[0] * other_vector.elements_[0],
        elements_[1] * other_vector.elements_[1]);
  }

  void operator/=(const Vector2& other_vector) noexcept {
    elements_[0] /= other_vector.elements_[0];
    elements_[1] /= other_vector.elements_[1];
  }

  Vector2 operator/(const Vector2& other_vector) const noexcept {
    return Vector2(
        elements_[0] / other_vector.elements_[0],
        elements_[1] / other_vector.elements_[1]);
  }

  friend Vector2 operator+(
      const Vector2& first_vector, Real scalar) noexcept {
    return Vector2(
      first_vector.GetX() + scalar,
      first_vector.GetY() + scalar);
  }

  friend Vector2 operator-(
      const Vector2& first_vector, Real scalar) noexcept {
    return Vector2(
      first_vector.GetX() - scalar,
      first_vector.GetY() - scalar);
  }

  friend Vector2 operator*(
      const Vector2& first_vector, Real scalar) noexcept {
    return Vector2(
      first_vector.GetX() * scalar,
      first_vector.GetY() * scalar);
  }

  friend Vector2 operator/(
      const Vector2& first_vector, Real scalar) noexcept {
    return Vector2(
      first_vector.GetX() / scalar,
      first_vector.GetY() / scalar);
  }
};

// ------------------------------- SCALAR OPERATORS ----------------------------

template<typename Vector>
Vector2 Vector2FromVector(Vector vec) noexcept {
  std::array<Real, 2> arr = vec.GetElements();
  return Vector2(arr);
}

/// @brief Get the length of a 2D vector.
/// @param vector Cartesian Coordinate vector.
/// @return The length of the vector. (sqrt(x^2 + y^2))
inline Real LengthOf(const Vector2& vector) noexcept {
  return SquareRootOf(PowerOf(vector.GetX(), 2) + PowerOf(vector.GetY(), 2));
}

/// @brief The distance between two 2D vectors.
/// @param first_vector The start vector.
/// @param second_vector The end vector.
/// @return THe distance between each vector.
inline Real DistanceBetween(
    const Vector2& first_vector, const Vector2& second_vector) noexcept {
  const Vector2 displacement_vector = first_vector - second_vector;
  return LengthOf(displacement_vector);
}

/// @brief Get the perimeter of a list of vectors.
/// @param vectors A list of 2D vectors. Ordering may affect result.
/// @return The perimeter of connecting all vectors.
inline Real PerimeterOf(const std::vector<Vector2>& vectors) {
  Real perimeter = 0.0f;

  for (size_t counter = 0; counter <= vectors.size() - 1; counter++) {
    perimeter += DistanceBetween(
        vectors[counter],
        vectors[(counter + 1) % vectors.size()]);
  }

  return perimeter;
}

/// @brief Converts a polar coordinate to a cartesian coordinate.
/// @param polar_vector The polar coordinate that contains length & angle in
/// radians as the vector components.
/// @return A Cartesian coordinate vector that contains the x & y components.
inline Vector2 PolarToCartesian(const Vector2& polar_vector) noexcept {
  const Real length = polar_vector.GetX();
  const Real angle = polar_vector.GetY();
  return Vector2(length * CosineOf(angle), length * SineOf(angle));
}

/// @brief Converts a cartesian coordinate to a polar coordinate.
/// @param cartesian_vector A vector containing x & y coordinates.
/// @return A polar vector containing it's length & angle in radians.
inline Vector2 CartesianToPolar(const Vector2& cartesian_vector) noexcept {
  const Real length = LengthOf(cartesian_vector);
  const Real angle = Atan2Of(
      cartesian_vector.GetY(), cartesian_vector.GetX());
  return Vector2(length, angle);
}

/// @brief Rotates a cartesian coordinate vector given an angle.
/// @param cartesian_vector The vector to rotate.
/// @param angle The angle to rotate the vector by (In radians)
/// @return A cartesian vector that has been rotated.
inline Vector2 RotateCartesianByAngle(
    const Vector2& cartesian_vector, Real angle) noexcept {
  Vector2 polar = CartesianToPolar(cartesian_vector);
  polar.SetY(polar.GetY() + angle);
  return PolarToCartesian(polar);
}

/// @brief Rotates a polar coordinate vector given an angle.
/// @param polar_vector The vector to rotate.
/// @param angle The angle to rotate the vector by (In radians)
/// @return A polar vector that has been rotated.
inline Vector2 RotatePolarByAngle(
    const Vector2& polar_vector, Real angle) noexcept {
  return Vector2(polar_vector.GetX(), polar_vector.GetY() + angle);
}

/// @brief Sum of multiple vectors.
/// @param vectors The list of vectors to sum.
/// @return a vector pointing to the sum of all vectors.
inline Vector2 SumOf(const std::vector<Vector2>& vectors) noexcept {
  Real x = 0, y = 0;

  for (const auto& vector : vectors) {
    x += vector.GetX();
    y += vector.GetY();
  }

  return Vector2(x, y);
}

}  // namespace lambda::math

#endif  // LAMBDA_SRC_LAMBDA_MATH_VECTOR2_H_
