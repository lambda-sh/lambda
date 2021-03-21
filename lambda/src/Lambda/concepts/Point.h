/// @file Point.h
/// @brief Concepts related to Points
#ifndef LAMBDA_SRC_LAMBDA_CONCEPTS_POINT_H_
#define LAMBDA_SRC_LAMBDA_CONCEPTS_POINT_H_

#include <array>
#include <concepts>
#include <vector>

#include <Lambda/concepts/Number.h>
#include <Lambda/math/shapes/Point.h>

namespace lambda::concepts {

// ---------------------------------- 2D POINTS -------------------------------

template<class Anything>
struct IsPoint2D : public std::false_type {};

template<NumberType Precision>
struct IsPoint2D<lambda::math::shapes::Point2D<Precision>>
  : public std::true_type {};

// ---------------------------------- 3D POINTS --------------------------------

template<class Anything>
struct IsPoint3D : public std::false_type {};

template<NumberType Precision>
struct IsPoint3D<lambda::math::shapes::Point3D<Precision>>
  : public std::true_type {};

// ------------------------------- POINT TYPE ---------------------------------

template<class MaybePoint>
concept PointType = IsPoint2D<MaybePoint>::value
  || IsPoint3D<MaybePoint>::value;

// ------------------------------ POINT ARRAYS --------------------------------

template<class Anything>
struct IsPointArray : public std::false_type {};

template<PointType Point, int Size>
struct IsPointArray<std::array<Point, Size>> : public std::true_type {};

template<class MaybeArray>
concept PointArray = IsPointArray<MaybeArray>::value;

// ------------------------------ POINT VECTORS -------------------------------

template<class Anything>
struct IsPointVector : public std::false_type {};

template<PointType Point, class Allocator>
struct IsPointVector<std::vector<Point, Allocator>> : public std::true_type {};

template<class MaybeVector>
concept PointVector = IsPointVector<MaybeVector>::value;

template<class MaybeContainer>
concept PointContainer = PointArray<MaybeContainer>
  || PointVector<MaybeContainer>;

}  // namespace lambda::concepts

#endif  // LAMBDA_SRC_LAMBDA_CONCEPTS_POINT_H_
