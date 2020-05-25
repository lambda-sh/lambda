#ifndef ENGINE_SRC_CORE_LOG_H_
#define ENGINE_SRC_CORE_LOG_H_

#include <memory>

#include "spdlog/spdlog.h"

#include "Core.h"

namespace engine {
namespace logging {

// The Log class allows us to obtain our log instances, both of which are
// exposed to the engine while only the client logger is exposed to the client.
class ENGINE_API Log {
 public:
  static void Init();

  inline static std::shared_ptr<spdlog::logger>& GetCoreLogger()
      { return s_CoreLogger; }
  inline static std::shared_ptr<spdlog::logger>& GetClientLogger()
      { return s_ClientLogger; }

 private:
  static std::shared_ptr<spdlog::logger> s_CoreLogger;
  static std::shared_ptr<spdlog::logger> s_ClientLogger;
};

}  // namespace logging
}  // namespace engine

// TODO(C3NZ): Evaluate if there's a better way to implement our logger.
// Ideally, I would like to attach these macros as functions of the Log class,
// but am not sure how to handle the variadic parameters as of now.

#define ENGINE_CORE_TRACE(...) \
    ::engine::logging::Log::GetCoreLogger()->trace(__VA_ARGS__)
#define ENGINE_CORE_INFO(...)  \
    ::engine::logging::Log::GetCoreLogger()->info(__VA_ARGS__)
#define ENGINE_CORE_WARN(...)  \
    ::engine::logging::Log::GetCoreLogger()->warn(__VA_ARGS__)
#define ENGINE_CORE_ERROR(...) \
    ::engine::logging::Log::GetCoreLogger()->error(__VA_ARGS__)
#define ENGINE_CORE_FATAL(...) \
    ::engine::logging::Log::GetCoreLogger()->fatal(__VA_ARGS__)

// Client log macros
#define ENGINE_CLIENT_TRACE(...) \
    ::engine::logging::Log::GetClientLogger()->trace(__VA_ARGS__)
#define ENGINE_CLIENT_INFO(...)  \
    ::engine::logging::Log::GetClientLogger()->info(__VA_ARGS__)
#define ENGINE_CLIENT_WARN(...)  \
    ::engine::logging::Log::GetClientLogger()->warn(__VA_ARGS__)
#define ENGINE_CLIENT_ERROR(...) \
    ::engine::logging::Log::GetClientLogger()->error(__VA_ARGS__)
#define ENGINE_CLIENT_FATAL(...) \
    ::engine::logging::Log::GetClientLogger()->fatal(__VA_ARGS__)

#endif  // ENGINE_SRC_CORE_LOG_H_
