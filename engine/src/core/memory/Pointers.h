#ifndef ENGINE_SRC_CORE_MEMORY_POINTERS_H_
#define ENGINE_SRC_CORE_MEMORY_POINTERS_H_

#include <memory>

namespace engine {
namespace core {
namespace memory {

template<typename T>
using Unique = std::unique_ptr<T>;

template<typename T>
using Shared = std::shared_ptr<T>;

template<typename T, class... Args>
Unique<T> CreateUnique(Args&&... args) {
  return std::make_unique<T>(args...);
}

template<typename T, class... Args>
Shared<T> CreateShared(Args&&... args) {
  return std::make_shared<T>(args...);
}

}  // namespace memory
}  // namespace core
}  // namespace engine

#endif  // ENGINE_SRC_CORE_MEMORY_POINTERS_H_

/**
 * @file Pointers.h
 * @brief Abstraction for handling pointers within the engine. Currently just
 * aliases for unique and shared pointers provided by c++11.
 */
