#ifndef ENGINE_SRC_CORE_EVENTS_EVENT_H_
#define ENGINE_SRC_CORE_EVENTS_EVENT_H_

#include <functional>
#include <string>

#include <spdlog/fmt/ostr.h>

#include "core/Core.h"

namespace engine {
namespace events {

// Event types to be used when registering an event with the engine.
enum class EventType {
  None = 0,
  kWindowClose, kWindowResize, kWindowFocus, kWindowLostFocus, kWindowMoved,
  kAppTick, kAppUpdate, kAppRender,
  kKeyPressed, kKeyReleased, kKeyTyped,
  kMouseButtonPressed, kMouseButtonReleased, kMouseMoved, kMouseScrolled
};

// BIT representations of categories
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

// Binds the address of a callback function to an object instance. It is needed
// for layers to pass callback functions into the
#define BIND_EVENT_FN(fn) std::bind(&fn, this, std::placeholders::_1)

// Base class for events.
class ENGINE_API Event {
  friend class EventDispatcher;
 public:
  // Created with EVENT_CLASS_TYPE(kAppEventType) in the header of any
  // child Event class.
  virtual EventType GetEventType() const = 0;

  // Created with EVENT_CLASS_TYPE(kAppEventType) in the header of any
  // child Event class.
  virtual const char* GetName() const = 0;

  // Created with EVENT_CLASS_CATEGORY(kEventCategoryType) in the header of any
  // child Event class.
  virtual int GetCategoryFlags() const = 0;
  virtual std::string ToString() const { return GetName(); }

  // This checks the bits of the event against any given category.
  inline bool IsInCategory(EventCategory category) {
    return GetCategoryFlags() & category;
  }

  // Allow the checking of if an event has been marked as completely handled.
  // Can only be done through the callback passed into the event dispatcher.
  inline bool HasBeenHandled() { return has_been_handled_; }

 protected:
  bool has_been_handled_ = false;
  inline void SetHandled(const bool success) { has_been_handled_ = success; }
};

// The Event Dispatcher handles all events by using a callback that takes in the
// event.
class EventDispatcher {
  template<typename T>
  using EventFn = const std::function<bool(const T&)>;
 public:
  explicit EventDispatcher(Event* event) : event_(event) {}

  // Dispatch an event given an EventFn that takes in a const reference to
  // an Event and returns a bool determining if the event has been handled.
  template<typename T>
  bool Dispatch(EventFn<T> func) {
    if (event_->GetEventType() == T::GetStaticType()) {
      event_->SetHandled(func(dynamic_cast<const T&>(*event_)));
      return true;
    }
    return false;
  }

 private:
  Event* event_;
};

// Overrides the output stream operator so that calls to the logging library
// with an event are easily handled.
inline std::ostream& operator<<(std::ostream& os, const Event& event) {
  return os << event.ToString();
}

}  // namespace events
}  // namespace engine



#endif  // ENGINE_SRC_CORE_EVENTS_EVENT_H_
