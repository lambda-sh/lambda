#ifndef ENGINE_SRC_CORE_APPLICATION_H_
#define ENGINE_SRC_CORE_APPLICATION_H_

#include <memory>

#include "core/Core.h"
#include "core/Layer.h"
#include "core/LayerStack.h"
#include "core/Window.h"
#include "core/events/ApplicationEvent.h"
#include "core/events/Event.h"
#include "core/imgui/ImGuiLayer.h"

namespace engine {

// An individual platform independent application instance that manages the
// lifecycle of core components of an application being created through our
// engine.
class ENGINE_API Application {
 public:
  Application();
  virtual ~Application();

  void Run();
  void OnEvent(events::Event* event);
  void PushLayer(Layer* layer);
  void PushOverlay(Layer* layer);

  inline const Window& GetWindow() const { return *window_; }

  inline static Application& GetApplication() {return *kApplication_; }
 private:
  static Application* kApplication_;

  bool running_ = true;
  std::unique_ptr<Window> window_;
  imgui::ImGuiLayer* imgui_layer_;
  LayerStack layer_stack_;

  bool OnWindowClosed(const events::WindowCloseEvent& event);
};

// To be defined in client.
Application* CreateApplication();

}  // namespace engine

#endif  // ENGINE_SRC_CORE_APPLICATION_H_
