/// @file MouseEvent.h
/// @brief All events related mouse input.
#ifndef LAMBDA_SRC_LAMBDA_CORE_EVENTS_MOUSEEVENT_H_
#define LAMBDA_SRC_LAMBDA_CORE_EVENTS_MOUSEEVENT_H_

#include <sstream>

#include "core/events/Event.h"

namespace lambda {
namespace core {
namespace events {

/// @brief An event generated whenever a mouse is moved within an application
/// that is running lambda.
class MouseMovedEvent : public Event {
 public:
  MouseMovedEvent(float x, float y) : mouse_x_(x), mouse_y_(y) {}

  float GetX() const { return mouse_x_; }
  float GetY() const { return mouse_y_; }

  std::string ToString() const override {
    std::stringstream event_string;
    event_string << "MouseMovedEvent: " << GetX() << ", " << GetY();
    return event_string.str();
  }

  EVENT_CLASS_TYPE(kMouseMoved)
  EVENT_CLASS_CATEGORY(kEventCategoryMouse | kEventCategoryInput)
 private:
  float mouse_x_, mouse_y_;
};

/// @brief An event generated whenever a mouse is scrolled within an application
/// that is running lambda.
class MouseScrolledEvent : public Event {
 public:
  MouseScrolledEvent(float x_offset, float y_offset)
      : x_offset_(x_offset), y_offset_(y_offset) {}

  float GetXOffset() const { return x_offset_; }
  float GetYOffset() const { return y_offset_; }

  std::string ToString() const override {
    std::stringstream event_string;
    event_string
        << "MouseScrolledEvent: "
        << GetXOffset()
        << ", "
        << GetYOffset();
    return event_string.str();
  }

  EVENT_CLASS_TYPE(kMouseScrolled)
  EVENT_CLASS_CATEGORY(kEventCategoryMouse | kEventCategoryInput)
 private:
  float x_offset_, y_offset_;
};

/// @brief The base event class for all events mouse button related.
class MouseButtonEvent : public Event {
 public:
  int GetMouseButton() const { return button_; }

  EVENT_CLASS_CATEGORY(kEventCategoryMouse | kEventCategoryInput)

 protected:
  explicit MouseButtonEvent(int button) : button_(button) {}
  int button_;
};

/// @brief An event generated whenever a Mouse button is pressed within an
/// application that is running lambda.
class MouseButtonPressedEvent : public MouseButtonEvent {
 public:
  explicit MouseButtonPressedEvent(int button) : MouseButtonEvent(button) {}

  std::string ToString() const override {
    std::stringstream event_string;
    event_string << "MouseButtonPressedEvent: " << button_;
    return event_string.str();
  }

  EVENT_CLASS_TYPE(kMouseButtonPressed)
};

/// @brief An event generated whenever a Mouse button is released within an
/// application that is running lambda.
class MouseButtonReleasedEvent : public MouseButtonEvent {
 public:
  explicit MouseButtonReleasedEvent(int button) : MouseButtonEvent(button) {}

  std::string ToString() const override {
    std::stringstream event_string;
    event_string << "MouseButtonReleasedEvent: " << button_;
    return event_string.str();
  }

  EVENT_CLASS_TYPE(kMouseButtonReleased)
};

}  // namespace events
}  // namespace core
}  // namespace lambda

#endif  // LAMBDA_SRC_LAMBDA_CORE_EVENTS_MOUSEEVENT_H_
