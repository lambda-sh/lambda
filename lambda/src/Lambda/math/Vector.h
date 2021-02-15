#ifndef LAMBDA_SRC_LAMBDA_MATH_VECTOR_H_
#define LAMBDA_SRC_LAMBDA_MATH_VECTOR_H_

#include <stdint.h>
#include <vector>

#include <Lambda/core/memory/Pointers.h>

namespace lambda {
namespace math {

template<class T>
class Vector {
 public:
  Vector(size_t size, std::vector<T> elements)
    : size_(size), elements_(elements) {}
  explicit Vector(std::vector<T> elements) : elements_(elements) {}

  const std::vector<T>& const GetRawElements() { return elements_; }

 private:
  size_t size_;
  std::vector<T> elements_;
};

}  // namespace math
}  // namespace lambda

#endif  // LAMBDA_SRC_LAMBDA_MATH_VECTOR_H_
