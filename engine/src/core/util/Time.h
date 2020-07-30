/**
 * @file Time.h
 * @brief Cross platform timing utility for the game engine.
 */
#ifndef ENGINE_SRC_CORE_UTIL_TIME_H_
#define ENGINE_SRC_CORE_UTIL_TIME_H_

#include <chrono>
#include <concepts>
#include <ratio>

#include "core/util/Assert.h"

namespace engine {
namespace util {

// Time concepts and aliases.

template<class T>
concept FloatType = std::floating_point<T>;

using Clock = std::chrono::steady_clock;
using TimePoint = std::chrono::time_point<Clock>;

// Forward declarations of both Time and DurationTo.

class Time;

template<FloatType T, typename Ratio>
const T DurationTo(const Time& start, const Time& end);

class Time {
 public:
  Time() noexcept : time_(Clock::now()) {}
  explicit Time(const TimePoint& t) : time_(t) {}

  const TimePoint InSeconds() const {
    return std::chrono::time_point_cast<std::chrono::seconds>(time_); }

  const TimePoint InMilliSeconds() const {
    return std::chrono::time_point_cast<std::chrono::milliseconds>(time_); }

  const TimePoint InMicroSeconds() const {
    return std::chrono::time_point_cast<std::chrono::microseconds>(time_); }

  const TimePoint GetTime() const { return time_; }

  Time AddMilliseconds(uint32_t milliseconds) {
    time_ += std::chrono::milliseconds(milliseconds);
    return Time(time_);
  }

  Time AddSeconds(uint32_t seconds) {
    time_ += std::chrono::seconds(seconds);
    return Time(time_);
  }

  bool IsAfter(const Time& t) {
    return DurationTo<float, std::milli>(t, *this) < 0;
  }

  bool IsBefore(const Time& t) {
    return DurationTo<float, std::milli>(t, *this) > 0;
  }

  bool HasPassed() {
    return DurationTo<float, std::milli>(Time(), *this) < 0;
  }

 private:
  TimePoint time_;
};

class TimeStep {
 public:
  TimeStep(Time start, Time stop) : start_(start), stop_(stop) {}

  template<FloatType T>
  const T InSeconds() const {
    return DurationTo<T, std::deci>(start_, stop_); }

  template<FloatType T>
  const T InMilliSeconds() const {
    return DurationTo<T, std::milli>(start_, stop_); }

  template<FloatType T>
  const T InMicroSeconds() const {
    return DurationTo<T, std::micro>(start_, stop_); }

  template<FloatType T>
  const T InNanoSeconds() const {
    return DurationTo<T, std::nano>(start_, stop_); }

 private:
  Time start_;
  Time stop_;
};

template<FloatType T, typename Ratio>
const T DurationTo(const Time& start, const Time& stop) {
  std::chrono::duration<T, Ratio> d(stop.GetTime() - start.GetTime());
  return d.count();
}

template<typename NumType, typename Ratio>
const std::chrono::duration<NumType, Ratio> ToDuration(NumType units) {
  return std::chrono::duration<NumType, Ratio>(units);
}

template<typename NumType>
const std::chrono::duration<NumType, std::milli> Nanoseconds(
    NumType nanoseconds) {
  return ToDuration<NumType, std::milli>(nanoseconds);
}

template<typename NumType>
const std::chrono::duration<NumType, std::milli> Milliseconds(
    NumType milliseconds) {
  return ToDuration<NumType, std::milli>(milliseconds);
}

template<typename NumType>
const std::chrono::duration<NumType, std::deci> Seconds(
    NumType seconds) {
  return ToDuration<NumType, std::milli>(seconds);
}

template<typename NumType>
const std::chrono::duration<NumType, std::nano> Microseconds(
    NumType microseconds) {
  return ToDuration<NumType, std::milli>(microseconds);
}

}  // namespace util
}  // namespace engine

#endif  // ENGINE_SRC_CORE_UTIL_TIME_H_

/**
 * @class engine::util::Time
 * @brief A wrapper for working with Time within the game engine. Uses
 * std::steady_clock for a platform independent Monotonic clock.
 */

/**
 * @fn engine::util::Time::InSeconds
 * @brief Get the current system time in seconds.
 */

/**
 * @fn engine::util::Time::InMilliSeconds
 * @brief Get the current system time in milliseconds.
 */

/**
 * @fn engine::util::Time::InMicroSeconds
 * @brief Get the current system time in microseconds.
 */

/**
 * @fn engine::util::Time::GetSeconds
 * @brief Get the current system time in nanoseconds.
 */

/**
 * @class engine::util::TimeStep
 * @brief The timestep between two time intervals. Primarily used for layers to
 * consistently update the engine.
 */

/**
 * @fn engine::util::TimeStep::InSeconds
 * @brief Get the interval between two Time objects in seconds.
 */

/**
 * @fn engine::util::TimeStep::InMilliSeconds
 * @brief Get the interval between two Time objects in milliseconds.
 */

/**
 * @fn engine::util::TimeStep::InMicroSeconds
 * @brief Get the interval between two Time objects in microseconds.
 */

/**
 * @fn engine::util::TimeStep::InNanoSeconds
 * @brief Get the interval between two Time objects in nanoseconds.
 */
