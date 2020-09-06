#include "Lambda/core/util/Log.h"

#include <spdlog/sinks/stdout_color_sinks.h>
#include <spdlog/spdlog.h>

namespace lambda {
namespace core {
namespace util {

std::shared_ptr<spdlog::logger> Log::kCoreLogger;
std::shared_ptr<spdlog::logger> Log::kClientLogger;

void Log::Init() {
  spdlog::set_pattern("%^[%T]-[%n]-[%s]: %v%$");
  kCoreLogger = spdlog::stdout_color_mt("Engine");
  kCoreLogger->set_level(spdlog::level::trace);

  kClientLogger = spdlog::stdout_color_mt("App");
  kCoreLogger->set_level(spdlog::level::trace);
}

}  // namespace util
}  // namespace core
}  // namespace lambda
