// TODO(C3NZ): Add documentation for this file.
#ifndef ENGINE_SRC_CORE_IO_EVENTLOOP_H_
#define ENGINE_SRC_CORE_IO_EVENTLOOP_H_

#include <concurrentqueue.h>

#include "core/io/AsyncTask.h"
#include "core/memory/Pointers.h"
#include "core/util/Time.h"

namespace engine {
namespace io {

using util::Time;

// TODO(C3NZ):
// Dispatch -- To diapatch callbacks to that are meant to run ASAP.
// SetInterval -- For Setting a callback to run in an interval.
// SetTimeout -- For setting a callback that should run in the future.
//
// Currently will be copy only, since accessing data across threads can lead
// to problems. (At least without locking or atomics.)
class EventLoop {
 public:
  explicit EventLoop(uint32_t size = 256)
      : running_(true), event_queue_(size) {}

  void Run() {
    while (running_) {
      std::this_thread::sleep_for(util::Milliseconds(100));

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
        next_task->ExecuteIn(next_task->GetIntervalInMilliseconds());
        bool has_space = event_queue_.enqueue(std::move(next_task));
        ENGINE_CORE_ASSERT(has_space, "The Event loop has run out of space")
      }
    }
  }

  bool SetTimeout(AsyncCallback callback, uint32_t milliseconds) {
    UniqueAsyncTask task = memory::CreateUnique<AsyncTask>(
        callback, milliseconds, false);
    return Dispatch(std::move(task));
  }

  bool SetInterval(AsyncCallback callback, uint32_t milliseconds) {
    UniqueAsyncTask task = memory::CreateUnique<AsyncTask>(
        callback, milliseconds, true);
    return Dispatch(std::move(task));
  }

  bool Dispatch(
      AsyncCallback callback,
      util::Time execute_at = util::Time(),
      util::Time expire_at = util::Time().AddSeconds(5)) {
    UniqueAsyncTask task = memory::CreateUnique<AsyncTask>(
        callback, execute_at, expire_at);
    return Dispatch(std::move(task));
  }

 private:
  // Private dispatch for putting the task into the queue.
  bool Dispatch(UniqueAsyncTask task) {
    bool has_space = event_queue_.enqueue(std::move(task));
    ENGINE_CORE_ASSERT(has_space, "The Event loop has run out of space")
    return has_space;
  }

  bool running_;
  // TODO(C3NZ): Investigate into the performance of std::atomic
  // vs using a mutex.
  moodycamel::ConcurrentQueue<UniqueAsyncTask> event_queue_;
};


}  // namespace io
}  // namespace engine

#endif  // ENGINE_SRC_CORE_IO_EVENTLOOP_H_
