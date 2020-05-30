/**
 * @file engine/src/core/events/Event.h
 * @brief The Event class and dispatcher core implementations.
 *
 * The event system is a core component of the game engine that enables the
 * engine to act upon user input by propagating the user input as event
 * across layers that are attached to the engines layer stack. This enables
 * events to be passed to prioritized layers. (More specifically, overlays)
 */
#ifndef ENGINE_SRC_CORE_EVENTS_EVENT_H_
#define ENGINE_SRC_CORE_EVENTS_EVENT_H_

#include <functional>
#include <ostream>
#include <string>

#include <spdlog/fmt/ostr.h>

#include "core/Core.h"

namespace engine {
namespace events {

// ----------------------- EVENT TYPES & CATEGORIES ---------------------------

/**
 * @enum EventType
 * @brief An Events specific type.
 *
 * This defines all the EventTypes that are available within the engine. Events
 * that are not registered here cannot be used for instantiating classes that
 * want to extend the base class Event and use the event dispatcher.
 */
enum class EventType {
  None = 0,
  kWindowClose, kWindowResize, kWindowFocus, kWindowLostFocus, kWindowMoved,
  kAppTick, kAppUpdate, kAppRender,
  kKeyPressed, kKeyReleased, kKeyTyped,
  kMouseButtonPressed, kMouseButtonReleased, kMouseMoved, kMouseScrolled
};

/**
 * @enum EventCategory
 * @brief An events specific category.
 *
 * This is primarily used for determining what categories an event belongs to.
 * EventCategory is set in any children class of Event using the macro
 * `EVENT_CLASS_CATEGORY(EventCategory::...)` like so.
 */
enum EventCategory {
  None = 0,
  kEventCategoryApplication = BIT(0),
  kEventCategoryInput = BIT(1),
  kEventCategoryKeyboard = BIT(2),
  kEventCategoryMouse = BIT(3),
  kEventCategoryMouseButton = BIT(4)
};

// ------------------------------------- MACROS --------------------------------

/**
 * @def EVENT_CLASS_TYPE(type)
 * @param type the child classes EventType.
 * @brief Helper macro to fill out child event classes.
 *
 * All children of the base Class Event are to implement this in their class
 * definition in order to be compatible with the EventDispatcher.
 */
#define EVENT_CLASS_TYPE(type) \
    static EventType GetStaticType() { return EventType::type; } \
    EventType GetEventType() const override { return GetStaticType(); } \
    const char* GetName() const override { return #type; }

/**
 * @def EVENT_CLASS_CATEGORY(category)
 * @param category The child classes EventCategory.
 * @brief Helper macro to fill out child event classes.
 *
 * All children of the base Class Event are to implement this in their class
 * definition in order to be compatible with the EventDispatcher
 */
#define EVENT_CLASS_CATEGORY(category) \
    int GetCategoryFlags() const override { return category; }

/**
 * @def BIND_EVENT_FN(fn)
 * @param fn The function being used to handle an event.
 * @brief Helper macro to bind event handlers to their parent classes.
 *
 * This is used to bind event handlers inside of classes to the callbacks
 * that they'd like to pass to the EventDispatcher.
 */
#define BIND_EVENT_FN(fn) std::bind(&fn, this, std::placeholders::_1)

// ----------------------------------- CLASSES ---------------------------------

/**
 * @class Event
 * @brief The abstract Event class.
 *
 * The base Event implementation that is the parent class for any Event that
 * would like to be passed into and handled by the EventDispatcher system. All
 * Children class must override the functions provided by this class in order
 * to be able to be propagated through the EventDispatcher. There are macros
 * provided in `core/events/Event.h` that are documented and make the
 * process of creating child classes of Event magnitudes easier.
 */
class ENGINE_API Event {
  friend class EventDispatcher;
 public:
  virtual EventType GetEventType() const = 0;
  virtual const char* GetName() const = 0;
  virtual int GetCategoryFlags() const = 0;
  virtual std::string ToString() const { return GetName(); }

  /**
   * @fn IsInCategory
   * @param category The category to be checked against.
   * @brief Check if an event belongs to an EventCategory
   */
  inline bool IsInCategory(EventCategory category) {
    return GetCategoryFlags() & category;
  }

  /**
   * @fn HasBeenHandled
   * @brief Check if an event has already been handled.
   *
   * Only the EventDispatcher is capable of setting an event as being handled.
   */
  inline bool HasBeenHandled() { return has_been_handled_; }

  /**
   * @brief Allows events to be overloaded into
   */
  inline std::ostream& operator<<(std::ostream& os) {
    return os << ToString();
  }

 protected:
  bool has_been_handled_ = false;
  inline void SetHandled(const bool success) { has_been_handled_ = success; }
};

/**
 * @class EventDispatcher
 * @brief The event handling system.
 *
 * The EventDispatcher is the key to handling all events. It is created per
 * event and streamlines the process of dispatching events with callbacks that
 * are invoked when the EventType of the event used to create the
 * EventDispatcher matches the EventType that the callback is looking for.
 */
class EventDispatcher {
  /**
   * @typedef EventFn
   * @brief The expected function header to be used for dispatching events.
   */
  template<typename T>
  using EventFn = const std::function<bool(const T&)>;

 public:
  explicit EventDispatcher(Event* event) : event_(event) {}

  /**
   * @fn Dispatch
   * @param func A function of type EventFn<T> that will be used to dispatch an
   * event if they have they have the same corresponding type.
   *
   * @brief Ensures that functions are disptached by event handlers that match
   * the type of event that is associated with the dispatcher.
   *
   * Functions passed into the dispatcher most likely need to be bound using
   * ```BIND_EVENT_FN(fn)``` when being passed to the Event Dispatcher. This is
   * to ensure that `this` for any class method is bound to the class that is
   * using the EventDispatcher.
   */
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

}  // namespace events
}  // namespace engine

#endif  // ENGINE_SRC_CORE_EVENTS_EVENT_H_
