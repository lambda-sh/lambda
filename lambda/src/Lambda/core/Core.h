/// @file Core.h
/// @brief Core macros to be used all throughout the engine for development and
/// production purposes.
///
/// This header includes core macros that are used all throughout the engine for
/// many different internal usage. You most likely won't need to use anything
/// from this file specifically and instead might find `core/Log.h` and
/// `core/Assert.h` more useful as they extend core and provide logging and
/// assertions (If enabled).
#ifndef LAMBDA_SRC_LAMBDA_CORE_CORE_H_
#define LAMBDA_SRC_LAMBDA_CORE_CORE_H_

#undef linux

/// @def LAMBDA_API
/// @brief Handles exporting the engine if it's being built as a dll.
/// Potentially deprecated and shouldn't be used outside of the engine.

/// @def LAMBDA_ENABLE_ASSERTS
/// @brief Allows the usage of `core/Assert.h` Assertions to execute in the
/// engine.

/// @def LAMBDA_DEBUG_BREAK()
/// @brief Allows the execution of breakpoints throughout the engine when
/// debuggin.

/// @def BIT(x)
/// @param x The number to represent in bits.
/// @brief Creates the bit representation of an integer.
///
/// Mostly for internal usage (Check `engine/src/core/events/Event.h`)

#ifdef LAMBDA_PLATFORM_WINDOWS
  #ifdef LAMBDA_BUILD_DLL
    #define LAMBDA_API __declspec(dllexport)
  #else
    #define LAMBDA_API __declspec(dllimport)
  #endif

  #ifdef LAMBDA_DEBUG
    #define LAMBDA_ENABLE_ASSERTS true
    #define LAMBDA_DEBUG_BREAK() __debugbreak()
  #else
    #define LAMBDA_DEBUG_BREAK()
  #endif
#elif defined LAMBDA_PLATFORM_LINUX
  #ifdef LAMBDA_BUILD_DLL
    #define LAMBDA_API __attribute__((visibility("default")))
  #else
    #define LAMBDA_API
  #endif

  #ifdef LAMBDA_DEBUG
    #define LAMBDA_ENABLE_ASSERTS true
    #define LAMBDA_DEBUG_BREAK() __builtin_trap()
  #else
    #define LAMBDA_DEBUG_BREAK()
  #endif
#else
  #define LAMBDA_API
  #define LAMBDA_ENABLE_ASSERTS false
  #define LAMBDA_DEBUG_BREAK()
#endif

#define BIT(x) (1 << x)

#endif  // LAMBDA_SRC_LAMBDA_CORE_CORE_H_
