/**
 * @file ApplicationEvent.h
 * @brief Events that are to be specifically handled at the application level.
 *
 * I can't really think of a use case for this class file if you're not directly
 * working on the game engine yourself.
 */
#ifndef ENGINE_SRC_CORE_EVENTS_APPLICATIONEVENT_H_
#define ENGINE_SRC_CORE_EVENTS_APPLICATIONEVENT_H_

#include <sstream>

#include "core/Core.h"
#include "core/events/Event.h"

namespace engine {
namespace events {

class WindowResizeEvent : public Event {
 public:
  WindowResizeEvent(unsigned int width, unsigned int height)
    : width_(width), height_(height) {}

  inline const unsigned int GetWidth() const { return width_; }
  inline const unsigned int GetHeight() const { return height_; }

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

class WindowCloseEvent: public Event {
 public:
  WindowCloseEvent() {}

  EVENT_CLASS_TYPE(kWindowClose)
  EVENT_CLASS_CATEGORY(kEventCategoryApplication)
};

class AppTickEvent : public Event {
 public:
  AppTickEvent() {}

  EVENT_CLASS_TYPE(kAppTick)
  EVENT_CLASS_CATEGORY(kEventCategoryApplication)
};

class AppUpdateEvent : public Event {
 public:
  AppUpdateEvent() {}

  EVENT_CLASS_TYPE(kAppUpdate)
  EVENT_CLASS_CATEGORY(kEventCategoryApplication)
};

class AppRenderEvent : public Event {
 public:
  AppRenderEvent() {}

  EVENT_CLASS_TYPE(kAppRender)
  EVENT_CLASS_CATEGORY(kEventCategoryApplication)
};

}  // namespace events
}  // namespace engine

#endif  // ENGINE_SRC_CORE_EVENTS_APPLICATIONEVENT_H_

/**
 * @class engine::events::WindowResizeEvent
 * @brief Generated whenever a window is resized.
 *
 * Platform independent.
 */

/**
 * @fn engine::events::WindowResizeEvent::GetWidth
 * @brief The new width that was registered with the event.
 */

/**
 * @fn engine::events::WindowResizeEvent::GetHeight
 * @brief The new height that was registered with the event.
 */

/**
 * @class engine::events::WindowCloseEvent
 * @brief Generated whenever a window is closed.
 */

/**
 * @class engine::events::AppTickEvent
 * @brief Generated whenever the app ticks.
 *
 * Currently not implemented.
 */

/**
 * @class engine::events::AppUpdateEvent
 * @brief Generated whenever the app updates.
 *
 * Currently not implemented.
 */

/**
 * @class engine::events::AppRenderEvent
 * @brief Generated whenever the app renders.
 *
 * Currently not implemented.
 */
