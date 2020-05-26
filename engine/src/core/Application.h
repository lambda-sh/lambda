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
#include "core/renderer/Buffer.h"
#include "core/renderer/Shader.h"

namespace engine {

/**
 * An individual platform independent application instance that manages the
 * lifecycle of the core and lower level components of the engine.
 */
class ENGINE_API Application {
 public:
  Application();
  virtual ~Application();

  /**
   * Controls the applications lifecycle and all lower level functionality like
   * input, events, rendering, networking, etc.
   */
  void Run();

  /**
   * The primary event handler for all applications.
   */
  void OnEvent(events::Event* event);

  /**
   * Attaches a layer to the application instance. This allows the application
   * instance to propage events, rendering, and any desired pieces of data into
   * the layer.
   */
  void PushLayer(Layer* layer);

  /**
   * Attaches an overlay to the application instance. This allows the
   * application instance to propage events, renderine, and any desired
   * pieces of data into the layer.
   */
  void PushOverlay(Layer* layer);

  /**
   * This gets the window implementiation for the current window system.
   * Currently, only opengl is known to be supported for both linux and windows
   * platforms.
   */
  inline const Window& GetWindow() const { return *window_; }

  /**
   * The application is instantiated at runtime and this function is independent
   * of any single application instance (There can currently only be one
   * instance running, anyways.)
   */
  inline static Application& GetApplication() {return *kApplication_; }

 private:
  LayerStack layer_stack_;
  bool running_ = true;
  imgui::ImGuiLayer* imgui_layer_;
  std::unique_ptr<Window> window_;
  std::unique_ptr<renderer::Shader> shader_;
  std::unique_ptr<renderer::VertexBuffer> vertex_buffer_;
  std::unique_ptr<renderer::IndexBuffer> index_buffer_;
  unsigned int vertex_array_;

  static Application* kApplication_;

  /**
   * Handles what to do when a window close event is received by the
   * application.
   */
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
