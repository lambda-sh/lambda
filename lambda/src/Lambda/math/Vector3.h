#ifndef LAMBDA_SRC_LAMBDA_MATH_VECTOR3_H_
#define LAMBDA_SRC_LAMBDA_MATH_VECTOR3_H_

#include <Lambda/math/Vector.h>

namespace lambda::math {

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

#endif  //  LAMBDA_SRC_LAMBDA_MATH_VECTOR3_H_
