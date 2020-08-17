/// @file AsyncTask.h
/// @brief A convenient wrapper for callback functions being dispatched into the
/// event loop
// TODO(C3NZ): Add documentation for this file.
#ifndef LAMBDA_SRC_CORE_IO_ASYNCTASK_H_
#define LAMBDA_SRC_CORE_IO_ASYNCTASK_H_

#include <bits/stdint-uintn.h>
#include <functional>

#include "core/memory/Pointers.h"
#include "core/util/Time.h"

namespace lambda {
namespace core {
namespace io {

class AsyncTask;

typedef std::function<bool()> AsyncCallback;
typedef memory::Unique<AsyncTask> UniqueAsyncTask;

/// @brief The execution status of the async function.
///
/// Must be ready in order to execute.
enum class AsyncStatus {
  None = 0,
  Deferred,
  Ready,
  Expired
};

/// @brief The result of calling the async function.
enum class AsyncResult {
  None = 0,
  Failure,
  Success
};

/// @brief A wrapper for callbacks that are supposed to be executed
/// asynchronously.
///
/// There is no need to use this class externally, as the EventLoop will create
/// these upon passing it a AsyncCallback.
class AsyncTask {
 public:
  /// @brief Construct a task that should execute as soon as possible.
  AsyncTask(
      AsyncCallback callback,
      util::Time execute_at = util::Time(),
      util::Time expires_at = util::Time().AddSeconds(5)) :
          callback_(callback),
          scheduled_at_(util::Time()),
          execute_at_(execute_at),
          expires_at_(expires_at) {}

  /// @brief Construct a task that should execute from a certain period from the
  /// current time. (And potentially in an interval)
  AsyncTask(
      AsyncCallback callback,
      uint32_t interval_in_ms,
      bool should_repeat) :
          callback_(callback),
          scheduled_at_(core::util::Time()),
          execute_at_(scheduled_at_.AddMilliseconds(interval_in_ms_)),
          expires_at_(execute_at_.AddSeconds(5)),
          should_repeat_(should_repeat),
          interval_in_ms_(interval_in_ms) {}

  /// @brief Executes the AsyncCallback and returns back the result.
  AsyncResult Execute();

  /// @brief Gets the execution status of the callback.
  AsyncStatus GetStatus();

  /// @brief Allows a task to be rescheduled with new times.
  void RescheduleTask(
      core::util::Time new_execution_time,
      core::util::Time new_expiration_time);

  /// @brief Get the name of the task (Currently not implemented.)
  // TODO(C3NZ): There should be overloads in the EventLoop that allow callback
  // functions to easily be named.
  const std::string& GetName() const { return name_; }

  /// @brief
  const uint32_t GetIntervalInMilliseconds() const { return interval_in_ms_; }

  /// @brief Allow the ability to see if the task is setup to repeat.
  ///
  /// It is by default set to false.
  bool ShouldRepeat() { return should_repeat_; }

 private:
  std::string name_;
  AsyncCallback callback_;
  bool should_repeat_ = false;
  uint32_t interval_in_ms_;
  util::Time scheduled_at_, execute_at_, executed_at_, expires_at_;
};

}  // namespace io
}  // namespace core
}  // namespace lambda

#endif  // LAMBDA_SRC_CORE_IO_ASYNCTASK_H_
