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
#include "core/memory/Pointers.h"
#include "core/util/Time.h"

namespace engine {

class ENGINE_API Application {
 public:
  Application();
  virtual ~Application();

  void OnEvent(events::Event* event);
  void PushLayer(memory::Shared<Layer> layer);
  void PushOverlay(memory::Shared<Layer> layer);
  void Run();

  inline const Window& GetWindow() const { return *window_; }
  inline static Application& GetApplication() {return *kApplication_; }

 private:
  LayerStack layer_stack_;
  bool running_ = true;
  memory::Shared<Window> window_;
  memory::Shared<imgui::ImGuiLayer> imgui_layer_;
  static memory::Unique<Application> kApplication_;
  util::Time last_frame_time_;

  bool OnWindowClosed(const events::WindowCloseEvent& event);
};

/**
 * This is an external function that is to be defined inside of the client. It
 * allows the game developers to simply write a CreateApplication() method that
 * initializes their game specific code inside of the engine.
 */
Application* CreateApplication();

}  // namespace engine

#endif  // ENGINE_SRC_CORE_APPLICATION_H_

/**
 * @file engine/src/core/Application.h
 * @brief Contains the Application class definitions.
 *
 * The Application class is the primary driver of all applications being run by
 * the engine. It is designed to handle everything from events to rendering
 * without having to expose itself to applications that are being created with
 * it.
 */

/**
 * @class Application
 * @brief The primary driver of all applications extending this engine.
 *
 * The engine implements the application runner as an individual platform
 * independent application instance that manages the lifecycle of the core and
 * lower level components of the engine.
 */

/**
 * @fn Run
 * @brief Controls the applications lifecycle and all lower level
 * functionality like input, events, rendering, networking, etc.
 */

/**
 * @fn OnEvent
 * @param event An event pointer generated to be handled by the application.
 * @brief Passes events to all the layers.
 */

/**
 * @param layer
 * @brief Attaches a layer to the application instance.
 *
 * This allows the application instance to propage events, rendering, and any
 * desired pieces of data into the layer.
 */

/**
 * @fn PushOverlay
 * @brief Attaches an overlay to the application instance.
 * This allows the application instance to propage events, renderine,
 * and any desired pieces of data into the layer.
 */

/**
 * @fn GetWindow
 * @brief Gets a read only reference to the window pointer that is currently
 * being used.
 */

/**
 * The application is instantiated at runtime and this function is independent
 * of any single application instance (There can currently only be one
 * instance running, anyways.)
 */
