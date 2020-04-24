#ifndef ENGINE_SRC_PLATFORM_LINUX_WINDOW_H_
#define ENGINE_SRC_PLATFORM_LINUX_WINDOW_H_

#if defined ENGINE_PLATFORM_LINUX || defined ENGINE_DEBUG

#include <string>

#include <GLFW/glfw3.h>

#include "core/Window.h"

namespace engine {
namespace platform {
namespace linux {

namespace internal {

struct Properties {
  std::string Title;
  unsigned int Width, Height;
  bool VerticalSync;

  Window::EventCallbackFn EventCallback;
};

}  // namespace internal


class WindowImplementation : public engine::Window {
 public:
  explicit WindowImplementation(const engine::WindowProperties& properties);
  virtual ~WindowImplementation();

  void OnUpdate() override;
  void SetVerticalSync(bool enabled) override;
  bool HasVerticalSync() const override;

  inline unsigned int GetWidth() const override { return properties_.Width; }
  inline unsigned int GetHeight() const override { return properties_.Height; }

  inline void SetEventCallback(const EventCallbackFn& callback) override
      { properties_.EventCallback = callback; }

  // TODO(C3NZ): Implement in the Window implementation for windows.
  inline void* GetNativeWindow() const override { return window_; }
 private:
  GLFWwindow* window_;
  internal::Properties properties_;

  virtual void Init(const engine::WindowProperties& properties);
  virtual void Shutdown();
};

}  // namespace linux
}  // namespace platform
}  // namespace engine

#endif  // ENGINE_PLATFORM_LINUX
#endif  // ENGINE_SRC_PLATFORM_LINUX_WINDOW_H_
