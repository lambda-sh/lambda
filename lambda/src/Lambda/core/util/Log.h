/**
 * @file Log.h
 * @brief The engines util utility.
 *
 * Can be used both in the engine and client application.
 */
#ifndef LAMBDA_SRC_LAMBDA_CORE_UTIL_LOG_H_
#define LAMBDA_SRC_LAMBDA_CORE_UTIL_LOG_H_

#include <memory>

#include <spdlog/spdlog.h>

#include "Lambda/core/Core.h"

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
    SPDLOG_LOGGER_TRACE( \
        ::lambda::core::util::Log::GetCoreLogger(), __VA_ARGS__);

/// @def LAMBDA_CORE_INFO
/// @brief Log informational information within the engine.
#define LAMBDA_CORE_INFO(...)  \
    SPDLOG_LOGGER_INFO(::lambda::core::util::Log::GetCoreLogger(), __VA_ARGS__);

/// @def LAMBDA_CORE_WARN
/// @brief Log warning information within the engine.
#define LAMBDA_CORE_WARN(...)  \
    SPDLOG_LOGGER_WARN(::lambda::core::util::Log::GetCoreLogger(), __VA_ARGS__);

/// @def LAMBDA_CORE_ERROR
/// @brief Log error information within the engine.
#define LAMBDA_CORE_ERROR(...) \
    SPDLOG_LOGGER_ERROR( \
        ::lambda::core::util::Log::GetCoreLogger(), __VA_ARGS__);

/// @def LAMBDA_CORE_ERROR
/// @brief Log fatal information within the engine.
#define LAMBDA_CORE_FATAL(...) \
    SPDLOG_LOGGER_FATAL( \
        ::lambda::core::util::Log::GetCoreLogger(), __VA_ARGS__);

/// @def LAMBDA_CLIENT_TRACE
/// @brief Log tracing information within the application.
#define LAMBDA_CLIENT_TRACE(...) \
    SPDLOG_LOGGER_TRACE( \
        ::lambda::core::util::Log::GetClientLogger(), __VA_ARGS__);

/// @def LAMBDA_CLIENT_INFO
/// @brief Log informational information within the application.
#define LAMBDA_CLIENT_INFO(...)  \
    SPDLOG_LOGGER_INFO(\
        ::lambda::core::util::Log::GetClientLogger(), __VA_ARGS__);

/// @def LAMBDA_CLIENT_WARN
/// @brief Log warning information within the application.
#define LAMBDA_CLIENT_WARN(...)  \
    SPDLOG_LOGGER_WARN( \
        ::lambda::core::util::Log::GetClientLogger(), __VA_ARGS__);

/// @def LAMBDA_CLIENT_ERROR
/// @brief Log error information within the application.
#define LAMBDA_CLIENT_ERROR(...) \
    SPDLOG_LOGGER_ERROR( \
        ::lambda::core::util::Log::GetClientLogger(), __VA_ARGS__);

/// @def LAMBDA_CLIENT_FATAL
/// @brief Log fatal information within the application.
#define LAMBDA_CLIENT_FATAL(...) \
    SPDLOG_LOGGER_FATAL( \
        ::lambda::core::util::Log::GetClientLogger(), __VA_ARGS__);

#endif  // LAMBDA_SRC_LAMBDA_CORE_UTIL_LOG_H_
