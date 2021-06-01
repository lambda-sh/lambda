/// @file ApplicationEvent.h
/// @brief Events that are to be specifically handled at the application level.
///
/// I can't really think of a use case for this class file if you're not
/// directly working on the game engine yourself.
#ifndef LAMBDA_SRC_LAMBDA_CORE_EVENTS_APPLICATIONEVENT_H_
#define LAMBDA_SRC_LAMBDA_CORE_EVENTS_APPLICATIONEVENT_H_

#include <sstream>

#include <Lambda/core/Core.h>
#include <Lambda/core/events/Event.h>

namespace lambda::core::events {

/// @brief An Event generated when the Window has resized.
/// @todo Get rid of macros at the bottom.
class WindowResizeEvent : public Event {
 public:
  WindowResizeEvent(size_t width, size_t height)
    : width_(width), height_(height) {}

  /// @brief Get the new width.
  [[nodiscard]] size_t GetWidth() const { return width_; }

  /// @brief Get the new height.
  [[nodiscard]] size_t GetHeight() const { return height_; }

  /// @brief Represents the resize event as a string.
  [[nodiscard]] std::string ToString() const override {
    std::stringstream event_string;
    event_string << "WindowResizeEvent: " << width_ << ", " << height_;
    return event_string.str();
  }

  EVENT_CLASS_TYPE(WindowResize)
  [[nodiscard]] int GetCategoryFlags() const override {
    return static_cast<int>(EventCategory::Application);
  }

 private:
  size_t width_, height_;
};

/// @brief An Event generated when the Window has closed.
class WindowCloseEvent: public Event {
 public:
  WindowCloseEvent() = default;

  EVENT_CLASS_TYPE(WindowClose)

  [[nodiscard]] int GetCategoryFlags() const override {
    return static_cast<int>(EventCategory::Application);
  }
};

/// @brief An Event generated when the app has ticked.
class AppTickEvent : public Event {
 public:
  AppTickEvent() = default;

  EVENT_CLASS_TYPE(AppTick)
  [[nodiscard]] int GetCategoryFlags() const override {
    return static_cast<int>(EventCategory::Application);
  }
};

/// @brief An event generated when the app has updated.
class AppUpdateEvent : public Event {
 public:
  AppUpdateEvent() = default;

  EVENT_CLASS_TYPE(AppUpdate)
  [[nodiscard]] int GetCategoryFlags() const override {
    return static_cast<int>(EventCategory::Application);
  }
};

/// @brief An event generated when the app has rendered.
class AppRenderEvent : public Event {
 public:
  AppRenderEvent() = default;

  EVENT_CLASS_TYPE(AppRender)
  [[nodiscard]] int GetCategoryFlags() const override {
    return static_cast<int>(EventCategory::Application);
  }
};

}  // namespace lambda::core::events

#endif  // LAMBDA_SRC_LAMBDA_CORE_EVENTS_APPLICATIONEVENT_H_
