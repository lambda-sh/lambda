/**
 * @file Application.h
 * @brief Contains the Application class definitions.
 *
 * The Application class is the primary driver of all applications being run by
 * the engine. It is designed to handle everything from events to rendering
 * without having to expose itself to applications that are being created with
 * it.
 */
#ifndef LAMBDA_SRC_CORE_APPLICATION_H_
#define LAMBDA_SRC_CORE_APPLICATION_H_

#include <memory>

#include "core/Core.h"
#include "core/layers/Layer.h"
#include "core/layers/LayerStack.h"
#include "core/Window.h"
#include "core/events/ApplicationEvent.h"
#include "core/events/Event.h"
#include "core/imgui/ImGuiLayer.h"
#include "core/memory/Pointers.h"
#include "core/util/Time.h"

namespace lambda {
namespace core {


class Application {
 public:
  Application();
  virtual ~Application();

  void OnEvent(memory::Shared<events::Event> event);
  void PushLayer(memory::Shared<layers::Layer> layer);
  void PushOverlay(memory::Shared<layers::Layer> layer);
  void Run();

  const Window& GetWindow() const { return *window_; }
  static Application& GetApplication() {return *kApplication_; }

 private:
  bool running_ = true;
  bool minimized_ = false;

  layers::LayerStack layer_stack_;
  memory::Shared<Window> window_;
  memory::Shared<imgui::ImGuiLayer> imgui_layer_;
  util::Time last_frame_time_;

  static memory::Unique<Application> kApplication_;

  bool OnWindowResize(const events::WindowResizeEvent& event);
  bool OnWindowClosed(const events::WindowCloseEvent& event);
};

Application* CreateApplication();

}  // namespace core
}  // namespace lambda

#endif  // LAMBDA_SRC_CORE_APPLICATION_H_

/**
 * @class lambda::Application
 * @brief The primary driver of all applications extending this engine.
 *
 * The engine implements the application runner as an individual platform
 * independent application instance that manages the lifecycle of the core and
 * lower level components of the engine.
 */

/**
 * @fn lambda::Application::Run
 * @brief Controls the applications lifecycle and all lower level
 * functionality like input, events, rendering, networking, etc.
 */

/**
 * @fn lambda::Application::OnEvent
 * @brief Passes events to all the layers.
 * @param event An event pointer generated to be handled by the application.
 */

/**
 * @fn lambda::Application::PushLayer
 * @brief Attaches a layer to the application instance.
 * @param layer
 *
 * This allows the application instance to propage events, rendering, and any
 * desired pieces of data into the layer.
 */

/**
 * @fn lambda::Application::PushOverlay
 * @brief Attaches an overlay to the application instance.
 *
 * This allows the application instance to propage events, rendering,
 * and any desired pieces of data into the layer.
 */

/**
 * @fn lambda::Application::OnWindowClosed
 * @brief Handles what to do when a window has been closed.
 */

/**
 * @fn lambda::CreateApplication
 * @brief An external function that is to be defined inside of the client.
 *
 * It allows the game developers to simply write a CreateApplication() method
 * that initializes their game specific code inside of the engine.
 */
