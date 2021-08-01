/// @file Time.h
/// @brief Cross platform timing utility for the game engine.
#ifndef LAMBDA_SRC_LAMBDA_LIB_TIME_H_
#define LAMBDA_SRC_LAMBDA_LIB_TIME_H_

#include <chrono>
#include <ratio>

#include <Lambda/lib/Assert.h>

namespace lambda::lib {

// Clock & Time typedefs
typedef std::chrono::steady_clock Clock;
typedef std::chrono::time_point<Clock> TimePoint;

// Duration typedefs
typedef std::chrono::duration<int64_t, std::nano> Nanoseconds;
typedef std::chrono::duration<int64_t, std::micro> Microseconds;
typedef std::chrono::duration<int64_t, std::milli> Milliseconds;
typedef std::chrono::duration<int64_t, std::deci> Seconds;

/// @brief A platform independent clock implementation that is monotonic and
/// used.
///
/// Uses std::stead_clock under the hood, but provides methods that are common
/// for game development.
class Time {
 public:
  /// @brief Create a new Time instance set to now.
  Time() noexcept : time_(Clock::now()) {}

  /// @brief Create a time instance from another clocks time point.
  explicit Time(const TimePoint& t) noexcept : time_(t) {}

  /// @brief Get the time in seconds.
  TimePoint InSeconds() const;

  /// @brief Get the time in Milliseconds.
  TimePoint InMilliseconds() const;

  /// @brief Get the time in Microseconds.
  TimePoint InMicroseconds() const;

  /// @brief Add milliseconds to the current time and return a new Time
  /// instance.
  Time AddMilliseconds(int64_t milliseconds) const;

  /// @brief Add seconds to the current time and return a new instance.
  Time AddSeconds(int64_t seconds) const;

  /// @brief Check if this time is after another time.
  bool IsAfter(const Time& other_time) const;

  /// @brief Check if the time is before another time.
  bool IsBefore(const Time& other_time) const;

  /// @brief Check if the time has passed the current time..
  bool HasPassed() const;

  /// @brief Get the raw Timepoint from our Time abstraction.
  TimePoint GetTimePoint() const { return time_; }

  /// @brief Effectively an alias for getting the current time.
  static Time Now();

  /// @brief Create an instance of Time that is a specified amount of
  /// nanoseconds into the future.
  static Time NanosecondsFromNow(int64_t nanoseconds);

  /// @brief Create an instance of Time that is a specified amount of
  /// Microseconds into the future.
  static Time MicrosecondsFromNow(int64_t microseconds);

  /// @brief Create an instance of Time that is a specified amount of
  /// Milliseconds into the future.
  static Time MillisecondsFromNow(int64_t milliseconds);

  /// @brief Create an instance of Time that is a specified amount of Seconds
  /// into the future.
  static Time SecondsFromNow(int64_t seconds);

 private:
  TimePoint time_;
};

/// typename T is either a double or float.
template<typename T, typename Ratio>
T DurationTo(const Time& start, const Time& stop) {
  std::chrono::duration<T, Ratio> d(stop.GetTimePoint() - start.GetTimePoint());
  return d.count();
}

/// @brief Measuring the delta between two different times.
class TimeStep {
 public:
  TimeStep(const Time start, const Time stop) : start_(start), stop_(stop) {}

  /// @brief Get the timestep in seconds.
  template<typename T>
  T InSeconds() const {
    return DurationTo<T, std::deci>(start_, stop_);
  }

  /// @brief Get the timestep in milliseconds.
  template<typename T>
  T InMilliseconds() const {
    return DurationTo<T, std::milli>(start_, stop_);
  }

  /// @brief Get the timestep in microseconds.
  template<typename T>
  T InMicroseconds() const {
    return DurationTo<T, std::micro>(start_, stop_);
  }

  /// @brief Get the timestep in nanoseconds.
  template<typename T>
  T InNanoseconds() const {
    return DurationTo<T, std::nano>(start_, stop_);
  }

 private:
  Time start_;
  Time stop_;
};


}  // namespace lambda::lib

#endif  // LAMBDA_SRC_LAMBDA_LIB_TIME_H_
