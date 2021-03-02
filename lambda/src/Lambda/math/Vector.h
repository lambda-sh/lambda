#ifndef LAMBDA_SRC_LAMBDA_MATH_VECTOR_H_
#define LAMBDA_SRC_LAMBDA_MATH_VECTOR_H_

#include <vector>

#include <Lambda/core/memory/Pointers.h>
#include <Lambda/math/Precision.h>

namespace lambda::math {

template<class T>
class Vector {
 public:
  Vector(const size_t size, std::vector<T> elements)
    : size_(size), elements_(elements) {}
  explicit Vector(std::vector<T> elements)
    : size_(elements.size()), elements_(elements) {}

  const std::vector<T>& GetRawElements() { return elements_; }

  [[nodiscard]] size_t GetSize() const { return size_; }

 private:
  size_t size_;
  std::vector<T> elements_;
};

template<class T>
class Vector3 {
 public:
  Vector3() : x_position_(0), y_position_(0), z_position_(0) {}
 private:
  Real x_position_, y_position_, z_position_;
};

}  // namespace lambda::math

#endif  // LAMBDA_SRC_LAMBDA_MATH_VECTOR_H_
