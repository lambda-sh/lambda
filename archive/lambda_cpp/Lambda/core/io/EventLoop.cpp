#include <Lambda/core/io/EventLoop.h>

#include <Lambda/lib/Time.h>

namespace lambda::core::io {

/// @todo (C3NZ): Investigate into the amount of time needed to sleep by the
/// thread that this loop is running in.
/// @todo (C3NZ): Is this as performant as it can possibly be?
/// @todo (C3NZ): There is no way to currently turn this off when there should
/// be. Especially if this is running in another thread.
void EventLoop::Run() {
  while (running_) {
    std::this_thread::sleep_for(lib::Milliseconds(50));

    UniqueAsyncTask next_task;
    if (const bool has_next = event_queue_.try_dequeue(next_task); !has_next) {
      continue;
    }

    const AsyncStatus task_status = next_task->GetStatus();

    // Callback has expired.
    if (task_status == AsyncStatus::Expired) {
      LAMBDA_CORE_TRACE("Task [{0}] has expired", next_task->GetName());
      continue;
    }

    // Still waiting to execute.
    if (task_status == AsyncStatus::Deferred) {
      const bool has_space = event_queue_.enqueue(std::move(next_task));
      LAMBDA_CORE_ASSERT(
          has_space,
          "The Event loop has run out of space with {} nodes",
          size_);
      continue;
    }

    // Handle failure.
    if (const AsyncResult result = next_task->Execute();
        result == AsyncResult::Failure) {
      LAMBDA_CORE_ERROR(
          "Task [{}] has failed to execute.",
          next_task->GetName());
      continue;
    }

    LAMBDA_CORE_TRACE("Task [{0}] has completed.", next_task->GetName());
  }
}

bool EventLoop::SetTimeout(
    AsyncCallback callback, const uint32_t milliseconds) {
  UniqueAsyncTask task = memory::CreateUnique<AsyncTask>(
      std::move(callback),
      lib::Time::Now(),
      lib::Time::MillisecondsFromNow(milliseconds));
  return Dispatch(std::move(task));
}

bool EventLoop::SetInterval(
    AsyncCallback callback, const uint32_t milliseconds) {
  UniqueAsyncTask task = memory::CreateUnique<AsyncTask>(
      std::move(callback),
      lib::Time::Now(),
      lib::Time::MillisecondsFromNow(milliseconds));
  return Dispatch(std::move(task));
}

bool EventLoop::Dispatch(
    AsyncCallback callback, lib::Time execute_at, lib::Time expire_at) {
  UniqueAsyncTask task = memory::CreateUnique<AsyncTask>(
      std::move(callback), execute_at, expire_at);
  return Dispatch(std::move(task));
}

/// TODO(C3NZ): Do we need to use std::move since objects with well
/// defined move semantics are copyable into the queue?
///
/// Private dispatch for putting the task into the queue.
bool EventLoop::Dispatch(UniqueAsyncTask task) {
  const bool has_space = event_queue_.enqueue(std::move(task));
  LAMBDA_CORE_ASSERT(
      has_space, "The Event loop has run out of space with {} nodes", size_);
  return has_space;
}

}  // namespace lambda::core::io
