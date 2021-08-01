#ifndef LAMBDA_SRC_LAMBDA_PLATFORM_LINUX_WINDOW_H_
#define LAMBDA_SRC_LAMBDA_PLATFORM_LINUX_WINDOW_H_

#if defined LAMBDA_PLATFORM_LINUX || defined LAMBDA_DEBUG

#include <string>

#include <Lambda/core/Window.h>
#include <Lambda/core/renderer/GraphicsContext.h>

#include <Lambda/platform/glfw/GLFW.h>

namespace lambda::platform::linux {

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

  void SetVerticalSync(bool enabled) override;
  bool HasVerticalSync() const override;

  unsigned int GetWidth() const override { return properties_.Width; }
  unsigned int GetHeight() const override { return properties_.Height; }

  void SetEventCallback(const EventCallbackFunction& callback) override {
      properties_.EventCallback = callback; }

  void* GetNativeWindow() const override { return window_; }

 private:
  GLFWwindow* window_;
  /// todo(C3NZ): Convert this into a Shared resource as opposed to just a raw
  /// pointer.
  core::renderer::GraphicsContext* context_;
  internal::Properties properties_;

  void Init(core::WindowProperties properties);
  void Shutdown();
};

}  // namespace lambda::platform::linux

#endif  // LAMBDA_PLATFORM_LINUX
#endif  // LAMBDA_SRC_LAMBDA_PLATFORM_LINUX_WINDOW_H_
