#include "core/io/EventLoop.h"

namespace engine {
namespace core {
namespace io {

// TODO(C3NZ): Investigate into the amount of time needed to sleep by the thread
// that this loop is running in.
// TODO(C3NZ): Is this as performant as it can possibly be?
void EventLoop::Run() {
  while (running_) {
    std::this_thread::sleep_for(util::Milliseconds(50));

    UniqueAsyncTask next_task;
    bool has_next = event_queue_.try_dequeue(next_task);

    if (!has_next) {
      std::this_thread::sleep_for(util::Milliseconds(50));
      continue;
    }

    AsyncStatus task_status = next_task->GetStatus();

    // Callback has expired.
    if (task_status == AsyncStatus::Expired) {
      ENGINE_CORE_TRACE("Task [{0}] has expired", next_task->GetName());
      continue;
    }

    // Still waiting to execute.
    if (task_status == AsyncStatus::Deferred) {
      bool has_space = event_queue_.enqueue(std::move(next_task));
      ENGINE_CORE_ASSERT(has_space, "The Event loop has run out of space")
      continue;
    }

    // Callback is ready.
    AsyncResult result = next_task->Execute();

    // Handle failure.
    if (result == AsyncResult::Failure) {
      ENGINE_CORE_ERROR(
          "Task [{}] has failed to execute.",
          next_task->GetName());
      continue;
    }

    ENGINE_CORE_TRACE("Task [{0}] has completed.", next_task->GetName());

    // Reschedule if it should repeat.
    if (next_task->ShouldRepeat()) {
      util::Time next_execution_time = core::util::Time().AddMilliseconds(
          next_task->GetIntervalInMilliseconds());
      util::Time next_expiration_time = next_execution_time.AddMilliseconds(
          5000);
      next_task->RescheduleTask(next_execution_time, next_expiration_time);

      bool has_space = event_queue_.enqueue(std::move(next_task));
      ENGINE_CORE_ASSERT(has_space, "The Event loop has run out of space")
    }
  }
}

bool EventLoop::SetTimeout(AsyncCallback callback, uint32_t milliseconds) {
  UniqueAsyncTask task = memory::CreateUnique<AsyncTask>(
      callback, milliseconds, false);
  return Dispatch(std::move(task));
}

bool EventLoop::SetInterval(AsyncCallback callback, uint32_t milliseconds) {
  UniqueAsyncTask task = memory::CreateUnique<AsyncTask>(
      callback, milliseconds, true);
  return Dispatch(std::move(task));
}

bool EventLoop::Dispatch(
    AsyncCallback callback, util::Time execute_at, util::Time expire_at) {
  UniqueAsyncTask task = memory::CreateUnique<AsyncTask>(
      callback, execute_at, expire_at);
  return Dispatch(std::move(task));
}

// TODO(C3NZ): Do we need to use std::move since objects with well defined move
// semantics are copyable into the queue?
// Private dispatch for putting the task into the queue.
bool EventLoop::Dispatch(UniqueAsyncTask task) {
  bool has_space = event_queue_.enqueue(std::move(task));
  ENGINE_CORE_ASSERT(has_space, "The Event loop has run out of space")
  return has_space;
}

}  // namespace io
}  // namespace core
}  // namespace engine
