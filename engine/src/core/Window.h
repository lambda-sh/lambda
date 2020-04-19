#ifndef ENGINE_SRC_CORE_WINDOW_H_
#define ENGINE_SRC_CORE_WINDOW_H_

#include <string>

#include "Core.h"
#include "events/Event.h"

namespace engine {

// Interface representing a desktop system based window.
struct WindowProperties {
  std::string Title;
  unsigned int Width;
  unsigned int Height;

  WindowProperties(
      const std::string& title = "Game Engine",
      unsigned int width = 1280,
      unsigned int height = 720)
          : Title(title), Width(width), Height(height) {}
};

class ENGINE_API Window {
 public:
  using EventCallbackFn = std::function<void(events::Event&)>;

  virtual ~Window() {}

  virtual void OnUpdate() = 0;

  virtual unsigned int GetWidth() const = 0;
  virtual unsigned int GetHeight() const = 0;

  virtual void SetEventCallback(const EventCallbackFn& callback) = 0;
  virtual void SetVSync(bool enabled) = 0;
  virtual bool IsVSync() const = 0;

  static Window* Create(
      const WindowProperties& properties = WindowProperties());
};

}  // namespace engine

#endif  // ENGINE_SRC_CORE_WINDOW_H_