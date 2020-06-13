#include "core/Application.h"

#include <functional>
#include <initializer_list>
#include <memory>

#include "core/Assert.h"
#include "core/Input.h"
#include "core/Layer.h"
#include "core/Log.h"
#include "core/Window.h"
#include "core/events/ApplicationEvent.h"
#include "core/events/Event.h"

namespace engine {

Application* Application::kApplication_ = nullptr;

Application::Application() {
  ENGINE_CORE_ASSERT(!kApplication_, "Application already exists.");
  kApplication_ = this;

  window_.reset(Window::Create());
  window_->SetEventCallback(BIND_EVENT_FN(Application::OnEvent));

  imgui_layer_ = new imgui::ImGuiLayer();
  PushLayer(imgui_layer_);
}

Application::~Application() {}

/**
 * This currently does a lot of custom rendering when in reality it should
 * be implemented by a child project that is running the game. This will change
 * in the future, but at the moment implements a lot of specific rendering
 * tests that are for ensuring that the renderer currently works.
 */
void Application::Run() {
  while (running_) {
    for (Layer* layer : layer_stack_) {
      layer->OnUpdate();
    }

    imgui_layer_->Begin();
    for (Layer* layer : layer_stack_) {
      layer->OnImGuiRender();
    }
    imgui_layer_->End();

    window_->OnUpdate();
  }
}

/**
 * This function only specifically listens for when the window is requested to
 * close before passing the event to layers on the LayerStack.
 */
void Application::OnEvent(events::Event* event) {
  events::EventDispatcher dispatcher(event);
  dispatcher.Dispatch<events::WindowCloseEvent>
      (BIND_EVENT_FN(Application::OnWindowClosed));

  // Pass the event to all needed layers on the stack.
  for (auto it = layer_stack_.end(); it != layer_stack_.begin();) {
    (*--it)->OnEvent(event);
    if (event->HasBeenHandled()) {
      break;
    }
  }
}

void Application::PushLayer(Layer* layer) {
  layer_stack_.PushLayer(layer);
  layer->OnAttach();
}

void Application::PushOverlay(Layer* layer) {
  layer_stack_.PushOverlay(layer);
  layer->OnAttach();
}

bool Application::OnWindowClosed(const events::WindowCloseEvent& event) {
  running_ = false;
  return true;
}


}  // namespace engine
