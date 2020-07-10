/**
 * @file Assert.h
 * @brief This is a utility file that is primarily used for debugging the
 * internals of the engine when incorrect behavior or invalid use of the API is
 * occurring.
 *
 * This should most likely not be deployed into any game extending
 * this engine. ENGINE_ENABLE_ASSERTS enables and disables assertions for both
 * the client and the engine at compile time.
 */
#ifndef ENGINE_SRC_CORE_UTIL_ASSERT_H_
#define ENGINE_SRC_CORE_UTIL_ASSERT_H_

#include "core/Core.h"
#include "core/util/Log.h"

#if ENGINE_ENABLE_ASSERTS
  #define ENGINE_CLIENT_ASSERT(x, ...) { \
      if (!(x)) { \
          ENGINE_CLIENT_ERROR("Assertion Failed: {0},", __VA_ARGS__); \
          ENGINE_DEBUG_BREAK(); }}

  #define ENGINE_CORE_ASSERT(x, ...) { \
      if (!(x)) { \
          ENGINE_CORE_ERROR("Assertion Failed: {0},", __VA_ARGS__); \
          ENGINE_DEBUG_BREAK(); }}
#else
  #define ENGINE_CLIENT_ASSERT(x, ...)
  #define ENGINE_CORE_ASSERT(x, ...)
#endif  // ENGINE_ENABLE_ASSERTS

#endif  // ENGINE_SRC_CORE_UTIL_ASSERT_H_

/**
 * @def ENGINE_CLIENT_ASSERT(x, ...)
 * @brief When assertions are enabled, the client is allowed to use asserts in
 * their code to halt their application whenever the condition being asserted is
 * false.
 */

/**
 * @def ENGINE_CORE_ASSERT(x, ...)
 * @brief When assertions are enabled, the engine is allowed to use asserts in
 * its core to halt the application whenever the condition being asserted is
 * false.
 */
