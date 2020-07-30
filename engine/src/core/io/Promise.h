#ifndef ENGINE_SRC_CORE_IO_PROMISE_H_
#define ENGINE_SRC_CORE_IO_PROMISE_H_

#include <string>
#include <mutex>
#include <functional>
#include <future>

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

  AsyncTask(EventFunc callback, uint32_t interval_in_ms) :
      callback_(callback),
      scheduled_at_(util::Time()),
      execute_at_(scheduled_at_.AddMilliseconds(interval_in_ms)),
      should_repeat_(true),
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
  void Run() {
    while (running_) {
      UniqueAsyncTask next_task;
      bool has_next = event_queue_.try_dequeue(next_task);

      if (!has_next) {
        std::this_thread::sleep_for(util::Milliseconds(5));
        continue;
      }

      AsyncStatus task_status = next_task->GetStatus();

      if (task_status == AsyncStatus::Deferred) {
        bool has_space = event_queue_.enqueue(std::move(next_task));
        ENGINE_CORE_ASSERT(has_space, "The Event loop has run out of space")
        continue;
      } else if (task_status == AsyncStatus::Expired) {
        ENGINE_CORE_TRACE("Task [{0}] has expired", next_task->GetName());
        continue;
      } else {
        AsyncResult result = next_task->Execute();
      }
    }
  }

  void Dispatch(EventFunc callback) {}

 private:
  bool running_;
  // TODO(C3NZ): Investigate into the performance of std::atomic
  // vs using a mutex.
  moodycamel::ConcurrentQueue<memory::Unique<AsyncTask>> event_queue_;
};


}  // namespace io
}  // namespace engine


#endif  // ENGINE_SRC_CORE_IO_PROMISE_H_
