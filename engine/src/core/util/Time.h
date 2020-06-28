#ifndef ENGINE_SRC_CORE_UTIL_TIME_H_
#define ENGINE_SRC_CORE_UTIL_TIME_H_

#include <concepts>
#include <chrono>
#include <ratio>
#include <string>

namespace engine {
namespace util {

template<class T>
concept FloatType = std::floating_point<T>;

using Clock = std::chrono::steady_clock;
using TimePoint = std::chrono::time_point<Clock>;

/**
 * @class Time
 * @brief A wrapper for working with Time within the game engine. Uses
 * std::steady_clock for a platform independent Monotonic clock.
 */
class Time {
 public:
  Time() noexcept : time_(Clock::now()) {}

  const TimePoint InSeconds() const {
    return std::chrono::time_point_cast<std::chrono::seconds>(time_); }

  const TimePoint InMilliSeconds() const {
    return std::chrono::time_point_cast<std::chrono::milliseconds>(time_); }

  const TimePoint InMicroSeconds() const {
    return std::chrono::time_point_cast<std::chrono::microseconds>(time_); }

  const TimePoint GetTime() const { return time_; }

 private:
  TimePoint time_;
};

/**
 * @class TimeStep
 * @brief The timestep between two time intervals. Primarily used for layers to
 * consistently update the engine.
 */
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
const T DurationTo(Time start, Time stop) {
  std::chrono::duration<T, Ratio> d(stop.GetTime() - start.GetTime());
  return d.count();
}

}  // namespace util
}  // namespace engine

#endif  // ENGINE_SRC_CORE_UTIL_TIME_H_
