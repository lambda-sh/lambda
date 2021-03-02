#ifndef LAMBDA_SRC_LAMBDA_MATH_PRECISION_H_
#define LAMBDA_SRC_LAMBDA_MATH_PRECISION_H_

#include <cfloat>

namespace lambda::math {

#if !(defined(LAMBDA_MATH_SINGLE_PRECISION) || \
    defined(LAMBDA_MATH_DOUBLE_PRECISION))
  #define LAMBDA_MATH_SINGLE_PRECISION
#endif

#ifdef LAMBDA_MATH_SINGLE_PRECISION
  typedef float Real;

  #define REAL_MAX FLT_MAX
  #define REAL_EPSILON FLT_EPSILON
  #define REAL_PI 3.14159f

  #define REAL_SQRT sqrtf
  #define REAL_ABS fabsf
  #define REAL_SIN sinf
  #define REAL_COS cosf
  #define REAL_POW powf
  #define REAL_FMOD fmodf

#elif defined(LAMBDA_MATH_DOUBLE_PRECISION)
  typedef double Real;

  #define REAL_MAX DBL_MAX
  #define REAL_EPSILON DBL_EPSILON
  #define REAL_PI 3.14159265358979

  #define REAL_SQRT sqrt
  #define REAL_ABS fabs
  #define REAL_SIN sin
  #define REAL_COS cos
  #define REAL_POW pow
  #define REAL_FMOD fmod

#endif  // defined(LAMBDA_MATH_DOUBLE_PRECISION)

#endif  // LAMBDA_SRC_LAMBDA_MATH_PRECISION_H_

}  // namespace lambda::math
