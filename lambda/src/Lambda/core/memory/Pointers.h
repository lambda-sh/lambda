/// @file Pointers.h
/// @brief Abstraction for handling pointers within the engine. Currently just
/// aliases for unique and shared pointers provided by c++11.
#ifndef LAMBDA_SRC_LAMBDA_CORE_MEMORY_POINTERS_H_
#define LAMBDA_SRC_LAMBDA_CORE_MEMORY_POINTERS_H_

#include <memory>

namespace lambda {
namespace core {
namespace memory {

/// @brief
template<typename T>
using Unique = std::unique_ptr<T>;

template<typename T>
using Shared = std::shared_ptr<T>;

template<typename T, typename... Args>
constexpr Unique<T> CreateUnique(Args&&... args) {
  return std::make_unique<T>(std::forward<Args>(args)...);
}

template<typename T, typename... Args>
constexpr Shared<T> CreateShared(Args&&... args) {
  return std::make_shared<T>(std::forward<Args>(args)...);
}

}  // namespace memory
}  // namespace core
}  // namespace lambda

#endif  // LAMBDA_SRC_LAMBDA_CORE_MEMORY_POINTERS_H_
