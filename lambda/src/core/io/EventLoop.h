// TODO(C3NZ): Add documentation for this file.
#ifndef LAMBDA_SRC_CORE_IO_EVENTLOOP_H_
#define LAMBDA_SRC_CORE_IO_EVENTLOOP_H_

#include <concurrentqueue.h>

#include "core/io/AsyncTask.h"
#include "core/memory/Pointers.h"
#include "core/util/Time.h"

namespace lambda {
namespace core {
namespace io {

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

  void Run();

  // Functions to interact with the event loop from another thread.
  bool SetTimeout(AsyncCallback callback, uint32_t milliseconds);
  bool SetInterval(AsyncCallback callback, uint32_t milliseconds);
  bool Dispatch(
      AsyncCallback callback,
      util::Time execute_at = util::Time(),
      util::Time expire_at = util::Time().AddSeconds(5));

 private:
  // Private dispatch that is used after a task is created.
  bool Dispatch(UniqueAsyncTask task);

  bool running_;
  // TODO(C3NZ): Investigate into the performance of std::atomic
  // vs using a mutex.
  moodycamel::ConcurrentQueue<UniqueAsyncTask> event_queue_;
};

}  // namespace io
}  // namespace core
}  // namespace lambda

#endif  // LAMBDA_SRC_CORE_IO_EVENTLOOP_H_
