/// @file Application.h
/// @brief Contains the Application class definitions.
///
/// The Application class is the primary driver of all applications being run by
/// the engine. It is designed to handle everything from events to rendering
/// without having to expose itself to applications that are being created with
/// it.
#ifndef LAMBDA_SRC_CORE_APPLICATION_H_
#define LAMBDA_SRC_CORE_APPLICATION_H_

#include <memory>

#include "core/Core.h"
#include "core/Window.h"
#include "core/events/ApplicationEvent.h"
#include "core/events/Event.h"
#include "core/imgui/ImGuiLayer.h"
#include "core/layers/Layer.h"
#include "core/layers/LayerStack.h"
#include "core/memory/Pointers.h"
#include "core/util/Time.h"

namespace lambda {
namespace core {

/// @brief The mind, body, and soul of Lambda. The Application class is the
///
/// interface into lambda that brings your application to life.
class Application {
 public:
  Application();
  virtual ~Application();

  /// @brief The primary responder to Event.
  void OnEvent(memory::Shared<events::Event> event);

  /// @brief Push a layer into the application.
  ///
  /// This and PushOverlay take ownership of the layers afterwards.
  void PushLayer(memory::Shared<layers::Layer> layer);

  /// @brief Push an overlay into the application. This gives it higher
  /// precedence over other layers and overlays in the stack.
  void PushOverlay(memory::Shared<layers::Layer> layer);

  /// @brief The main application loop. Manages the applications lifecycle,
  /// memory, updating, and pretty much anything else needed for an application
  /// to be run.
  void Run();

  /// @brief Get the window.
  const Window& GetWindow() const { return *window_.get(); }

  /// @brief Get a reference to the singleton application. (There will always
  /// be ONE application per lambda engine instance.)
  static Application& GetApplication() { return *kApplication_; }

 private:
  bool running_ = true;
  bool minimized_ = false;

  layers::LayerStack layer_stack_;
  memory::Shared<Window> window_;
  memory::Shared<imgui::ImGuiLayer> imgui_layer_;
  util::Time last_frame_time_;

  static memory::Unique<Application> kApplication_;

  // Event handlers.
  bool OnWindowResize(const events::WindowResizeEvent& event);
  bool OnWindowClosed(const events::WindowCloseEvent& event);
};

/// @brief Used for creating an instance of the game engine.
memory::Unique<Application> CreateApplication();

}  // namespace core
}  // namespace lambda

#endif  // LAMBDA_SRC_CORE_APPLICATION_H_
