/// @file Application.h
/// @brief Contains the Application class definitions.
///
/// The Application class is the primary driver of all applications being run by
/// the engine. It is designed to handle everything from events to rendering
/// without having to expose itself to applications that are being created with
/// it.
#ifndef LAMBDA_SRC_LAMBDA_CORE_APPLICATION_H_
#define LAMBDA_SRC_LAMBDA_CORE_APPLICATION_H_

#include <memory>

#include <Lambda/core/Window.h>
#include <Lambda/core/events/ApplicationEvent.h>
#include <Lambda/core/events/Event.h>
#include <Lambda/core/imgui/ImGuiLayer.h>
#include <Lambda/core/layers/Layer.h>
#include <Lambda/core/layers/LayerStack.h>
#include <Lambda/core/memory/Pointers.h>
#include <Lambda/lib/Time.h>

namespace lambda::core {

/// @brief The mind, body, and soul of Lambda.
///
/// The Application class is the interface into lambda that brings your
/// application to life.
class Application {
 public:
  Application();
  virtual ~Application();

  /// @brief The primary responder to Event.
  void OnEvent(memory::Unique<events::Event> event);

  /// @brief Push a layer into the application.
  ///
  /// This and PushOverlay take ownership of the layers afterwards.
  void PushLayer(memory::Unique<layers::Layer> layer);

  /// @brief Push an overlay into the application. This gives it higher
  /// precedence over other layers and overlays in the stack.
  void PushOverlay(memory::Unique<layers::Layer> layer);

  /// @brief The main application loop. Manages the applications lifecycle,
  /// memory, updating, and pretty much anything else needed for an application
  /// to be run.
  void Run();

  /// @brief Get a const pointer to the window.
  const Window* const GetWindow() const { return window_.get(); }

  /// @brief Get a reference to the singleton application. (There will always
  /// be ONE application per lambda engine instance.)
  static const Application& GetApplication() { return *kApplication_; }

 private:
  bool running_ = true;
  bool minimized_ = false;

  layers::LayerStack layer_stack_;
  memory::Unique<Window> window_;
  memory::Unique<imgui::ImGuiLayer> imgui_layer_;
  lib::Time last_frame_time_;

  static memory::Unique<Application> kApplication_;

  // Event handlers.
  bool OnWindowResize(const events::WindowResizeEvent& event);
  bool OnWindowClosed(const events::WindowCloseEvent& event);
};

/// @brief Used for creating an instance of the game engine.
memory::Unique<Application> CreateApplication();

}  // namespace lambda::core

#endif  // LAMBDA_SRC_LAMBDA_CORE_APPLICATION_H_
