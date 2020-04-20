
#ifndef ENGINE_SRC_PLATFORM_LINUX_WINDOW_H_
#define ENGINE_SRC_PLATFORM_LINUX_WINDOW_H_

#ifdef ENGINE_PLATFORM_LINUX

#include <string>

#include <GLFW/glfw3.h>

#include "core/Window.h"

namespace engine {
namespace platform {
namespace linux {

class WindowImplementation : public engine::Window {
 public:
  explicit WindowImplementation(const engine::WindowProperties& properties);
  virtual ~WindowImplementation();

  void OnUpdate() override;

  inline unsigned int GetWidth() const override { return properties_.Width; }
  inline unsigned int GetHeight() const override { return properties_.Height; }

  inline void SetEventCallback(const EventCallbackFn& callback) override
      { properties_.EventCallback = callback; }

  void SetVSync(bool enabled) override;
  bool IsVSync() const override;

 private:
  virtual void Init(const engine::WindowProperties& properties);
  virtual void Shutdown();

  GLFWwindow* window_;

  struct Properties {
    std::string Title;
    unsigned int Width, Height;
    bool Vsync;

    EventCallbackFn EventCallback;
  };

  Properties properties_;
};

}  // namespace linux
}  // namespace platform
}  // namespace engine

#endif  // ENGINE_PLATFORM_LINUX
#endif  // ENGINE_SRC_PLATFORM_LINUX_WINDOW_H_
