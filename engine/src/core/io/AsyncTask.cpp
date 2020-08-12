#include "core/io/AsyncTask.h"

#include "core/util/Time.h"

namespace engine {
namespace core {
namespace io {

using util::Time;
using util::Milliseconds;

// TODO(C3NZ): Callbacks should be made more generic. Is it possible to allow
// values to escape the callback once it's been resolved/rejected?
AsyncResult AsyncTask::Execute() {
  if (callback_()) {
    return AsyncResult::Success;
  }
  return AsyncResult::Failure;
}

AsyncStatus AsyncTask::GetStatus() {
  if (expires_at_.HasPassed()) {
    return AsyncStatus::Expired;
  }

  if (execute_at_.HasPassed()) {
    return AsyncStatus::Ready;
  }

  return AsyncStatus::Deferred;
}

void AsyncTask::RescheduleTask(
    Time new_execution_time, Time new_expiration_time) {
  execute_at_ = new_execution_time;
  expires_at_ = new_expiration_time;
}

}  // namespace io
}  // namespace core
}  // namespace engine
