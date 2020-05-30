/**
 * @file engine/src/core/events/KeyEvent.h
 * @brief Events that specifically deal with key input.
 *
 * These events are all platform specific and allow for the capturing of
 * user input via the keyboard.
 */
#ifndef ENGINE_SRC_CORE_EVENTS_KEYEVENT_H_
#define ENGINE_SRC_CORE_EVENTS_KEYEVENT_H_

#include <sstream>

#include "core/events/Event.h"

namespace engine {
namespace events {

/**
 * @class KeyEvent
 * @brief The base event for all other Key input events.
 *
 * Registers the Event category as both a keyboard and input event.
 * (See EventCategory for more types of event categories)
 */
class ENGINE_API KeyEvent : public Event {
 public:
  /**
   * @fn GetKeyCode
   * @brief obtain the key code that generated the user had input into the
   * application.
   *
   * This should reference engine key codes, and not any platform specific ones.
   */
  inline int GetKeyCode() const { return key_code_; }

  EVENT_CLASS_CATEGORY(kEventCategoryKeyboard | kEventCategoryInput)

 protected:
  int key_code_;

  /**
   * @fn KeyEvent
   * @brief Only classes that derive from the KeyEvent class are allowed to
   * invoke the constructor of the KeyEvent class.
   */
  explicit KeyEvent(int key_code) : key_code_(key_code) {}
};

/**
 * @class KeyPressedEvent
 * @brief Generated when a key is pressed by the user in the application.
 *
 */
class ENGINE_API KeyPressedEvent : public KeyEvent {
 public:
  /**
   * @fn KeyPressedEvent
   * @brief Generated whenever a key is pressed by the user.
   */
  KeyPressedEvent(int key_code, int repeat_count)
    : KeyEvent(key_code), repeat_count_(repeat_count) {}

  /**
   * @fn GetRepeatCount
   * @brief Gets the count of which the key code associated with this event was
   * pressed.
   */
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

/**
 * @class KeyReleasedEvent
 * @brief Generated when a key is released by the user in the application.
 */
class ENGINE_API KeyReleasedEvent : public KeyEvent {
 public:
  explicit KeyReleasedEvent(int key_code) : KeyEvent(key_code) {}

  std::string ToString() const override {
    std::stringstream event_string;
    event_string << "KeyReleasedEvent: " << key_code_;
    return event_string.str();
  }

  EVENT_CLASS_TYPE(kKeyReleased);
};

/**
 * @class KeyTypedEvent
 * @brief Generated when a key is typed by the user in the application.
 */
class ENGINE_API KeyTypedEvent : public KeyEvent {
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
}  // namespace engine

#endif  // ENGINE_SRC_CORE_EVENTS_KEYEVENT_H_
