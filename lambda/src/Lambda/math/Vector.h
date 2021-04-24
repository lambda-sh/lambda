#ifndef LAMBDA_SRC_LAMBDA_MATH_VECTOR_H_
#define LAMBDA_SRC_LAMBDA_MATH_VECTOR_H_

#include <array>
#include <vector>

#include <Lambda/lib/Assert.h>
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

  explicit Vector(const Vector& vec) = default;
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

class Vector2 : public Vector<Real, std::array<Real, 2>> {
 public:
  Vector2() noexcept : Vector({0, 0}) {}
  Vector2(const Real x, const Real y) noexcept : Vector({x, y}) {}

  ~Vector2() = default;

  explicit Vector2(const std::array<Real, 2> elements) : Vector(elements) {}

  explicit Vector2(const Vector2& vec) = default;

  explicit Vector2(Vector2&& vec) = default;

  Vector2& operator=(const Vector2& vec) = default;
  Vector2& operator=(Vector2&& vec) = default;

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


template<typename Vector>
static Vector Add(Vector first, Vector second) {
  return first + second;
}

}  // namespace lambda::math

#endif  // LAMBDA_SRC_LAMBDA_MATH_VECTOR_H_
