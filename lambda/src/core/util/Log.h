/**
 * @file Log.h
 * @brief The engines util utility.
 *
 * Can be used both in the engine and client application.
 */
#ifndef LAMBDA_SRC_CORE_UTIL_LOG_H_
#define LAMBDA_SRC_CORE_UTIL_LOG_H_

#include <memory>

#include <spdlog/spdlog.h>

#include "core/Core.h"

namespace lambda {
namespace core {
namespace util {

/// @brief The engine wide logging API. Should primarily be used through the
/// macros exposed at the bottom of the API.
class Log {
 public:
  static void Init();

  static std::shared_ptr<spdlog::logger> GetCoreLogger() {
      return kCoreLogger; }

  static std::shared_ptr<spdlog::logger> GetClientLogger() {
      return kClientLogger; }

 private:
  static std::shared_ptr<spdlog::logger> kCoreLogger;
  static std::shared_ptr<spdlog::logger> kClientLogger;
};

}  // namespace util
}  // namespace core
}  // namespace lambda

/// @def LAMBDA_CORE_TRACE
/// @brief Log tracing information within the engine.
#define LAMBDA_CORE_TRACE(...) \
    ::lambda::core::util::Log::GetCoreLogger()->trace(__VA_ARGS__)

/// @def LAMBDA_CORE_INFO
/// @brief Log informational information within the engine.
#define LAMBDA_CORE_INFO(...)  \
    ::lambda::core::util::Log::GetCoreLogger()->info(__VA_ARGS__)

/// @def LAMBDA_CORE_WARN
/// @brief Log warning information within the engine.
#define LAMBDA_CORE_WARN(...)  \
    ::lambda::core::util::Log::GetCoreLogger()->warn(__VA_ARGS__)

/// @def LAMBDA_CORE_ERROR
/// @brief Log error information within the engine.
#define LAMBDA_CORE_ERROR(...) \
    ::lambda::core::util::Log::GetCoreLogger()->error(__VA_ARGS__)

/// @def LAMBDA_CORE_ERROR
/// @brief Log fatal information within the engine.
#define LAMBDA_CORE_FATAL(...) \
    ::lambda::core::util::Log::GetCoreLogger()->fatal(__VA_ARGS__)

/// @def LAMBDA_CLIENT_TRACE
/// @brief Log tracing information within the application.
#define LAMBDA_CLIENT_TRACE(...) \
    ::lambda::core::util::Log::GetClientLogger()->trace(__VA_ARGS__)

/// @def LAMBDA_CLIENT_INFO
/// @brief Log informational information within the application.
#define LAMBDA_CLIENT_INFO(...)  \
    ::lambda::core::util::Log::GetClientLogger()->info(__VA_ARGS__)

/// @def LAMBDA_CLIENT_WARN
/// @brief Log warning information within the application.
#define LAMBDA_CLIENT_WARN(...)  \
    ::lambda::core::util::Log::GetClientLogger()->warn(__VA_ARGS__)

/// @def LAMBDA_CLIENT_ERROR
/// @brief Log error information within the application.
#define LAMBDA_CLIENT_ERROR(...) \
    ::lambda::core::util::Log::GetClientLogger()->error(__VA_ARGS__)

/// @def LAMBDA_CLIENT_FATAL
/// @brief Log fatal information within the application.
#define LAMBDA_CLIENT_FATAL(...) \
    ::lambda::core::util::Log::GetClientLogger()->fatal(__VA_ARGS__)

#endif  // LAMBDA_SRC_CORE_UTIL_LOG_H_
