#ifndef LAMBDA_SRC_LAMBDA_MATH_SHAPES_POINT_H_
#define LAMBDA_SRC_LAMBDA_MATH_SHAPES_POINT_H_

#include <Lambda/concepts/Number.h>
#include <Lambda/math/Precision.h>

namespace lambda::math::shapes {

/// @brief A container for a set of 2D points
/// @tparam Precision The precision of the grid the points will use.
template<concepts::NumberType Precision = lambda::math::Real>
struct Point2D {
  Precision x;
  Precision y;
};

/// @brief A container for a set of 3D points
/// @tparam Precision The precision of the grid the points will use.
template<concepts::NumberType Precision = lambda::math::Real>
struct Point3D {
  Precision x;
  Precision y;
  Precision z;
};

}  // namespace lambda::math::shapes

#endif  // LAMBDA_SRC_LAMBDA_MATH_SHAPES_POINT_H_
