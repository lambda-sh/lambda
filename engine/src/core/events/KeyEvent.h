#ifndef ENGINE_SRC_CORE_EVENTS_KEYEVENT_H_
#define ENGINE_SRC_CORE_EVENTS_KEYEVENT_H_

#include <sstream>

#include "Event.h"

namespace engine {
namespace events {

class ENGINE_API KeyEvent : public Event {
 public:
  inline int GetKeyCode() const { return key_code_; }
  EVENT_CLASS_CATEGORY(kEventCategoryKeyboard | kEventCategoryInput)

 protected:
  int key_code_;
  explicit KeyEvent(int key_code) : key_code_(key_code) {}
};

class ENGINE_API KeyPressedEvent : public KeyEvent {
 public:
  KeyPressedEvent(int key_code, int repeat_count)
    : KeyEvent(key_code), repeat_count_(repeat_count) {}

  inline int GetRepeatCount() const { return repeat_count_; }

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

class ENGINE_API KeyReleasedEvent : public KeyEvent {
 public:
  explicit KeyReleasedEvent(int key_code) : KeyEvent(key_code) {}

  std::string ToString() const override {
    std::stringstream event_string;
    event_string << "KeyReleasedEvent: " << key_code_;
    return event_string.str();
  }

  EVENT_CLASS_TYPE(kKeyReleased)
};


}  // namespace events
}  // namespace engine

#endif  // ENGINE_SRC_CORE_EVENTS_KEYEVENT_H_
