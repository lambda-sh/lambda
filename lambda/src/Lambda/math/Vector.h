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

  Vector& operator=(const Vector& vec) noexcept = default;
  Vector& operator=(Vector&& vec) noexcept = default;

  const Container& GetRawElements() noexcept {
    return elements_;
  }

  [[nodiscard]] size_t GetSize() const noexcept {
    return elements_.size();
  }

  void ApplyInPlace(std::function<Type(Type)> lambda) noexcept {
    std::for_each(elements_.begin(), elements_.end(), lambda);
  }

  Vector Apply(std::function<Type(Type)> lambda) noexcept {
    Container new_elements(elements_.size());
    std::transform(elements_.begin(), elements_.end(), &new_elements, lambda);
    return Vector(std::move(new_elements));
  }

  Vector operator+(const Vector& other_vector) noexcept {
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

  void operator+=(const Vector& other_vector) noexcept {
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

}  // namespace lambda::math

#endif  // LAMBDA_SRC_LAMBDA_MATH_VECTOR_H_
