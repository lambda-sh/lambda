#ifndef LAMBDA_SRC_LAMBDA_PLATFORM_WINDOWS_WINDOW_H_
#define LAMBDA_SRC_LAMBDA_PLATFORM_WINDOWS_WINDOW_H_

#if defined LAMBDA_PLATFORM_WINDOWS || defined LAMBDA_DEBUG

#include <string>

#include <Lambda/core/Window.h>
#include <Lambda/core/renderer/GraphicsContext.h>

#include <Lambda/platform/glfw/GLFW.h>

namespace lambda::platform::windows {

namespace internal {

struct Properties {
  std::string Title;
  unsigned int Width, Height;
  bool VerticalSync;

  core::Window::EventCallbackFunction EventCallback;
};

}  // namespace internal

class Window : public core::Window {
 public:
  explicit Window(core::WindowProperties properties);
  ~Window() override;

  void OnUpdate() override;

  unsigned int GetWidth() const override { return properties_.Width; }
  unsigned int GetHeight() const override { return properties_.Height; }
  void SetVerticalSync(bool enabled) override;
  bool HasVerticalSync() const override;

  void SetEventCallback(const EventCallbackFunction& callback) override
      { properties_.EventCallback = callback; }
  void* GetNativeWindow() const override { return window_; }
 private:
  GLFWwindow* window_;
  core::renderer::GraphicsContext* context_;
  internal::Properties properties_;

  void Init(core::WindowProperties properties);
  void Shutdown();
};

}  // namespace lambda::platform::windows

#endif  // LAMBDA_PLATFORM_WINDOWS
#endif  // LAMBDA_SRC_LAMBDA_PLATFORM_WINDOWS_WINDOW_H_
