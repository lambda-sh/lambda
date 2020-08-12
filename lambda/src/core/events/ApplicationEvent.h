/// @file ApplicationEvent.h
/// @brief Events that are to be specifically handled at the application level.
///
/// I can't really think of a use case for this class file if you're not
/// directly working on the game engine yourself.
#ifndef LAMBDA_SRC_CORE_EVENTS_APPLICATIONEVENT_H_
#define LAMBDA_SRC_CORE_EVENTS_APPLICATIONEVENT_H_

#include <sstream>

#include "core/Core.h"
#include "core/events/Event.h"

namespace lambda {
namespace core {
namespace events {

/// @brief An Event generated when the Window has resized.
class WindowResizeEvent : public Event {
 public:
  WindowResizeEvent(unsigned int width, unsigned int height)
    : width_(width), height_(height) {}

  /// @brief Get the new width.
  const unsigned int GetWidth() const { return width_; }

  /// @brief Get the new height.
  const unsigned int GetHeight() const { return height_; }

  /// @brief Represents the resize event as a string.
  std::string ToString() const override {
    std::stringstream event_string;
    event_string << "WindowResizeEvent: " << width_ << ", " << height_;
    return event_string.str();
  }

  EVENT_CLASS_TYPE(kWindowResize)
  EVENT_CLASS_CATEGORY(kEventCategoryApplication)

 private:
  unsigned int width_, height_;
};

/// @brief An Event generated when the Window has closed.
class WindowCloseEvent: public Event {
 public:
  WindowCloseEvent() {}

  EVENT_CLASS_TYPE(kWindowClose)
  EVENT_CLASS_CATEGORY(kEventCategoryApplication)
};

/// @brief An Event generated when the app has ticked.
class AppTickEvent : public Event {
 public:
  AppTickEvent() {}

  EVENT_CLASS_TYPE(kAppTick)
  EVENT_CLASS_CATEGORY(kEventCategoryApplication)
};

/// @brief An event generated when the app has updated.
class AppUpdateEvent : public Event {
 public:
  AppUpdateEvent() {}

  EVENT_CLASS_TYPE(kAppUpdate)
  EVENT_CLASS_CATEGORY(kEventCategoryApplication)
};

/// @brief An event generated when the app has rendered.
class AppRenderEvent : public Event {
 public:
  AppRenderEvent() {}

  EVENT_CLASS_TYPE(kAppRender)
  EVENT_CLASS_CATEGORY(kEventCategoryApplication)
};

}  // namespace events
}  // namespace core
}  // namespace lambda

#endif  // LAMBDA_SRC_CORE_EVENTS_APPLICATIONEVENT_H_
