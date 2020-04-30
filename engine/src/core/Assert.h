#ifndef ENGINE_SRC_CORE_ASSERT_H_
#define ENGINE_SRC_CORE_ASSERT_H_

#include "core/Core.h"
#include "core/Log.h"

#ifdef ENGINE_ENABLE_ASSERTS
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
#endif

#endif  // ENGINE_SRC_CORE_ASSERT_H_
