#ifndef LAMBDA_SRC_LAMBDA_PLATFORM_LINUX_WINDOW_H_
#define LAMBDA_SRC_LAMBDA_PLATFORM_LINUX_WINDOW_H_

#include "core/events/ApplicationEvent.h"
#if defined LAMBDA_PLATFORM_LINUX || defined LAMBDA_DEBUG

#include <string>

#include <glad/glad.h>
#include <GLFW/glfw3.h>

#include "core/Window.h"
#include "core/memory/Pointers.h"
#include "core/renderer/GraphicsContext.h"

namespace lambda {
namespace platform {
namespace linux {

namespace internal {

struct Properties {
  std::string Title;
  unsigned int Width, Height;
  bool VerticalSync;

  core::Window::EventCallbackFunction EventCallback;
};

}  // namespace internal

class WindowImplementation : public core::Window {
 public:
  explicit WindowImplementation(const core::WindowProperties& properties);
  virtual ~WindowImplementation();

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
  /// TODO(C3NZ): Convert this into a Shared resource as opposed to just a raw
  // pointer.
  core::renderer::GraphicsContext* context_;
  internal::Properties properties_;

  virtual void Init(const core::WindowProperties& properties);
  virtual void Shutdown();
};

}  // namespace linux
}  // namespace platform
}  // namespace lambda

#endif  // LAMBDA_PLATFORM_LINUX
#endif  // LAMBDA_SRC_LAMBDA_PLATFORM_LINUX_WINDOW_H_
