/// @file KeyEvent.h
/// @brief Events that specifically deal with key input.
///
/// These events are all platform independent and allow for the capturing of
/// user input via the keyboard.
#ifndef LAMBDA_SRC_LAMBDA_CORE_EVENTS_KEYEVENT_H_
#define LAMBDA_SRC_LAMBDA_CORE_EVENTS_KEYEVENT_H_

#include <sstream>

#include "core/events/Event.h"

namespace lambda {
namespace core {
namespace events {

/// @brief The Base Key Event used for other Key Events.
class KeyEvent : public Event {
 public:
  int GetKeyCode() const { return key_code_; }

  EVENT_CLASS_CATEGORY(kEventCategoryKeyboard | kEventCategoryInput)

 protected:
  int key_code_;

  explicit KeyEvent(int key_code) : key_code_(key_code) {}
};

/// @brief An Event generated whenever a key is pressed within an application
/// that is running lambda.
class KeyPressedEvent : public KeyEvent {
 public:
  KeyPressedEvent(int key_code, int repeat_count)
    : KeyEvent(key_code), repeat_count_(repeat_count) {}

  int GetRepeatCount() const { return repeat_count_; }

  std::string ToString() const override {
    std::stringstream event_string;
    event_string
        << "KeyPressedEvent: "
        << key_code_
        << "("
        << repeat_count_
        << "repeats)";
    return event_string.str();
  }

  EVENT_CLASS_TYPE(kKeyPressed);

 private:
  int repeat_count_;
};

/// @brief An Event generated whenever a key is released within an application
/// that is running lambda.
class KeyReleasedEvent : public KeyEvent {
 public:
  explicit KeyReleasedEvent(int key_code) : KeyEvent(key_code) {}

  std::string ToString() const override {
    std::stringstream event_string;
    event_string << "KeyReleasedEvent: " << key_code_;
    return event_string.str();
  }

  EVENT_CLASS_TYPE(kKeyReleased);
};

/// @brief An Event generated whenever a key is typed within an application that
/// is running lambda. (Keys typed do not track any repeat counts.)
class KeyTypedEvent : public KeyEvent {
 public:
  explicit KeyTypedEvent(int key_code) : KeyEvent(key_code) {}

  std::string ToString() const override {
    std::stringstream event_string;
    event_string << "KeyTypedEvent: " << key_code_;
    return event_string.str();
  }

  EVENT_CLASS_TYPE(kKeyTyped);
};

}  // namespace events
}  // namespace core
}  // namespace lambda

#endif  // LAMBDA_SRC_LAMBDA_CORE_EVENTS_KEYEVENT_H_
