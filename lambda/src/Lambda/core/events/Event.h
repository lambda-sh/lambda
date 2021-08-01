/// @file Event.h
/// @brief The Event class and dispatcher core implementations.
///
/// The event system is a core component of the game engine that enables the
/// engine to act upon user input by propagating the user input as event
/// across layers that are attached to the engines layer stack. This enables
/// events to be passed to prioritized layers. (More specifically, overlays)
#ifndef LAMBDA_SRC_LAMBDA_CORE_EVENTS_EVENT_H_
#define LAMBDA_SRC_LAMBDA_CORE_EVENTS_EVENT_H_

#include <functional>
#include <ostream>
#include <string>

#include <Lambda/core/Core.h>
#include <Lambda/core/memory/Pointers.h>

namespace lambda::core::events {

// ----------------------- EVENT TYPES & CATEGORIES ---------------------------

/// @brief Event types natively supported by lambda.
enum class EventType {
  None = 0,
  WindowClose, WindowResize, WindowFocus, WindowLostFocus, WindowMoved,
  AppTick, AppUpdate, AppRender,
  KeyPressed, KeyReleased, KeyTyped,
  MouseButtonPressed, MouseButtonReleased, MouseMoved, MouseScrolled
};

enum class EventCategory : int {
  None = 0,
  Application = BIT(0),
  Input = BIT(1),
  Keyboard = BIT(2),
  Mouse = BIT(3),
  MouseButton = BIT(4)
};

// ------------------------------------- MACROS --------------------------------

/// @brief Utility macro to make events compatible with lambdas Dispatcher.
#define EVENT_CLASS_TYPE(type) \
    static EventType GetStaticType() { return EventType::type; } \
    EventType GetEventType() const override { return GetStaticType(); } \
    const char* GetName() const override { return #type; }

/// @brief Utility function used for binding functions to lambdas
/// Dispatcher.
#define BIND_EVENT_HANDLER(fn) std::bind(&fn, this, std::placeholders::_1)

/// @brief New Constexpr implementation for binding event listeners to
template<typename FunctionAddress, typename ClassPointer>
inline constexpr auto Bind(FunctionAddress function, ClassPointer* pointer) {
  return std::bind(function, pointer, std::placeholders::_1);
}

// ----------------------------------- CLASSES ---------------------------------

/// @brief The base Event class for events that are propagated throughout
/// lambda.
class Event {
  friend class Dispatcher;
 public:
  virtual ~Event() = default;
  [[nodiscard]] virtual EventType GetEventType() const = 0;
  [[nodiscard]] virtual const char* GetName() const = 0;
  [[nodiscard]] virtual int GetCategoryFlags() const = 0;
  [[nodiscard]] virtual std::string ToString() const { return GetName(); }

  /// @brief Checks if the Event has been handled.
  [[nodiscard]] bool HasBeenHandled() const { return has_been_handled_; }

  /// @brief Checks if the Event belongs to a specific category.
  [[nodiscard]] bool IsInCategory(EventCategory category) const {
    return GetCategoryFlags() & static_cast<int>(category);
  }

  /// @brief Support the use of << with Events.
  std::ostream& operator<<(std::ostream& os) const { return os << ToString(); }

 protected:
  bool has_been_handled_ = false;
  void SetHandled(bool success) { has_been_handled_ = success; }
};

/// @brief The primary way of allowing the application and layers in lambda
/// the capability of handling events propagated throughout the application.
class Dispatcher {
  template<typename T>
  using EventFn = const std::function<bool(const T&)>;

 public:
  /// @brief HandleWhen an event to be handled if it matches the Event
  /// associated with the handler function being passed in.
  template<class DesiredEvent>
  static bool HandleWhen(EventFn<DesiredEvent> func, Event* const event) {
    if (event->GetEventType() == DesiredEvent::GetStaticType()) {
      const DesiredEvent& casted_event = dynamic_cast<
          const DesiredEvent&>(*event);
      const bool success = func(casted_event);
      event->SetHandled(success);
      return true;
    }
    return false;
  }
};

}  // namespace lambda::core::events

#endif  // LAMBDA_SRC_LAMBDA_CORE_EVENTS_EVENT_H_
