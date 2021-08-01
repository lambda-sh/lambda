/// @file EventLoop.h
/// @brief  An Attempt to make an Asynchronous dispatcher to run in another
/// thread.
///
/// Hopefully, the interface included in here will enable applications consuming
/// lambda to offload i/o intensive work to another thread.
/// TODO(C3NZ): Add documentation for this file.
#ifndef LAMBDA_SRC_LAMBDA_CORE_IO_EVENTLOOP_H_
#define LAMBDA_SRC_LAMBDA_CORE_IO_EVENTLOOP_H_

#include <concurrentqueue.h>

#include <Lambda/core/io/AsyncTask.h>
#include <Lambda/lib/Time.h>

namespace lambda::core::io {

/// @brief Asynchronous Event Loop that allows the execution of code to happen
/// in another thread. This is not recommended for production as of yet.
///
/// This currently depends on everything passed into the queue being atomic
/// or protected by locks to ensure that data isn't corrupted. The easy way to
/// remedy this is to copy data into your callback as opposed to using instances
/// of it, as that has the potential for major issues.
class EventLoop {
 public:
  explicit EventLoop(const uint32_t size = 256)
      : running_(true), size_(size), event_queue_(size_) {}

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
      lib::Time execute_at = lib::Time(),
      lib::Time expire_at = lib::Time().AddSeconds(5));

 private:
  bool running_;
  uint32_t size_;
  moodycamel::ConcurrentQueue<UniqueAsyncTask> event_queue_;

  /// @brief Private dispatch that is used after a task is created.
  bool Dispatch(UniqueAsyncTask task);
};

}  // namespace lambda::core::io

#endif  // LAMBDA_SRC_LAMBDA_CORE_IO_EVENTLOOP_H_
