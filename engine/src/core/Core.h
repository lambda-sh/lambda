#ifndef SRC_CORE_CORE
#define SRC_CORE_CORE

  #ifdef ENGINE_PLATFORM_WINDOWS
    #ifdef ENGINE_BUILD_DLL
      #define ENGINE_API __declspec(dllexport)
    #else
      #define ENGINE_API __declspec(dllimport)
    #endif
  #elif ENGINE_PLATFORM_LINUX
    #ifdef ENGINE_BUILD_DLL
      #define ENGINE_API  __attribute__((visibility("default"))) 
    #else
      #define ENGINE_API
    #endif
  #endif

#endif
