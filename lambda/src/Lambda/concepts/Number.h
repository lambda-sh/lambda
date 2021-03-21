/// @file Number.h
/// @brief Concepts for defining Numbers and Number Containers.
#ifndef LAMBDA_SRC_LAMBDA_CONCEPTS_NUMBER_H_
#define LAMBDA_SRC_LAMBDA_CONCEPTS_NUMBER_H_

#include <array>
#include <concepts>
#include <vector>

namespace lambda::concepts {

template<class Type>
concept NumberType = std::floating_point<Type> || std::integral<Type>;

// ------------------------------- NUMBER ARRAYS -------------------------------

template<class Anything>
struct IsNumberArray : public std::false_type {};

template<NumberType Precision, int Size>
struct IsNumberArray<std::array<Precision, Size>> : public std::true_type {};

template<class MaybeArray>
concept NumberArray = IsNumberArray<MaybeArray>::value;

// ------------------------------- NUMBER VECTORS ------------------------------

template<class Anything>
struct IsNumberVector : public std::false_type {};

template<NumberType Precision, class Allocator>
struct IsNumberVector <std::vector<Precision, Allocator>>
  : public std::true_type {};

template<class MaybeVector>
concept NumberVector = IsNumberVector<MaybeVector>::value;

// ------------------------------ NUMBER CONTAINER -----------------------------

template<class MaybeContainer>
concept NumberContainer = NumberArray<MaybeContainer>
  || NumberVector<MaybeContainer>;

}  // namespace lambda::concepts

#endif  // LAMBDA_SRC_LAMBDA_CONCEPTS_NUMBER_H_
