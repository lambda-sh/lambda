#ifndef ENGINE_SRC_CORE_EVENTS_EVENT_H_
#define ENGINE_SRC_CORE_EVENTS_EVENT_H_

#include <string>
#include <functional>

#include "core/Core.h"
#include "spdlog/fmt/ostr.h"

namespace engine {
namespace events {

enum class EventType {
  None = 0,
  kWindowClose, kWindowResize, kWindowFocus, kWindowLostFocus, kWindowMoved,
  kAppTick, kAppUpdate, kAppRender,
  kKeyPressed, kKeyReleased,
  kMouseButtonPressed, kMouseButtonReleased, kMouseMoved, kMouseScrolled
};

enum EventCategory {
  None = 0,
  kEventCategoryApplication = BIT(0),
  kEventCategoryInput = BIT(1),
  kEventCategoryKeyboard = BIT(2),
  kEventCategoryMouse = BIT(3),
  kEventCategoryMouseButton = BIT(4)
};

// Attach these functions to any given event by calling this macro with one of
// the EventType enums.
#define EVENT_CLASS_TYPE(type) \
    static EventType GetStaticType() { return EventType::type; } \
    EventType GetEventType() const override { return GetStaticType(); } \
    const char* GetName() const override { return #type; }

// Attach category flags to an event by calling this macro with an
// EventCategory.
#define EVENT_CLASS_CATEGORY(category) \
    int GetCategoryFlags() const override { return category; }

// Base class for events.
class ENGINE_API Event {
  friend class EventDispatcher;
 public:
  virtual EventType GetEventType() const = 0;
  virtual const char* GetName() const = 0;
  virtual int GetCategoryFlags() const = 0;
  virtual std::string ToString() const { return GetName(); }

  // Check if the event is in a specific category.
  inline bool IsInCategory(EventCategory category) {
    return GetCategoryFlags() & category;
  }
 protected:
  bool has_been_handled_ = false;
};

// The Event Dispatcher handles all events by using a callback that takes in the
// event.
class EventDispatcher {
  template<typename T>
  using EventFn = std::function<bool(T&)>;
 public:
  explicit EventDispatcher(Event& event) : event_(event) {}

  template<typename T>
  bool Dispatch(EventFn<T> func) {
    if (event_.GetEventType() == T::GetStaticType()) {
      event_.has_been_handled_ = func(dynamic_cast<T&>(&event_));
      return true;
    }

    return false;
  }

 private:
  Event& event_;
};

// Overrides the output stream operator so that calls to the logging library
// with an event are easily handled.
inline std::ostream& operator<<(std::ostream& os, const Event& event) {
  return os << event.ToString();
}

}  // namespace events
}  // namespace engine

#endif  // ENGINE_SRC_CORE_EVENTS_EVENT_H_
