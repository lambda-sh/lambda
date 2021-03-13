#ifndef LAMBDA_SRC_LAMBDA_MATH_VECTOR_H_
#define LAMBDA_SRC_LAMBDA_MATH_VECTOR_H_

#include <array>
#include <vector>

#include <Lambda/core/memory/Pointers.h>
#include <Lambda/math/Precision.h>

namespace lambda::math {

/// @brief Implementation for Vectors of varying length.
/// @tparam Type is the type of the element being stored within Container.
/// @tparam Container The container to use for storing elements within.
template<class Type = Real, class Container = std::vector<Real>>
class Vector {
 public:
  Vector(const size_t size, Container elements)
    : size_(size), elements_(std::move(elements)) {}

  explicit Vector(Container elements)
    : size_(elements.size()), elements_(std::move(elements)) {}

  const Container& GetRawElements() { return elements_; }

  [[nodiscard]] size_t GetSize() const { return size_; }

 protected:
  size_t size_;
  Container elements_;
};

/// @brief Implementation for Vectors of length 3.
class Vector3 : public Vector<Real, std::array<Real, 3>> {
 public:
  Vector3() : Vector({0, 0, 0}) {}

  inline void SetX(const Real x) { elements_[0] = x; }
  [[nodiscard]] inline Real GetX() const { return elements_[0]; }

  inline void SetY(const Real y) { elements_[1] = y; }
  [[nodiscard]] inline Real GetY() const { return elements_[1]; }

  inline void SetZ(const Real z) { elements_[2] = z; }
  [[nodiscard]] inline Real GetZ() const { return elements_[2]; }
};

}  // namespace lambda::math

#endif  // LAMBDA_SRC_LAMBDA_MATH_VECTOR_H_
