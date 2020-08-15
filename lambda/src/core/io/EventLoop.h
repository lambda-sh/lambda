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

/// @brief Asynchronous Event Loop that allows the execution of code to happen
/// in another thread. This is not recommended for production as of yet.
///
/// This currently depends on everything passed into the queue being atomic
/// or protected by locks to ensure that data isn't corrupted. The easy way to
/// remedy this is to copy data into your callback as opposed to using instances
/// of it, as that has the potential for major issues.
class EventLoop {
 public:
  explicit EventLoop(uint32_t size = 256)
      : running_(true), event_queue_(size) {}

  /// @brief Runs the event loop. This will block any thread it's running in and
  /// should not be used in the main thread.
  void Run();

  /// @brief Set a callback function to execute in a certain amount of
  /// milliseconds
  bool SetTimeout(AsyncCallback callback, uint32_t milliseconds);

  /// @brief Set a callback function to execute in an interval every specified
  /// amount of milliseconds.
  bool SetInterval(AsyncCallback callback, uint32_t milliseconds);

  /// @brief Dispatch a callback to run immediately. (naturally expires after 5
  /// seconds of not being run.)
  bool Dispatch(
      AsyncCallback callback,
      core::util::Time execute_at = core::util::Time(),
      core::util::Time expire_at = core::util::Time().AddSeconds(5));

 private:
  /// @brief Private dispatch that is used after a task is created.
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
