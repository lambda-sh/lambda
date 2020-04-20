#ifndef ENGINE_SRC_CORE_CORE_H_
#define ENGINE_SRC_CORE_CORE_H_

#undef linux

#ifdef ENGINE_PLATFORM_WINDOWS
  #ifdef ENGINE_BUILD_DLL
    #define ENGINE_API __declspec(dllexport)
  #else
    #define ENGINE_API __declspec(dllimport)
  #endif

  #define ENGINE_DEBUG_BREAK() __debugbreak()
#elif defined ENGINE_PLATFORM_LINUX
  #ifdef ENGINE_BUILD_DLL
    #define ENGINE_API __attribute__((visibility("default")))
  #else
    #define ENGINE_API
  #endif

  #define ENGINE_DEBUG_BREAK() __builtin_trap()
#else
  #define ENGINE_API
  #define ENGINE_DEBUG_BREAK()
#endif

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

#define BIT(x) (1 << x)

#endif  // ENGINE_SRC_CORE_CORE_H_
