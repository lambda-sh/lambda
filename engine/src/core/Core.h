#ifndef ENGINE_CORE_CORE_H_
#define ENGINE_CORE_CORE_H_

  #ifdef ENGINE_PLATFORM_WINDOWS
    #ifdef ENGINE_BUILD_DLL
      #define ENGINE_API __declspec(dllexport)
    #else
      #define ENGINE_API __declspec(dllimport)
    #endif
  #elif defined ENGINE_PLATFORM_LINUX
    #ifdef ENGINE_BUILD_DLL
      #define ENGINE_API __attribute__((visibility("default"))) 
    #else
      #define ENGINE_API
    #endif
  #else 
    #define ENGINE_API
  #endif

#endif
