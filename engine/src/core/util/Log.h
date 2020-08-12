/**
 * @file Log.h
 * @brief The engines util utility.
 *
 * Can be used both in the engine and client application.
 */
#ifndef ENGINE_SRC_CORE_UTIL_LOG_H_
#define ENGINE_SRC_CORE_UTIL_LOG_H_

#include <memory>

#include <spdlog/spdlog.h>

#include "core/Core.h"

namespace engine {
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
}  // namespace engine

#define ENGINE_CORE_TRACE(...) \
    ::engine::core::util::Log::GetCoreLogger()->trace(__VA_ARGS__)

#define ENGINE_CORE_INFO(...)  \
    ::engine::core::util::Log::GetCoreLogger()->info(__VA_ARGS__)

#define ENGINE_CORE_WARN(...)  \
    ::engine::core::util::Log::GetCoreLogger()->warn(__VA_ARGS__)

#define ENGINE_CORE_ERROR(...) \
    ::engine::core::util::Log::GetCoreLogger()->error(__VA_ARGS__)

#define ENGINE_CORE_FATAL(...) \
    ::engine::core::util::Log::GetCoreLogger()->fatal(__VA_ARGS__)

#define ENGINE_CLIENT_TRACE(...) \
    ::engine::core::util::Log::GetClientLogger()->trace(__VA_ARGS__)

#define ENGINE_CLIENT_INFO(...)  \
    ::engine::core::util::Log::GetClientLogger()->info(__VA_ARGS__)

#define ENGINE_CLIENT_WARN(...)  \
    ::engine::core::util::Log::GetClientLogger()->warn(__VA_ARGS__)

#define ENGINE_CLIENT_ERROR(...) \
    ::engine::core::util::Log::GetClientLogger()->error(__VA_ARGS__)

#define ENGINE_CLIENT_FATAL(...) \
    ::engine::core::util::Log::GetClientLogger()->fatal(__VA_ARGS__)

#endif  // ENGINE_SRC_CORE_UTIL_LOG_H_

/**
 * @class engine::core::util::Log
 * @brief The container class for managing static instances of the engine and
 * client loggers.
 */

/**
 * @def ENGINE_CORE_TRACE(...)
 * @brief Engine core trace output.
 */

/**
 * @def ENGINE_CORE_INFO(...)
 * @brief Engine core info output.
 */

/**
 * @def ENGINE_CORE_WARN(...)
 * @brief Engine core warning output.
 */

/**
 * @def ENGINE_CORE_ERROR(...)
 * @brief Engine core error output.
 */

/**
 * @def ENGINE_CORE_FATAL(...)
 * @brief Engine core fatal output. Exits the application.
 */

/**
 * @def ENGINE_CLIENT_TRACE(...)
 * @brief Engine client trace output.
 */

/**
 * @def ENGINE_CLIENT_INFO(...)
 * @brief Engine client info output.
 */

/**
 * @def ENGINE_CLIENT_WARN(...)
 * @brief Engine client warning output.
 */

/**
 * @def ENGINE_CLIENT_ERROR(...)
 * @brief Engine client error output.
 */

/**
 * @def ENGINE_CLIENT_FATAL(...)
 * @brief Engine client fatal output.
 */
