#ifndef LAMBDA_SRC_LAMBDA_MATH_PRECISION_H_
#define LAMBDA_SRC_LAMBDA_MATH_PRECISION_H_

#include <cfloat>
#include <cmath>

namespace lambda::math {

#if !(defined(LAMBDA_MATH_SINGLE_PRECISION) || \
    defined(LAMBDA_MATH_DOUBLE_PRECISION))
  #define LAMBDA_MATH_SINGLE_PRECISION
#endif  // LAMBDA_MATH_SINGLE_PRECISION || LAMBDA_MATH_DOUBLE_PRECISION

#ifdef LAMBDA_MATH_SINGLE_PRECISION

typedef float Real;

constexpr Real REAL_PI = 3.14159f;
constexpr Real REAL_MAX = FLT_MAX;
constexpr Real REAL_EPSILON = FLT_EPSILON;

inline Real SquareRootOf(Real number) noexcept {
    return sqrtf(number);
}

/// @brief Returns the absolute value of a number.
inline Real AbsoluteValueOf(Real number) noexcept {
  return fabsf(number);
}

inline Real SineOf(const Real radians) noexcept {
  return sinf(radians);
}

inline Real CosineOf(const Real radians) noexcept {
  return cos(radians);
}

inline Real Atan2Of(const Real y, const Real x) noexcept {
  return atan2f(y, x);
}

inline Real PowerOf(const Real number, const Real power) noexcept {
  return powf(number, power);
}

inline Real ModulusOf(const Real x, const Real y) noexcept {
  return fmodf(x, y);
}

#elif defined(LAMBDA_MATH_DOUBLE_PRECISION)
  typedef double Real;

const Real REAL_PI = 3.14159265358979;
const Real REAL_MAX = DBL_MAX;
const Real REAL_EPSILON = DBL_EPSILON;

constexpr auto REAL_SQRT = sqrt;
constexpr auto REAL_ABS = fabs;
constexpr auto REAL_SIN = sin;
constexpr auto REAL_COS = cos;
constexpr auto REAL_ATAN2 = atan2;
constexpr auto REAL_POW = pow;
constexpr auto REAL_FMOD = fmod;

#endif  // defined(LAMBDA_MATH_DOUBLE_PRECISION)

constexpr Real DegreeToRadians(const Real degrees) noexcept {
  return degrees * (REAL_PI / 180.0f);
}

constexpr Real RadiansToDegree(const Real radians) noexcept {
  return radians * (180.0f / REAL_PI);
}

}  // namespace lambda::math

#endif  // LAMBDA_SRC_LAMBDA_MATH_PRECISION_H_
