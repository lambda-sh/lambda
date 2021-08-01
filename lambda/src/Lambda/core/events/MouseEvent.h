/// @file MouseEvent.h
/// @brief All events related mouse input.
#ifndef LAMBDA_SRC_LAMBDA_CORE_EVENTS_MOUSEEVENT_H_
#define LAMBDA_SRC_LAMBDA_CORE_EVENTS_MOUSEEVENT_H_

#include <sstream>

#include <Lambda/core/events/Event.h>

namespace lambda::core::events {

/// @brief An event generated whenever a mouse is moved within an application
/// that is running lambda.
class MouseMovedEvent : public Event {
 public:
  MouseMovedEvent(const float x, const float y) : mouse_x_(x), mouse_y_(y) {}

  [[nodiscard]] float GetX() const { return mouse_x_; }
  [[nodiscard]] float GetY() const { return mouse_y_; }

  [[nodiscard]] std::string ToString() const override {
    std::stringstream event_string;
    event_string << "MouseMovedEvent: " << GetX() << ", " << GetY();
    return event_string.str();
  }

  EVENT_CLASS_TYPE(MouseMoved)
  [[nodiscard]] int GetCategoryFlags() const override {
    return static_cast<int>(EventCategory::Mouse)
        | static_cast<int>(EventCategory::Input);
  }

 private:
  float mouse_x_, mouse_y_;
};

/// @brief An event generated whenever a mouse is scrolled within an application
/// that is running lambda.
class MouseScrolledEvent : public Event {
 public:
  MouseScrolledEvent(const float x_offset, const float y_offset)
      : x_offset_(x_offset), y_offset_(y_offset) {}

  [[nodiscard]] float GetXOffset() const { return x_offset_; }
  [[nodiscard]] float GetYOffset() const { return y_offset_; }

  [[nodiscard]] std::string ToString() const override {
    std::stringstream event_string;
    event_string
        << "MouseScrolledEvent: "
        << GetXOffset()
        << ", "
        << GetYOffset();
    return event_string.str();
  }

  EVENT_CLASS_TYPE(MouseScrolled)

  [[nodiscard]] int GetCategoryFlags() const override {
    return static_cast<int>(EventCategory::Mouse)
        |static_cast<int>(EventCategory::Input);
  }

 private:
  float x_offset_, y_offset_;
};

/// @brief The base event class for all events mouse button related.
class MouseButtonEvent : public Event {
 public:
  [[nodiscard]] int GetMouseButton() const { return button_; }

  [[nodiscard]] int GetCategoryFlags() const override {
    return static_cast<int>(
        EventCategory::Mouse) | static_cast<int>(EventCategory::Input);
  }

 protected:
  explicit MouseButtonEvent(int button) : button_(button) {}
  int button_;
};

/// @brief An event generated whenever a Mouse button is pressed within an
/// application that is running lambda.
class MouseButtonPressedEvent final : public MouseButtonEvent {
 public:
  explicit MouseButtonPressedEvent(int button) : MouseButtonEvent(button) {}

  [[nodiscard]] std::string ToString() const override {
    std::stringstream event_string;
    event_string << "MouseButtonPressedEvent: " << button_;
    return event_string.str();
  }

  EVENT_CLASS_TYPE(MouseButtonPressed)
};

/// @brief An event generated whenever a Mouse button is released within an
/// application that is running lambda.
class MouseButtonReleasedEvent final : public MouseButtonEvent {
 public:
  explicit MouseButtonReleasedEvent(const int button)
      : MouseButtonEvent(button) {}

  [[nodiscard]] std::string ToString() const override {
    std::stringstream event_string;
    event_string << "MouseButtonReleasedEvent: " << button_;
    return event_string.str();
  }

  EVENT_CLASS_TYPE(MouseButtonReleased)
};

}  // namespace lambda::core::events

#endif  // LAMBDA_SRC_LAMBDA_CORE_EVENTS_MOUSEEVENT_H_
