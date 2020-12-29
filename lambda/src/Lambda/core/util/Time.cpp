#include "Lambda/core/util/Time.h"

#include <chrono>

namespace lambda {
namespace core {
namespace util {

/// FloatType T is either a double or float.
template<FloatType T, typename Ratio>
const T DurationTo(const Time& start, const Time& stop) {
  std::chrono::duration<T, Ratio> d(stop.GetTimePoint() - start.GetTimePoint());
  return d.count();
}

// ----------------------------------- TIME ------------------------------------

const TimePoint Time::InSeconds() const {
  return std::chrono::time_point_cast<std::chrono::seconds>(time_);
}

const TimePoint Time::InMilliSeconds() const {
  return std::chrono::time_point_cast<std::chrono::milliseconds>(time_);
}

const TimePoint Time::InMicroSeconds() const {
  return std::chrono::time_point_cast<std::chrono::microseconds>(time_);
}

Time Time::AddMilliseconds(int64_t milliseconds) const {
  return Time(time_ + Milliseconds(milliseconds));
}

Time Time::AddSeconds(int64_t seconds) const {
  return Time(time_ + Seconds(seconds));
}

bool Time::IsAfter(const Time& other_time) const {
  return DurationTo<float, std::milli>(other_time, *this) < 0;
}

bool Time::IsBefore(const Time& other_time) const {
  return DurationTo<float, std::milli>(other_time, *this) > 0;
}

bool Time::HasPassed() const {
  return DurationTo<float, std::milli>(Time(), *this) < 0;
}

/// @brief Effectively an alias for getting the current time.
Time Time::Now() { return Time(); }

/// @brief Create an instance of Time that is a specified amount of
/// nanoseconds into the future.
Time Time::NanosecondsFromNow(int64_t nanoseconds) {
  return Time().AddMilliseconds(nanoseconds);
}

/// @brief Create an instance of Time that is a specified amount of
/// Microseconds into the future.
Time Time::MicrosecondsFromNow(int64_t microseconds) {
  return Time().AddMilliseconds(microseconds);
}

/// @brief Create an instance of Time that is a specified amount of
/// Milliseconds into the future.
Time Time::MillisecondsFromNow(int64_t milliseconds) {
  return Time().AddMilliseconds(milliseconds);
}

/// @brief Create an instance of Time that is a specified amount of Seconds
/// into the future.
Time Time::SecondsFromNow(int64_t seconds) {
  return Time().AddSeconds(seconds);
}

}  // namespace util
}  // namespace core
}  // namespace lambda
