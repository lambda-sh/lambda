#ifndef LAMBDA_SRC_LAMBDA_PROFILER_TIMER_H_
#define LAMBDA_SRC_LAMBDA_PROFILER_TIMER_H_

#include "Lambda/core/util/Log.h"
#include "Lambda/core/util/Time.h"

namespace lambda {
namespace profiler {


/// @brief A basic RAII timer used to profile computation within Lambda.
class Timer {
 public:
  explicit Timer(const char* name) : name_(name), stopped_(false), start_() {}

  ~Timer() {
    if (!stopped_) {
      Stop();
    }
  }


 private:
  bool stopped_;
  const char* name_;
  core::util::Time start_;

  /// @brief Compute the duration of a function (Called within the Timers
  /// destructor.)
  void Stop() {
    core::util::Time end;
    core::util::TimeStep duration(start_, end);

    stopped_ = true;

    LAMBDA_CORE_INFO(
        "Duration of {0}: {1} ms", duration.InMilliSeconds<float>());
  }
};

}  // namespace profiler
}  // namespace lambda

#endif  // LAMBDA_SRC_LAMBDA_PROFILER_TIMER_H_
