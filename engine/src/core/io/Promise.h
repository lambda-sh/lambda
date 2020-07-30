#ifndef ENGINE_SRC_CORE_IO_PROMISE_H_
#define ENGINE_SRC_CORE_IO_PROMISE_H_

#include <string>
#include <functional>

#include <concurrentqueue.h>

#include "core/memory/Pointers.h"
#include "core/io/Dispatcher.h"
#include "core/util/Time.h"

namespace engine {
namespace io {

class AsyncTask;

typedef std::function<bool()> EventFunc;
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
      EventFunc callback,
      util::Time execute_at = util::Time(),
      util::Time expires_at = util::Time().AddSeconds(5)) :
          callback_(callback),
          scheduled_at_(util::Time()),
          execute_at_(execute_at),
          expires_at_(expires_at) {}

  AsyncTask(EventFunc callback, uint32_t interval_in_ms, bool should_repeat) :
      callback_(callback),
      should_repeat_(should_repeat),
      interval_in_ms_(interval_in_ms) {
        scheduled_at_ = util::Time();
        execute_at_ = scheduled_at_.AddMilliseconds(interval_in_ms_);
        expires_at_ = util::Time(execute_at_.AddSeconds(5));
      }

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
  EventFunc callback_;
  bool should_repeat_ = false;
  uint32_t interval_in_ms_;
  util::Time scheduled_at_, execute_at_, executed_at_, expires_at_;
};

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

  bool SetTimeout(EventFunc callback, uint32_t milliseconds) {
    UniqueAsyncTask task = memory::CreateUnique<AsyncTask>(
        callback, milliseconds, false);
    return Dispatch(std::move(task));
  }

  bool SetInterval(EventFunc callback, uint32_t milliseconds) {
    UniqueAsyncTask task = memory::CreateUnique<AsyncTask>(
        callback, milliseconds, true);
    return Dispatch(std::move(task));
  }

  bool Dispatch(
      EventFunc callback,
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
  moodycamel::ConcurrentQueue<memory::Unique<AsyncTask>> event_queue_;
};


}  // namespace io
}  // namespace engine


#endif  // ENGINE_SRC_CORE_IO_PROMISE_H_
