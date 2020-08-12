/**
 * @file Reverse.h
 * @brief A lightweight utility for Reversing STL based containers.
 */
#ifndef ENGINE_SRC_CORE_UTIL_REVERSE_H_
#define ENGINE_SRC_CORE_UTIL_REVERSE_H_

#include <ranges>

namespace lambda {
namespace core {
namespace util {
namespace internal {

template<class Container>
concept Iterable = std::ranges::bidirectional_range<Container>;

}  // namespace internal

template<internal::Iterable Container>
class Reverse {
 public:
  explicit Reverse(Container& container) : container_(container) {}
  auto begin() { return container_.rbegin(); }
  auto end() { return container_.rend(); }

 private:
  Container& container_;
};

}  // namespace util
}  // namespace core
}  // namespace lambda

#endif  // ENGINE_SRC_CORE_UTIL_REVERSE_H_

/**
 * @class lambda::util::Reverse
 * @brief Provides a clean interface for iterating through any container that
 * implements rbegin and rend.
 */
