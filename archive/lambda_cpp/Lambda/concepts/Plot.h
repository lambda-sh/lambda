/// @file Number.h
/// @brief Concepts for defining Numbers and Number Containers.
#ifndef LAMBDA_CONCEPTS_PLOT_H_
#define LAMBDA_CONCEPTS_PLOT_H_

#include <array>
#include <concepts>
#include <vector>

#include <Lambda/concepts/Number.h>
#include <Lambda/concepts/Point.h>
#include <Lambda/math/plot/Graph.h>

namespace lambda::concepts {

template<class Anything>
struct IsGraph : public std::false_type {};

template<
    concepts::NumberType Precision,
    PointType Point,
    PointContainer Points>
struct IsGraph<math::plot::Graph2D<Precision, Point, Points>>
    : public std::true_type {};

template <class MaybeGraph>
concept Graph = IsGraph<MaybeGraph>::value;

}  // namespace lambda::concepts

#endif  // LAMBDA_CONCEPTS_PLOT_H_
