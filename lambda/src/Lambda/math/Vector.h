#ifndef LAMBDA_SRC_LAMBDA_MATH_VECTOR_H_
#define LAMBDA_SRC_LAMBDA_MATH_VECTOR_H_

#include <array>
#include <vector>

#include <Lambda/lib/Assert.h>
#include <Lambda/math/Precision.h>

namespace lambda::math {

// ----------------------------------- VECTOR ----------------------------------

/// @brief Implementation for Vectors of varying length.
/// @tparam Type is the type of the element being stored within Container.
/// @tparam Container The container to use for storing elements within.
template<class Type = Real, class Container = std::vector<Type>>
class Vector {
 public:
  Vector() noexcept : elements_(Container()) {}

  explicit Vector(Container elements) noexcept
    : elements_(std::move(elements)) {}

  explicit Vector(const Vector& vec) noexcept = default;
  explicit Vector(const Vector&& vec) noexcept
      : elements_(std::move(vec.elements_)) {}

  Vector& operator=(const Vector& vec) = default;
  Vector& operator=(Vector&& vec) = default;

  const Container& GetRawElements() {
    return elements_;
  }

  [[nodiscard]] size_t GetSize() const {
    return elements_.size();
  }

  void ApplyInPlace(std::function<Type(Type)> lambda) {
    std::for_each(elements_.begin(), elements_.end(), lambda);
  }

  Vector Apply(std::function<Type(Type)> lambda) {
    Container new_elements(elements_.size());
    std::transform(elements_.begin(), elements_.end(), &new_elements, lambda);
    return Vector(std::move(new_elements));
  }

  Vector operator+(const Vector& other_vector) {
    LAMBDA_CORE_ASSERT(
        GetSize() == other_vector.GetSize(),
        "Vectors are not the same size",
        "");
    Container new_elements(GetSize());

    std::transform(
        elements_.begin(),
        elements_.end(),
        other_vector.elements_.begin(),
        new_elements.begin(),
        [](Type x, Type y) -> Type {
          return x + y;
        });

    return Vector(new_elements);
  }

  void operator+=(const Vector& other_vector) {
    std::transform(
        elements_.begin(),
        elements_.end(),
        other_vector.elements_.begin(),
        elements_.begin(),
        [](Type x, Type y) {
          return x + y;
        });
  }

 protected:
  Container elements_;
};

// ----------------------------------- VECTOR2 ---------------------------------

class Vector2 : public Vector<Real, std::array<Real, 2>> {
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
  void SetX(const Real x) {
    elements_[0] = x;
  }

  [[nodiscard]] Real GetX() const {
    return elements_[0];
    }

  /// @brief Set the y component of the current vector.
  void SetY(const Real y) {
      elements_[1] = y;
    }

  [[nodiscard]] Real GetY() const {
    return elements_[1];
  }

  [[nodiscard]] Real GetLength() const {
    return REAL_SQRT(REAL_POW(elements_[0], 2) + REAL_POW(elements_[1], 2));
  }

  void operator+=(const Vector2& other_vector) {
    elements_[0] = elements_[0] + other_vector.GetX();
    elements_[1] = elements_[1] + other_vector.GetY();
  }

  void operator-=(const Vector2& other_vector) {
    elements_[0] = elements_[0] - other_vector.GetX();
    elements_[1] = elements_[1] - other_vector.GetY();
  }

  void operator*=(const Vector2& other_vector) {
    elements_[0] = elements_[0] * other_vector.GetX();
    elements_[1] = elements_[1] * other_vector.GetY();
  }

