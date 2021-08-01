/// @file Concepts.h
/// @brief Enables the support of concepts for all platforms.
///
/// This file is only needed until clang gets native concept support or I get
/// g++ to compile Lambda on macos.
#ifndef LAMBDA_SRC_LAMBDA_LIB_CONCEPTS_H_
#define LAMBDA_SRC_LAMBDA_LIB_CONCEPTS_H_

#ifdef LAMBDA_PLATFORM_MACOS
  #include <Lambda/platform/macos/clang/Concepts.h>
#else
  #include <concepts>
#endif

#endif  // LAMBDA_SRC_LAMBDA_LIB_CONCEPTS_H_
