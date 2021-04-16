#ifndef LAMBDA_SRC_LAMBDA_MATH_VECTOR_H_
#define LAMBDA_SRC_LAMBDA_MATH_VECTOR_H_

#include <array>
#include <vector>

#include <Lambda/math/Precision.h>

namespace lambda::math {

/// @brief Implementation for Vectors of varying length.
/// @tparam Type is the type of the element being stored within Container.
/// @tparam Container The container to use for storing elements within.
template<class Type = Real, class Container = std::vector<Real>>
class Vector {
 public:
  Vector() : elements_(Container()) {}
  explicit Vector(Container elements)
    : elements_(std::move(elements)) {}

  const Container& GetRawElements() {
    return elements_;
  }

  [[nodiscard]] size_t GetSize() const {
    return elements_.size();
  }

  void ApplyInPlace(std::function<Type(Type)> lambda) {
    std::for_each(elements_.begin(), elements_.end(), lambda);
  }

  Container Apply(std::function<Type(Type)> lambda) {
    Container new_elements(elements_.size());
    std::transform(elements_.begin(), elements_.end(), &new_elements, lambda);
    return new_elements;
  }

 protected:
  Container elements_;
};

class Vector2 : public Vector<Real, std::array<Real, 2>> {
 public:
  Vector2() : Vector({0, 0}) {}
  Vector2(const Real x, const Real y) : Vector({x, y}) {}
  explicit Vector2(const std::array<Real, 2> elements) : Vector(elements) {}

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
};

/// @brief Implementation for Vectors of length 3.
class Vector3 : public Vector<Real, std::array<Real, 3>> {
 public:
  Vector3() : Vector({0, 0, 0}) {}
  explicit Vector3(const std::array<Real, 3> elements) : Vector(elements) {}
  Vector3(const Real x, const Real y) : Vector({x, y}) {}


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
