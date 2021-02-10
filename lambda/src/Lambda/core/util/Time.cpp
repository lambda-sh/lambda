#include "Lambda/core/util/Time.h"

#include <chrono>

namespace lambda {
namespace core {
namespace util {


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

Time Time::Now() { return Time(); }

Time Time::NanosecondsFromNow(int64_t nanoseconds) {
  return Time().AddMilliseconds(nanoseconds);
}

Time Time::MicrosecondsFromNow(int64_t microseconds) {
  return Time().AddMilliseconds(microseconds);
}

Time Time::MillisecondsFromNow(int64_t milliseconds) {
  return Time().AddMilliseconds(milliseconds);
}

Time Time::SecondsFromNow(int64_t seconds) {
  return Time().AddSeconds(seconds);
}

}  // namespace util
}  // namespace core
}  // namespace lambda
