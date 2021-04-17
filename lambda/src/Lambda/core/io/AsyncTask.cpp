#include <Lambda/core/io/AsyncTask.h>

#include <Lambda/lib/Time.h>

namespace lambda::core::io {

/// TODO(C3NZ): Callbacks should be made more generic. Is it possible to allow
// values to escape the callback once it's been resolved/rejected?
/// Executes the task if and returns a success if the callback succeeds.
AsyncResult AsyncTask::Execute() const {
  if (callback_()) {
    return AsyncResult::Success;
  }
  return AsyncResult::Failure;
}

/// The status refers to whether or not the task has executed.
AsyncStatus AsyncTask::GetStatus() const {
  if (expires_at_.HasPassed()) {
    return AsyncStatus::Expired;
  }

  if (execute_at_.HasPassed()) {
    return AsyncStatus::Ready;
  }

  return AsyncStatus::Deferred;
}

/// Resets the task to execute at a future time. Usually done through the event
/// loop.
void AsyncTask::RescheduleTask(
    const lib::Time new_execution_time, const lib::Time new_expiration_time) {
  execute_at_ = new_execution_time;
  expires_at_ = new_expiration_time;
}

}  // namespace lambda::core::io
