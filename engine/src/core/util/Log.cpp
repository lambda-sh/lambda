#include "core/Log.h"

#include <spdlog/spdlog.h>
#include <spdlog/sinks/stdout_color_sinks.h>

namespace engine {
namespace logging {

std::shared_ptr<spdlog::logger> Log::s_CoreLogger;
std::shared_ptr<spdlog::logger> Log::s_ClientLogger;

// Initializes both the client and core logger.
void Log::Init() {
  spdlog::set_pattern("%^[%T] %n: %v%$");
  s_CoreLogger = spdlog::stdout_color_mt("Engine");
  s_CoreLogger->set_level(spdlog::level::trace);

  s_ClientLogger = spdlog::stdout_color_mt("App");
  s_CoreLogger->set_level(spdlog::level::trace);
}

}  // namespace logging
}  // namespace engine
