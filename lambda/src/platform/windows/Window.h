#ifndef LAMBDA_SRC_PLATFORM_WINDOWS_WINDOW_H_
#define LAMBDA_SRC_PLATFORM_WINDOWS_WINDOW_H_

#if defined LAMBDA_PLATFORM_WINDOWS || defined LAMBDA_DEBUG

#include <string>

#include <glad/glad.h>
#include <GLFW/glfw3.h>

#include "core/Window.h"
#include "core/renderer/GraphicsContext.h"

namespace lambda {
namespace platform {
namespace windows {

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

  inline unsigned int GetWidth() const override { return properties_.Width; }
  inline unsigned int GetHeight() const override { return properties_.Height; }
  void SetVerticalSync(bool enabled) override;
  bool HasVerticalSync() const override;

  inline void SetEventCallback(const EventCallbackFunction& callback) override
      { properties_.EventCallback = callback; }
  inline void* GetNativeWindow() const override { return window_; }
 private:
  GLFWwindow* window_;
  core::renderer::GraphicsContext* context_;
  internal::Properties properties_;

  virtual void Init(const core::WindowProperties& properties);
  virtual void Shutdown();
};

}  // namespace windows
}  // namespace platform
}  // namespace lambda

#endif  // LAMBDA_PLATFORM_WINDOWS
#endif  // LAMBDA_SRC_PLATFORM_WINDOWS_WINDOW_H_
