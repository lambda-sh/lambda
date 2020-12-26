/// @file Reverse.h
/// @brief A lightweight utility for Reversing STL based containers.
#ifndef LAMBDA_SRC_LAMBDA_CORE_UTIL_REVERSE_H_
#define LAMBDA_SRC_LAMBDA_CORE_UTIL_REVERSE_H_

#ifndef LAMBDA_PLATFORM_MACOS
  #include <ranges>
#endif  // LAMBDA_PLATFORM_MACOS

namespace lambda {
namespace core {
namespace util {

#ifndef LAMBDA_PLATFORM_MACOS
template<class Container>
concept Iterable = std::ranges::bidirectional_range<Container>;
#else
  #define Iterable typename;
#endif  // LAMBDA_PLATFORM_MACOS

/// @brief A clean container for iterating through any container that implements
/// rbegin and rend.
template<Iterable Container>
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

#endif  // LAMBDA_SRC_LAMBDA_CORE_UTIL_REVERSE_H_