  void operator/=(const Vector2& other_vector) {
    elements_[0] = elements_[0] / other_vector.GetX();
    elements_[1] = elements_[1] / other_vector.GetY();
  }
};

inline Vector2 operator+(
    const Vector2& first_vector, const Vector2& second_vector) {
  return Vector2(
    first_vector.GetX() + second_vector.GetX(),
    first_vector.GetY() + second_vector.GetY());
}

inline Vector2 operator+(
    const Vector2& first_vector, Real scalar) {
  return Vector2(
    first_vector.GetX() + scalar,
    first_vector.GetY() + scalar);
}

inline Vector2 operator-(
    const Vector2& first_vector, const Vector2& second_vector) {
  return Vector2(
    first_vector.GetX() - second_vector.GetX(),
    first_vector.GetY() - second_vector.GetY());
}

inline Vector2 operator-(
    const Vector2& first_vector, Real scalar) {
  return Vector2(
    first_vector.GetX() - scalar,
    first_vector.GetY() - scalar);
}

inline Vector2 operator*(
    const Vector2& first_vector, const Vector2& second_vector) {
  return Vector2(
    first_vector.GetX() * second_vector.GetX(),
    first_vector.GetY() * second_vector.GetY());
}

inline Vector2 operator*(
    const Vector2& first_vector, Real scalar) {
  return Vector2(
    first_vector.GetX() * scalar,
    first_vector.GetY() * scalar);
}

inline Vector2 operator/(
    const Vector2& first_vector, const Vector2& second_vector) {
  return Vector2(
    first_vector.GetX() / second_vector.GetX(),
    first_vector.GetY() / second_vector.GetY());
}

inline Vector2 operator/(
    const Vector2& first_vector, Real scalar) {
  return Vector2(
    first_vector.GetX() / scalar,
    first_vector.GetY() / scalar);
}

/// @brief Get the length of a 2D vector.
/// @param vector Cartesian Coordinate vector.
/// @return The length of the vector. (sqrt(a^2 + b^2))
inline Real LengthOf(const Vector2& vector) {
  return REAL_SQRT(REAL_POW(vector.GetX(), 2) + REAL_POW(vector.GetY(), 2));
}

/// @brief The distance between two 2D vectors.
/// @param first_vector The start vector.
/// @param second_vector The end vector.
/// @return THe distance between each vector.
inline Real DistanceBetween(
    const Vector2& first_vector, const Vector2& second_vector) {
  const Vector2 displacement_vector = first_vector - second_vector;
  return LengthOf(displacement_vector);
}

/// @brief Get the perimeter of a list of vectors.
/// @param vectors The list of 2D vectors to find the coordinate of (Must be
/// in order).
/// @return The perimeter of connecting all vectors.
inline Real PerimeterOf(std::vector<Vector2> vectors) {
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
inline Vector2 PolarToCartesian(const Vector2& polar_vector) {
  const Real length = polar_vector.GetX();
  const Real angle = polar_vector.GetY();
  return Vector2(length * REAL_COS(angle), length * REAL_SIN(angle));
}

/// @brief Converts a cartesian coordinate to a polar coordinate.
/// @param cartesian_vector A vector containing x & y coordinates.
/// @return A polar vector containing it's length & angle in radians.
inline Vector2 CartesianToPolar(const Vector2& cartesian_vector) {
  const Real length = LengthOf(cartesian_vector);
  const Real angle = REAL_ATAN2(
      cartesian_vector.GetY(), cartesian_vector.GetX());
  return Vector2(length, angle);
}

/// @brief Rotates a cartesian coordinate vector given an angle.
/// @param cartesian_vector The vector to rotate.
/// @param angle The angle to rotate the vector by (In radians)
/// @return A cartesian vector that has been rotated.
inline Vector2 RotateCartesian(const Vector2& cartesian_vector, Real angle) {
  Vector2 polar = CartesianToPolar(cartesian_vector);
  polar.SetY(polar.GetY() + angle);
  return PolarToCartesian(polar);
}

/// @brief Rotates a polar coordinate vector given an angle.
/// @param polar_vector The vector to rotate.
/// @param angle The angle to rotate the vector by (In radians)
/// @return A polar vector that has been rotated.
inline Vector2 RotatePolar(const Vector2& polar_vector, Real angle) {
  return Vector2(polar_vector.GetX(), polar_vector.GetY() + angle);
}

// ----------------------------------- VECTOR3 ---------------------------------

/// @brief Implementation for Vectors of length 3.
class Vector3 : public Vector<Real, std::array<Real, 3>> {
 public:
  Vector3() : Vector({0, 0, 0}) {}
  explicit Vector3(const std::array<Real, 3> elements) : Vector(elements) {}
  Vector3(const Real x, const Real y) noexcept : Vector({x, y}) {}

  /// @brief Set the x component of the current vector.
  void SetX(const Real x) {
    elements_[0] = x;
  }
  [[nodiscard]] Real GetX() const {
    return elements_[0];
  }

  /// @brief Set the y component of the current vector.
  void SetY(const Real y) {
    elements_[1] = y;
  }

  /// @brief Get the y component of the current vector.
  [[nodiscard]] Real GetY() const {
    return elements_[1];
  }

  /// @brief Set the z component of the current vector.
  void SetZ(const Real z) {
    elements_[2] = z;
  }
  [[nodiscard]] Real GetZ() const {
    return elements_[2];
  }
};


}  // namespace lambda::math

#endif  // LAMBDA_SRC_LAMBDA_MATH_VECTOR_H_
