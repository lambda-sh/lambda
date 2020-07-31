// TODO(C3NZ): Add documentation for this file.
#ifndef ENGINE_SRC_CORE_IO_ASYNCTASK_H_
#define ENGINE_SRC_CORE_IO_ASYNCTASK_H_

#include <bits/stdint-uintn.h>
#include <functional>

#include "core/memory/Pointers.h"
#include "core/util/Time.h"

namespace engine {
namespace io {

class AsyncTask;

typedef std::function<bool()> AsyncCallback;
typedef memory::Unique<AsyncTask> UniqueAsyncTask;

enum class AsyncStatus {
  None = 0,
  Deferred,
  Ready,
  Expired
};

enum class AsyncResult {
  None = 0,
  Failure,
  Success
};

class AsyncTask {
 public:
  AsyncTask(
      AsyncCallback callback,
      util::Time execute_at = util::Time(),
      util::Time expires_at = util::Time().AddSeconds(5)) :
          callback_(callback),
          scheduled_at_(util::Time()),
          execute_at_(execute_at),
          expires_at_(expires_at) {}

  AsyncTask(
      AsyncCallback callback,
      uint32_t interval_in_ms,
      bool should_repeat) :
          callback_(callback),
          scheduled_at_(util::Time()),
          execute_at_(scheduled_at_.AddMilliseconds(interval_in_ms_)),
          expires_at_(execute_at_.AddSeconds(5)),
          should_repeat_(should_repeat),
          interval_in_ms_(interval_in_ms) {}

  bool ShouldRepeat() { return should_repeat_; }

  AsyncResult Execute() {
    if (callback_()) {
      return AsyncResult::Success;
    }
    return AsyncResult::Failure;
  }

  AsyncStatus GetStatus() {
    if (expires_at_.HasPassed()) {
      return AsyncStatus::Expired;
    }

    if (execute_at_.HasPassed()) {
      return AsyncStatus::Ready;
    }

    return AsyncStatus::Deferred;
  }

  const std::string& GetName() const { return name_; }
  const uint32_t GetIntervalInMilliseconds() const { return interval_in_ms_; }

  // Sets when to execute at.
  void ExecuteAt(util::Time new_time) {
    execute_at_ = new_time;
  }

  void ExecuteIn(uint32_t milliseconds) {
    execute_at_ = execute_at_.AddMilliseconds(milliseconds);
    expires_at_ = execute_at_.AddSeconds(5);
  }

 private:
  std::string name_;
  AsyncCallback callback_;
  bool should_repeat_ = false;
  uint32_t interval_in_ms_;
  util::Time scheduled_at_, execute_at_, executed_at_, expires_at_;
};

}  // namespace io
}  // namespace engine

#endif  // ENGINE_SRC_CORE_IO_ASYNCTASK_H_
