#ifndef ENGINE_SRC_CORE_CORE_H_
#define ENGINE_SRC_CORE_CORE_H_

#undef linux

#ifdef ENGINE_PLATFORM_WINDOWS
  #ifdef ENGINE_BUILD_DLL
    #define ENGINE_API __declspec(dllexport)
  #else
    #define ENGINE_API __declspec(dllimport)
  #endif

  #ifdef ENGINE_DEBUG
    #define ENGINE_ENABLE_ASSERTS
    #define ENGINE_DEBUG_BREAK() __debugbreak()
  #else
    #define ENGINE_DEBUG_BREAK()
  #endif
#elif defined ENGINE_PLATFORM_LINUX
  #ifdef ENGINE_BUILD_DLL
    #define ENGINE_API __attribute__((visibility("default")))
  #else
    #define ENGINE_API
  #endif

  #ifdef ENGINE_DEBUG
    #define ENGINE_ENABLE_ASSERTS
    #define ENGINE_DEBUG_BREAK() __builtin_trap()
  #else
    #define ENGINE_DEBUG_BREAK()
  #endif
#else
  #define ENGINE_API
  #define ENGINE_DEBUG_BREAK()
#endif

#define BIT(x) (1 << x)

#endif  // ENGINE_SRC_CORE_CORE_H_
