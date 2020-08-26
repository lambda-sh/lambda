/**
 * @file Time.h
 * @brief Cross platform timing utility for the game engine.
 */
#ifndef LAMBDA_SRC_LAMBDA_CORE_UTIL_TIME_H_
#define LAMBDA_SRC_LAMBDA_CORE_UTIL_TIME_H_

#include <chrono>
#include <concepts>
#include <ratio>

#include "Lambda/core/util/Assert.h"

namespace lambda {
namespace core {
namespace util {

template<class T>
concept FloatType = std::floating_point<T>;

// Clock & Time typedefs
typedef std::chrono::steady_clock Clock;
typedef std::chrono::time_point<Clock> TimePoint;

// Duration typedefs
typedef std::chrono::duration<int64_t, std::nano> Nanoseconds;
typedef std::chrono::duration<int64_t, std::micro> Microseconds;
typedef std::chrono::duration<int64_t, std::milli> Milliseconds;
typedef std::chrono::duration<int64_t, std::deci> Seconds;

// Forward declarations of both Time and DurationTo.

class Time;

template<FloatType T, typename Ratio>
const T DurationTo(const Time& start, const Time& end);

/// @brief A platform independent clock implementation that is monotonic and
/// used.
///
/// Uses std::stead_clock under the hood, but provides methods that are common
/// for game development.
class Time {
 public:
  /// @brief Create a new Time instance set to now.
  Time() noexcept : time_(Clock::now()) {}

  /// @brief Create a new Time instance as a copy from another time instance.
  Time(Time& t) noexcept : time_(t.GetTime()) {}

  /// @brief Create a time instance from another clocks time point.
  explicit Time(const TimePoint& t) noexcept : time_(t) {}

  /// @brief Get the time in seconds.
  const TimePoint InSeconds() const {
    return std::chrono::time_point_cast<std::chrono::seconds>(time_); }

  /// @brief Get the time in Milliseconds.
  const TimePoint InMilliSeconds() const {
    return std::chrono::time_point_cast<std::chrono::milliseconds>(time_); }

  /// @brief Get the time in Microseconds.
  const TimePoint InMicroSeconds() const {
    return std::chrono::time_point_cast<std::chrono::microseconds>(time_); }

  /// @brief Get the raw Timepoint of the wrapper.
  const TimePoint GetTime() const { return time_; }

  /// @brief Add milliseconds to the current time and return a new Time
  /// instance.
  Time AddMilliseconds(int64_t milliseconds) {
    return Time(time_ + Milliseconds(milliseconds));
  }

  /// @brief Add seconds to the current time and return a new instance.
  Time AddSeconds(int64_t seconds) {
    return Time(time_ + Seconds(seconds));
  }

  /// @brief Check if this time is after another time.
  bool IsAfter(const Time& t) {
    return DurationTo<float, std::milli>(t, *this) < 0;
  }

  /// @brief Check if the time is before another time.
  bool IsBefore(const Time& t) {
    return DurationTo<float, std::milli>(t, *this) > 0;
  }

  /// @brief Check if the time has passed the current time..
  bool HasPassed() {
    return DurationTo<float, std::milli>(Time(), *this) < 0;
  }

  /// @brief Effectively an alias for getting the current time.
  static Time Now() { return Time(); }

  /// @brief Create an instance of Time that is a specified amount of
  /// nanoseconds into the future.
  static Time NanosecondsFromNow(int64_t nanoseconds) {
    return Time().AddMilliseconds(nanoseconds);
  }

  /// @brief Create an instance of Time that is a specified amount of
  /// Microseconds into the future.
  static Time MicrosecondsFromNow(int64_t microseconds) {
    return Time().AddMilliseconds(microseconds);
  }

  /// @brief Create an instance of Time that is a specified amount of
  /// Milliseconds into the future.
  static Time MillisecondsFromNow(int64_t milliseconds) {
    return Time().AddMilliseconds(milliseconds);
  }

  /// @brief Create an instance of Time that is a specified amount of Seconds
  /// into the future.
  static Time SecondsFromNow(int64_t seconds) {
    return Time().AddSeconds(seconds);
  }

 private:
  TimePoint time_;
};

/// @brief Measuring the delta between two different times.
class TimeStep {
 public:
  TimeStep(Time start, Time stop) : start_(start), stop_(stop) {}

  /// @brief Get the timestep in seconds.
  template<FloatType T>
  const T InSeconds() const {
    return DurationTo<T, std::deci>(start_, stop_); }

  /// @brief Get the timestep in milliseconds.
  template<FloatType T>
  const T InMilliSeconds() const {
    return DurationTo<T, std::milli>(start_, stop_); }

  /// @brief Get the timestep in microseconds.
  template<FloatType T>
  const T InMicroSeconds() const {
    return DurationTo<T, std::micro>(start_, stop_); }

  /// @brief Get the timestep in nanoseconds.
  template<FloatType T>
  const T InNanoSeconds() const {
    return DurationTo<T, std::nano>(start_, stop_); }

 private:
  Time start_;
  Time stop_;
};

/// @brief Convert two Time instances into a duration of float type T.
///
/// FloatType T is either a double or float.
template<FloatType T, typename Ratio>
const T DurationTo(const Time& start, const Time& stop) {
  std::chrono::duration<T, Ratio> d(stop.GetTime() - start.GetTime());
  return d.count();
}

}  // namespace util
}  // namespace core
}  // namespace lambda

#endif  // LAMBDA_SRC_LAMBDA_CORE_UTIL_TIME_H_
