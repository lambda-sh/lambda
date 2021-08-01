#include <Lambda/lib/Time.h>

#include <chrono>

namespace lambda::lib {

// ----------------------------------- TIME ------------------------------------

TimePoint Time::InSeconds() const {
  return std::chrono::time_point_cast<std::chrono::seconds>(time_);
}

TimePoint Time::InMilliseconds() const {
  return std::chrono::time_point_cast<std::chrono::milliseconds>(time_);
}

TimePoint Time::InMicroseconds() const {
  return std::chrono::time_point_cast<std::chrono::microseconds>(time_);
}

Time Time::AddMilliseconds(const int64_t milliseconds) const {
  return Time(time_ + Milliseconds(milliseconds));
}

Time Time::AddSeconds(const int64_t seconds) const {
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

Time Time::Now() {
  return Time();
}

Time Time::NanosecondsFromNow(const int64_t nanoseconds) {
  return Time().AddMilliseconds(nanoseconds);
}

Time Time::MicrosecondsFromNow(const int64_t microseconds) {
  return Time().AddMilliseconds(microseconds);
}

Time Time::MillisecondsFromNow(const int64_t milliseconds) {
  return Time().AddMilliseconds(milliseconds);
}

Time Time::SecondsFromNow(const int64_t seconds) {
  return Time().AddSeconds(seconds);
}

}  // namespace lambda::lib
