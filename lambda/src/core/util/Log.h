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

class Log {
 public:
  static void Init();

  inline static std::shared_ptr<spdlog::logger>& GetCoreLogger() {
      return kCoreLogger; }

  inline static std::shared_ptr<spdlog::logger>& GetClientLogger() {
      return kClientLogger; }

 private:
  static std::shared_ptr<spdlog::logger> kCoreLogger;
  static std::shared_ptr<spdlog::logger> kClientLogger;
};

}  // namespace util
}  // namespace core
}  // namespace lambda

#define LAMBDA_CORE_TRACE(...) \
    ::lambda::core::util::Log::GetCoreLogger()->trace(__VA_ARGS__)

#define LAMBDA_CORE_INFO(...)  \
    ::lambda::core::util::Log::GetCoreLogger()->info(__VA_ARGS__)

#define LAMBDA_CORE_WARN(...)  \
    ::lambda::core::util::Log::GetCoreLogger()->warn(__VA_ARGS__)

#define LAMBDA_CORE_ERROR(...) \
    ::lambda::core::util::Log::GetCoreLogger()->error(__VA_ARGS__)

#define LAMBDA_CORE_FATAL(...) \
    ::lambda::core::util::Log::GetCoreLogger()->fatal(__VA_ARGS__)

#define LAMBDA_CLIENT_TRACE(...) \
    ::lambda::core::util::Log::GetClientLogger()->trace(__VA_ARGS__)

#define LAMBDA_CLIENT_INFO(...)  \
    ::lambda::core::util::Log::GetClientLogger()->info(__VA_ARGS__)

#define LAMBDA_CLIENT_WARN(...)  \
    ::lambda::core::util::Log::GetClientLogger()->warn(__VA_ARGS__)

#define LAMBDA_CLIENT_ERROR(...) \
    ::lambda::core::util::Log::GetClientLogger()->error(__VA_ARGS__)

#define LAMBDA_CLIENT_FATAL(...) \
    ::lambda::core::util::Log::GetClientLogger()->fatal(__VA_ARGS__)

#endif  // LAMBDA_SRC_CORE_UTIL_LOG_H_

/**
 * @class lambda::core::util::Log
 * @brief The container class for managing static instances of the engine and
 * client loggers.
 */

/**
 * @def LAMBDA_CORE_TRACE(...)
 * @brief Engine core trace output.
 */

/**
 * @def LAMBDA_CORE_INFO(...)
 * @brief Engine core info output.
 */

/**
 * @def LAMBDA_CORE_WARN(...)
 * @brief Engine core warning output.
 */

/**
 * @def LAMBDA_CORE_ERROR(...)
 * @brief Engine core error output.
 */

/**
 * @def LAMBDA_CORE_FATAL(...)
 * @brief Engine core fatal output. Exits the application.
 */

/**
 * @def LAMBDA_CLIENT_TRACE(...)
 * @brief Engine client trace output.
 */

/**
 * @def LAMBDA_CLIENT_INFO(...)
 * @brief Engine client info output.
 */

/**
 * @def LAMBDA_CLIENT_WARN(...)
 * @brief Engine client warning output.
 */

/**
 * @def LAMBDA_CLIENT_ERROR(...)
 * @brief Engine client error output.
 */

/**
 * @def LAMBDA_CLIENT_FATAL(...)
 * @brief Engine client fatal output.
 */
