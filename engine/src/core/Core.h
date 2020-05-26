/**
 * @file engine/src/core/Core.h
 * @brief Core macros to be used all throughout the engine for development and
 * production purposes.
 *
 * This header includes core macros that are used all throughout the engine for
 * many different internal usage. You most likely won't need to use anything
 * from this file specifically and instead might find `core/Log.h` and
 * `core/Assert.h` more useful as they extend core and provide logging and
 * assertions (If enabled).
 */

/**
 * @def ENGINE_API
 * @brief Handles exporting the engine if it's being built as a dll.
 * Potentially deprecated and shouldn't be used outside of the engine.
 */

/**
 * @def ENGINE_ENABLE_ASSERTS
 * @brief Allows the usage of `core/Assert.h` Assertions to execute in the
 * engine.
 */

/**
 * @def ENGINE_DEBUG_BREAK()
 * @brief Allows the execution of breakpoints throughout the engine when
 * debuggin.
 */

/**
 * @def BIT(x)
 * @brief Creates the bit representation of an integer.
 */

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
    #define ENGINE_ENABLE_ASSERTS true
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
    #define ENGINE_ENABLE_ASSERTS true
    #define ENGINE_DEBUG_BREAK() __builtin_trap()
  #else
    #define ENGINE_DEBUG_BREAK()
  #endif
#else
  #define ENGINE_API
  #define ENGINE_ENABLE_ASSERTS false
  #define ENGINE_DEBUG_BREAK()
#endif

#define BIT(x) (1 << x)

#endif  // ENGINE_SRC_CORE_CORE_H_
