/// @file AsyncTask.h
/// @brief A convenient wrapper for callback functions being dispatched into the
/// event loop
#ifndef LAMBDA_SRC_LAMBDA_CORE_IO_ASYNCTASK_H_
#define LAMBDA_SRC_LAMBDA_CORE_IO_ASYNCTASK_H_

#include <functional>

#include <Lambda/core/memory/Pointers.h>
#include <Lambda/lib/Time.h>

namespace lambda::core::io {

class AsyncTask;

typedef std::function<bool()> AsyncCallback;
typedef memory::Unique<AsyncTask> UniqueAsyncTask;

/// @brief The execution status of the async function.
///
/// Must be ready in order to execute.
enum class AsyncStatus {
  None = 0,
  Deferred,
  Ready,
  Expired
};

/// @brief The result of calling the async function.
enum class AsyncResult {
  None = 0,
  Failure,
  Success
};

/// @brief A wrapper for callbacks that are supposed to be executed
/// asynchronously.
///
/// There is no need to use this class externally, as the EventLoop will create
/// these upon passing it a AsyncCallback.
class AsyncTask {
 public:
  /// @brief Construct a task that should execute as soon as possible.
  AsyncTask(
      AsyncCallback callback,
      const lib::Time execute_at = lib::Time(),
      const lib::Time expires_at = lib::Time().AddSeconds(5)) :
          callback_(std::move(callback)),
          scheduled_at_(lib::Time()),
          execute_at_(execute_at),
          expires_at_(expires_at) {}

  /// @brief Executes the AsyncCallback and returns back the result.
  [[nodiscard]] AsyncResult Execute() const;

  /// @brief Gets the execution status of the callback.
  [[nodiscard]] AsyncStatus GetStatus() const;

  /// @brief Allows a task to be rescheduled with new times.
  void RescheduleTask(
      lib::Time new_execution_time,
      lib::Time new_expiration_time);

  /// @brief Get the name of the task (Currently not implemented.)
  /// TODO(C3NZ): There should be overloads in the EventLoop that allow callback
  // functions to easily be named.
  [[nodiscard]] const std::string& GetName() const { return name_; }

  /// @brief Allow the ability to see if the task is setup to repeat.
  ///
  /// It is by default set to false.
  [[nodiscard]] bool ShouldRepeat() const { return should_repeat_; }

 private:
  std::string name_;
  AsyncCallback callback_;
  bool should_repeat_ = false;
  lib::Time scheduled_at_, execute_at_, executed_at_, expires_at_;
};

}  // namespace lambda::core::io

#endif  // LAMBDA_SRC_LAMBDA_CORE_IO_ASYNCTASK_H_
