#ifndef ENGINE_SRC_PLATFORM_WINDOWS_WINDOW_H_
#define ENGINE_SRC_PLATFORM_WINDOWS_WINDOW_H_

#if defined ENGINE_PLATFORM_WINDOWS || defined ENGINE_DEBUG

#include <string>

#include <glad/glad.h>
#include <GLFW/glfw3.h>

#include "core/Window.h"
#include "core/renderer/GraphicsContext.h"

namespace engine {
namespace platform {
namespace windows {

namespace internal {

struct Properties {
  std::string Title;
  unsigned int Width, Height;
  bool VerticalSync;

  Window::EventCallbackFunction EventCallback;
};

}  // namespace internal

class WindowImplementation : public engine::Window {
 public:
  explicit WindowImplementation(const engine::WindowProperties& properties);
  virtual ~WindowImplementation();

  void OnUpdate() override;

  inline unsigned int GetWidth() const override { return properties_.Width; }
  inline unsigned int GetHeight() const override { return properties_.Height; }
  void SetVerticalSync(bool enabled) override;
  bool HasVerticalSync() const override;

  inline void SetEventCallback(const EventCallbackFunction& callback) override
      { properties_.EventCallback = callback; }
  inline void* GetNativeWindow() const override { return window_; }
 private:
  GLFWwindow* window_;
  renderer::GraphicsContext* context_;
  internal::Properties properties_;

  virtual void Init(const engine::WindowProperties& properties);
  virtual void Shutdown();
};

}  // namespace windows
}  // namespace platform
}  // namespace engine

#endif  // ENGINE_PLATFORM_WINDOWS
#endif  // ENGINE_SRC_PLATFORM_WINDOWS_WINDOW_H_
