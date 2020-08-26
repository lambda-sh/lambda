/// @file Assert.h
/// @brief This is a utility file that is primarily used for debugging the
/// internals of the engine when incorrect behavior or invalid use of the API is
/// occurring.
///
/// This should most likely not be deployed into any game extending
/// this engine. LAMBDA_ENABLE_ASSERTS enables and disables assertions for both
/// the client and the engine at compile time.
#ifndef LAMBDA_SRC_LAMBDA_CORE_UTIL_ASSERT_H_
#define LAMBDA_SRC_LAMBDA_CORE_UTIL_ASSERT_H_

#include "Lambda/core/Core.h"
#include "Lambda/core/util/Log.h"

/// @def LAMBDA_CLIENT_ASSERT(x, ...)
/// @brief When assertions are enabled, the client is allowed to use asserts
/// in their code to halt their application whenever the condition being
/// asserted is false.

/// @def LAMBDA_CORE_ASSERT(x, ...)
/// @brief When assertions are enabled, the engine is allowed to use asserts
/// in its core to halt the application whenever the condition being asserted
/// is false.

#if LAMBDA_ENABLE_ASSERTS
  #define LAMBDA_CLIENT_ASSERT(x, ...) { \
      if (!(x)) { \
          LAMBDA_CLIENT_ERROR("Assertion Failed: {0},", __VA_ARGS__); \
          LAMBDA_DEBUG_BREAK(); }}

  #define LAMBDA_CORE_ASSERT(x, ...) { \
      if (!(x)) { \
          LAMBDA_CORE_ERROR("Assertion Failed: {0},", __VA_ARGS__); \
          LAMBDA_DEBUG_BREAK(); }}
#else
  #define LAMBDA_CLIENT_ASSERT(x, ...)
  #define LAMBDA_CORE_ASSERT(x, ...)
#endif  // LAMBDA_ENABLE_ASSERTS

#endif  // LAMBDA_SRC_LAMBDA_CORE_UTIL_ASSERT_H_
